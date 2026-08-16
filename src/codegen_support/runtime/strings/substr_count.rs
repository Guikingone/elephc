//! Purpose:
//! Emits the `__rt_substr_count` runtime helper assembly for the PHP `substr_count` builtin.
//! Counts non-overlapping needle occurrences inside an already-sliced haystack window.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The helper receives the WINDOW, not the original subject: `substr_count()`'s `$offset`
//!   and `$length` are normalized (and their `ValueError`s raised) in the backend lowering,
//!   which then passes `haystack + offset` and the clamped length. That keeps the catchable
//!   diagnostics out of the runtime, where a fatal could not be caught.
//! - Matching is NON-OVERLAPPING, exactly like php-src: a hit advances the cursor by the full
//!   needle length, so `substr_count("aaaa", "aa")` is 2 rather than 3.
//! - The helper allocates nothing and calls nothing, so it needs no frame and no concat
//!   scratch reservation.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_substr_count` runtime helper for the `substr_count` builtin.
///
/// ABI (AArch64):
///   Input:  `x1` = window pointer, `x2` = window length, `x3` = needle pointer,
///           `x4` = needle length.
///   Output: `x0` = number of non-overlapping matches.
///
/// ABI (x86_64 System V):
///   Input:  `rdi` = window pointer, `rsi` = window length, `rdx` = needle pointer,
///           `rcx` = needle length.
///   Output: `rax` = number of non-overlapping matches.
///
/// An empty needle and a needle longer than the window both yield zero; the empty case
/// never reaches the helper because the lowering raises PHP's `ValueError` first, but the
/// guard keeps the loop from spinning if it ever did.
pub fn emit_substr_count(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_substr_count_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: substr_count ---");
    emitter.label_global("__rt_substr_count");

    emitter.instruction("mov x0, #0");                                          // start the match counter at zero
    emitter.instruction("cbz x4, __rt_substr_count_done");                      // an empty needle can never be counted
    emitter.instruction("cmp x4, x2");                                          // compare the needle length against the searchable window
    emitter.instruction("b.gt __rt_substr_count_done");                         // a needle longer than the window cannot match
    emitter.instruction("sub x10, x2, x4");                                     // compute the last window offset where the needle still fits
    emitter.instruction("mov x5, #0");                                          // start scanning at window offset zero

    emitter.label("__rt_substr_count_outer");
    emitter.instruction("cmp x5, x10");                                         // has the cursor passed the last candidate start offset?
    emitter.instruction("b.gt __rt_substr_count_done");                         // stop once no full needle can start here
    emitter.instruction("mov x6, #0");                                          // restart the needle comparison at byte zero

    emitter.label("__rt_substr_count_inner");
    emitter.instruction("cmp x6, x4");                                          // did every needle byte match at this candidate offset?
    emitter.instruction("b.hs __rt_substr_count_hit");                          // a complete needle match was found
    emitter.instruction("add x7, x5, x6");                                      // compute the window index of the byte under comparison
    emitter.instruction("ldrb w9, [x1, x7]");                                   // load the window byte at the candidate position
    emitter.instruction("ldrb w11, [x3, x6]");                                  // load the needle byte at the same relative position
    emitter.instruction("cmp w9, w11");                                         // compare the window and needle bytes
    emitter.instruction("b.ne __rt_substr_count_next");                         // abandon this candidate on the first mismatch
    emitter.instruction("add x6, x6, #1");                                      // advance to the next needle byte
    emitter.instruction("b __rt_substr_count_inner");                           // keep comparing the current candidate

    emitter.label("__rt_substr_count_hit");
    emitter.instruction("add x0, x0, #1");                                      // record one more non-overlapping match
    emitter.instruction("add x5, x5, x4");                                      // skip the whole matched needle so matches never overlap
    emitter.instruction("b __rt_substr_count_outer");                           // resume scanning after the counted match

    emitter.label("__rt_substr_count_next");
    emitter.instruction("add x5, x5, #1");                                      // slide the candidate start one byte forward
    emitter.instruction("b __rt_substr_count_outer");                           // retry the match at the next window offset

    emitter.label("__rt_substr_count_done");
    emitter.instruction("ret");                                                 // return the accumulated match count
}

/// Emits `__rt_substr_count` for x86_64 Linux using the System V ABI.
///
/// `rsi` is repurposed as the last valid start offset once the window length has been used,
/// which frees `r11` for the per-candidate window pointer and leaves `rax` as the only byte
/// scratch register the inner comparison needs.
fn emit_substr_count_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: substr_count ---");
    emitter.label_global("__rt_substr_count");

    emitter.instruction("xor r10d, r10d");                                      // start the match counter at zero
    emitter.instruction("test rcx, rcx");                                       // is the needle empty?
    emitter.instruction("jz __rt_substr_count_done_linux_x86_64");              // an empty needle can never be counted
    emitter.instruction("cmp rcx, rsi");                                        // compare the needle length against the searchable window
    emitter.instruction("jg __rt_substr_count_done_linux_x86_64");              // a needle longer than the window cannot match
    emitter.instruction("sub rsi, rcx");                                        // reuse the window length as the last valid start offset
    emitter.instruction("xor r8d, r8d");                                        // start scanning at window offset zero

    emitter.label("__rt_substr_count_outer_linux_x86_64");
    emitter.instruction("cmp r8, rsi");                                         // has the cursor passed the last candidate start offset?
    emitter.instruction("jg __rt_substr_count_done_linux_x86_64");              // stop once no full needle can start here
    emitter.instruction("lea r11, [rdi + r8]");                                 // point at the window bytes for the current candidate
    emitter.instruction("xor r9d, r9d");                                        // restart the needle comparison at byte zero

    emitter.label("__rt_substr_count_inner_linux_x86_64");
    emitter.instruction("cmp r9, rcx");                                         // did every needle byte match at this candidate offset?
    emitter.instruction("jae __rt_substr_count_hit_linux_x86_64");              // a complete needle match was found
    emitter.instruction("movzx eax, BYTE PTR [r11 + r9]");                      // load the window byte at the candidate position
    emitter.instruction("cmp al, BYTE PTR [rdx + r9]");                         // compare it against the needle byte at the same position
    emitter.instruction("jne __rt_substr_count_next_linux_x86_64");             // abandon this candidate on the first mismatch
    emitter.instruction("add r9, 1");                                           // advance to the next needle byte
    emitter.instruction("jmp __rt_substr_count_inner_linux_x86_64");            // keep comparing the current candidate

    emitter.label("__rt_substr_count_hit_linux_x86_64");
    emitter.instruction("add r10, 1");                                          // record one more non-overlapping match
    emitter.instruction("add r8, rcx");                                         // skip the whole matched needle so matches never overlap
    emitter.instruction("jmp __rt_substr_count_outer_linux_x86_64");            // resume scanning after the counted match

    emitter.label("__rt_substr_count_next_linux_x86_64");
    emitter.instruction("add r8, 1");                                           // slide the candidate start one byte forward
    emitter.instruction("jmp __rt_substr_count_outer_linux_x86_64");            // retry the match at the next window offset

    emitter.label("__rt_substr_count_done_linux_x86_64");
    emitter.instruction("mov rax, r10");                                        // return the accumulated match count
    emitter.instruction("ret");                                                 // hand the count back to the caller
}
