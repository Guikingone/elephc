//! Purpose:
//! Emits the `__rt_strpbrk` runtime helper assembly for strpbrk.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_strpbrk` runtime helper for PHP `strpbrk()`.
///
/// Returns the suffix of `string` beginning at the first byte that occurs in
/// `characters`, or a null pointer when no character matches (PHP `false`).
///
/// Register contract (ARM64):
/// - Input: x1 = subject ptr, x2 = subject len, x3 = chars ptr, x4 = chars len
/// - Output: x1 = result ptr (0 if none), x2 = result len
///
/// Register contract (x86_64 System V):
/// - Input: rdi = subject ptr, rsi = subject len, rdx = chars ptr, rcx = chars len
/// - Output: rax = result ptr (0 if none), rdx = result len
pub fn emit_strpbrk(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_strpbrk_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: strpbrk ---");
    emitter.label_global("__rt_strpbrk");

    emitter.instruction("mov x5, #0");                                          // subject scan index = 0
    emitter.label("__rt_strpbrk_outer");
    emitter.instruction("cmp x5, x2");                                          // have we scanned the whole subject?
    emitter.instruction("b.ge __rt_strpbrk_miss");                              // no characters byte was found -> return false
    emitter.instruction("ldrb w6, [x1, x5]");                                   // load the current subject byte
    emitter.instruction("mov x7, #0");                                          // character-set index = 0
    emitter.label("__rt_strpbrk_inner");
    emitter.instruction("cmp x7, x4");                                          // have we compared against every character-set byte?
    emitter.instruction("b.ge __rt_strpbrk_nextc");                             // subject byte not in the set -> try the next subject byte
    emitter.instruction("ldrb w8, [x3, x7]");                                   // load the current character-set byte
    emitter.instruction("cmp w6, w8");                                          // does the subject byte match this set byte?
    emitter.instruction("b.eq __rt_strpbrk_found");                             // match -> return the suffix starting here
    emitter.instruction("add x7, x7, #1");                                      // advance to the next character-set byte
    emitter.instruction("b __rt_strpbrk_inner");                                // keep scanning the character set
    emitter.label("__rt_strpbrk_nextc");
    emitter.instruction("add x5, x5, #1");                                      // advance to the next subject byte
    emitter.instruction("b __rt_strpbrk_outer");                                // test the next subject byte
    emitter.label("__rt_strpbrk_found");
    emitter.instruction("add x1, x1, x5");                                      // result pointer = subject ptr + match index
    emitter.instruction("sub x2, x2, x5");                                      // result length = subject len - match index
    emitter.instruction("ret");                                                 // return the matching suffix in x1/x2
    emitter.label("__rt_strpbrk_miss");
    emitter.instruction("mov x1, #0");                                          // null pointer signals PHP false (no match)
    emitter.instruction("mov x2, #0");                                          // zero length for the not-found result
    emitter.instruction("ret");                                                 // return the not-found sentinel to the caller
}

/// Emits the x86_64 Linux implementation of `__rt_strpbrk`.
///
/// Input: rdi = subject ptr, rsi = subject len, rdx = chars ptr, rcx = chars len.
/// Output: rax = result ptr (0 if none), rdx = result len.
fn emit_strpbrk_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: strpbrk ---");
    emitter.label_global("__rt_strpbrk");

    emitter.instruction("xor r8d, r8d");                                        // subject scan index = 0
    emitter.label("__rt_strpbrk_outer_x86_64");
    emitter.instruction("cmp r8, rsi");                                         // have we scanned the whole subject?
    emitter.instruction("jge __rt_strpbrk_miss_x86_64");                        // no characters byte was found -> return false
    emitter.instruction("movzx r9d, BYTE PTR [rdi + r8]");                      // load the current subject byte
    emitter.instruction("xor r10d, r10d");                                      // character-set index = 0
    emitter.label("__rt_strpbrk_inner_x86_64");
    emitter.instruction("cmp r10, rcx");                                        // have we compared against every character-set byte?
    emitter.instruction("jge __rt_strpbrk_nextc_x86_64");                       // subject byte not in the set -> try the next subject byte
    emitter.instruction("movzx r11d, BYTE PTR [rdx + r10]");                    // load the current character-set byte
    emitter.instruction("cmp r9b, r11b");                                       // does the subject byte match this set byte?
    emitter.instruction("je __rt_strpbrk_found_x86_64");                        // match -> return the suffix starting here
    emitter.instruction("add r10, 1");                                          // advance to the next character-set byte
    emitter.instruction("jmp __rt_strpbrk_inner_x86_64");                       // keep scanning the character set
    emitter.label("__rt_strpbrk_nextc_x86_64");
    emitter.instruction("add r8, 1");                                           // advance to the next subject byte
    emitter.instruction("jmp __rt_strpbrk_outer_x86_64");                       // test the next subject byte
    emitter.label("__rt_strpbrk_found_x86_64");
    emitter.instruction("mov rax, rdi");                                        // seed the result pointer from the subject pointer
    emitter.instruction("add rax, r8");                                         // result pointer = subject ptr + match index
    emitter.instruction("mov rdx, rsi");                                        // seed the result length from the subject length
    emitter.instruction("sub rdx, r8");                                         // result length = subject len - match index
    emitter.instruction("ret");                                                 // return the matching suffix in rax/rdx
    emitter.label("__rt_strpbrk_miss_x86_64");
    emitter.instruction("xor eax, eax");                                        // null pointer signals PHP false (no match)
    emitter.instruction("xor edx, edx");                                        // zero length for the not-found result
    emitter.instruction("ret");                                                 // return the not-found sentinel to the caller
}
