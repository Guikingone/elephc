//! Purpose:
//! Emits the `__rt_substr_count` runtime helper assembly for counting non-overlapping
//! substring occurrences.  Implements the 2-argument form of PHP's `substr_count()`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - AArch64 input: x1=hay_ptr, x2=hay_len, x3=needle_ptr, x4=needle_len; output: x0=count.
//! - x86_64 input:  rdi=hay_ptr, rsi=hay_len, rdx=needle_ptr, rcx=needle_len; output: rax=count.
//! - An empty needle returns 0 (PHP behavior: warns and returns false for empty needle, but
//!   the AOT type-checker should reject empty-string literals before reaching this point).
//! - After each match the search position advances by needle_len (non-overlapping).

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_substr_count` runtime helper for counting non-overlapping occurrences.
///
/// Scans `haystack` for every non-overlapping occurrence of `needle` and returns the count.
/// After a match the search cursor advances by `needle_len`, so overlapping occurrences are
/// not counted (matching PHP semantics).
///
/// # Input registers
/// - AArch64: x1 = haystack pointer, x2 = haystack length, x3 = needle pointer, x4 = needle length
/// - x86_64:  rdi = haystack pointer, rsi = haystack length, rdx = needle pointer, rcx = needle length
///
/// # Output registers
/// - AArch64: x0 = occurrence count
/// - x86_64:  rax = occurrence count
pub fn emit_substr_count(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_substr_count_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: substr_count ---");
    emitter.label_global("__rt_substr_count");

    // -- initialize count and validate inputs --
    emitter.instruction("mov x0, #0");                                            // initialize the occurrence counter to zero
    emitter.instruction("cbz x4, __rt_substr_count_done");                        // an empty needle matches nothing; return 0
    emitter.instruction("cmp x4, x2");                                            // needle longer than haystack cannot match
    emitter.instruction("b.gt __rt_substr_count_done");                           // return 0 when needle exceeds haystack length
    emitter.instruction("mov x5, #0");                                            // initialize search cursor to position 0

    // -- outer loop: advance cursor and try to match needle --
    emitter.label("__rt_substr_count_outer");
    emitter.instruction("sub x9, x2, x4");                                        // last valid start = haystack_len - needle_len
    emitter.instruction("cmp x5, x9");                                            // check whether cursor exceeds last valid start
    emitter.instruction("b.gt __rt_substr_count_done");                           // no room for another match; return count

    // -- inner loop: compare needle bytes at the current cursor position --
    emitter.instruction("mov x6, #0");                                            // reset needle comparison index to 0
    emitter.label("__rt_substr_count_inner");
    emitter.instruction("cmp x6, x4");                                            // check whether all needle bytes have matched
    emitter.instruction("b.ge __rt_substr_count_match");                          // all bytes matched; record the occurrence
    emitter.instruction("add x7, x5, x6");                                        // compute haystack byte index = cursor + needle_idx
    emitter.instruction("ldrb w8, [x1, x7]");                                     // load haystack byte at the computed index
    emitter.instruction("ldrb w9, [x3, x6]");                                     // load needle byte at the current comparison index
    emitter.instruction("cmp w8, w9");                                            // compare the two bytes
    emitter.instruction("b.ne __rt_substr_count_next");                           // mismatch: advance cursor and retry
    emitter.instruction("add x6, x6, #1");                                        // advance needle comparison index
    emitter.instruction("b __rt_substr_count_inner");                             // continue comparing remaining needle bytes

    // -- match recorded: advance cursor past the matched needle --
    emitter.label("__rt_substr_count_match");
    emitter.instruction("add x0, x0, #1");                                        // increment occurrence counter
    emitter.instruction("add x5, x5, x4");                                        // advance cursor past the full needle length
    emitter.instruction("b __rt_substr_count_outer");                             // search for the next occurrence

    // -- no match at this position: advance cursor by one byte --
    emitter.label("__rt_substr_count_next");
    emitter.instruction("add x5, x5, #1");                                        // step cursor forward by one byte
    emitter.instruction("b __rt_substr_count_outer");                             // retry from the new cursor position

    emitter.label("__rt_substr_count_done");
    emitter.instruction("ret");                                                   // return occurrence count in x0 to caller
}

/// Emits the x86_64 Linux `__rt_substr_count` runtime helper.
///
/// Identical semantics to the ARM64 path but uses the SysV AMD64 calling convention:
/// rdi=haystack_ptr, rsi=haystack_len, rdx=needle_ptr, rcx=needle_len, result in rax.
fn emit_substr_count_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: substr_count ---");
    emitter.label_global("__rt_substr_count");

    // -- initialize count and validate inputs --
    emitter.instruction("xor eax, eax");                                          // initialize the occurrence counter to zero
    emitter.instruction("test rcx, rcx");                                         // check for an empty needle
    emitter.instruction("jz __rt_substr_count_done_x86_64");                      // an empty needle matches nothing; return 0
    emitter.instruction("cmp rcx, rsi");                                          // needle longer than haystack cannot match
    emitter.instruction("jg __rt_substr_count_done_x86_64");                      // return 0 when needle exceeds haystack length
    emitter.instruction("xor r10, r10");                                          // initialize search cursor to position 0

    // -- outer loop: advance cursor and try to match needle --
    emitter.label("__rt_substr_count_outer_x86_64");
    emitter.instruction("mov r11, rsi");                                          // last valid start = haystack_len - needle_len
    emitter.instruction("sub r11, rcx");                                          // compute last valid cursor position
    emitter.instruction("cmp r10, r11");                                          // check whether cursor exceeds last valid start
    emitter.instruction("jg __rt_substr_count_done_x86_64");                      // no room for another match; return count

    // -- inner loop: compare needle bytes at the current cursor position --
    emitter.instruction("xor r8, r8");                                            // reset needle comparison index to 0
    emitter.label("__rt_substr_count_inner_x86_64");
    emitter.instruction("cmp r8, rcx");                                           // check whether all needle bytes have matched
    emitter.instruction("jge __rt_substr_count_match_x86_64");                    // all bytes matched; record the occurrence
    emitter.instruction("mov r9, r10");                                           // start computing haystack byte index
    emitter.instruction("add r9, r8");                                            // add needle comparison index to cursor
    emitter.instruction("movzx r9, BYTE PTR [rdi + r9]");                         // load haystack byte without sign-extension
    emitter.instruction("movzx r12d, BYTE PTR [rdx + r8]");                       // load needle byte without sign-extension
    emitter.instruction("cmp r9b, r12b");                                         // compare haystack byte and needle byte
    emitter.instruction("jne __rt_substr_count_next_x86_64");                     // mismatch: advance cursor and retry
    emitter.instruction("inc r8");                                                // advance needle comparison index
    emitter.instruction("jmp __rt_substr_count_inner_x86_64");                    // continue comparing remaining needle bytes

    // -- match recorded: advance cursor past the matched needle --
    emitter.label("__rt_substr_count_match_x86_64");
    emitter.instruction("inc eax");                                               // increment occurrence counter
    emitter.instruction("add r10, rcx");                                          // advance cursor past the full needle length
    emitter.instruction("jmp __rt_substr_count_outer_x86_64");                    // search for the next occurrence

    // -- no match at this position: advance cursor by one byte --
    emitter.label("__rt_substr_count_next_x86_64");
    emitter.instruction("inc r10");                                               // step cursor forward by one byte
    emitter.instruction("jmp __rt_substr_count_outer_x86_64");                    // retry from the new cursor position

    emitter.label("__rt_substr_count_done_x86_64");
    emitter.instruction("ret");                                                   // return occurrence count in rax to caller
}
