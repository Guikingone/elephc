//! Purpose:
//! Emits `__rt_mixed_numeric_div`: PHP `/` on two boxed Mixed operands, returning an int box
//! when both operands are int-like and the division is exact, and a float box otherwise.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::MixedNumericBinop` with the `div` operation (`crate::codegen::lower_inst::arithmetic`).
//!
//! Key details:
//! - Int, bool and null payloads divide as ints through `__rt_int_div_checked` (`10 / 2` is
//!   `int(5)`, `7 / 2` is `float(3.5)`); any other payload divides as a float.
//! - A zero divisor returns a null pointer: the caller throws `DivisionByZeroError` in its own
//!   frame so the exception unwinds through the generated code's handlers.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_mixed_numeric_div`.
/// Input:  x0 = left Mixed*, x1 = right Mixed* (x86_64: rax, rdi)
/// Output: owned boxed Mixed (int or float) in x0 / rax, or 0 for a zero divisor
pub fn emit_mixed_numeric_div(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_numeric_div_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: mixed_numeric_div ---");
    emitter.label_global("__rt_mixed_numeric_div");
    emitter.instruction("sub sp, sp, #48");                                     // slots: left box, right box, left tag / left value
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the helper frame
    emitter.instruction("str x0, [sp, #0]");                                    // keep the left box
    emitter.instruction("str x1, [sp, #8]");                                    // keep the right box
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = left runtime tag
    emitter.instruction("str x0, [sp, #16]");                                   // keep the left tag
    emitter.instruction("ldr x0, [sp, #8]");                                    // right box
    emitter.instruction("bl __rt_mixed_unbox");                                 // x0 = right runtime tag
    emitter.instruction("ldr x9, [sp, #16]");                                   // left tag
    for (reg, next) in [("x9", "1"), ("x0", "2")] {
        emitter.instruction(&format!("cmp {}, #0", reg));                       // int payload?
        emitter.instruction(&format!("b.eq {}f", next));                        // int-like
        emitter.instruction(&format!("cmp {}, #3", reg));                       // bool payload?
        emitter.instruction(&format!("b.eq {}f", next));                        // int-like
        emitter.instruction(&format!("cmp {}, #8", reg));                       // null payload?
        emitter.instruction("b.ne 5f");                                         // anything else divides as a float
        emitter.label(next);
    }
    emitter.instruction("ldr x0, [sp, #0]");                                    // left box
    emitter.instruction("bl __rt_mixed_cast_int");                              // x0 = left int
    emitter.instruction("str x0, [sp, #16]");                                   // keep it across the right cast
    emitter.instruction("ldr x0, [sp, #8]");                                    // right box
    emitter.instruction("bl __rt_mixed_cast_int");                              // x0 = right int
    emitter.instruction("cbz x0, 8f");                                          // zero divisor: the caller throws
    emitter.instruction("mov x1, x0");                                          // divisor
    emitter.instruction("ldr x0, [sp, #16]");                                   // dividend
    emitter.instruction("bl __rt_int_div_checked");                             // boxed exact int or float quotient
    emitter.instruction("b 9f");                                                // return it
    emitter.label("5");
    emitter.instruction("ldr x0, [sp, #0]");                                    // left box
    emitter.instruction("bl __rt_mixed_cast_float");                            // d0 = left float
    emitter.instruction("str d0, [sp, #16]");                                   // keep it across the right cast
    emitter.instruction("ldr x0, [sp, #8]");                                    // right box
    emitter.instruction("bl __rt_mixed_cast_float");                            // d0 = right float
    emitter.instruction("fcmp d0, #0.0");                                       // zero divisor?
    emitter.instruction("b.eq 8f");                                             // the caller throws
    emitter.instruction("fmov d1, d0");                                         // divisor
    emitter.instruction("ldr d0, [sp, #16]");                                   // dividend
    emitter.instruction("fdiv d0, d0, d1");                                     // floating-point quotient
    emitter.instruction("fmov x1, d0");                                         // double bits as the payload
    emitter.instruction("mov x2, xzr");                                         // double payloads have no high word
    emitter.instruction("mov x0, #2");                                          // runtime tag 2 = float
    emitter.instruction("bl __rt_mixed_box_scalar");                            // box the float quotient
    emitter.instruction("b 9f");                                                // return it
    emitter.label("8");
    emitter.instruction("mov x0, #0");                                          // no box: division by zero
    emitter.label("9");
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return the boxed quotient
}

/// Emits the x86_64 variant of `__rt_mixed_numeric_div` (rax, rdi → rax).
fn emit_mixed_numeric_div_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_numeric_div ---");
    emitter.label_global("__rt_mixed_numeric_div");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 32");                                         // slots: left box, right box, left tag / left value
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // keep the left box
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // keep the right box
    emitter.instruction("call __rt_mixed_unbox");                               // rax = left runtime tag
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // keep the left tag
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // right box
    emitter.instruction("call __rt_mixed_unbox");                               // rax = right runtime tag
    emitter.instruction("mov rcx, QWORD PTR [rbp - 24]");                       // left tag
    for (reg, next) in [("rcx", "1"), ("rax", "2")] {
        emitter.instruction(&format!("cmp {}, 0", reg));                        // int payload?
        emitter.instruction(&format!("je {}f", next));                          // int-like
        emitter.instruction(&format!("cmp {}, 3", reg));                        // bool payload?
        emitter.instruction(&format!("je {}f", next));                          // int-like
        emitter.instruction(&format!("cmp {}, 8", reg));                        // null payload?
        emitter.instruction("jne 5f");                                          // anything else divides as a float
        emitter.label(next);
    }
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // left box
    emitter.instruction("mov rdi, rax");                                        // also in rdi for helpers taking it there
    emitter.instruction("call __rt_mixed_cast_int");                            // rax = left int
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // keep it across the right cast
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // right box
    emitter.instruction("mov rdi, rax");                                        // also in rdi for helpers taking it there
    emitter.instruction("call __rt_mixed_cast_int");                            // rax = right int
    emitter.instruction("test rax, rax");                                       // zero divisor?
    emitter.instruction("jz 8f");                                               // the caller throws
    emitter.instruction("mov rsi, rax");                                        // divisor
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // dividend
    emitter.instruction("call __rt_int_div_checked");                           // boxed exact int or float quotient
    emitter.instruction("jmp 9f");                                              // return it
    emitter.label("5");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // left box
    emitter.instruction("mov rdi, rax");                                        // also in rdi for helpers taking it there
    emitter.instruction("call __rt_mixed_cast_float");                          // xmm0 = left float
    emitter.instruction("movsd QWORD PTR [rbp - 24], xmm0");                    // keep it across the right cast
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // right box
    emitter.instruction("mov rdi, rax");                                        // also in rdi for helpers taking it there
    emitter.instruction("call __rt_mixed_cast_float");                          // xmm0 = right float
    emitter.instruction("xorpd xmm1, xmm1");                                    // 0.0
    emitter.instruction("ucomisd xmm0, xmm1");                                  // zero divisor?
    emitter.instruction("jp 3f");                                               // NaN is not zero
    emitter.instruction("je 8f");                                               // the caller throws
    emitter.label("3");
    emitter.instruction("movapd xmm1, xmm0");                                   // divisor
    emitter.instruction("movsd xmm0, QWORD PTR [rbp - 24]");                    // dividend
    emitter.instruction("divsd xmm0, xmm1");                                    // floating-point quotient
    emitter.instruction("movq rdi, xmm0");                                      // double bits as the payload
    emitter.instruction("xor esi, esi");                                        // double payloads have no high word
    emitter.instruction("mov eax, 2");                                          // runtime tag 2 = float
    emitter.instruction("call __rt_mixed_box_scalar");                          // box the float quotient
    emitter.instruction("jmp 9f");                                              // return it
    emitter.label("8");
    emitter.instruction("xor eax, eax");                                        // no box: division by zero
    emitter.label("9");
    emitter.instruction("leave");                                               // release the helper frame
    emitter.instruction("ret");                                                 // return the boxed quotient
}
