//! Purpose:
//! Emits the `__rt_strrchr` runtime helper assembly for strrchr.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; the returned suffix aliases the haystack storage (no allocation).

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_strrchr` runtime helper for PHP `strrchr()`.
///
/// Returns the suffix of `haystack` beginning at the last occurrence of the first
/// byte of `needle`, or a null pointer when the byte never appears (PHP `false`).
/// An empty `haystack` or empty `needle` also yields the null sentinel. Only the
/// first byte of `needle` is used, matching PHP's multi-byte-needle behavior. The
/// result aliases the caller's haystack storage, so no allocation is performed.
///
/// Register contract (ARM64):
/// - Input: x1 = haystack ptr, x2 = haystack len, x3 = needle ptr, x4 = needle len
/// - Output: x1 = result ptr (0 if none), x2 = result len
///
/// Register contract (x86_64 System V):
/// - Input: rdi = haystack ptr, rsi = haystack len, rdx = needle ptr, rcx = needle len
/// - Output: rax = result ptr (0 if none), rdx = result len
pub fn emit_strrchr(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_strrchr_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: strrchr ---");
    emitter.label_global("__rt_strrchr");

    emitter.instruction("cbz x4, __rt_strrchr_miss");                           // an empty needle never matches -> PHP false
    emitter.instruction("cbz x2, __rt_strrchr_miss");                           // an empty haystack never matches -> PHP false
    emitter.instruction("ldrb w5, [x3]");                                       // load the search byte (first byte of needle)
    emitter.instruction("mov x6, x2");                                          // scan index starts one past the last byte

    emitter.label("__rt_strrchr_loop");
    emitter.instruction("sub x6, x6, #1");                                      // step back to the next byte to test
    emitter.instruction("ldrb w7, [x1, x6]");                                   // load the current haystack byte
    emitter.instruction("cmp w7, w5");                                          // does it equal the search byte?
    emitter.instruction("b.eq __rt_strrchr_found");                             // last occurrence found -> return the suffix
    emitter.instruction("cbnz x6, __rt_strrchr_loop");                          // keep scanning while bytes remain
    emitter.instruction("b __rt_strrchr_miss");                                 // index 0 reached with no match -> PHP false

    emitter.label("__rt_strrchr_found");
    emitter.instruction("add x1, x1, x6");                                      // result pointer = haystack ptr + match index
    emitter.instruction("sub x2, x2, x6");                                      // result length = haystack len - match index
    emitter.instruction("ret");                                                 // return the matching suffix in x1/x2

    emitter.label("__rt_strrchr_miss");
    emitter.instruction("mov x1, #0");                                          // null pointer signals PHP false (no match)
    emitter.instruction("mov x2, #0");                                          // zero length for the not-found result
    emitter.instruction("ret");                                                 // return the not-found sentinel to the caller
}

/// Emits the x86_64 Linux implementation of `__rt_strrchr`.
///
/// Input: rdi = haystack ptr, rsi = haystack len, rdx = needle ptr, rcx = needle len.
/// Output: rax = result ptr (0 if none), rdx = result len.
fn emit_strrchr_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: strrchr ---");
    emitter.label_global("__rt_strrchr");

    emitter.instruction("test rcx, rcx");                                       // an empty needle never matches -> PHP false
    emitter.instruction("jz __rt_strrchr_miss_x86_64");                         // jump to the miss sentinel for an empty needle
    emitter.instruction("test rsi, rsi");                                       // an empty haystack never matches -> PHP false
    emitter.instruction("jz __rt_strrchr_miss_x86_64");                         // jump to the miss sentinel for an empty haystack
    emitter.instruction("movzx r8d, BYTE PTR [rdx]");                           // load the search byte (first byte of needle)
    emitter.instruction("mov r9, rsi");                                         // scan index starts one past the last byte

    emitter.label("__rt_strrchr_loop_x86_64");
    emitter.instruction("sub r9, 1");                                           // step back to the next byte to test
    emitter.instruction("movzx r10d, BYTE PTR [rdi + r9]");                     // load the current haystack byte
    emitter.instruction("cmp r10b, r8b");                                       // does it equal the search byte?
    emitter.instruction("je __rt_strrchr_found_x86_64");                        // last occurrence found -> return the suffix
    emitter.instruction("test r9, r9");                                         // have we reached index zero?
    emitter.instruction("jnz __rt_strrchr_loop_x86_64");                        // keep scanning while bytes remain
    emitter.instruction("jmp __rt_strrchr_miss_x86_64");                        // index 0 reached with no match -> PHP false

    emitter.label("__rt_strrchr_found_x86_64");
    emitter.instruction("mov rax, rdi");                                        // seed the result pointer from the haystack pointer
    emitter.instruction("add rax, r9");                                         // result pointer = haystack ptr + match index
    emitter.instruction("mov rdx, rsi");                                        // seed the result length from the haystack length
    emitter.instruction("sub rdx, r9");                                         // result length = haystack len - match index
    emitter.instruction("ret");                                                 // return the matching suffix in rax/rdx

    emitter.label("__rt_strrchr_miss_x86_64");
    emitter.instruction("xor eax, eax");                                        // null pointer signals PHP false (no match)
    emitter.instruction("xor edx, edx");                                        // zero length for the not-found result
    emitter.instruction("ret");                                                 // return the not-found sentinel to the caller
}
