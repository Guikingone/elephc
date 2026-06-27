//! Purpose:
//! Emits runtime helpers for checked integer add/sub/mul with PHP overflow-to-float promotion.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::arrays`.
//!
//! Key details:
//! - Helpers take two raw I64 values and return a boxed Mixed cell (int or float).
//! - On overflow, the result is promoted to double to match PHP semantics.
//! - These helpers are used for non-constant integer arithmetic where the type
//!   checker cannot prove the result fits in int at compile time.

use crate::codegen::emit::Emitter;
use crate::codegen::{abi, platform::Arch};

/// Emits the checked integer add/sub/mul helpers for both AArch64 and x86_64.
///
/// Input (AArch64):  x0 = left I64, x1 = right I64
/// Input (x86_64):   rdi = left I64, rsi = right I64
/// Output: boxed Mixed pointer in the integer result register (x0 / rax)
pub fn emit_int_checked_binops(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_int_checked_binops_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: int_checked_binops ---");

    emit_aarch64_checked(emitter, "__rt_int_add_checked", 0);
    emit_aarch64_checked(emitter, "__rt_int_sub_checked", 1);
    emit_aarch64_checked(emitter, "__rt_int_mul_checked", 2);

    emitter.label("__rt_int_checked_common");
    emitter.instruction("str x0, [sp, #0]");                                    // save the left integer operand across the right operand load
    emitter.instruction("str x1, [sp, #8]");                                    // save the right integer operand for the arithmetic path
    emitter.instruction("str x9, [sp, #16]");                                   // save the selected arithmetic opcode for overflow dispatch

    // -- integer path with PHP overflow promotion --
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the left integer operand into x1
    emitter.instruction("ldr x2, [sp, #8]");                                     // reload the right integer operand into x2
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the selected arithmetic opcode
    emitter.instruction("cmp x9, #1");                                          // is this helper handling subtraction?
    emitter.instruction("b.eq __rt_int_checked_sub_op");                         // branch to the subtraction overflow sequence
    emitter.instruction("cmp x9, #2");                                          // is this helper handling multiplication?
    emitter.instruction("b.eq __rt_int_checked_mul_op");                         // branch to the multiplication overflow sequence

    emitter.label("__rt_int_checked_add_op");
    emitter.instruction("adds x0, x1, x2");                                     // compute integer addition and set overflow flags
    emitter.instruction("b.vs __rt_int_checked_overflow");                      // promote to double when signed addition overflowed
    emitter.instruction("b __rt_int_checked_box_int");                           // box the in-range integer result

    emitter.label("__rt_int_checked_sub_op");
    emitter.instruction("subs x0, x1, x2");                                     // compute integer subtraction and set overflow flags
    emitter.instruction("b.vs __rt_int_checked_overflow");                      // promote to double when signed subtraction overflowed
    emitter.instruction("b __rt_int_checked_box_int");                           // box the in-range integer result

    emitter.label("__rt_int_checked_mul_op");
    emitter.instruction("mul x0, x1, x2");                                       // compute the low half of the signed integer product
    emitter.instruction("smulh x3, x1, x2");                                     // compute the high half needed for overflow detection
    emitter.instruction("cmp x3, x0, asr #63");                                 // high half must equal the sign extension of the low half
    emitter.instruction("b.ne __rt_int_checked_overflow");                      // promote to double when signed multiplication overflowed

    emitter.label("__rt_int_checked_box_int");
    emitter.instruction("mov x1, x0");                                           // move the integer result into the Mixed helper payload register
    emitter.instruction("mov x2, xzr");                                          // integer payloads do not use a high word
    emitter.instruction("mov x0, #0");                                           // runtime tag 0 = integer
    emitter.instruction("bl __rt_mixed_from_value");                            // box the integer result into a Mixed cell
    emitter.instruction("b __rt_int_checked_done");                              // restore the helper frame and return the boxed result

    emitter.label("__rt_int_checked_overflow");
    emitter.instruction("scvtf d0, x1");                                         // convert the original left integer to double for PHP overflow promotion
    emitter.instruction("scvtf d1, x2");                                         // convert the original right integer to double for PHP overflow promotion
    emitter.instruction("ldr x9, [sp, #16]");                                   // reload the selected arithmetic opcode for the double fallback
    emitter.instruction("cmp x9, #1");                                          // is this overflow fallback for subtraction?
    emitter.instruction("b.eq __rt_int_checked_float_sub");                      // use floating-point subtraction for an overflowing integer subtraction
    emitter.instruction("cmp x9, #2");                                          // is this overflow fallback for multiplication?
    emitter.instruction("b.eq __rt_int_checked_float_mul");                      // use floating-point multiplication for an overflowing integer multiplication

    emitter.label("__rt_int_checked_float_add");
    emitter.instruction("fadd d0, d0, d1");                                      // compute the double addition result
    emitter.instruction("b __rt_int_checked_box_float");                         // box the double result

    emitter.label("__rt_int_checked_float_sub");
    emitter.instruction("fsub d0, d0, d1");                                      // compute the double subtraction result
    emitter.instruction("b __rt_int_checked_box_float");                         // box the double result

    emitter.label("__rt_int_checked_float_mul");
    emitter.instruction("fmul d0, d0, d1");                                      // compute the double multiplication result

    emitter.label("__rt_int_checked_box_float");
    emitter.instruction("fmov x1, d0");                                          // move the double bits into the Mixed helper payload register
    emitter.instruction("mov x2, xzr");                                          // double payloads do not use a high word
    emitter.instruction("mov x0, #2");                                           // runtime tag 2 = double
    emitter.instruction("bl __rt_mixed_from_value");                            // box the double result into a Mixed cell

    emitter.label("__rt_int_checked_done");
    emitter.instruction("ldp x29, x30, [sp, #32]");                              // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                      // release the helper stack frame
    emitter.instruction("ret");                                                  // return to generated code with boxed Mixed result in x0
}

/// Emits one AArch64 checked integer helper entry point.
///
/// Allocates a 48-byte helper frame, saves FP/LR, loads the opcode into x9,
/// and branches to the shared `__rt_int_checked_common` implementation.
fn emit_aarch64_checked(emitter: &mut Emitter, label: &str, opcode: i64) {
    emitter.label_global(label);
    emitter.instruction("sub sp, sp, #48");                                       // allocate a helper frame for operands and saved FP state
    emitter.instruction("stp x29, x30, [sp, #32]");                              // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                     // establish a stable helper frame pointer
    abi::emit_load_int_immediate(emitter, "x9", opcode);
    emitter.instruction("b __rt_int_checked_common");                           // enter the shared checked integer implementation
}

/// Emits the Linux x86_64 checked integer add/sub/mul helpers.
fn emit_int_checked_binops_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: int_checked_binops ---");

    emit_x86_64_checked_entry(emitter, "__rt_int_add_checked", 0);
    emit_x86_64_checked_entry(emitter, "__rt_int_sub_checked", 1);
    emit_x86_64_checked_entry(emitter, "__rt_int_mul_checked", 2);

    emitter.label("__rt_int_checked_common_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                         // save the left integer operand
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                        // save the right integer operand
    emitter.instruction("mov QWORD PTR [rbp - 24], r10");                       // save the selected arithmetic opcode

    // -- integer path with PHP overflow promotion --
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                         // reload the left integer operand
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                        // reload the right integer operand
    emitter.instruction("mov r8, r10");                                          // preserve the original left integer for overflow promotion
    emitter.instruction("mov r9, r11");                                          // preserve the original right integer for overflow promotion
    emitter.instruction("cmp QWORD PTR [rbp - 24], 1");                         // is this helper handling subtraction?
    emitter.instruction("je __rt_int_checked_sub_x86_64");                       // branch to the subtraction overflow sequence
    emitter.instruction("cmp QWORD PTR [rbp - 24], 2");                         // is this helper handling multiplication?
    emitter.instruction("je __rt_int_checked_mul_x86_64");                      // branch to the multiplication overflow sequence

    emitter.label("__rt_int_checked_add_x86_64");
    emitter.instruction("add r10, r11");                                         // compute integer addition and set overflow flags
    emitter.instruction("jo __rt_int_checked_overflow_x86_64");                  // promote to double when signed addition overflowed
    emitter.instruction("jmp __rt_int_checked_box_int_x86_64");                  // box the in-range integer result

    emitter.label("__rt_int_checked_sub_x86_64");
    emitter.instruction("sub r10, r11");                                         // compute integer subtraction and set overflow flags
    emitter.instruction("jo __rt_int_checked_overflow_x86_64");                  // promote to double when signed subtraction overflowed
    emitter.instruction("jmp __rt_int_checked_box_int_x86_64");                  // box the in-range integer result

    emitter.label("__rt_int_checked_mul_x86_64");
    emitter.instruction("mov rax, r10");                                         // move the left operand into rax for one-operand signed multiply
    emitter.instruction("imul r11");                                            // compute signed multiplication and set overflow flags
    emitter.instruction("jo __rt_int_checked_overflow_x86_64");                  // promote to double when signed multiplication overflowed
    emitter.instruction("mov r10, rax");                                         // keep the in-range product in the integer result scratch

    emitter.label("__rt_int_checked_box_int_x86_64");
    emitter.instruction("mov rdi, r10");                                         // move the integer result into the Mixed helper payload register
    emitter.instruction("xor rsi, rsi");                                        // integer payloads do not use a high word
    emitter.instruction("mov rax, 0");                                           // runtime tag 0 = integer
    emitter.instruction("call __rt_mixed_from_value");                          // box the integer result into a Mixed cell
    emitter.instruction("jmp __rt_int_checked_done_x86_64");                     // restore the helper frame and return the boxed result

    emitter.label("__rt_int_checked_overflow_x86_64");
    emitter.instruction("cvtsi2sd xmm0, r8");                                    // convert the original left integer to double for PHP overflow promotion
    emitter.instruction("cvtsi2sd xmm1, r9");                                   // convert the original right integer to double for PHP overflow promotion
    emitter.instruction("cmp QWORD PTR [rbp - 24], 1");                         // is this overflow fallback for subtraction?
    emitter.instruction("je __rt_int_checked_float_sub_x86_64");                // use floating-point subtraction for an overflowing integer subtraction
    emitter.instruction("cmp QWORD PTR [rbp - 24], 2");                         // is this overflow fallback for multiplication?
    emitter.instruction("je __rt_int_checked_float_mul_x86_64");                 // use floating-point multiplication for an overflowing integer multiplication
    emitter.instruction("jmp __rt_int_checked_float_add_x86_64");               // use floating-point addition for an overflowing integer addition

    emitter.label("__rt_int_checked_float_add_x86_64");
    emitter.instruction("addsd xmm0, xmm1");                                     // compute the double addition result
    emitter.instruction("jmp __rt_int_checked_box_float_x86_64");                // box the double result

    emitter.label("__rt_int_checked_float_sub_x86_64");
    emitter.instruction("subsd xmm0, xmm1");                                     // compute the double subtraction result
    emitter.instruction("jmp __rt_int_checked_box_float_x86_64");                // box the double result

    emitter.label("__rt_int_checked_float_mul_x86_64");
    emitter.instruction("mulsd xmm0, xmm1");                                     // compute the double multiplication result

    emitter.label("__rt_int_checked_box_float_x86_64");
    emitter.instruction("movq rdi, xmm0");                                      // move the double bits into the Mixed helper payload register
    emitter.instruction("xor rsi, rsi");                                        // double payloads do not use a high word
    emitter.instruction("mov rax, 2");                                          // runtime tag 2 = double
    emitter.instruction("call __rt_mixed_from_value");                          // box the double result into a Mixed cell

    emitter.label("__rt_int_checked_done_x86_64");
    emitter.instruction("add rsp, 48");                                          // release the helper stack frame
    emitter.instruction("pop rbp");                                              // restore the caller frame pointer
    emitter.instruction("ret");                                                  // return to generated code with boxed Mixed result in rax
}

/// Emits one x86_64 checked integer helper entry point.
fn emit_x86_64_checked_entry(emitter: &mut Emitter, label: &str, opcode: i64) {
    emitter.label_global(label);
    emitter.instruction("push rbp");                                             // save the caller frame pointer before nested runtime calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame pointer
    emitter.instruction("sub rsp, 48");                                          // allocate aligned helper slots for operands and saved FP state
    abi::emit_load_int_immediate(emitter, "r10", opcode);
    emitter.instruction("jmp __rt_int_checked_common_x86_64");                   // enter the shared checked integer implementation
}