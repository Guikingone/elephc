//! Purpose:
//! Emits the `__rt_bindec` runtime helper assembly for binary-string-to-integer conversion.
//! Implements PHP's `bindec()`: parses `0`/`1` characters from a byte string and returns
//! the equivalent signed 64-bit integer.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - AArch64 input: x1=ptr, x2=len; output: x0=result.
//! - x86_64 input: rdi=ptr, rsi=len; output: rax=result.
//! - Non-binary characters (not `0` or `1`) are ignored, matching PHP's behaviour.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_bindec` runtime helper for binary-string-to-integer parsing.
///
/// Reads a PHP byte-string pointer/length pair, parses valid binary digits (`0`/`1`),
/// and returns the decimal-equivalent signed 64-bit integer. Non-binary characters
/// are ignored (matching PHP). An empty string returns 0.
///
/// # Input registers
/// - AArch64: x1 = string pointer, x2 = byte length
/// - x86_64: rdi = string pointer, rsi = byte length
///
/// # Output registers
/// - AArch64: x0 = result integer
/// - x86_64: rax = result integer
pub fn emit_bindec(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_bindec_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: bindec ---");
    emitter.label_global("__rt_bindec");

    emitter.instruction("mov x0, #0");                                          // initialize result accumulator to zero
    emitter.instruction("mov x5, #0");                                          // byte scan index = 0
    emitter.label("__rt_bindec_loop");
    emitter.instruction("cmp x5, x2");                                          // have we consumed every input byte?
    emitter.instruction("b.ge __rt_bindec_done");                               // yes -> return the accumulated value
    emitter.instruction("ldrb w6, [x1, x5]");                                   // load the current input byte
    emitter.instruction("sub w7, w6, #48");                                     // compute byte - '0' to get digit value
    emitter.instruction("cmp w7, #1");                                          // is the byte '0' or '1'?
    emitter.instruction("b.hi __rt_bindec_skip");                               // no -> ignore this non-binary byte
    emitter.instruction("lsl x0, x0, #1");                                      // shift the accumulated value left by one binary digit
    emitter.instruction("add x0, x0, w7, uxtw");                                // add the current binary digit (0 or 1)
    emitter.label("__rt_bindec_skip");
    emitter.instruction("add x5, x5, #1");                                      // advance to the next input byte
    emitter.instruction("b __rt_bindec_loop");                                  // continue parsing the input string
    emitter.label("__rt_bindec_done");
    emitter.instruction("ret");                                                 // return the parsed integer in x0
}

/// Emits the x86_64 Linux `__rt_bindec` runtime helper.
///
/// Input: rdi = string pointer, rsi = byte length. Output: rax = result integer.
fn emit_bindec_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: bindec ---");
    emitter.label_global("__rt_bindec");

    emitter.instruction("xor eax, eax");                                        // initialize result accumulator to zero
    emitter.instruction("xor r8d, r8d");                                        // byte scan index = 0
    emitter.label("__rt_bindec_loop_x86_64");
    emitter.instruction("cmp r8, rsi");                                         // have we consumed every input byte?
    emitter.instruction("jge __rt_bindec_done_x86_64");                         // yes -> return the accumulated value
    emitter.instruction("movzx r9d, BYTE PTR [rdi + r8]");                      // load the current input byte
    emitter.instruction("lea r10d, [r9 - 48]");                                 // compute byte - '0' to get digit value
    emitter.instruction("cmp r10d, 1");                                         // is the byte '0' or '1'?
    emitter.instruction("ja __rt_bindec_skip_x86_64");                          // no -> ignore this non-binary byte
    emitter.instruction("shl rax, 1");                                          // shift the accumulated value left by one binary digit
    emitter.instruction("add rax, r10");                                        // add the current binary digit (0 or 1)
    emitter.label("__rt_bindec_skip_x86_64");
    emitter.instruction("add r8, 1");                                           // advance to the next input byte
    emitter.instruction("jmp __rt_bindec_loop_x86_64");                         // continue parsing the input string
    emitter.label("__rt_bindec_done_x86_64");
    emitter.instruction("ret");                                                 // return the parsed integer in rax
}
