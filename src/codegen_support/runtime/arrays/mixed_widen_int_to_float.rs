//! Purpose:
//! Emits `__rt_mixed_widen_int_to_float`: PHP's coercive widening of an `int` held in a boxed
//! `mixed` value reaching a `float`-accepting declared type that does not accept `int`
//! (`?float`, `float|string`, ...): an int box becomes a fresh float box.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - Generated code for `Op::MixedWidenIntToFloat` (`crate::codegen::lower_inst::conversions`).
//!
//! Key details:
//! - The input is borrowed: the result is always a new owned reference (a fresh float box for an
//!   int payload, the same box retained otherwise).

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_mixed_widen_int_to_float`.
/// Input: x0 / rax = boxed Mixed. Output: x0 / rax = owned boxed Mixed.
pub fn emit_mixed_widen_int_to_float(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_widen_int_to_float ---");
    emitter.label_global("__rt_mixed_widen_int_to_float");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("cbz x0, 1f");                                  // no box: retain path (no-op)
            emitter.instruction("ldr x9, [x0]");                                // payload tag
            emitter.instruction("cbnz x9, 1f");                                 // not an int: keep the value
            emitter.instruction("ldr x9, [x0, #8]");                            // the int payload
            emitter.instruction("scvtf d0, x9");                                // as a double
            emitter.instruction("stp x29, x30, [sp, #-16]!");                   // frame for the boxing call
            emitter.instruction("mov x29, sp");
            emitter.instruction("fmov x1, d0");                                 // double bits as the payload
            emitter.instruction("mov x2, xzr");                                 // no high word
            emitter.instruction("mov x0, #2");                                  // runtime tag 2 = float
            emitter.instruction("bl __rt_mixed_box_scalar");                    // fresh float box
            emitter.instruction("ldp x29, x30, [sp], #16");
            emitter.instruction("ret");
            emitter.label("1");
            emitter.instruction("b __rt_incref");                               // the same value, one more reference
        }
        Arch::X86_64 => {
            emitter.instruction("test rax, rax");                               // no box: retain path (no-op)
            emitter.instruction("jz 1f");
            emitter.instruction("cmp QWORD PTR [rax], 0");                      // an int payload?
            emitter.instruction("jne 1f");                                      // no: keep the value
            emitter.instruction("cvtsi2sd xmm0, QWORD PTR [rax + 8]");          // the int payload as a double
            emitter.instruction("push rbp");                                    // aligned frame for the boxing call
            emitter.instruction("mov rbp, rsp");
            emitter.instruction("movq rdi, xmm0");                              // double bits as the payload
            emitter.instruction("xor esi, esi");                                // no high word
            emitter.instruction("mov eax, 2");                                  // runtime tag 2 = float
            emitter.instruction("call __rt_mixed_box_scalar");                  // fresh float box
            emitter.instruction("pop rbp");
            emitter.instruction("ret");
            emitter.label("1");
            emitter.instruction("jmp __rt_incref");                             // the same value, one more reference
        }
    }
}
