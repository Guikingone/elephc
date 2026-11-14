//! Purpose:
//! Emits `__rt_array_mixed_to_scalar`: converts an indexed array of boxed Mixed elements into a
//! fresh indexed array of raw int, float or bool payloads.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::ArrayMixedToScalar` (`crate::codegen::lower_inst::arrays`).
//!
//! Key details:
//! - Used at a call boundary when the checker specialized an `array` parameter to scalar
//!   elements while the caller built the argument with boxed elements (an untyped caller
//!   parameter keeps a `mixed` EIR contract).
//! - Each element goes through the regular `__rt_mixed_cast_int` / `_float` / `_bool` helper.
//! - Like the other container conversions it consumes its input (one reference released).

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_array_mixed_to_scalar`.
/// Input:  x0 = source array<mixed>, x1 = target value tag (0 int, 2 float, 3 bool)
///         (x86_64: rdi, rsi)
/// Output: x0 = new owned scalar array (x86_64: rax)
pub fn emit_array_mixed_to_scalar(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_mixed_to_scalar_x86_64(emitter);
        return;
    }
    emitter.blank();
    emitter.comment("--- runtime: array_mixed_to_scalar ---");
    emitter.label_global("__rt_array_mixed_to_scalar");
    emitter.instruction("sub sp, sp, #64");                                     // slots: source, tag, length, result, index
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // establish the helper frame
    emitter.instruction("str x0, [sp, #0]");                                    // keep the source array
    emitter.instruction("str x1, [sp, #8]");                                    // keep the target value tag
    emitter.instruction("ldr x0, [x0]");                                        // source length
    emitter.instruction("str x0, [sp, #16]");                                   // keep the length (also the capacity)
    emitter.instruction("mov x1, #8");                                          // scalar slots are one word
    emitter.instruction("bl __rt_array_new");                                   // x0 = fresh scalar array
    emitter.instruction("str x0, [sp, #24]");                                   // keep the result
    emitter.instruction("ldr x9, [sp, #8]");                                    // target value tag
    emitter.instruction("lsl x9, x9, #8");                                      // into the kind word's value_type lane
    emitter.instruction("ldr x10, [x0, #-8]");                                  // fresh kind word
    emitter.instruction("orr x10, x10, x9");                                    // stamp the element value_type
    emitter.instruction("str x10, [x0, #-8]");                                  // store the stamped kind word
    emitter.instruction("str xzr, [sp, #32]");                                  // index = 0
    emitter.label("1");
    emitter.instruction("ldr x9, [sp, #32]");                                   // index
    emitter.instruction("ldr x10, [sp, #16]");                                  // length
    emitter.instruction("cmp x9, x10");                                         // all elements converted?
    emitter.instruction("b.hs 8f");                                             // yes: finish
    emitter.instruction("ldr x11, [sp, #0]");                                   // source array
    emitter.instruction("add x11, x11, #24");                                   // source payload base
    emitter.instruction("ldr x0, [x11, x9, lsl #3]");                           // the boxed element
    emitter.instruction("ldr x12, [sp, #8]");                                   // target value tag
    emitter.instruction("cmp x12, #2");                                         // float target?
    emitter.instruction("b.eq 3f");                                             // convert to float
    emitter.instruction("cmp x12, #3");                                         // bool target?
    emitter.instruction("b.eq 4f");                                             // convert to bool
    emitter.instruction("bl __rt_mixed_cast_int");                              // x0 = int payload
    emitter.instruction("b 5f");                                                // store it
    emitter.label("3");
    emitter.instruction("bl __rt_mixed_cast_float");                            // d0 = float payload
    emitter.instruction("fmov x0, d0");                                         // float bits as the slot word
    emitter.instruction("b 5f");                                                // store it
    emitter.label("4");
    emitter.instruction("bl __rt_mixed_cast_bool");                             // x0 = bool payload
    emitter.label("5");
    emitter.instruction("ldr x9, [sp, #32]");                                   // index
    emitter.instruction("ldr x11, [sp, #24]");                                  // result array
    emitter.instruction("add x12, x11, #24");                                   // result payload base
    emitter.instruction("str x0, [x12, x9, lsl #3]");                           // store the converted element
    emitter.instruction("add x9, x9, #1");                                      // next index
    emitter.instruction("str x9, [sp, #32]");                                   // keep it
    emitter.instruction("str x9, [x11]");                                       // result length follows the copied prefix
    emitter.instruction("b 1b");                                                // continue
    emitter.label("8");
    emitter.instruction("ldr x0, [sp, #0]");                                    // source array
    emitter.instruction("bl __rt_decref_any");                                  // the conversion consumes its input
    emitter.instruction("ldr x0, [sp, #24]");                                   // return the scalar array
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the helper frame
    emitter.instruction("ret");                                                 // return
}

/// Emits the x86_64 variant of `__rt_array_mixed_to_scalar` (rdi, rsi → rax).
fn emit_array_mixed_to_scalar_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_mixed_to_scalar ---");
    emitter.label_global("__rt_array_mixed_to_scalar");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 48");                                         // slots: source, tag, length, result, index
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // keep the source array
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // keep the target value tag
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // source length
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // keep the length (also the capacity)
    emitter.instruction("mov rdi, rax");                                        // capacity
    emitter.instruction("mov esi, 8");                                          // scalar slots are one word
    emitter.instruction("call __rt_array_new");                                 // rax = fresh scalar array
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // keep the result
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // target value tag
    emitter.instruction("shl rcx, 8");                                          // into the kind word's value_type lane
    emitter.instruction("or QWORD PTR [rax - 8], rcx");                         // stamp the element value_type
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // index = 0
    emitter.label("1");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // index
    emitter.instruction("cmp rcx, QWORD PTR [rbp - 24]");                       // all elements converted?
    emitter.instruction("jae 8f");                                              // yes: finish
    emitter.instruction("mov rdx, QWORD PTR [rbp - 8]");                        // source array
    emitter.instruction("mov rax, QWORD PTR [rdx + rcx * 8 + 24]");             // the boxed element
    emitter.instruction("mov rdi, rax");                                        // also in rdi for helpers taking it there
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // target value tag
    emitter.instruction("cmp rdx, 2");                                          // float target?
    emitter.instruction("je 3f");                                               // convert to float
    emitter.instruction("cmp rdx, 3");                                          // bool target?
    emitter.instruction("je 4f");                                               // convert to bool
    emitter.instruction("call __rt_mixed_cast_int");                            // rax = int payload
    emitter.instruction("jmp 5f");                                              // store it
    emitter.label("3");
    emitter.instruction("call __rt_mixed_cast_float");                          // xmm0 = float payload
    emitter.instruction("movq rax, xmm0");                                      // float bits as the slot word
    emitter.instruction("jmp 5f");                                              // store it
    emitter.label("4");
    emitter.instruction("call __rt_mixed_cast_bool");                           // rax = bool payload
    emitter.label("5");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // index
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // result array
    emitter.instruction("mov QWORD PTR [rdx + rcx * 8 + 24], rax");             // store the converted element
    emitter.instruction("add rcx, 1");                                          // next index
    emitter.instruction("mov QWORD PTR [rbp - 40], rcx");                       // keep it
    emitter.instruction("mov QWORD PTR [rdx], rcx");                            // result length follows the copied prefix
    emitter.instruction("jmp 1b");                                              // continue
    emitter.label("8");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // source array
    emitter.instruction("call __rt_decref_any");                                // the conversion consumes its input
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // return the scalar array
    emitter.instruction("leave");                                               // release the helper frame
    emitter.instruction("ret");                                                 // return
}
