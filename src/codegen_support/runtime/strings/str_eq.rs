//! Purpose:
//! Emits the `__rt_str_eq`, `__rt_str_eq_false` runtime helper assembly for string equality comparison.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers scan or transform byte ranges and return target ABI pointer/length pairs for generated call sites.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_str_eq` runtime helper for byte-string equality.
/// Dispatches to the x86_64 Linux variant when targeting that architecture; otherwise
/// emits a portable ARM64 implementation.
/// ABI: ARM64 calling convention — x1=ptr_a, x2=len_a, x3=ptr_b, x4=len_b → x0=result (1=equal, 0=different).
pub fn emit_str_eq(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_eq_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_eq ---");
    emitter.label_global("__rt_str_eq");

    // -- lengths first, then overlapping word compares (never reading past either string) --
    emitter.instruction("cmp x2, x4");                                          // compare lengths of both strings
    emitter.instruction("b.ne __rt_str_eq_false");                              // if lengths differ, strings are not equal
    emitter.instruction("cmp x1, x3");                                          // the same bytes?
    emitter.instruction("b.eq __rt_str_eq_true");                               // identical pointers are equal
    emitter.instruction("cbz x2, __rt_str_eq_true");                            // two empty strings are equal
    emitter.instruction("cmp x2, #8");                                          // at least one full word?
    emitter.instruction("b.lo __rt_str_eq_small");                              // 1..7 bytes use 4/2/1-byte compares
    emitter.instruction("sub x2, x2, #8");                                      // bytes before the final word
    emitter.instruction("ldr x5, [x1, x2]");                                    // final (possibly overlapping) word of A
    emitter.instruction("ldr x6, [x3, x2]");                                    // final (possibly overlapping) word of B
    emitter.instruction("cmp x5, x6");                                          // compare the final words
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    emitter.label("__rt_str_eq_loop");
    emitter.instruction("cmp x2, #0");                                          // any leading words left?
    emitter.instruction("b.le __rt_str_eq_true");                               // all words matched
    emitter.instruction("ldr x5, [x1], #8");                                    // next word of A
    emitter.instruction("ldr x6, [x3], #8");                                    // next word of B
    emitter.instruction("cmp x5, x6");                                          // compare the words
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    emitter.instruction("sub x2, x2, #8");                                      // eight fewer leading bytes
    emitter.instruction("b __rt_str_eq_loop");                                  // continue with the next word
    emitter.label("__rt_str_eq_small");
    emitter.instruction("cmp x2, #4");                                          // at least four bytes?
    emitter.instruction("b.lo __rt_str_eq_tiny");                               // 1..3 bytes
    emitter.instruction("ldr w5, [x1]");                                        // first four bytes of A
    emitter.instruction("ldr w6, [x3]");                                        // first four bytes of B
    emitter.instruction("cmp w5, w6");                                          // compare them
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    emitter.instruction("sub x2, x2, #4");                                      // offset of the last four bytes
    emitter.instruction("ldr w5, [x1, x2]");                                    // last four bytes of A
    emitter.instruction("ldr w6, [x3, x2]");                                    // last four bytes of B
    emitter.instruction("cmp w5, w6");                                          // compare them
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    emitter.instruction("b __rt_str_eq_true");                                  // equal
    emitter.label("__rt_str_eq_tiny");
    emitter.instruction("ldrb w5, [x1]");                                       // first byte of A
    emitter.instruction("ldrb w6, [x3]");                                       // first byte of B
    emitter.instruction("cmp w5, w6");                                          // compare them
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    emitter.instruction("cmp x2, #1");                                          // a single byte?
    emitter.instruction("b.eq __rt_str_eq_true");                               // equal
    emitter.instruction("sub x2, x2, #2");                                      // offset of the last two bytes
    emitter.instruction("ldrh w5, [x1, x2]");                                   // last two bytes of A
    emitter.instruction("ldrh w6, [x3, x2]");                                   // last two bytes of B
    emitter.instruction("cmp w5, w6");                                          // compare them
    emitter.instruction("b.ne __rt_str_eq_false");                              // mismatch
    // -- strings are equal --
    emitter.label("__rt_str_eq_true");
    emitter.instruction("mov x0, #1");                                          // return 1 (true: strings are equal)
    emitter.instruction("ret");                                                 // return to caller

    // -- strings are not equal --
    emitter.label("__rt_str_eq_false");
    emitter.instruction("mov x0, #0");                                          // return 0 (false: strings differ)
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux variant of `__rt_str_eq`.
/// ABI: System V AMD64 — rdi=ptr_a, rsi=len_a, rdx=ptr_b, rcx=len_b → rax=result (1=equal, 0=different).
/// Compares lengths, then overlapping word/half-word loads that never read past either string.
fn emit_str_eq_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_eq ---");
    emitter.label_global("__rt_str_eq");

    emitter.instruction("cmp rsi, rcx");                                        // compare both string lengths
    emitter.instruction("jne __rt_str_eq_false");                               // unequal lengths cannot hold equal bytes
    emitter.instruction("cmp rdi, rdx");                                        // the same bytes?
    emitter.instruction("je __rt_str_eq_true");                                 // identical pointers are equal
    emitter.instruction("test rsi, rsi");                                       // two empty strings?
    emitter.instruction("je __rt_str_eq_true");                                 // are equal
    emitter.instruction("cmp rsi, 8");                                          // at least one full word?
    emitter.instruction("jb __rt_str_eq_small");                                // 1..7 bytes use 4/2/1-byte compares
    emitter.instruction("sub rsi, 8");                                          // bytes before the final word
    emitter.instruction("mov r8, QWORD PTR [rdi + rsi]");                       // final (possibly overlapping) word of the left string
    emitter.instruction("cmp r8, QWORD PTR [rdx + rsi]");                       // compare it with the right string's final word
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.label("__rt_str_eq_loop");
    emitter.instruction("test rsi, rsi");                                       // any leading words left?
    emitter.instruction("jle __rt_str_eq_true");                                // all words matched
    emitter.instruction("mov r8, QWORD PTR [rdi]");                             // next word of the left string
    emitter.instruction("cmp r8, QWORD PTR [rdx]");                             // compare it with the right string
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.instruction("add rdi, 8");                                          // advance the left string
    emitter.instruction("add rdx, 8");                                          // advance the right string
    emitter.instruction("sub rsi, 8");                                          // eight fewer leading bytes
    emitter.instruction("jmp __rt_str_eq_loop");                                // continue with the next word
    emitter.label("__rt_str_eq_small");
    emitter.instruction("cmp rsi, 4");                                          // at least four bytes?
    emitter.instruction("jb __rt_str_eq_tiny");                                 // 1..3 bytes
    emitter.instruction("mov r8d, DWORD PTR [rdi]");                            // first four bytes of the left string
    emitter.instruction("cmp r8d, DWORD PTR [rdx]");                            // compare them
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.instruction("mov r8d, DWORD PTR [rdi + rsi - 4]");                  // last four bytes of the left string
    emitter.instruction("cmp r8d, DWORD PTR [rdx + rsi - 4]");                  // compare them
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.instruction("jmp __rt_str_eq_true");                                // equal
    emitter.label("__rt_str_eq_tiny");
    emitter.instruction("movzx r8d, BYTE PTR [rdi]");                           // first byte of the left string
    emitter.instruction("cmp r8b, BYTE PTR [rdx]");                             // compare it
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.instruction("cmp rsi, 1");                                          // a single byte?
    emitter.instruction("je __rt_str_eq_true");                                 // equal
    emitter.instruction("movzx r8d, WORD PTR [rdi + rsi - 2]");                 // last two bytes of the left string
    emitter.instruction("cmp r8w, WORD PTR [rdx + rsi - 2]");                   // compare them
    emitter.instruction("jne __rt_str_eq_false");                               // mismatch
    emitter.label("__rt_str_eq_true");
    emitter.instruction("mov rax, 1");                                          // return 1 to signal that both strings are equal
    emitter.instruction("ret");                                                 // return to the caller after the equality fast path or completed byte loop

    emitter.label("__rt_str_eq_false");
    emitter.instruction("xor rax, rax");                                        // return 0 to signal that the strings differ
    emitter.instruction("ret");                                                 // return immediately after the mismatch or length check failure
}
