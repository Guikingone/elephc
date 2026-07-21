//! Purpose:
//! Emits the `__rt_strcspn` runtime helper assembly for strcspn.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_strcspn` runtime helper for PHP `strcspn()`.
///
/// Returns the length of the initial segment of `string` that does not contain
/// any byte present in `characters`.
///
/// Register contract (ARM64):
/// - Input: x1 = subject ptr, x2 = subject len, x3 = chars ptr, x4 = chars len
/// - Output: x0 = span length
///
/// Register contract (x86_64 System V):
/// - Input: rdi = subject ptr, rsi = subject len, rdx = chars ptr, rcx = chars len
/// - Output: rax = span length
pub fn emit_strcspn(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_strcspn_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: strcspn ---");
    emitter.label_global("__rt_strcspn");

    emitter.instruction("mov x0, #0");                                          // span length accumulator = 0
    emitter.label("__rt_strcspn_outer");
    emitter.instruction("cmp x0, x2");                                          // have we scanned the whole subject string?
    emitter.instruction("b.ge __rt_strcspn_done");                              // yes -> the entire subject is part of the span
    emitter.instruction("ldrb w5, [x1, x0]");                                   // load the current subject byte
    emitter.instruction("mov x6, #0");                                          // character-set index = 0
    emitter.label("__rt_strcspn_inner");
    emitter.instruction("cmp x6, x4");                                          // have we compared against every character-set byte?
    emitter.instruction("b.ge __rt_strcspn_next");                              // subject byte is not in the set -> it extends the span
    emitter.instruction("ldrb w7, [x3, x6]");                                   // load the current character-set byte
    emitter.instruction("cmp w5, w7");                                          // does the subject byte match this set byte?
    emitter.instruction("b.eq __rt_strcspn_done");                              // match -> the byte is in the set, span ends here
    emitter.instruction("add x6, x6, #1");                                      // advance to the next character-set byte
    emitter.instruction("b __rt_strcspn_inner");                                // keep scanning the character set
    emitter.label("__rt_strcspn_next");
    emitter.instruction("add x0, x0, #1");                                      // extend the span by one subject byte
    emitter.instruction("b __rt_strcspn_outer");                                // test the next subject byte
    emitter.label("__rt_strcspn_done");
    emitter.instruction("ret");                                                 // return the span length in x0
}

/// Emits the x86_64 Linux implementation of `__rt_strcspn`.
///
/// Input: rdi = subject ptr, rsi = subject len, rdx = chars ptr, rcx = chars len.
/// Output: rax = span length.
fn emit_strcspn_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: strcspn ---");
    emitter.label_global("__rt_strcspn");

    emitter.instruction("xor eax, eax");                                        // span length accumulator = 0
    emitter.label("__rt_strcspn_outer_x86_64");
    emitter.instruction("cmp rax, rsi");                                        // have we scanned the whole subject string?
    emitter.instruction("jge __rt_strcspn_done_x86_64");                        // yes -> the entire subject is part of the span
    emitter.instruction("movzx r8d, BYTE PTR [rdi + rax]");                     // load the current subject byte
    emitter.instruction("xor r9d, r9d");                                        // character-set index = 0
    emitter.label("__rt_strcspn_inner_x86_64");
    emitter.instruction("cmp r9, rcx");                                         // have we compared against every character-set byte?
    emitter.instruction("jge __rt_strcspn_next_x86_64");                        // subject byte is not in the set -> it extends the span
    emitter.instruction("movzx r10d, BYTE PTR [rdx + r9]");                     // load the current character-set byte
    emitter.instruction("cmp r8b, r10b");                                       // does the subject byte match this set byte?
    emitter.instruction("je __rt_strcspn_done_x86_64");                         // match -> the byte is in the set, span ends here
    emitter.instruction("add r9, 1");                                           // advance to the next character-set byte
    emitter.instruction("jmp __rt_strcspn_inner_x86_64");                       // keep scanning the character set
    emitter.label("__rt_strcspn_next_x86_64");
    emitter.instruction("add rax, 1");                                          // extend the span by one subject byte
    emitter.instruction("jmp __rt_strcspn_outer_x86_64");                       // test the next subject byte
    emitter.label("__rt_strcspn_done_x86_64");
    emitter.instruction("ret");                                                 // return the span length in rax
}
