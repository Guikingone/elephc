//! Purpose:
//! Emits the `__rt_addcslashes` runtime helper assembly for addcslashes.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_addcslashes` runtime helper for PHP's `addcslashes()`.
///
/// For every source byte that is a member of the `characters` set, writes a
/// C-style escape into the shared concat buffer: control bytes `\t \n \v \f \r
/// \a \b` map to their two-character escapes, other non-printable bytes (`< 32`
/// or `> 126`) become `\ooo` three-digit octal, and printable members are simply
/// prefixed with a backslash. Non-member bytes copy through unchanged. The
/// `characters` set expands `a..z` ranges exactly like PHP's `php_charmask`
/// (ascending ranges only; malformed `..` sequences degrade to literal bytes).
/// Membership is tested by scanning the (small) character set for each source
/// byte, so no 256-byte mask table is materialized.
///
/// ## ARM64 ABI (default)
/// - Input: `x1` = source ptr, `x2` = source len, `x3` = characters ptr, `x4` = characters len
/// - Output: `x1` = result ptr, `x2` = result len
/// - Uses the concat buffer (`_concat_buf` / `_concat_off`) for output storage
///
/// ## x86_64 Linux ABI
/// - Input: `rdi` = source ptr, `rsi` = source len, `rdx` = characters ptr, `rcx` = characters len
/// - Output: `rax` = result ptr, `rdx` = result len
/// - Uses the concat buffer (`_concat_buf` / `_concat_off`) for output storage
pub fn emit_addcslashes(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_addcslashes_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: addcslashes ---");
    emitter.label_global("__rt_addcslashes");

    // -- set up concat_buf destination and scan cursor --
    abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load the current concat-buffer offset
    abi::emit_symbol_address(emitter, "x7", "_concat_buf");
    emitter.instruction("add x9, x7, x8");                                      // destination write pointer = buffer base + offset
    emitter.instruction("mov x10, x9");                                         // remember the result start pointer
    emitter.instruction("mov x11, x2");                                         // remaining source byte count

    emitter.label("__rt_addcslashes_loop");
    emitter.instruction("cbz x11, __rt_addcslashes_done");                      // stop once every source byte is consumed
    emitter.instruction("ldrb w12, [x1], #1");                                  // load the current source byte and advance
    emitter.instruction("sub x11, x11, #1");                                    // account for the consumed source byte

    // -- membership scan: is the byte listed in the characters set? --
    emitter.instruction("mov x13, #0");                                         // character-set scan index = 0
    emitter.label("__rt_addcslashes_mem");
    emitter.instruction("cmp x13, x4");                                         // scanned the whole character set?
    emitter.instruction("b.ge __rt_addcslashes_copy");                          // byte not in the set -> copy it unchanged
    emitter.instruction("add x15, x13, #3");                                    // index of the range's closing byte
    emitter.instruction("cmp x15, x4");                                         // is there room for an `a..z` range here?
    emitter.instruction("b.ge __rt_addcslashes_single");                        // too few bytes left -> single-byte membership test
    emitter.instruction("ldrb w14, [x3, x13]");                                 // load the candidate range start byte
    emitter.instruction("add x0, x13, #1");                                     // offset of the first range separator
    emitter.instruction("ldrb w15, [x3, x0]");                                  // load the byte after the range start
    emitter.instruction("cmp w15, #46");                                        // is it a '.' range separator?
    emitter.instruction("b.ne __rt_addcslashes_single_c");                      // not a range -> single-byte test on the start byte
    emitter.instruction("add x0, x13, #2");                                     // offset of the second range separator
    emitter.instruction("ldrb w15, [x3, x0]");                                  // load the second potential separator byte
    emitter.instruction("cmp w15, #46");                                        // is it the second '.' of the range?
    emitter.instruction("b.ne __rt_addcslashes_single_c");                      // not a range -> single-byte test on the start byte
    emitter.instruction("add x0, x13, #3");                                     // offset of the range's closing byte
    emitter.instruction("ldrb w15, [x3, x0]");                                  // load the range end byte
    emitter.instruction("cmp w15, w14");                                        // is the range ascending (end >= start)?
    emitter.instruction("b.lo __rt_addcslashes_single_c");                      // descending ranges degrade to a literal start byte
    emitter.instruction("cmp w12, w14");                                        // source byte below the range start?
    emitter.instruction("b.lo __rt_addcslashes_range_skip");                    // below the range -> keep scanning
    emitter.instruction("cmp w12, w15");                                        // source byte at or below the range end?
    emitter.instruction("b.ls __rt_addcslashes_escape");                        // within the range -> escape the byte
    emitter.label("__rt_addcslashes_range_skip");
    emitter.instruction("add x13, x13, #4");                                    // skip the four range bytes and continue scanning
    emitter.instruction("b __rt_addcslashes_mem");                              // test the next character-set entry
    emitter.label("__rt_addcslashes_single");
    emitter.instruction("ldrb w14, [x3, x13]");                                 // load the single character-set byte
    emitter.label("__rt_addcslashes_single_c");
    emitter.instruction("cmp w12, w14");                                        // does the source byte match this set byte?
    emitter.instruction("b.eq __rt_addcslashes_escape");                        // exact match -> escape the byte
    emitter.instruction("add x13, x13, #1");                                    // advance to the next character-set byte
    emitter.instruction("b __rt_addcslashes_mem");                              // keep scanning the character set

    emitter.label("__rt_addcslashes_copy");
    emitter.instruction("strb w12, [x9], #1");                                  // copy a non-member byte unchanged
    emitter.instruction("b __rt_addcslashes_loop");                             // continue with the next source byte

    // -- escape a member byte --
    emitter.label("__rt_addcslashes_escape");
    emitter.instruction("cmp w12, #32");                                        // is the byte a non-printable control byte?
    emitter.instruction("b.lo __rt_addcslashes_ctl");                           // bytes below space use C escapes or octal
    emitter.instruction("cmp w12, #126");                                       // is the byte above the printable ASCII range?
    emitter.instruction("b.hi __rt_addcslashes_ctl");                           // bytes above '~' use octal escapes
    emitter.instruction("mov w14, #92");                                        // backslash escape prefix
    emitter.instruction("strb w14, [x9], #1");                                  // write the backslash prefix
    emitter.instruction("strb w12, [x9], #1");                                  // write the printable member byte
    emitter.instruction("b __rt_addcslashes_loop");                             // continue with the next source byte

    emitter.label("__rt_addcslashes_ctl");
    emitter.instruction("cmp w12, #9");                                         // horizontal tab?
    emitter.instruction("b.eq __rt_addcslashes_t");                             // emit '\\t'
    emitter.instruction("cmp w12, #10");                                        // line feed?
    emitter.instruction("b.eq __rt_addcslashes_n");                             // emit '\\n'
    emitter.instruction("cmp w12, #11");                                        // vertical tab?
    emitter.instruction("b.eq __rt_addcslashes_v");                             // emit '\\v'
    emitter.instruction("cmp w12, #12");                                        // form feed?
    emitter.instruction("b.eq __rt_addcslashes_f");                             // emit '\\f'
    emitter.instruction("cmp w12, #13");                                        // carriage return?
    emitter.instruction("b.eq __rt_addcslashes_r");                             // emit '\\r'
    emitter.instruction("cmp w12, #7");                                         // alert/bell?
    emitter.instruction("b.eq __rt_addcslashes_a");                             // emit '\\a'
    emitter.instruction("cmp w12, #8");                                         // backspace?
    emitter.instruction("b.eq __rt_addcslashes_b");                             // emit '\\b'

    // -- octal escape: backslash followed by three octal digits --
    emitter.instruction("mov w14, #92");                                        // backslash escape prefix
    emitter.instruction("strb w14, [x9], #1");                                  // write the backslash prefix
    emitter.instruction("lsr w15, w12, #6");                                    // high octal digit = (byte >> 6)
    emitter.instruction("and w15, w15, #7");                                    // mask to three bits
    emitter.instruction("add w15, w15, #48");                                   // convert to an ASCII digit
    emitter.instruction("strb w15, [x9], #1");                                  // write the high octal digit
    emitter.instruction("lsr w15, w12, #3");                                    // middle octal digit = (byte >> 3)
    emitter.instruction("and w15, w15, #7");                                    // mask to three bits
    emitter.instruction("add w15, w15, #48");                                   // convert to an ASCII digit
    emitter.instruction("strb w15, [x9], #1");                                  // write the middle octal digit
    emitter.instruction("and w15, w12, #7");                                    // low octal digit = (byte & 7)
    emitter.instruction("add w15, w15, #48");                                   // convert to an ASCII digit
    emitter.instruction("strb w15, [x9], #1");                                  // write the low octal digit
    emitter.instruction("b __rt_addcslashes_loop");                             // continue with the next source byte

    emitter.label("__rt_addcslashes_t");
    emitter.instruction("mov w15, #116");                                       // escape letter 't'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_n");
    emitter.instruction("mov w15, #110");                                       // escape letter 'n'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_v");
    emitter.instruction("mov w15, #118");                                       // escape letter 'v'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_f");
    emitter.instruction("mov w15, #102");                                       // escape letter 'f'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_r");
    emitter.instruction("mov w15, #114");                                       // escape letter 'r'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_a");
    emitter.instruction("mov w15, #97");                                        // escape letter 'a'
    emitter.instruction("b __rt_addcslashes_bs_char");                          // emit '\\' + letter
    emitter.label("__rt_addcslashes_b");
    emitter.instruction("mov w15, #98");                                        // escape letter 'b'
    emitter.label("__rt_addcslashes_bs_char");
    emitter.instruction("mov w14, #92");                                        // backslash escape prefix
    emitter.instruction("strb w14, [x9], #1");                                  // write the backslash prefix
    emitter.instruction("strb w15, [x9], #1");                                  // write the escape letter
    emitter.instruction("b __rt_addcslashes_loop");                             // continue with the next source byte

    emitter.label("__rt_addcslashes_done");
    emitter.instruction("mov x1, x10");                                         // result pointer = saved start
    emitter.instruction("sub x2, x9, x10");                                     // result length = write pointer - start
    emitter.instruction("ldr x8, [x6]");                                        // reload the concat-buffer offset
    emitter.instruction("add x8, x8, x2");                                      // advance the offset by the result length
    emitter.instruction("str x8, [x6]");                                        // publish the updated concat-buffer offset
    emitter.instruction("ret");                                                 // return the escaped string slice
}

/// Emits the x86_64 Linux implementation of `__rt_addcslashes`.
///
/// Same escaping semantics as the ARM64 variant using System V registers:
/// - Input: `rdi` = source ptr, `rsi` = source len, `rdx` = characters ptr, `rcx` = characters len
/// - Output: `rax` = result ptr, `rdx` = result len
/// - `rdx`/`rcx` hold the characters set during the loop and are only repurposed at the end.
fn emit_addcslashes_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: addcslashes ---");
    emitter.label_global("__rt_addcslashes");

    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);              // load the current concat-buffer offset
    abi::emit_symbol_address(emitter, "r9", "_concat_buf");                     // materialize the concat-buffer base pointer
    emitter.instruction("add r9, r8");                                          // destination write pointer = base + offset
    emitter.instruction("mov r10, r9");                                         // remember the result start pointer

    emitter.label("__rt_addcslashes_loop_x86_64");
    emitter.instruction("test rsi, rsi");                                       // stop once every source byte is consumed
    emitter.instruction("je __rt_addcslashes_done_x86_64");                     // finish when no source bytes remain
    emitter.instruction("movzx r8d, BYTE PTR [rdi]");                           // load the current source byte
    emitter.instruction("add rdi, 1");                                          // advance the source pointer
    emitter.instruction("sub rsi, 1");                                          // account for the consumed source byte

    // -- membership scan: is the byte listed in the characters set? --
    emitter.instruction("xor r11d, r11d");                                      // character-set scan index = 0
    emitter.label("__rt_addcslashes_mem_x86_64");
    emitter.instruction("cmp r11, rcx");                                        // scanned the whole character set?
    emitter.instruction("jae __rt_addcslashes_copy_x86_64");                    // byte not in the set -> copy it unchanged
    emitter.instruction("mov rax, r11");                                        // copy the scan index to test for range room
    emitter.instruction("add rax, 3");                                          // index of the range's closing byte
    emitter.instruction("cmp rax, rcx");                                        // is there room for an `a..z` range here?
    emitter.instruction("jae __rt_addcslashes_single_x86_64");                  // too few bytes left -> single-byte membership test
    emitter.instruction("movzx eax, BYTE PTR [rdx + r11 + 1]");                 // load the byte after the range start
    emitter.instruction("cmp al, 46");                                          // is it a '.' range separator?
    emitter.instruction("jne __rt_addcslashes_single_x86_64");                  // not a range -> single-byte test on the start byte
    emitter.instruction("movzx eax, BYTE PTR [rdx + r11 + 2]");                 // load the second potential separator byte
    emitter.instruction("cmp al, 46");                                          // is it the second '.' of the range?
    emitter.instruction("jne __rt_addcslashes_single_x86_64");                  // not a range -> single-byte test on the start byte
    emitter.instruction("movzx eax, BYTE PTR [rdx + r11 + 3]");                 // load the range end byte
    emitter.instruction("cmp al, BYTE PTR [rdx + r11]");                        // is the range ascending (end >= start)?
    emitter.instruction("jb __rt_addcslashes_single_x86_64");                   // descending ranges degrade to a literal start byte
    emitter.instruction("cmp r8b, BYTE PTR [rdx + r11]");                       // source byte below the range start?
    emitter.instruction("jb __rt_addcslashes_range_skip_x86_64");               // below the range -> keep scanning
    emitter.instruction("cmp r8b, al");                                         // source byte at or below the range end?
    emitter.instruction("jbe __rt_addcslashes_escape_x86_64");                  // within the range -> escape the byte
    emitter.label("__rt_addcslashes_range_skip_x86_64");
    emitter.instruction("add r11, 4");                                          // skip the four range bytes and continue scanning
    emitter.instruction("jmp __rt_addcslashes_mem_x86_64");                     // test the next character-set entry
    emitter.label("__rt_addcslashes_single_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [rdx + r11]");                     // load the single character-set byte
    emitter.instruction("cmp r8b, al");                                         // does the source byte match this set byte?
    emitter.instruction("je __rt_addcslashes_escape_x86_64");                   // exact match -> escape the byte
    emitter.instruction("add r11, 1");                                          // advance to the next character-set byte
    emitter.instruction("jmp __rt_addcslashes_mem_x86_64");                     // keep scanning the character set

    emitter.label("__rt_addcslashes_copy_x86_64");
    emitter.instruction("mov BYTE PTR [r9], r8b");                              // copy a non-member byte unchanged
    emitter.instruction("add r9, 1");                                           // advance the output write pointer
    emitter.instruction("jmp __rt_addcslashes_loop_x86_64");                    // continue with the next source byte

    // -- escape a member byte --
    emitter.label("__rt_addcslashes_escape_x86_64");
    emitter.instruction("cmp r8b, 32");                                         // is the byte a non-printable control byte?
    emitter.instruction("jb __rt_addcslashes_ctl_x86_64");                      // bytes below space use C escapes or octal
    emitter.instruction("cmp r8b, 126");                                        // is the byte above the printable ASCII range?
    emitter.instruction("ja __rt_addcslashes_ctl_x86_64");                      // bytes above '~' use octal escapes
    emitter.instruction("mov BYTE PTR [r9], 92");                               // write the backslash prefix
    emitter.instruction("mov BYTE PTR [r9 + 1], r8b");                          // write the printable member byte
    emitter.instruction("add r9, 2");                                           // advance past the two-byte escape
    emitter.instruction("jmp __rt_addcslashes_loop_x86_64");                    // continue with the next source byte

    emitter.label("__rt_addcslashes_ctl_x86_64");
    emitter.instruction("cmp r8b, 9");                                          // horizontal tab?
    emitter.instruction("je __rt_addcslashes_t_x86_64");                        // emit '\\t'
    emitter.instruction("cmp r8b, 10");                                         // line feed?
    emitter.instruction("je __rt_addcslashes_n_x86_64");                        // emit '\\n'
    emitter.instruction("cmp r8b, 11");                                         // vertical tab?
    emitter.instruction("je __rt_addcslashes_v_x86_64");                        // emit '\\v'
    emitter.instruction("cmp r8b, 12");                                         // form feed?
    emitter.instruction("je __rt_addcslashes_f_x86_64");                        // emit '\\f'
    emitter.instruction("cmp r8b, 13");                                         // carriage return?
    emitter.instruction("je __rt_addcslashes_r_x86_64");                        // emit '\\r'
    emitter.instruction("cmp r8b, 7");                                          // alert/bell?
    emitter.instruction("je __rt_addcslashes_a_x86_64");                        // emit '\\a'
    emitter.instruction("cmp r8b, 8");                                          // backspace?
    emitter.instruction("je __rt_addcslashes_b_x86_64");                        // emit '\\b'

    // -- octal escape: backslash followed by three octal digits --
    emitter.instruction("mov BYTE PTR [r9], 92");                               // write the backslash prefix
    emitter.instruction("mov eax, r8d");                                        // copy the byte for the high octal digit
    emitter.instruction("shr eax, 6");                                          // high octal digit = (byte >> 6)
    emitter.instruction("and eax, 7");                                          // mask to three bits
    emitter.instruction("add eax, 48");                                         // convert to an ASCII digit
    emitter.instruction("mov BYTE PTR [r9 + 1], al");                           // write the high octal digit
    emitter.instruction("mov eax, r8d");                                        // copy the byte for the middle octal digit
    emitter.instruction("shr eax, 3");                                          // middle octal digit = (byte >> 3)
    emitter.instruction("and eax, 7");                                          // mask to three bits
    emitter.instruction("add eax, 48");                                         // convert to an ASCII digit
    emitter.instruction("mov BYTE PTR [r9 + 2], al");                           // write the middle octal digit
    emitter.instruction("mov eax, r8d");                                        // copy the byte for the low octal digit
    emitter.instruction("and eax, 7");                                          // low octal digit = (byte & 7)
    emitter.instruction("add eax, 48");                                         // convert to an ASCII digit
    emitter.instruction("mov BYTE PTR [r9 + 3], al");                           // write the low octal digit
    emitter.instruction("add r9, 4");                                           // advance past the four-byte octal escape
    emitter.instruction("jmp __rt_addcslashes_loop_x86_64");                    // continue with the next source byte

    emitter.label("__rt_addcslashes_t_x86_64");
    emitter.instruction("mov eax, 116");                                        // escape letter 't'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_n_x86_64");
    emitter.instruction("mov eax, 110");                                        // escape letter 'n'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_v_x86_64");
    emitter.instruction("mov eax, 118");                                        // escape letter 'v'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_f_x86_64");
    emitter.instruction("mov eax, 102");                                        // escape letter 'f'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_r_x86_64");
    emitter.instruction("mov eax, 114");                                        // escape letter 'r'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_a_x86_64");
    emitter.instruction("mov eax, 97");                                         // escape letter 'a'
    emitter.instruction("jmp __rt_addcslashes_bs_char_x86_64");                 // emit '\\' + letter
    emitter.label("__rt_addcslashes_b_x86_64");
    emitter.instruction("mov eax, 98");                                         // escape letter 'b'
    emitter.label("__rt_addcslashes_bs_char_x86_64");
    emitter.instruction("mov BYTE PTR [r9], 92");                               // write the backslash prefix
    emitter.instruction("mov BYTE PTR [r9 + 1], al");                           // write the escape letter
    emitter.instruction("add r9, 2");                                           // advance past the two-byte escape
    emitter.instruction("jmp __rt_addcslashes_loop_x86_64");                    // continue with the next source byte

    emitter.label("__rt_addcslashes_done_x86_64");
    emitter.instruction("mov rax, r10");                                        // result pointer = saved start
    emitter.instruction("mov rdx, r9");                                         // snapshot the final write pointer
    emitter.instruction("sub rdx, r10");                                        // result length = write pointer - start
    abi::emit_load_symbol_to_reg(emitter, "r8", "_concat_off", 0);             // reload the concat-buffer offset
    emitter.instruction("add r8, rdx");                                         // advance the offset by the result length
    abi::emit_store_reg_to_symbol(emitter, "r8", "_concat_off", 0);            // publish the updated concat-buffer offset
    emitter.instruction("ret");                                                 // return the escaped string slice
}

