//! Purpose:
//! Emits `__rt_int_div_checked`: PHP `/` on two ints, returning a boxed int when the division is
//! exact and a boxed float otherwise (`10 / 2` is `int(5)`, `7 / 2` is `float(3.5)`).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::ICheckedDiv` (`crate::codegen::lower_inst::arithmetic`).
//!
//! Key details:
//! - The caller has already thrown `DivisionByZeroError` for a zero divisor.
//! - `PHP_INT_MIN / -1` does not fit in an int, so it takes the float path (and never reaches
//!   the trapping `idiv` on x86_64).

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_int_div_checked`.
/// Input:  x0 = dividend, x1 = non-zero divisor (x86_64: rdi, rsi)
/// Output: owned boxed Mixed (int or float) in x0 / rax
pub fn emit_int_div_checked(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_int_div_checked_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: int_div_checked ---");
    emitter.label_global("__rt_int_div_checked");
    emitter.instruction("stp x29, x30, [sp, #-16]!");                           // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the helper frame
    emitter.instruction("cmn x1, #1");                                          // is the divisor -1?
    emitter.instruction("b.ne 1f");                                             // no: divide normally
    crate::codegen_support::abi::emit_load_int_immediate(emitter, "x9", i64::MIN);
    emitter.instruction("cmp x0, x9");                                          // PHP_INT_MIN / -1 overflows the int range
    emitter.instruction("b.eq 3f");                                             // promote that quotient to float
    emitter.label("1");
    emitter.instruction("sdiv x9, x0, x1");                                     // truncated quotient
    emitter.instruction("msub x10, x9, x1, x0");                                // remainder = dividend - quotient * divisor
    emitter.instruction("cbnz x10, 3f");                                        // inexact: the result is a float
    emitter.instruction("mov x1, x9");                                          // exact quotient as the int payload
    emitter.instruction("mov x2, xzr");                                         // int payloads have no high word
    emitter.instruction("mov x0, #0");                                          // runtime tag 0 = int
    emitter.instruction("bl __rt_mixed_box_scalar");                            // box the int quotient
    emitter.instruction("b 9f");                                                // return it
    emitter.label("3");
    emitter.instruction("scvtf d0, x0");                                        // dividend as double
    emitter.instruction("scvtf d1, x1");                                        // divisor as double
    emitter.instruction("fdiv d0, d0, d1");                                     // floating-point quotient
    emitter.instruction("fmov x1, d0");                                         // double bits as the payload
    emitter.instruction("mov x2, xzr");                                         // double payloads have no high word
    emitter.instruction("mov x0, #2");                                          // runtime tag 2 = float
    emitter.instruction("bl __rt_mixed_box_scalar");                            // box the float quotient
    emitter.label("9");
    emitter.instruction("ldp x29, x30, [sp], #16");                             // restore frame pointer and return address
    emitter.instruction("ret");                                                 // return the boxed quotient
}

/// Emits the x86_64 variant of `__rt_int_div_checked` (rdi, rsi → rax).
fn emit_int_div_checked_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: int_div_checked ---");
    emitter.label_global("__rt_int_div_checked");
    emitter.instruction("push rbp");                                            // keep the stack 16-byte aligned for the box call
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("cmp rsi, -1");                                         // is the divisor -1?
    emitter.instruction("jne 1f");                                              // no: divide normally
    emitter.instruction("mov rax, 0x8000000000000000");                         // PHP_INT_MIN
    emitter.instruction("cmp rdi, rax");                                        // PHP_INT_MIN / -1 overflows (and would trap in idiv)
    emitter.instruction("je 3f");                                               // promote that quotient to float
    emitter.label("1");
    emitter.instruction("mov rax, rdi");                                        // dividend for idiv
    emitter.instruction("cqo");                                                 // sign-extend into rdx:rax
    emitter.instruction("idiv rsi");                                            // rax = quotient, rdx = remainder
    emitter.instruction("test rdx, rdx");                                       // exact division?
    emitter.instruction("jnz 3f");                                              // inexact: the result is a float
    emitter.instruction("mov rdi, rax");                                        // exact quotient as the int payload
    emitter.instruction("xor esi, esi");                                        // int payloads have no high word
    emitter.instruction("xor eax, eax");                                        // runtime tag 0 = int
    emitter.instruction("call __rt_mixed_box_scalar");                          // box the int quotient
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boxed int
    emitter.label("3");
    emitter.instruction("cvtsi2sd xmm0, rdi");                                  // dividend as double
    emitter.instruction("cvtsi2sd xmm1, rsi");                                  // divisor as double
    emitter.instruction("divsd xmm0, xmm1");                                    // floating-point quotient
    emitter.instruction("movq rdi, xmm0");                                      // double bits as the payload
    emitter.instruction("xor esi, esi");                                        // double payloads have no high word
    emitter.instruction("mov eax, 2");                                          // runtime tag 2 = float
    emitter.instruction("call __rt_mixed_box_scalar");                          // box the float quotient
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boxed float
}
