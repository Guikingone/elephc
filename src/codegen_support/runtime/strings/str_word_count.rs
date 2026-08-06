//! Purpose:
//! Emits the `__rt_str_word_count` runtime helper assembly for PHP's `str_word_count`:
//! scans the subject for php-src's word definition and materializes the requested result
//! shape (a count, a list of words, or a byte-offset map).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - php-src's word alphabet is `isalpha()` under the C locale plus `'` and `-`, extended by
//!   every byte in the optional `$characters` list. A 256-byte membership table is built on
//!   the helper's own frame, so the scan never re-walks the character list.
//! - php-src trims a LEADING `'` or `-` and a TRAILING `-` before scanning, unless the
//!   character list explicitly re-admits that byte. Both adjustments are reproduced verbatim.
//! - Format 1 pushes each word through `__rt_array_push_str`, which persists the bytes, and
//!   format 2 persists the word itself before `__rt_hash_set` stores it under its byte offset.
//!   Neither result can alias the subject, which is what the `Fresh` ownership contract on
//!   `RuntimeFnId::StrWordCount` promises.
//! - An unknown `$format` never reaches this helper: the EIR lowering raises php-src's
//!   catchable `ValueError` before the call.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Byte offset of the 256-entry word-character membership table inside the helper frame.
const MASK_BYTES: usize = 256;

/// Emits the `__rt_str_word_count` runtime helper.
///
/// ABI (AArch64):
///   Input:  `x1` = subject pointer, `x2` = subject length, `x3` = format (0, 1, or 2),
///           `x4` = character-list pointer, `x5` = character-list length.
///   Output: `x0` = word count (format 0), indexed array pointer (format 1), or hash
///           pointer (format 2).
///
/// ABI (x86_64 System V):
///   Input:  `rax` = subject pointer, `rdx` = subject length, `rdi` = format,
///           `rcx` = character-list pointer, `r8` = character-list length.
///   Output: `rax` = the same three result shapes.
///
/// Clobbers every caller-saved register: the container paths reach `__rt_array_new`,
/// `__rt_hash_new`, `__rt_array_push_str`, `__rt_str_persist`, and `__rt_hash_set`.
pub fn emit_str_word_count(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_word_count_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_word_count ---");
    emitter.label_global("__rt_str_word_count");

    // Frame layout (352 bytes):
    //   [sp, #0]   = 256-byte word-character membership table
    //   [sp, #256] = subject base pointer
    //   [sp, #264] = subject length
    //   [sp, #272] = requested format
    //   [sp, #280] = scan cursor `p`
    //   [sp, #288] = scan end `e`
    //   [sp, #296] = running result (count, array pointer, or hash pointer)
    //   [sp, #304] = current word start
    //   [sp, #312] = current word length
    //   [sp, #320] = current word byte offset (format 2 hash key)
    //   [sp, #336] = saved x29/x30
    emitter.instruction("sub sp, sp, #352");                                    // allocate the word-scan frame with its membership table
    emitter.instruction("stp x29, x30, [sp, #336]");                            // save the frame pointer and return address across the container helper calls
    emitter.instruction("add x29, sp, #336");                                   // establish the str_word_count helper frame pointer
    emitter.instruction("str x1, [sp, #256]");                                  // save the subject base pointer for the format 2 offset keys
    emitter.instruction("str x2, [sp, #264]");                                  // save the subject length across the container allocation
    emitter.instruction("str x3, [sp, #272]");                                  // save the requested result format

    // -- clear the 256-entry word-character membership table --
    emitter.instruction("mov x10, sp");                                         // x10 = membership table base, recomputed after every helper call
    emitter.instruction("mov x9, #0");                                          // start clearing at the first table word
    emitter.label("__rt_str_word_count_zero");
    emitter.instruction("str xzr, [x10, x9]");                                  // clear eight membership entries at a time
    emitter.instruction("add x9, x9, #8");                                      // advance to the next table word
    emitter.instruction(&format!("cmp x9, #{}", MASK_BYTES));                   // has the whole table been cleared?
    emitter.instruction("b.lo __rt_str_word_count_zero");                       // keep clearing until the table is fully zeroed

    // -- admit every byte of the optional `$characters` list as a word character --
    emitter.instruction("mov x9, #0");                                          // start at the first character-list byte
    emitter.label("__rt_str_word_count_mask");
    emitter.instruction("cmp x9, x5");                                          // has the whole character list been consumed?
    emitter.instruction("b.hs __rt_str_word_count_mask_done");                  // the membership table is complete
    emitter.instruction("ldrb w11, [x4, x9]");                                  // load the next extra word character
    emitter.instruction("mov w12, #1");                                         // membership marker for an admitted byte
    emitter.instruction("strb w12, [x10, x11]");                                // admit the byte into the word alphabet
    emitter.instruction("add x9, x9, #1");                                      // advance to the next character-list byte
    emitter.instruction("b __rt_str_word_count_mask");                          // keep admitting character-list bytes
    emitter.label("__rt_str_word_count_mask_done");

    // -- allocate the container the requested format needs --
    emitter.instruction("mov x0, xzr");                                         // format 0 accumulates a plain word count starting at zero
    emitter.instruction("ldr x3, [sp, #272]");                                  // reload the requested result format
    emitter.instruction("cbz x3, __rt_str_word_count_container_ready");         // format 0 needs no container at all
    emitter.instruction("cmp x3, #1");                                          // is the caller asking for the list of words?
    emitter.instruction("b.ne __rt_str_word_count_map_new");                    // format 2 builds the byte-offset map instead
    emitter.instruction("mov x0, #16");                                         // seed the word list with room for sixteen entries
    emitter.instruction("mov x1, #16");                                         // 16-byte slots hold the (pointer, length) string pairs
    emitter.instruction("bl __rt_array_new");                                   // allocate the indexed result array
    emitter.instruction("b __rt_str_word_count_container_ready");               // the list container is ready
    emitter.label("__rt_str_word_count_map_new");
    emitter.instruction("mov x0, #8");                                          // seed the byte-offset map with the minimum hash capacity
    emitter.instruction("mov x1, #1");                                          // value_type 1 = string values
    emitter.instruction("bl __rt_hash_new");                                    // allocate the byte-offset result hash
    emitter.label("__rt_str_word_count_container_ready");
    emitter.instruction("str x0, [sp, #296]");                                  // publish the running result before the scan starts
    emitter.instruction("mov x10, sp");                                         // restore the membership table base after the container allocation

    // -- an empty subject yields the count 0 or the empty container --
    emitter.instruction("ldr x1, [sp, #256]");                                  // reload the subject base pointer
    emitter.instruction("ldr x2, [sp, #264]");                                  // reload the subject length
    emitter.instruction("cbz x2, __rt_str_word_count_done");                    // php-src returns immediately for an empty subject
    emitter.instruction("add x3, x1, x2");                                      // scan end = subject base + subject length

    // -- php-src drops a leading `'` or `-` unless the character list re-admits it --
    emitter.instruction("ldrb w11, [x1]");                                      // load the subject's first byte
    emitter.instruction("cmp w11, #39");                                        // is the first byte an apostrophe?
    emitter.instruction("b.ne __rt_str_word_count_first_dash");                 // check the leading hyphen case instead
    emitter.instruction("ldrb w13, [x10, #39]");                                // is the apostrophe an explicitly admitted word character?
    emitter.instruction("cbnz w13, __rt_str_word_count_first_done");            // an admitted apostrophe may open a word
    emitter.instruction("add x1, x1, #1");                                      // skip the leading apostrophe
    emitter.instruction("b __rt_str_word_count_first_done");                    // php-src performs at most one leading skip
    emitter.label("__rt_str_word_count_first_dash");
    emitter.instruction("cmp w11, #45");                                        // is the first byte a hyphen?
    emitter.instruction("b.ne __rt_str_word_count_first_done");                 // any other first byte is scanned normally
    emitter.instruction("ldrb w13, [x10, #45]");                                // is the hyphen an explicitly admitted word character?
    emitter.instruction("cbnz w13, __rt_str_word_count_first_done");            // an admitted hyphen may open a word
    emitter.instruction("add x1, x1, #1");                                      // skip the leading hyphen
    emitter.label("__rt_str_word_count_first_done");

    // -- php-src drops a trailing `-` unless the character list re-admits it --
    emitter.instruction("sub x9, x3, #1");                                      // address the subject's last byte
    emitter.instruction("ldrb w11, [x9]");                                      // load the subject's last byte
    emitter.instruction("cmp w11, #45");                                        // is the last byte a hyphen?
    emitter.instruction("b.ne __rt_str_word_count_last_done");                  // any other last byte stays inside the scan range
    emitter.instruction("ldrb w13, [x10, #45]");                                // is the hyphen an explicitly admitted word character?
    emitter.instruction("cbnz w13, __rt_str_word_count_last_done");             // an admitted hyphen may close a word
    emitter.instruction("sub x3, x3, #1");                                      // exclude the trailing hyphen from the scan range
    emitter.label("__rt_str_word_count_last_done");
    emitter.instruction("str x1, [sp, #280]");                                  // publish the adjusted scan cursor
    emitter.instruction("str x3, [sp, #288]");                                  // publish the adjusted scan end

    // -- walk the subject one candidate word at a time --
    emitter.label("__rt_str_word_count_scan");
    emitter.instruction("ldr x1, [sp, #280]");                                  // reload the scan cursor
    emitter.instruction("ldr x3, [sp, #288]");                                  // reload the scan end
    emitter.instruction("cmp x1, x3");                                          // has the scan reached the adjusted end?
    emitter.instruction("b.hs __rt_str_word_count_done");                       // every candidate word has been visited
    emitter.instruction("mov x4, x1");                                          // remember where this candidate word starts
    emitter.instruction("str x4, [sp, #304]");                                  // save the word start across the recording helper calls

    emitter.label("__rt_str_word_count_word");
    emitter.instruction("cmp x1, x3");                                          // is the cursor still inside the scan range?
    emitter.instruction("b.hs __rt_str_word_count_word_end");                   // the word ends at the scan end
    emitter.instruction("ldrb w11, [x1]");                                      // load the candidate word byte
    emitter.instruction("orr w12, w11, #0x20");                                 // fold the byte to lower case for the C-locale isalpha() test
    emitter.instruction("sub w12, w12, #97");                                   // rebase the folded byte on 'a'
    emitter.instruction("cmp w12, #26");                                        // is the byte an ASCII letter?
    emitter.instruction("b.lo __rt_str_word_count_word_char");                  // letters always continue the word
    emitter.instruction("ldrb w13, [x10, x11]");                                // is the byte an explicitly admitted word character?
    emitter.instruction("cbnz w13, __rt_str_word_count_word_char");             // admitted bytes continue the word
    emitter.instruction("cmp w11, #39");                                        // is the byte an interior apostrophe?
    emitter.instruction("b.eq __rt_str_word_count_word_char");                  // apostrophes always continue the word
    emitter.instruction("cmp w11, #45");                                        // is the byte an interior hyphen?
    emitter.instruction("b.eq __rt_str_word_count_word_char");                  // hyphens always continue the word
    emitter.instruction("b __rt_str_word_count_word_end");                      // any other byte terminates the word
    emitter.label("__rt_str_word_count_word_char");
    emitter.instruction("add x1, x1, #1");                                      // consume the word character
    emitter.instruction("b __rt_str_word_count_word");                          // test the next byte

    emitter.label("__rt_str_word_count_word_end");
    emitter.instruction("str x1, [sp, #280]");                                  // publish the cursor before any recording helper call
    emitter.instruction("subs x5, x1, x4");                                     // how many bytes did this candidate word cover?
    emitter.instruction("b.eq __rt_str_word_count_advance");                    // a zero-length candidate is a separator, not a word
    emitter.instruction("str x5, [sp, #312]");                                  // save the word length across the recording helper calls
    emitter.instruction("ldr x3, [sp, #272]");                                  // reload the requested result format
    emitter.instruction("cbnz x3, __rt_str_word_count_record");                 // formats 1 and 2 materialize the word itself
    emitter.instruction("ldr x9, [sp, #296]");                                  // reload the running word count
    emitter.instruction("add x9, x9, #1");                                      // count one more word
    emitter.instruction("str x9, [sp, #296]");                                  // publish the updated word count
    emitter.instruction("b __rt_str_word_count_advance");                       // move past the terminating byte

    emitter.label("__rt_str_word_count_record");
    emitter.instruction("cmp x3, #1");                                          // is the caller collecting the plain list of words?
    emitter.instruction("b.ne __rt_str_word_count_record_map");                 // format 2 stores the word under its byte offset
    emitter.instruction("ldr x0, [sp, #296]");                                  // reload the result array pointer
    emitter.instruction("ldr x1, [sp, #304]");                                  // pass the word start to the append helper
    emitter.instruction("ldr x2, [sp, #312]");                                  // pass the word length to the append helper
    emitter.instruction("bl __rt_array_push_str");                              // append a persisted copy of the word
    emitter.instruction("str x0, [sp, #296]");                                  // republish the array pointer after possible growth
    emitter.instruction("b __rt_str_word_count_advance");                       // move past the terminating byte

    emitter.label("__rt_str_word_count_record_map");
    emitter.instruction("ldr x9, [sp, #304]");                                  // reload the word start
    emitter.instruction("ldr x11, [sp, #256]");                                 // reload the subject base pointer
    emitter.instruction("sub x9, x9, x11");                                     // the map key is the word's byte offset in the subject
    emitter.instruction("str x9, [sp, #320]");                                  // save the map key across the persist call
    emitter.instruction("ldr x1, [sp, #304]");                                  // pass the word start to the persist helper
    emitter.instruction("ldr x2, [sp, #312]");                                  // pass the word length to the persist helper
    emitter.instruction("bl __rt_str_persist");                                 // copy the word so the map owns its own bytes
    emitter.instruction("mov x3, x1");                                          // value_lo = owned word pointer
    emitter.instruction("mov x4, x2");                                          // value_hi = owned word length
    emitter.instruction("ldr x0, [sp, #296]");                                  // reload the result hash pointer
    emitter.instruction("ldr x1, [sp, #320]");                                  // key_lo = the word's byte offset
    emitter.instruction("mov x2, #-1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("mov x5, #1");                                          // runtime tag 1 marks the stored value as a string
    emitter.instruction("bl __rt_hash_set");                                    // insert the word under its byte offset
    emitter.instruction("str x0, [sp, #296]");                                  // republish the hash pointer after possible growth

    emitter.label("__rt_str_word_count_advance");
    emitter.instruction("mov x10, sp");                                         // restore the membership table base after the recording helper calls
    emitter.instruction("ldr x1, [sp, #280]");                                  // reload the scan cursor
    emitter.instruction("add x1, x1, #1");                                      // php-src always steps past the terminating byte
    emitter.instruction("str x1, [sp, #280]");                                  // publish the advanced scan cursor
    emitter.instruction("b __rt_str_word_count_scan");                          // look for the next candidate word

    emitter.label("__rt_str_word_count_done");
    emitter.instruction("ldr x0, [sp, #296]");                                  // return the count or the finished container
    emitter.instruction("ldp x29, x30, [sp, #336]");                            // restore the frame pointer and return address
    emitter.instruction("add sp, sp, #352");                                    // release the word-scan frame
    emitter.instruction("ret");                                                 // return the requested str_word_count result
}

/// Emits `__rt_str_word_count` for x86_64 Linux using the System V ABI.
///
/// The membership table sits at the bottom of the frame and is addressed as
/// `[rbp + index - 384]`, so no register has to survive the container helper calls to
/// keep it reachable. The saved-value slots start at `[rbp-32]` to stay clear of the
/// `[rbp-8]`..`[rbp-24]` window other runtime emitters reserve for pushed callee-saved
/// registers.
fn emit_str_word_count_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_word_count ---");
    emitter.label_global("__rt_str_word_count");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the container helper calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the scan state and membership table
    emitter.instruction("sub rsp, 384");                                        // reserve the scan slots plus the 256-byte membership table
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the subject base pointer for the format 2 offset keys
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // save the subject length across the container allocation
    emitter.instruction("mov QWORD PTR [rbp - 48], rdi");                       // save the requested result format

    // -- clear the 256-entry word-character membership table --
    emitter.instruction("xor r9d, r9d");                                        // start clearing at the first table word
    emitter.label("__rt_str_word_count_zero_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp + r9 - 384], 0");                   // clear eight membership entries at a time
    emitter.instruction("add r9, 8");                                           // advance to the next table word
    emitter.instruction(&format!("cmp r9, {}", MASK_BYTES));                    // has the whole table been cleared?
    emitter.instruction("jb __rt_str_word_count_zero_linux_x86_64");            // keep clearing until the table is fully zeroed

    // -- admit every byte of the optional `$characters` list as a word character --
    emitter.instruction("xor r9d, r9d");                                        // start at the first character-list byte
    emitter.label("__rt_str_word_count_mask_linux_x86_64");
    emitter.instruction("cmp r9, r8");                                          // has the whole character list been consumed?
    emitter.instruction("jae __rt_str_word_count_mask_done_linux_x86_64");      // the membership table is complete
    emitter.instruction("movzx r10d, BYTE PTR [rcx + r9]");                     // load the next extra word character
    emitter.instruction("mov BYTE PTR [rbp + r10 - 384], 1");                   // admit the byte into the word alphabet
    emitter.instruction("add r9, 1");                                           // advance to the next character-list byte
    emitter.instruction("jmp __rt_str_word_count_mask_linux_x86_64");           // keep admitting character-list bytes
    emitter.label("__rt_str_word_count_mask_done_linux_x86_64");

    // -- allocate the container the requested format needs --
    emitter.instruction("xor eax, eax");                                        // format 0 accumulates a plain word count starting at zero
    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // reload the requested result format
    emitter.instruction("test r9, r9");                                         // is the caller only asking for the word count?
    emitter.instruction("jz __rt_str_word_count_ready_linux_x86_64");           // format 0 needs no container at all
    emitter.instruction("cmp r9, 1");                                           // is the caller asking for the list of words?
    emitter.instruction("jne __rt_str_word_count_map_new_linux_x86_64");        // format 2 builds the byte-offset map instead
    emitter.instruction("mov edi, 16");                                         // seed the word list with room for sixteen entries
    emitter.instruction("mov esi, 16");                                         // 16-byte slots hold the (pointer, length) string pairs
    emitter.instruction("call __rt_array_new");                                 // allocate the indexed result array
    emitter.instruction("jmp __rt_str_word_count_ready_linux_x86_64");          // the list container is ready
    emitter.label("__rt_str_word_count_map_new_linux_x86_64");
    emitter.instruction("cmp r9, 2");                                           // is the caller asking for the byte-offset map?
    emitter.instruction("jne __rt_str_word_count_ready_linux_x86_64");          // an unknown format never reaches this helper
    emitter.instruction("mov edi, 8");                                          // seed the byte-offset map with the minimum hash capacity
    emitter.instruction("mov esi, 1");                                          // value_type 1 = string values
    emitter.instruction("call __rt_hash_new");                                  // allocate the byte-offset result hash
    emitter.label("__rt_str_word_count_ready_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // publish the running result before the scan starts

    // -- an empty subject yields the count 0 or the empty container --
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // reload the subject base pointer as the scan cursor
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // reload the subject length
    emitter.instruction("test rdx, rdx");                                       // is the subject empty?
    emitter.instruction("jz __rt_str_word_count_done_linux_x86_64");            // php-src returns immediately for an empty subject
    emitter.instruction("mov rdi, rsi");                                        // copy the subject base before deriving the scan end
    emitter.instruction("add rdi, rdx");                                        // scan end = subject base + subject length

    // -- php-src drops a leading `'` or `-` unless the character list re-admits it --
    emitter.instruction("movzx r9d, BYTE PTR [rsi]");                           // load the subject's first byte
    emitter.instruction("cmp r9d, 39");                                         // is the first byte an apostrophe?
    emitter.instruction("jne __rt_str_word_count_first_dash_linux_x86_64");     // check the leading hyphen case instead
    emitter.instruction("cmp BYTE PTR [rbp + r9 - 384], 0");                    // is the apostrophe an explicitly admitted word character?
    emitter.instruction("jne __rt_str_word_count_first_done_linux_x86_64");     // an admitted apostrophe may open a word
    emitter.instruction("add rsi, 1");                                          // skip the leading apostrophe
    emitter.instruction("jmp __rt_str_word_count_first_done_linux_x86_64");     // php-src performs at most one leading skip
    emitter.label("__rt_str_word_count_first_dash_linux_x86_64");
    emitter.instruction("cmp r9d, 45");                                         // is the first byte a hyphen?
    emitter.instruction("jne __rt_str_word_count_first_done_linux_x86_64");     // any other first byte is scanned normally
    emitter.instruction("cmp BYTE PTR [rbp + r9 - 384], 0");                    // is the hyphen an explicitly admitted word character?
    emitter.instruction("jne __rt_str_word_count_first_done_linux_x86_64");     // an admitted hyphen may open a word
    emitter.instruction("add rsi, 1");                                          // skip the leading hyphen
    emitter.label("__rt_str_word_count_first_done_linux_x86_64");

    // -- php-src drops a trailing `-` unless the character list re-admits it --
    emitter.instruction("movzx r9d, BYTE PTR [rdi - 1]");                       // load the subject's last byte
    emitter.instruction("cmp r9d, 45");                                         // is the last byte a hyphen?
    emitter.instruction("jne __rt_str_word_count_last_done_linux_x86_64");      // any other last byte stays inside the scan range
    emitter.instruction("cmp BYTE PTR [rbp + r9 - 384], 0");                    // is the hyphen an explicitly admitted word character?
    emitter.instruction("jne __rt_str_word_count_last_done_linux_x86_64");      // an admitted hyphen may close a word
    emitter.instruction("sub rdi, 1");                                          // exclude the trailing hyphen from the scan range
    emitter.label("__rt_str_word_count_last_done_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 56], rsi");                       // publish the adjusted scan cursor
    emitter.instruction("mov QWORD PTR [rbp - 64], rdi");                       // publish the adjusted scan end

    // -- walk the subject one candidate word at a time --
    emitter.label("__rt_str_word_count_scan_linux_x86_64");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // reload the scan cursor
    emitter.instruction("mov rdi, QWORD PTR [rbp - 64]");                       // reload the scan end
    emitter.instruction("cmp rsi, rdi");                                        // has the scan reached the adjusted end?
    emitter.instruction("jae __rt_str_word_count_done_linux_x86_64");           // every candidate word has been visited
    emitter.instruction("mov QWORD PTR [rbp - 80], rsi");                       // remember where this candidate word starts

    emitter.label("__rt_str_word_count_word_linux_x86_64");
    emitter.instruction("cmp rsi, rdi");                                        // is the cursor still inside the scan range?
    emitter.instruction("jae __rt_str_word_count_word_end_linux_x86_64");       // the word ends at the scan end
    emitter.instruction("movzx r9d, BYTE PTR [rsi]");                           // load the candidate word byte
    emitter.instruction("mov r10d, r9d");                                       // copy the byte before folding its case
    emitter.instruction("or r10d, 32");                                         // fold the byte to lower case for the C-locale isalpha() test
    emitter.instruction("sub r10d, 97");                                        // rebase the folded byte on 'a'
    emitter.instruction("cmp r10d, 26");                                        // is the byte an ASCII letter?
    emitter.instruction("jb __rt_str_word_count_word_char_linux_x86_64");       // letters always continue the word
    emitter.instruction("cmp BYTE PTR [rbp + r9 - 384], 0");                    // is the byte an explicitly admitted word character?
    emitter.instruction("jne __rt_str_word_count_word_char_linux_x86_64");      // admitted bytes continue the word
    emitter.instruction("cmp r9d, 39");                                         // is the byte an interior apostrophe?
    emitter.instruction("je __rt_str_word_count_word_char_linux_x86_64");       // apostrophes always continue the word
    emitter.instruction("cmp r9d, 45");                                         // is the byte an interior hyphen?
    emitter.instruction("je __rt_str_word_count_word_char_linux_x86_64");       // hyphens always continue the word
    emitter.instruction("jmp __rt_str_word_count_word_end_linux_x86_64");       // any other byte terminates the word
    emitter.label("__rt_str_word_count_word_char_linux_x86_64");
    emitter.instruction("add rsi, 1");                                          // consume the word character
    emitter.instruction("jmp __rt_str_word_count_word_linux_x86_64");           // test the next byte

    emitter.label("__rt_str_word_count_word_end_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 56], rsi");                       // publish the cursor before any recording helper call
    emitter.instruction("mov r11, rsi");                                        // copy the cursor before deriving the word length
    emitter.instruction("sub r11, QWORD PTR [rbp - 80]");                       // how many bytes did this candidate word cover?
    emitter.instruction("test r11, r11");                                       // was the candidate empty?
    emitter.instruction("jz __rt_str_word_count_advance_linux_x86_64");         // a zero-length candidate is a separator, not a word
    emitter.instruction("mov QWORD PTR [rbp - 88], r11");                       // save the word length across the recording helper calls
    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // reload the requested result format
    emitter.instruction("test r9, r9");                                         // is the caller only counting words?
    emitter.instruction("jnz __rt_str_word_count_record_linux_x86_64");         // formats 1 and 2 materialize the word itself
    emitter.instruction("add QWORD PTR [rbp - 72], 1");                         // count one more word
    emitter.instruction("jmp __rt_str_word_count_advance_linux_x86_64");        // move past the terminating byte

    emitter.label("__rt_str_word_count_record_linux_x86_64");
    emitter.instruction("cmp r9, 1");                                           // is the caller collecting the plain list of words?
    emitter.instruction("jne __rt_str_word_count_record_map_linux_x86_64");     // format 2 stores the word under its byte offset
    emitter.instruction("mov rdi, QWORD PTR [rbp - 72]");                       // reload the result array pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // pass the word start in the append helper's SysV string-pointer register
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // pass the word length to the append helper
    emitter.instruction("call __rt_array_push_str");                            // append a persisted copy of the word
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // republish the array pointer after possible growth
    emitter.instruction("jmp __rt_str_word_count_advance_linux_x86_64");        // move past the terminating byte

    emitter.label("__rt_str_word_count_record_map_linux_x86_64");
    emitter.instruction("mov r10, QWORD PTR [rbp - 80]");                       // reload the word start
    emitter.instruction("sub r10, QWORD PTR [rbp - 32]");                       // the map key is the word's byte offset in the subject
    emitter.instruction("mov QWORD PTR [rbp - 96], r10");                       // save the map key across the persist call
    emitter.instruction("mov rax, QWORD PTR [rbp - 80]");                       // pass the word start to the persist helper
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // pass the word length to the persist helper
    emitter.instruction("call __rt_str_persist");                               // copy the word so the map owns its own bytes
    emitter.instruction("mov rcx, rax");                                        // value_lo = owned word pointer
    emitter.instruction("mov r8, rdx");                                         // value_hi = owned word length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 72]");                       // reload the result hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 96]");                       // key_lo = the word's byte offset
    emitter.instruction("mov rdx, -1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("mov r9d, 1");                                          // runtime tag 1 marks the stored value as a string
    emitter.instruction("call __rt_hash_set");                                  // insert the word under its byte offset
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // republish the hash pointer after possible growth

    emitter.label("__rt_str_word_count_advance_linux_x86_64");
    emitter.instruction("mov rsi, QWORD PTR [rbp - 56]");                       // reload the scan cursor
    emitter.instruction("add rsi, 1");                                          // php-src always steps past the terminating byte
    emitter.instruction("mov QWORD PTR [rbp - 56], rsi");                       // publish the advanced scan cursor
    emitter.instruction("jmp __rt_str_word_count_scan_linux_x86_64");           // look for the next candidate word

    emitter.label("__rt_str_word_count_done_linux_x86_64");
    emitter.instruction("mov rax, QWORD PTR [rbp - 72]");                       // return the count or the finished container
    emitter.instruction("add rsp, 384");                                        // release the word-scan frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the requested str_word_count result
}
