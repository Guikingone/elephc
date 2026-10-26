//! Purpose:
//! Emits the `__rt_fputcsv` runtime helper assembly for fputcsv.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - I/O helpers bridge PHP strings, resources, descriptors, and libc calls while returning runtime arrays or pointer/length strings.
//! - The configured separator and enclosure travel in as pointer/length pairs, so `fputcsv()` honours
//!   php's `separator`/`enclosure` arguments instead of hardcoding `,` and `"`; a zero length selects
//!   the corresponding default. The scan that decides whether a field needs quoting compares each
//!   payload byte against the separator byte, the enclosure byte, and a newline, matching php's rule.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_fputcsv` runtime helper that writes a PHP string array as a CSV line to a file descriptor.
///
/// Each array element is written as a CSV field. Fields containing the separator, the enclosure, or
/// a newline are wrapped in the enclosure; internal enclosure characters are escaped by doubling.
///
/// A trailing newline is written after the last field. The total byte count written (including separators,
/// quotes, and newline) is returned in `x0` (ARM64) or `rax` (x86_64).
///
/// # Inputs
/// - `x0` / `rdi`: file descriptor to write to
/// - `x1` / `rsi`: pointer to a runtime array of PHP strings (each string has a 24-byte header followed by char* and length)
/// - `x2`/`x3` or `rdx`/`rcx`: separator pointer/length, or zero/zero for php's default `","`
/// - `x4`/`x5` or `r8`/`r9`: enclosure pointer/length, or zero/zero for php's default `"\""`
///
/// # Outputs
/// - `x0` / `rax`: total bytes written across all fields, separators, quotes, and trailing newline
///
/// # ABI notes
/// - Each field/separator/quote/newline segment is emitted through `__rt_fd_write`
///   (not a bare `write`), so a synthetic userspace-wrapper fd transparently routes
///   into the wrapper's `stream_write` while a normal fd takes the raw `write` path.
/// - ARM64: `bl __rt_fd_write` per segment; 144-byte stack frame
/// - x86_64: `call __rt_fd_write` per segment; 144-byte frame with callee-saved `rbp`
pub fn emit_fputcsv(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_fputcsv_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: fputcsv ---");
    emitter.label_global("__rt_fputcsv");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #144");                                    // allocate the CSV writer frame
    emitter.instruction("stp x29, x30, [sp, #128]");                            // save frame pointer and return address
    emitter.instruction("add x29, sp, #128");                                   // establish new frame pointer

    // -- save inputs --
    emitter.instruction("str x0, [sp, #0]");                                    // save fd
    emitter.instruction("str x1, [sp, #8]");                                    // save array pointer
    emitter.instruction("str xzr, [sp, #16]");                                  // total bytes written = 0
    emitter.instruction("str xzr, [sp, #24]");                                  // current element index = 0

    // -- resolve the separator: a zero length selects php's default comma --
    emitter.instruction("cbnz x3, __rt_fputcsv_sep_given");                     // a caller-supplied separator wins
    emitter.adrp("x9", "__rt_fputcsv_comma_lit");                // load the default comma literal address
    emitter.add_lo12("x9", "x9", "__rt_fputcsv_comma_lit");          // resolve exact address
    emitter.instruction("mov x2, x9");                                          // separator pointer = the comma literal
    emitter.instruction("mov x3, #1");                                          // separator length = 1 byte
    emitter.label("__rt_fputcsv_sep_given");
    emitter.instruction("str x2, [sp, #80]");                                   // save separator pointer
    emitter.instruction("str x3, [sp, #88]");                                   // save separator length
    emitter.instruction("ldrb w9, [x2]");                                       // isolate the separator's first byte for the quote scan
    emitter.instruction("str x9, [sp, #112]");                                  // save the separator byte

    // -- resolve the enclosure: a zero length selects php's default double quote --
    emitter.instruction("cbnz x5, __rt_fputcsv_enc_given");                     // a caller-supplied enclosure wins
    emitter.adrp("x9", "__rt_fputcsv_quote_lit");                // load the default quote literal address
    emitter.add_lo12("x9", "x9", "__rt_fputcsv_quote_lit");          // resolve exact address
    emitter.instruction("mov x4, x9");                                          // enclosure pointer = the quote literal
    emitter.instruction("mov x5, #1");                                          // enclosure length = 1 byte
    emitter.label("__rt_fputcsv_enc_given");
    emitter.instruction("str x4, [sp, #96]");                                   // save enclosure pointer
    emitter.instruction("str x5, [sp, #104]");                                  // save enclosure length
    emitter.instruction("ldrb w9, [x4]");                                       // isolate the enclosure's first byte
    emitter.instruction("str x9, [sp, #120]");                                  // save the enclosure byte

    // -- get array length --
    emitter.instruction("ldr x9, [x1]");                                        // load array length from header
    emitter.instruction("str x9, [sp, #32]");                                   // save array length

    // -- classify the element layout: a Mixed array casts each element, a Str array reads slots --
    emitter.instruction("ldr x9, [x1, #-8]");                                   // load the packed array kind word
    emitter.instruction("lsr x9, x9, #8");                                      // move the element value_type into the low bits
    emitter.instruction("and x9, x9, #0x0f");                                   // isolate the element value_type
    emitter.instruction("str x9, [sp, #64]");                                   // save the element value_type for the field loop
    emitter.instruction("str xzr, [sp, #72]");                                  // no owned cast field yet

    // -- main loop: iterate over array elements --
    emitter.label("__rt_fputcsv_loop");
    emitter.instruction("ldr x9, [sp, #24]");                                   // load current index
    emitter.instruction("ldr x10, [sp, #32]");                                  // load array length
    emitter.instruction("cmp x9, x10");                                         // check if we've processed all elements
    emitter.instruction("b.hs __rt_fputcsv_newline");                           // if done, write trailing newline

    // -- write the separator before 2nd+ fields --
    emitter.instruction("cbz x9, __rt_fputcsv_field");                          // skip the separator for the first field
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #80]");                                   // reload separator pointer
    emitter.instruction("ldr x2, [sp, #88]");                                   // reload separator length
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total

    // -- load current field from array --
    emitter.label("__rt_fputcsv_field");
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload current index
    emitter.instruction("ldr x10, [sp, #8]");                                   // reload array pointer
    emitter.instruction("ldr x11, [sp, #64]");                                  // reload the element value_type
    emitter.instruction("cmp x11, #7");                                         // is this a boxed Mixed array?
    emitter.instruction("b.eq __rt_fputcsv_field_mixed");                       // Mixed elements cast to a string per field
    emitter.instruction("lsl x11, x9, #4");                                     // byte offset = index * 16
    emitter.instruction("add x11, x10, x11");                                   // element address = array + offset
    emitter.instruction("ldr x3, [x11, #24]");                                  // load string pointer (skip 24-byte header)
    emitter.instruction("ldr x4, [x11, #32]");                                  // load string length
    emitter.instruction("str xzr, [sp, #72]");                                  // a borrowed string slot owns nothing to free
    emitter.instruction("b __rt_fputcsv_field_ready");                          // the field is loaded

    emitter.label("__rt_fputcsv_field_mixed");
    emitter.instruction("add x11, x10, #24");                                   // skip the 24-byte array header
    emitter.instruction("ldr x0, [x11, x9, lsl #3]");                           // load the boxed Mixed cell pointer
    emitter.instruction("cbnz x0, __rt_fputcsv_field_cast");                    // a present cell is cast to a string
    emitter.instruction("mov x3, #0");                                          // a null cell renders as an empty field
    emitter.instruction("mov x4, #0");                                          // with zero length
    emitter.instruction("str xzr, [sp, #72]");                                  // nothing to free for a null cell
    emitter.instruction("b __rt_fputcsv_field_ready");                          // the empty field is loaded

    emitter.label("__rt_fputcsv_field_cast");
    emitter.instruction("bl __rt_mixed_cast_string");                           // x1=string ptr, x2=string length for the boxed value
    emitter.instruction("mov x3, x1");                                          // field pointer = cast string pointer
    emitter.instruction("mov x4, x2");                                          // field length = cast string length
    emitter.instruction("mov x9, #1");                                          // mark the cast result owned (heap_free ignores scratch)
    emitter.instruction("str x9, [sp, #72]");                                   // save the owned-field flag for the next step

    emitter.label("__rt_fputcsv_field_ready");

    // -- check if field needs quoting (contains the separator, the enclosure, or a newline) --
    emitter.instruction("stp x3, x4, [sp, #40]");                               // save field ptr and len
    emitter.instruction("mov x5, #0");                                          // needs_quote flag = 0
    emitter.instruction("mov x6, #0");                                          // scan index = 0
    emitter.label("__rt_fputcsv_scan");
    emitter.instruction("cmp x6, x4");                                          // check if scan complete
    emitter.instruction("b.hs __rt_fputcsv_write");                             // if done scanning, proceed to write
    emitter.instruction("ldrb w7, [x3, x6]");                                   // load byte at current position
    emitter.instruction("ldr x9, [sp, #112]");                                  // reload the separator byte
    emitter.instruction("cmp w7, w9");                                          // check for the configured separator
    emitter.instruction("b.eq __rt_fputcsv_need_q");                            // separator found, needs quoting
    emitter.instruction("ldr x9, [sp, #120]");                                  // reload the enclosure byte
    emitter.instruction("cmp w7, w9");                                          // check for the configured enclosure
    emitter.instruction("b.eq __rt_fputcsv_need_q");                            // enclosure found, needs quoting
    emitter.instruction("cmp w7, #0x0A");                                       // check for newline
    emitter.instruction("b.eq __rt_fputcsv_need_q");                            // newline found, needs quoting
    emitter.instruction("add x6, x6, #1");                                      // increment scan index
    emitter.instruction("b __rt_fputcsv_scan");                                 // continue scanning

    emitter.label("__rt_fputcsv_need_q");
    emitter.instruction("mov x5, #1");                                          // set needs_quote flag

    // -- write the field (quoted or unquoted) --
    emitter.label("__rt_fputcsv_write");
    emitter.instruction("ldp x3, x4, [sp, #40]");                               // reload field ptr and len
    emitter.instruction("cbz x5, __rt_fputcsv_plain");                          // if no quoting needed, write directly

    // -- write opening enclosure --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #96]");                                   // reload enclosure pointer
    emitter.instruction("ldr x2, [sp, #104]");                                  // reload enclosure length
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total

    // -- write field contents, escaping internal enclosure characters --
    emitter.instruction("ldp x3, x4, [sp, #40]");                               // reload field ptr and len
    emitter.instruction("mov x6, #0");                                          // byte index = 0
    emitter.label("__rt_fputcsv_qloop");
    emitter.instruction("cmp x6, x4");                                          // check if all bytes written
    emitter.instruction("b.hs __rt_fputcsv_close_q");                           // if done, write closing quote
    emitter.instruction("ldrb w7, [x3, x6]");                                   // load current byte
    emitter.instruction("add x6, x6, #1");                                      // advance index
    emitter.instruction("str x6, [sp, #56]");                                   // save current index
    emitter.instruction("ldr x9, [sp, #120]");                                  // reload the enclosure byte
    emitter.instruction("cmp w7, w9");                                          // check if byte is the enclosure character
    emitter.instruction("b.ne __rt_fputcsv_qchar");                             // if not, write normally

    // -- escape the enclosure character by writing it twice --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #96]");                                   // reload enclosure pointer
    emitter.instruction("ldr x2, [sp, #104]");                                  // reload enclosure length
    emitter.instruction("bl __rt_fd_write");                                    // write the escape copy
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total

    // -- write the actual character --
    emitter.label("__rt_fputcsv_qchar");
    emitter.instruction("ldp x3, x4, [sp, #40]");                               // reload field ptr and len
    emitter.instruction("ldr x6, [sp, #56]");                                   // reload byte index
    emitter.instruction("sub x9, x6, #1");                                      // index of byte to write
    emitter.instruction("add x1, x3, x9");                                      // pointer to the byte
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("mov x2, #1");                                          // write 1 byte
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total
    emitter.instruction("ldr x6, [sp, #56]");                                   // reload byte index
    emitter.instruction("ldp x3, x4, [sp, #40]");                               // reload field ptr and len
    emitter.instruction("b __rt_fputcsv_qloop");                                // continue writing

    // -- write closing enclosure --
    emitter.label("__rt_fputcsv_close_q");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("ldr x1, [sp, #96]");                                   // reload enclosure pointer
    emitter.instruction("ldr x2, [sp, #104]");                                  // reload enclosure length
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total
    emitter.instruction("b __rt_fputcsv_next");                                 // proceed to next field

    // -- write plain field (no quoting needed) --
    emitter.label("__rt_fputcsv_plain");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.instruction("mov x1, x3");                                          // field pointer
    emitter.instruction("mov x2, x4");                                          // field length
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save updated total

    // -- advance to next element --
    emitter.label("__rt_fputcsv_next");
    emitter.instruction("ldr x9, [sp, #72]");                                   // was the field a cast result we own?
    emitter.instruction("cbz x9, __rt_fputcsv_next_advance");                   // borrowed string slots are never freed
    emitter.instruction("ldr x0, [sp, #40]");                                   // reload the cast field pointer
    emitter.instruction("bl __rt_heap_free");                                   // release the cast result (a no-op for scratch pointers)
    emitter.instruction("str xzr, [sp, #72]");                                  // clear the owned-field flag
    emitter.label("__rt_fputcsv_next_advance");
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload current index
    emitter.instruction("add x9, x9, #1");                                      // increment index
    emitter.instruction("str x9, [sp, #24]");                                   // save updated index
    emitter.instruction("b __rt_fputcsv_loop");                                 // continue loop

    // -- write trailing newline --
    emitter.label("__rt_fputcsv_newline");
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload fd
    emitter.adrp("x1", "__rt_fputcsv_nl_lit");                   // load newline literal address
    emitter.add_lo12("x1", "x1", "__rt_fputcsv_nl_lit");             // resolve exact address
    emitter.instruction("mov x2, #1");                                          // write 1 byte (newline)
    emitter.instruction("bl __rt_fd_write");                                    // write this segment (wrapper-aware: stream_write or raw write)
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload total bytes
    emitter.instruction("add x9, x9, x0");                                      // add final bytes written
    emitter.instruction("str x9, [sp, #16]");                                   // save final total

    // -- return total bytes written --
    emitter.instruction("ldr x0, [sp, #16]");                                   // return total bytes written

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #128]");                            // restore frame pointer and return address
    emitter.instruction("add sp, sp, #144");                                    // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller

    // -- literal data for comma, quote, and newline characters --
    emitter.label("__rt_fputcsv_comma_lit");
    emitter.instruction(".ascii \",\"");                                        // comma character literal
    emitter.label("__rt_fputcsv_quote_lit");
    emitter.instruction(".ascii \"\\\"\"");                                     // double quote character literal
    emitter.label("__rt_fputcsv_nl_lit");
    emitter.instruction(".ascii \"\\n\"");                                      // newline character literal
}

/// x86_64 Linux implementation of the `__rt_fputcsv` runtime helper.
///
/// Uses the System V AMD64 ABI: `rdi` = fd, `rsi` = array pointer, `rdx`/`rcx` = separator
/// pointer/length, `r8`/`r9` = enclosure pointer/length. Calls `__rt_fd_write()` for each output
/// segment (separators, field content, enclosures, newline). Preserves and restores `rbp` as frame pointer.
/// The total byte count is returned in `rax`.
///
/// # Stack layout (152 bytes)
/// - `[rbp - 8]`  : file descriptor (preserved across all steps)
/// - `[rbp - 16]` : array pointer (preserved across field loads)
/// - `[rbp - 24]` : running total bytes written
/// - `[rbp - 32]` : current field index
/// - `[rbp - 40]` : array length (for loop termination)
/// - `[rbp - 48]` : current field string pointer
/// - `[rbp - 56]` : current field string length
/// - `[rbp - 64]` : needs-quote flag (0 or 1)
/// - `[rbp - 72]` : byte index inside quoted field loop
/// - `[rbp - 80]` : element value_type (Str reads slots, Mixed casts per element)
/// - `[rbp - 88]` : owned cast-field flag
/// - `[rbp - 96]` : separator pointer
/// - `[rbp - 104]`: separator length
/// - `[rbp - 112]`: enclosure pointer
/// - `[rbp - 120]`: enclosure length
/// - `[rbp - 128]`: separator first byte
/// - `[rbp - 136]`: enclosure first byte
fn emit_fputcsv_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: fputcsv ---");
    emitter.label_global("__rt_fputcsv");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer while fputcsv() keeps stream and field state in stack slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the file descriptor, array pointer, and CSV writer bookkeeping
    emitter.instruction("sub rsp, 152");                                        // reserve aligned stack space for the CSV writer state across repeated __rt_fd_write() calls
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the destination file descriptor across all field-scan and write helper steps
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve the source string-array pointer across repeated field loads
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // total written bytes start at zero before any CSV separator or field bytes are emitted
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // current field index starts at zero before iterating the source array

    // -- resolve the separator and enclosure, defaulting a zero length to php's comma/quote --
    emitter.instruction("test rcx, rcx");                                       // did the caller supply a separator?
    emitter.instruction("jnz __rt_fputcsv_sep_given_x86");                      // yes, keep it
    emitter.instruction("lea rdx, [rip + __rt_fputcsv_comma_lit]");             // default separator is the comma literal
    emitter.instruction("mov rcx, 1");                                          // one byte long
    emitter.label("__rt_fputcsv_sep_given_x86");
    emitter.instruction("mov QWORD PTR [rbp - 96], rdx");                       // save separator pointer
    emitter.instruction("mov QWORD PTR [rbp - 104], rcx");                      // save separator length
    emitter.instruction("movzx r10d, BYTE PTR [rdx]");                          // isolate the separator byte for the quote scan
    emitter.instruction("mov QWORD PTR [rbp - 128], r10");                      // save the separator byte

    emitter.instruction("test r9, r9");                                         // did the caller supply an enclosure?
    emitter.instruction("jnz __rt_fputcsv_enc_given_x86");                      // yes, keep it
    emitter.instruction("lea r8, [rip + __rt_fputcsv_quote_lit]");              // default enclosure is the quote literal
    emitter.instruction("mov r9, 1");                                           // one byte long
    emitter.label("__rt_fputcsv_enc_given_x86");
    emitter.instruction("mov QWORD PTR [rbp - 112], r8");                       // save enclosure pointer
    emitter.instruction("mov QWORD PTR [rbp - 120], r9");                       // save enclosure length
    emitter.instruction("movzx r10d, BYTE PTR [r8]");                           // isolate the enclosure byte
    emitter.instruction("mov QWORD PTR [rbp - 136], r10");                      // save the enclosure byte

    emitter.instruction("mov r10, QWORD PTR [rsi]");                            // load the source string-array logical length before entering the CSV writer loop
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve the source string-array length for the loop termination check
    emitter.instruction("mov r10, QWORD PTR [rsi - 8]");                        // load the packed array kind word before classifying the element layout
    emitter.instruction("shr r10, 8");                                          // move the element value_type into the low bits
    emitter.instruction("and r10, 0x0f");                                       // isolate the element value_type
    emitter.instruction("mov QWORD PTR [rbp - 80], r10");                       // save the element value_type for the field loop
    emitter.instruction("mov QWORD PTR [rbp - 88], 0");                         // no owned cast field yet

    emitter.label("__rt_fputcsv_loop_x86");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the current field index before checking loop completion
    emitter.instruction("cmp r10, QWORD PTR [rbp - 40]");                       // have we already emitted every field from the source string array?
    emitter.instruction("jae __rt_fputcsv_newline_x86");                        // write the trailing newline once every field has been emitted
    emitter.instruction("test r10, r10");                                       // is the current field index zero, meaning this is the first CSV field?
    emitter.instruction("jz __rt_fputcsv_field_x86");                           // skip the separator before the first field
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the separator
    emitter.instruction("mov rsi, QWORD PTR [rbp - 96]");                       // pass the configured separator address as the second __rt_fd_write() argument
    emitter.instruction("mov rdx, QWORD PTR [rbp - 104]");                      // pass the configured separator length
    emitter.instruction("call __rt_fd_write");                                  // emit the separator through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the separator byte count into the running CSV write total

    emitter.label("__rt_fputcsv_field_x86");
    emitter.instruction("mov r10, QWORD PTR [rbp - 32]");                       // reload the current field index before loading the next string slot from the array
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // reload the source string-array pointer before computing the current field slot address
    emitter.instruction("cmp QWORD PTR [rbp - 80], 7");                         // is this a boxed Mixed array?
    emitter.instruction("je __rt_fputcsv_field_mixed_x86");                     // Mixed elements cast to a string per field
    emitter.instruction("mov rcx, r10");                                        // copy the field index before scaling it into the 16-byte string-slot offset
    emitter.instruction("shl rcx, 4");                                          // convert the field index into the byte offset of the current 16-byte string slot
    emitter.instruction("lea rcx, [r11 + rcx + 24]");                           // compute the current string-slot address inside the source array payload region
    emitter.instruction("mov r8, QWORD PTR [rcx]");                             // load the current field string pointer from the source array slot
    emitter.instruction("mov r9, QWORD PTR [rcx + 8]");                         // load the current field string length from the source array slot
    emitter.instruction("mov QWORD PTR [rbp - 88], 0");                         // a borrowed string slot owns nothing to free
    emitter.instruction("jmp __rt_fputcsv_field_ready_x86");                    // the field is loaded

    emitter.label("__rt_fputcsv_field_mixed_x86");
    emitter.instruction("mov rax, QWORD PTR [r11 + r10 * 8 + 24]");             // load the boxed Mixed cell pointer for this field
    emitter.instruction("test rax, rax");                                       // is the cell present?
    emitter.instruction("jnz __rt_fputcsv_field_cast_x86");                     // a present cell is cast to a string
    emitter.instruction("xor r8d, r8d");                                        // a null cell renders as an empty field
    emitter.instruction("xor r9d, r9d");                                        // with zero length
    emitter.instruction("mov QWORD PTR [rbp - 88], 0");                         // nothing to free for a null cell
    emitter.instruction("jmp __rt_fputcsv_field_ready_x86");                    // the empty field is loaded

    emitter.label("__rt_fputcsv_field_cast_x86");
    emitter.instruction("mov rdi, rax");                                        // pass the boxed Mixed cell to the string cast helper
    emitter.instruction("call __rt_mixed_cast_string");                         // rax=string ptr, rdx=string length for the boxed value
    emitter.instruction("mov r8, rax");                                         // field pointer = cast string pointer
    emitter.instruction("mov r9, rdx");                                         // field length = cast string length
    emitter.instruction("mov QWORD PTR [rbp - 88], 1");                         // mark the cast result owned (heap_free ignores scratch)

    emitter.label("__rt_fputcsv_field_ready_x86");
    emitter.instruction("mov QWORD PTR [rbp - 48], r8");                        // preserve the current field string pointer across the quote scan and repeated write() calls
    emitter.instruction("mov QWORD PTR [rbp - 56], r9");                        // preserve the current field string length across the quote scan and repeated write() calls
    emitter.instruction("mov QWORD PTR [rbp - 64], 0");                         // needs_quote starts false before scanning the current field payload
    emitter.instruction("xor ecx, ecx");                                        // start scanning the current field payload from byte index zero

    emitter.label("__rt_fputcsv_scan_x86");
    emitter.instruction("cmp rcx, r9");                                         // have we scanned every byte of the current field payload?
    emitter.instruction("jae __rt_fputcsv_write_x86");                          // proceed to field emission once the quote scan reaches the end of the payload
    emitter.instruction("movzx edx, BYTE PTR [r8 + rcx]");                      // load the current field byte while deciding whether CSV quoting is required
    emitter.instruction("cmp dl, BYTE PTR [rbp - 128]");                        // does the current field byte contain the configured separator?
    emitter.instruction("je __rt_fputcsv_need_q_x86");                          // quote the field when it contains the separator byte
    emitter.instruction("cmp dl, BYTE PTR [rbp - 136]");                        // does the current field byte contain the configured enclosure?
    emitter.instruction("je __rt_fputcsv_need_q_x86");                          // quote the field when it contains the enclosure byte
    emitter.instruction("cmp dl, 0x0A");                                        // does the current field byte contain a newline?
    emitter.instruction("je __rt_fputcsv_need_q_x86");                          // quote the field when it contains a newline byte
    emitter.instruction("add rcx, 1");                                          // advance to the next field byte while scanning for CSV quote triggers
    emitter.instruction("jmp __rt_fputcsv_scan_x86");                           // continue scanning the field payload for quote-triggering bytes

    emitter.label("__rt_fputcsv_need_q_x86");
    emitter.instruction("mov QWORD PTR [rbp - 64], 1");                         // remember that the current field must be emitted inside CSV quotes

    emitter.label("__rt_fputcsv_write_x86");
    emitter.instruction("cmp QWORD PTR [rbp - 64], 0");                         // does the current field require CSV quoting based on the scan result?
    emitter.instruction("je __rt_fputcsv_plain_x86");                           // write the field directly when no separators or enclosures were found
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the opening enclosure
    emitter.instruction("mov rsi, QWORD PTR [rbp - 112]");                      // pass the configured enclosure address as the second __rt_fd_write() argument
    emitter.instruction("mov rdx, QWORD PTR [rbp - 120]");                      // pass the configured enclosure length
    emitter.instruction("call __rt_fd_write");                                  // emit the opening enclosure through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the opening-enclosure byte count into the running CSV write total
    emitter.instruction("mov QWORD PTR [rbp - 72], 0");                         // current byte index inside the quoted field starts at zero before the per-byte writer loop

    emitter.label("__rt_fputcsv_qloop_x86");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 72]");                       // reload the current byte index before checking whether the quoted field payload is finished
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 56]");                       // have we emitted every byte from the quoted field payload?
    emitter.instruction("jae __rt_fputcsv_close_q_x86");                        // write the closing enclosure once all quoted field bytes have been emitted
    emitter.instruction("mov r8, QWORD PTR [rbp - 48]");                        // reload the current field string pointer before fetching the next quoted field byte
    emitter.instruction("movzx edx, BYTE PTR [r8 + rcx]");                      // load the current field byte while deciding whether it must be escaped
    emitter.instruction("cmp dl, BYTE PTR [rbp - 136]");                        // is the current field byte itself the enclosure character?
    emitter.instruction("jne __rt_fputcsv_qchar_x86");                          // skip the escape-prefix write when the current byte is not the enclosure character
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the escaped enclosure prefix
    emitter.instruction("mov rsi, QWORD PTR [rbp - 112]");                      // pass the configured enclosure address for the escaped enclosure prefix
    emitter.instruction("mov rdx, QWORD PTR [rbp - 120]");                      // pass the configured enclosure length for the escaped enclosure prefix
    emitter.instruction("call __rt_fd_write");                                  // emit the escape-prefix enclosure through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the escape-prefix byte count into the running CSV write total

    emitter.label("__rt_fputcsv_qchar_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the current field byte
    emitter.instruction("mov r8, QWORD PTR [rbp - 48]");                        // reload the current field string pointer before writing the current field byte
    emitter.instruction("mov rcx, QWORD PTR [rbp - 72]");                       // reload the current byte index before computing the source pointer of the current field byte
    emitter.instruction("lea rsi, [r8 + rcx]");                                 // point __rt_fd_write() at the current field byte inside the source string payload
    emitter.instruction("mov edx, 1");                                          // write exactly one payload byte from the quoted field
    emitter.instruction("call __rt_fd_write");                                  // emit the current field byte through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the current field-byte count into the running CSV write total
    emitter.instruction("add QWORD PTR [rbp - 72], 1");                         // advance to the next byte inside the quoted field payload
    emitter.instruction("jmp __rt_fputcsv_qloop_x86");                          // continue emitting the quoted field payload byte-by-byte

    emitter.label("__rt_fputcsv_close_q_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the closing enclosure
    emitter.instruction("mov rsi, QWORD PTR [rbp - 112]");                      // pass the configured enclosure address for the closing enclosure
    emitter.instruction("mov rdx, QWORD PTR [rbp - 120]");                      // pass the configured enclosure length for the closing enclosure
    emitter.instruction("call __rt_fd_write");                                  // emit the closing enclosure through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the closing-enclosure byte count into the running CSV write total
    emitter.instruction("jmp __rt_fputcsv_next_x86");                           // advance to the next field after finishing the quoted field emission

    emitter.label("__rt_fputcsv_plain_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the plain field path
    emitter.instruction("mov rsi, QWORD PTR [rbp - 48]");                       // pass the current field string pointer as the second __rt_fd_write() argument for the plain field path
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // pass the current field string length as the third __rt_fd_write() argument for the plain field path
    emitter.instruction("call __rt_fd_write");                                  // emit the entire unquoted field payload through one __rt_fd_write() call
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the plain field byte count into the running CSV write total

    emitter.label("__rt_fputcsv_next_x86");
    emitter.instruction("cmp QWORD PTR [rbp - 88], 0");                         // was the field a cast result we own?
    emitter.instruction("je __rt_fputcsv_next_advance_x86");                    // borrowed string slots are never freed
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // reload the cast field pointer
    emitter.instruction("call __rt_heap_free");                                 // release the cast result (a no-op for scratch pointers)
    emitter.instruction("mov QWORD PTR [rbp - 88], 0");                         // clear the owned-field flag
    emitter.label("__rt_fputcsv_next_advance_x86");
    emitter.instruction("add QWORD PTR [rbp - 32], 1");                         // advance to the next field index before looping back to the CSV field iterator
    emitter.instruction("jmp __rt_fputcsv_loop_x86");                           // continue emitting the remaining CSV fields from the source string array

    emitter.label("__rt_fputcsv_newline_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the destination file descriptor as the first __rt_fd_write() argument for the trailing newline
    emitter.instruction("lea rsi, [rip + __rt_fputcsv_nl_lit]");                // pass the newline literal address as the second __rt_fd_write() argument
    emitter.instruction("mov edx, 1");                                          // write exactly one trailing newline byte after the last CSV field
    emitter.instruction("call __rt_fd_write");                                  // emit the trailing newline through __rt_fd_write()
    emitter.instruction("add QWORD PTR [rbp - 24], rax");                       // accumulate the trailing newline byte count into the running CSV write total
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // return the total number of bytes that fputcsv() emitted through __rt_fd_write()
    emitter.instruction("add rsp, 152");                                        // release the CSV writer spill slots before returning to the caller
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the x86_64 CSV writer completes
    emitter.instruction("ret");                                                 // return the total written byte count in the x86_64 integer result register

    emitter.label("__rt_fputcsv_comma_lit");
    emitter.instruction(".ascii \",\"");                                        // comma character literal used as the default CSV field separator
    emitter.label("__rt_fputcsv_quote_lit");
    emitter.instruction(".ascii \"\\\"\"");                                     // double quote character literal used as the default CSV enclosure
    emitter.label("__rt_fputcsv_nl_lit");
    emitter.instruction(".ascii \"\\n\"");                                      // trailing newline character literal written after the final CSV field
}
