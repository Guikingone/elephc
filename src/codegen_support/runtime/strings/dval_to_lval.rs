//! Purpose:
//! Emits `__rt_dval_to_lval_slow`, the out-of-range half of PHP's float-to-int conversion
//! (`zend_dval_to_lval`), reached from the inline `(int)` conversion only when the machine
//! truncation saturated (AArch64) or produced the x86_64 "integer indefinite" value.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//! - Inline conversions from `crate::codegen_support::abi::emit_float_result_to_int_result`.
//!
//! Key details:
//! - Input: d0 / xmm0. Output: x0 / rax. Every other register is preserved, so the inline
//!   conversion keeps the "no caller-saved clobber" contract the register allocator assumes.
//! - NaN and infinities give 0; a finite value at or beyond 2^63 in magnitude is an integer,
//!   so its value modulo 2^64 is the 53-bit significand shifted left by `exponent - 52`
//!   (0 once the shift reaches 64), negated for negative inputs.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_dval_to_lval_slow` for the active target.
pub fn emit_dval_to_lval_slow(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: dval_to_lval_slow ---");
    emitter.label_global("__rt_dval_to_lval_slow");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("stp x9, x10, [sp, #-16]!");                    // preserve the scratch registers
            emitter.instruction("fmov x0, d0");                                 // raw IEEE-754 bits
            emitter.instruction("ubfx x9, x0, #52, #11");                       // biased exponent
            emitter.instruction("cmp x9, #2047");                               // NaN or infinity?
            emitter.instruction("b.eq __rt_dval_to_lval_zero");                 // PHP converts them to 0
            emitter.instruction("subs x9, x9, #1075");                          // shift = exponent - 1023 - 52
            emitter.instruction("b.lt __rt_dval_to_lval_small");                // fractional value: plain truncation
            emitter.instruction("cmp x9, #64");                                 // shifted past every result bit?
            emitter.instruction("b.ge __rt_dval_to_lval_zero");                 // the value is a multiple of 2^64
            emitter.instruction("ubfx x10, x0, #0, #52");                       // stored significand bits
            emitter.instruction("orr x10, x10, #0x10000000000000");             // add the implicit leading 1
            emitter.instruction("lsl x10, x10, x9");                            // value modulo 2^64
            emitter.instruction("cmp x0, #0");                                  // negative input?
            emitter.instruction("cneg x0, x10, lt");                            // two's-complement wrap of the negative value
            emitter.instruction("b __rt_dval_to_lval_done");                    // result ready
            emitter.label("__rt_dval_to_lval_small");
            emitter.instruction("fcvtzs x0, d0");                               // in range: truncate toward zero
            emitter.instruction("b __rt_dval_to_lval_done");                    // result ready
            emitter.label("__rt_dval_to_lval_zero");
            emitter.instruction("mov x0, #0");                                  // NaN, infinity, or a multiple of 2^64
            emitter.label("__rt_dval_to_lval_done");
            emitter.instruction("ldp x9, x10, [sp], #16");                      // restore the scratch registers
            emitter.instruction("ret");                                         // return the PHP integer in x0
        }
        Arch::X86_64 => {
            emitter.instruction("push rcx");                                    // preserve the shift-count register
            emitter.instruction("push rdx");                                    // preserve the scratch register
            emitter.instruction("movq rax, xmm0");                              // raw IEEE-754 bits
            emitter.instruction("mov rdx, rax");                                // copy for the exponent
            emitter.instruction("shr rdx, 52");                                 // drop the significand
            emitter.instruction("and edx, 2047");                               // biased exponent
            emitter.instruction("cmp edx, 2047");                               // NaN or infinity?
            emitter.instruction("je __rt_dval_to_lval_zero");                   // PHP converts them to 0
            emitter.instruction("sub edx, 1075");                               // shift = exponent - 1023 - 52
            emitter.instruction("jl __rt_dval_to_lval_small");                  // fractional value: plain truncation
            emitter.instruction("cmp edx, 64");                                 // shifted past every result bit?
            emitter.instruction("jge __rt_dval_to_lval_zero");                  // the value is a multiple of 2^64
            emitter.instruction("mov ecx, edx");                                // shift count
            emitter.instruction("mov rdx, rax");                                // keep the sign
            emitter.instruction("shl rax, 12");                                 // clear sign and exponent ...
            emitter.instruction("shr rax, 12");                                 // ... leaving the stored significand
            emitter.instruction("bts rax, 52");                                 // add the implicit leading 1
            emitter.instruction("shl rax, cl");                                 // value modulo 2^64
            emitter.instruction("test rdx, rdx");                               // negative input?
            emitter.instruction("jns __rt_dval_to_lval_done");                  // no: done
            emitter.instruction("neg rax");                                     // two's-complement wrap of the negative value
            emitter.instruction("jmp __rt_dval_to_lval_done");                  // result ready
            emitter.label("__rt_dval_to_lval_small");
            emitter.instruction("cvttsd2si rax, xmm0");                         // in range: truncate toward zero
            emitter.instruction("jmp __rt_dval_to_lval_done");                  // result ready
            emitter.label("__rt_dval_to_lval_zero");
            emitter.instruction("xor eax, eax");                                // NaN, infinity, or a multiple of 2^64
            emitter.label("__rt_dval_to_lval_done");
            emitter.instruction("pop rdx");                                     // restore the scratch register
            emitter.instruction("pop rcx");                                     // restore the shift-count register
            emitter.instruction("ret");                                         // return the PHP integer in rax
        }
    }
}
