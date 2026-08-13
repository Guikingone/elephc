//! Purpose:
//! Emits the `__rt_strncasecmp` runtime helper assembly for the PHP `strncasecmp` builtin.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Mirrors php-src's `zend_binary_strncasecmp`: bytes are folded with the ASCII-only
//!   `zend_tolower_ascii` before comparison, exactly like `__rt_strcasecmp`, so bytes outside
//!   `A`-`Z` are never rewritten and no locale is consulted.
//! - The compared prefix and the equal-prefix tiebreak both use the TRUNCATED lengths
//!   `min($length, strlen($a))` / `min($length, strlen($b))`, matching `__rt_strncmp`.
//! - The result is the raw folded-byte difference, not a clamped `-1/0/1`.
//! - `$length` is validated as non-negative by the backend lowering, which raises PHP's
//!   catchable `ValueError`; the helper may therefore treat it as an unsigned bound.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_strncasecmp` runtime helper for length-limited case-insensitive comparison.
///
/// Register contract (AArch64):
/// - Input: `x1` = ptr_a, `x2` = len_a, `x3` = ptr_b, `x4` = len_b, `x5` = compare length
/// - Output: `x0` = result (`< 0` if a < b, `0` if equal, `> 0` if a > b)
///
/// Register contract (x86_64 System V):
/// - Input: `rdi` = ptr_a, `rsi` = len_a, `rdx` = ptr_b, `rcx` = len_b, `r8` = compare length
/// - Output: `rax` = result (`< 0` if a < b, `0` if equal, `> 0` if a > b)
pub fn emit_strncasecmp(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_strncasecmp_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: strncasecmp ---");
    emitter.label_global("__rt_strncasecmp");

    // -- truncate both operands to the requested comparison length --
    emitter.instruction("cmp x2, x5");                                          // compare the first string length against the requested bound
    emitter.instruction("csel x9, x2, x5, lo");                                 // x9 = min(len_a, length), the first effective length
    emitter.instruction("cmp x4, x5");                                          // compare the second string length against the requested bound
    emitter.instruction("csel x10, x4, x5, lo");                                // x10 = min(len_b, length), the second effective length
    emitter.instruction("cmp x9, x10");                                         // compare both effective lengths
    emitter.instruction("csel x11, x9, x10, lo");                               // x11 = shared prefix actually compared byte by byte
    emitter.instruction("mov x6, #0");                                          // start comparing at byte offset zero

    emitter.label("__rt_strncasecmp_loop");
    emitter.instruction("cmp x6, x11");                                         // has the shared prefix been fully compared?
    emitter.instruction("b.hs __rt_strncasecmp_len");                           // fall back to the effective-length tiebreak
    emitter.instruction("ldrb w7, [x1, x6]");                                   // load the current byte of the first string
    emitter.instruction("ldrb w8, [x3, x6]");                                   // load the current byte of the second string

    // -- ASCII-fold the first string byte --
    emitter.instruction("cmp w7, #65");                                         // is the first byte at or above 'A'?
    emitter.instruction("b.lt __rt_strncasecmp_b");                             // bytes below 'A' are compared unchanged
    emitter.instruction("cmp w7, #90");                                         // is the first byte at or below 'Z'?
    emitter.instruction("b.gt __rt_strncasecmp_b");                             // bytes above 'Z' are compared unchanged
    emitter.instruction("add w7, w7, #32");                                     // fold the uppercase ASCII letter to lowercase

    // -- ASCII-fold the second string byte --
    emitter.label("__rt_strncasecmp_b");
    emitter.instruction("cmp w8, #65");                                         // is the second byte at or above 'A'?
    emitter.instruction("b.lt __rt_strncasecmp_cmp");                           // bytes below 'A' are compared unchanged
    emitter.instruction("cmp w8, #90");                                         // is the second byte at or below 'Z'?
    emitter.instruction("b.gt __rt_strncasecmp_cmp");                           // bytes above 'Z' are compared unchanged
    emitter.instruction("add w8, w8, #32");                                     // fold the uppercase ASCII letter to lowercase

    emitter.label("__rt_strncasecmp_cmp");
    emitter.instruction("cmp w7, w8");                                          // compare the two folded bytes
    emitter.instruction("b.ne __rt_strncasecmp_diff");                          // report the byte difference on the first mismatch
    emitter.instruction("add x6, x6, #1");                                      // advance to the next shared-prefix byte
    emitter.instruction("b __rt_strncasecmp_loop");                             // keep comparing the shared prefix

    emitter.label("__rt_strncasecmp_diff");
    emitter.instruction("sub x0, x7, x8");                                      // return the signed folded-byte difference
    emitter.instruction("ret");                                                 // hand the byte difference back to the caller

    emitter.label("__rt_strncasecmp_len");
    emitter.instruction("sub x0, x9, x10");                                     // tiebreak on the TRUNCATED lengths, never the raw ones
    emitter.instruction("ret");                                                 // hand the length difference back to the caller
}

/// Emits the x86_64 Linux implementation of `__rt_strncasecmp`.
///
/// `rsi` and `rcx` are reused as the byte index and second byte scratch once both raw
/// lengths have been truncated into `r9`/`r10`, so the helper needs no callee-saved register.
fn emit_strncasecmp_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: strncasecmp ---");
    emitter.label_global("__rt_strncasecmp");

    // -- truncate both operands to the requested comparison length --
    emitter.instruction("mov r9, rsi");                                         // seed the first effective length from the first string length
    emitter.instruction("cmp r9, r8");                                          // compare the first string length against the requested bound
    emitter.instruction("cmova r9, r8");                                        // r9 = min(len_a, length), the first effective length
    emitter.instruction("mov r10, rcx");                                        // seed the second effective length from the second string length
    emitter.instruction("cmp r10, r8");                                         // compare the second string length against the requested bound
    emitter.instruction("cmova r10, r8");                                       // r10 = min(len_b, length), the second effective length
    emitter.instruction("mov r11, r9");                                         // seed the shared prefix bound from the first effective length
    emitter.instruction("cmp r11, r10");                                        // compare both effective lengths
    emitter.instruction("cmova r11, r10");                                      // r11 = shared prefix actually compared byte by byte
    emitter.instruction("xor esi, esi");                                        // start comparing at byte offset zero

    emitter.label("__rt_strncasecmp_loop_linux_x86_64");
    emitter.instruction("cmp rsi, r11");                                        // has the shared prefix been fully compared?
    emitter.instruction("jae __rt_strncasecmp_len_linux_x86_64");               // fall back to the effective-length tiebreak
    emitter.instruction("movzx rax, BYTE PTR [rdi + rsi]");                     // load the current byte of the first string
    emitter.instruction("movzx rcx, BYTE PTR [rdx + rsi]");                     // load the current byte of the second string

    // -- ASCII-fold the first string byte --
    emitter.instruction("cmp al, 65");                                          // is the first byte at or above 'A'?
    emitter.instruction("jb __rt_strncasecmp_second_linux_x86_64");             // bytes below 'A' are compared unchanged
    emitter.instruction("cmp al, 90");                                          // is the first byte at or below 'Z'?
    emitter.instruction("ja __rt_strncasecmp_second_linux_x86_64");             // bytes above 'Z' are compared unchanged
    emitter.instruction("add al, 32");                                          // fold the uppercase ASCII letter to lowercase

    // -- ASCII-fold the second string byte --
    emitter.label("__rt_strncasecmp_second_linux_x86_64");
    emitter.instruction("cmp cl, 65");                                          // is the second byte at or above 'A'?
    emitter.instruction("jb __rt_strncasecmp_cmp_linux_x86_64");                // bytes below 'A' are compared unchanged
    emitter.instruction("cmp cl, 90");                                          // is the second byte at or below 'Z'?
    emitter.instruction("ja __rt_strncasecmp_cmp_linux_x86_64");                // bytes above 'Z' are compared unchanged
    emitter.instruction("add cl, 32");                                          // fold the uppercase ASCII letter to lowercase

    emitter.label("__rt_strncasecmp_cmp_linux_x86_64");
    emitter.instruction("cmp rax, rcx");                                        // compare the two folded bytes
    emitter.instruction("jne __rt_strncasecmp_diff_linux_x86_64");              // report the byte difference on the first mismatch
    emitter.instruction("add rsi, 1");                                          // advance to the next shared-prefix byte
    emitter.instruction("jmp __rt_strncasecmp_loop_linux_x86_64");              // keep comparing the shared prefix

    emitter.label("__rt_strncasecmp_diff_linux_x86_64");
    emitter.instruction("sub rax, rcx");                                        // return the signed folded-byte difference
    emitter.instruction("ret");                                                 // hand the byte difference back to the caller

    emitter.label("__rt_strncasecmp_len_linux_x86_64");
    emitter.instruction("mov rax, r9");                                         // seed the tiebreak from the first effective length
    emitter.instruction("sub rax, r10");                                        // tiebreak on the TRUNCATED lengths, never the raw ones
    emitter.instruction("ret");                                                 // hand the length difference back to the caller
}
