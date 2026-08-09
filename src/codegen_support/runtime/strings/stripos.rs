//! Purpose:
//! Emits the `__rt_stripos` runtime helper assembly: the case-insensitive twin of
//! `__rt_strpos`, scanning left to right for the first occurrence of a needle.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The ABI is byte-for-byte identical to `__rt_strpos`, so `lower_string_position` drives
//!   both spellings through the same `$offset` normalization, `ValueError` guard, and match
//!   rebase. Only the per-byte comparison differs.
//! - Folding is ASCII-only (`A`-`Z` -> `a`-`z`), matching php-src's locale-independent
//!   `zend_tolower_ascii`. That is why `stripos("Été", "é")` is 3 rather than 1: the two
//!   non-ASCII lead bytes are compared verbatim.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_stripos` runtime helper for AArch64.
///
/// ABI:
///   Input:  x1=haystack_ptr, x2=haystack_len, x3=needle_ptr, x4=needle_len
///   Output: x0 = byte offset of the first case-insensitive match, or -1 when absent
///
/// An empty needle matches at offset 0 and a needle longer than the haystack cannot match,
/// exactly as `__rt_strpos` decides those two edge cases.
/// Dispatches to `emit_stripos_linux_x86_64` on x86_64.
pub fn emit_stripos(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_stripos_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: stripos ---");
    emitter.label_global("__rt_stripos");

    // -- edge cases --
    emitter.instruction("cbz x4, __rt_stripos_empty");                          // empty needle always matches at position 0
    emitter.instruction("cmp x4, x2");                                          // compare needle length with haystack length
    emitter.instruction("b.gt __rt_stripos_notfound");                          // needle longer than haystack, can't match
    emitter.instruction("mov x5, #0");                                          // initialize search position to 0

    // -- outer loop: try matching needle at each position --
    emitter.label("__rt_stripos_outer");
    emitter.instruction("sub x9, x2, x4");                                      // last valid start = haystack_len - needle_len
    emitter.instruction("cmp x5, x9");                                          // check if position exceeds last valid start
    emitter.instruction("b.gt __rt_stripos_notfound");                          // past end, needle not found

    // -- inner loop: compare needle bytes at current position, case-insensitively --
    emitter.instruction("mov x6, #0");                                          // needle comparison index = 0
    emitter.label("__rt_stripos_inner");
    emitter.instruction("cmp x6, x4");                                          // check if all needle bytes matched
    emitter.instruction("b.ge __rt_stripos_found");                             // all matched, found at position x5
    emitter.instruction("add x7, x5, x6");                                      // compute haystack index = pos + needle_idx
    emitter.instruction("ldrb w8, [x1, x7]");                                   // load haystack byte at computed index
    emitter.instruction("ldrb w10, [x3, x6]");                                  // load needle byte at current index

    // -- fold the haystack byte to lowercase, ASCII range only --
    emitter.instruction("cmp w8, #65");                                         // is the haystack byte at or above 'A'?
    emitter.instruction("b.lt __rt_stripos_fold_needle");                       // bytes below 'A' are compared verbatim
    emitter.instruction("cmp w8, #90");                                         // is the haystack byte at or below 'Z'?
    emitter.instruction("b.gt __rt_stripos_fold_needle");                       // bytes above 'Z' are compared verbatim
    emitter.instruction("add w8, w8, #32");                                     // fold the uppercase haystack byte to lowercase

    // -- fold the needle byte to lowercase, ASCII range only --
    emitter.label("__rt_stripos_fold_needle");
    emitter.instruction("cmp w10, #65");                                        // is the needle byte at or above 'A'?
    emitter.instruction("b.lt __rt_stripos_cmp");                               // bytes below 'A' are compared verbatim
    emitter.instruction("cmp w10, #90");                                        // is the needle byte at or below 'Z'?
    emitter.instruction("b.gt __rt_stripos_cmp");                               // bytes above 'Z' are compared verbatim
    emitter.instruction("add w10, w10, #32");                                   // fold the uppercase needle byte to lowercase

    emitter.label("__rt_stripos_cmp");
    emitter.instruction("cmp w8, w10");                                         // compare the folded haystack and needle bytes
    emitter.instruction("b.ne __rt_stripos_next");                              // mismatch, try next position
    emitter.instruction("add x6, x6, #1");                                      // advance needle index
    emitter.instruction("b __rt_stripos_inner");                                // continue comparing

    // -- advance to next haystack position --
    emitter.label("__rt_stripos_next");
    emitter.instruction("add x5, x5, #1");                                      // increment search position
    emitter.instruction("b __rt_stripos_outer");                                // retry from new position

    // -- return results --
    emitter.label("__rt_stripos_found");
    emitter.instruction("mov x0, x5");                                          // return match position
    emitter.instruction("ret");                                                 // return to caller
    emitter.label("__rt_stripos_empty");
    emitter.instruction("mov x0, #0");                                          // empty needle found at position 0
    emitter.instruction("ret");                                                 // return to caller
    emitter.label("__rt_stripos_notfound");
    emitter.instruction("mov x0, #-1");                                         // return -1 (not found)
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the `__rt_stripos` runtime helper for Linux x86_64.
///
/// ABI:
///   Input:  rdi=haystack_ptr, rsi=haystack_len, rdx=needle_ptr, rcx=needle_len
///   Output: rax = byte offset of the first case-insensitive match, or -1 when absent
///
/// `rsi` doubles as the needle-byte scratch register once the last valid start offset has
/// been derived from it, the same reuse `__rt_strpos` makes.
/// Called exclusively from `emit_stripos` when `emitter.target.arch == Arch::X86_64`.
fn emit_stripos_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stripos ---");
    emitter.label_global("__rt_stripos");

    emitter.instruction("test rcx, rcx");                                       // empty needles match immediately at offset zero
    emitter.instruction("jz __rt_stripos_empty_linux_x86_64");                  // return zero when stripos() receives an empty needle
    emitter.instruction("cmp rcx, rsi");                                        // reject searches whose needle is longer than the haystack
    emitter.instruction("jg __rt_stripos_notfound_linux_x86_64");               // return the not-found sentinel when the needle cannot fit
    emitter.instruction("mov r10, rsi");                                        // copy the haystack length so the final valid start offset can be computed once
    emitter.instruction("sub r10, rcx");                                        // compute the last haystack offset where the full needle can still fit
    emitter.instruction("mov r8, rdi");                                         // seed the current haystack candidate pointer at the start of the haystack
    emitter.instruction("xor r9d, r9d");                                        // start scanning from haystack offset zero

    emitter.label("__rt_stripos_outer_linux_x86_64");
    emitter.instruction("cmp r9, r10");                                         // have we advanced beyond the last valid haystack start offset?
    emitter.instruction("jg __rt_stripos_notfound_linux_x86_64");               // stop once there are no more candidate start offsets to test
    emitter.instruction("xor r11d, r11d");                                      // start the needle byte comparison from index zero for the current candidate

    emitter.label("__rt_stripos_inner_linux_x86_64");
    emitter.instruction("cmp r11, rcx");                                        // did every byte in the needle match at the current haystack offset?
    emitter.instruction("jge __rt_stripos_found_linux_x86_64");                 // return the current haystack offset once the full needle matches
    emitter.instruction("movzx eax, BYTE PTR [r8 + r11]");                      // load the current haystack byte for the candidate comparison
    emitter.instruction("movzx esi, BYTE PTR [rdx + r11]");                     // load the current needle byte for the candidate comparison
    emitter.instruction("cmp al, 65");                                          // is the haystack byte at or above 'A'?
    emitter.instruction("jb __rt_stripos_fold_needle_linux_x86_64");            // bytes below 'A' are compared verbatim
    emitter.instruction("cmp al, 90");                                          // is the haystack byte at or below 'Z'?
    emitter.instruction("ja __rt_stripos_fold_needle_linux_x86_64");            // bytes above 'Z' are compared verbatim
    emitter.instruction("add al, 32");                                          // fold the uppercase haystack byte to lowercase

    emitter.label("__rt_stripos_fold_needle_linux_x86_64");
    emitter.instruction("cmp sil, 65");                                         // is the needle byte at or above 'A'?
    emitter.instruction("jb __rt_stripos_cmp_linux_x86_64");                    // bytes below 'A' are compared verbatim
    emitter.instruction("cmp sil, 90");                                         // is the needle byte at or below 'Z'?
    emitter.instruction("ja __rt_stripos_cmp_linux_x86_64");                    // bytes above 'Z' are compared verbatim
    emitter.instruction("add sil, 32");                                         // fold the uppercase needle byte to lowercase

    emitter.label("__rt_stripos_cmp_linux_x86_64");
    emitter.instruction("cmp al, sil");                                         // compare the folded haystack and needle bytes
    emitter.instruction("jne __rt_stripos_next_linux_x86_64");                  // abandon this candidate start offset on the first mismatching byte
    emitter.instruction("add r11, 1");                                          // advance to the next byte within the current needle comparison
    emitter.instruction("jmp __rt_stripos_inner_linux_x86_64");                 // continue matching bytes against the current candidate start offset

    emitter.label("__rt_stripos_next_linux_x86_64");
    emitter.instruction("add r8, 1");                                           // advance the haystack candidate pointer to the next possible start offset
    emitter.instruction("add r9, 1");                                           // advance the logical haystack offset returned on a successful future match
    emitter.instruction("jmp __rt_stripos_outer_linux_x86_64");                 // retry the needle comparison from the next haystack start offset

    emitter.label("__rt_stripos_found_linux_x86_64");
    emitter.instruction("mov rax, r9");                                         // return the first haystack offset whose bytes matched the full needle
    emitter.instruction("ret");                                                 // return the signed match offset to the caller

    emitter.label("__rt_stripos_empty_linux_x86_64");
    emitter.instruction("xor eax, eax");                                        // empty needles match at offset zero
    emitter.instruction("ret");                                                 // return the empty-needle offset to the caller

    emitter.label("__rt_stripos_notfound_linux_x86_64");
    emitter.instruction("mov rax, -1");                                         // return the not-found sentinel when no haystack offset matches the needle
    emitter.instruction("ret");                                                 // return the not-found sentinel to the caller
}
