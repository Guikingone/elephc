//! Purpose:
//! Emits the `__rt_octdec` runtime helper assembly for octal-string-to-integer conversion.
//! Implements PHP's `octdec()`: parses octal digits (0-7) from a byte string and returns
//! the equivalent signed 64-bit decimal integer.  Stops at the first non-octal character.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - AArch64 input: x1=ptr, x2=len; output: x0=result.
//! - x86_64 input:  rax=ptr, rdx=len (elephc string-value convention); output: rax=result.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits the `__rt_octdec` runtime helper for octal-string-to-integer parsing.
///
/// Reads a PHP byte-string pointer/length pair, parses valid octal digits (0–7), and
/// returns the decimal-equivalent signed 64-bit integer.  Parsing stops at the first
/// character that is not in [0-7]; the accumulated result up to that point is returned.
/// An empty string or one with no leading octal digits returns 0.
///
/// # Input registers
/// - AArch64: x1 = string pointer, x2 = byte length
/// - x86_64:  rax = string pointer, rdx = byte length (elephc string-value convention)
///
/// # Output registers
/// - AArch64: x0 = result integer
/// - x86_64:  rax = result integer
pub fn emit_octdec(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_octdec_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: octdec ---");
    emitter.label_global("__rt_octdec");

    // -- initialize result accumulator --
    emitter.instruction("mov x0, #0");                                            // initialize result accumulator to zero
    emitter.instruction("cbz x2, __rt_octdec_done");                              // return 0 immediately for an empty string

    // -- parse octal digits: result = result * 8 + digit --
    emitter.label("__rt_octdec_loop");
    emitter.instruction("cbz x2, __rt_octdec_done");                              // stop when all bytes are consumed
    emitter.instruction("ldrb w3, [x1], #1");                                     // load next byte and advance pointer
    emitter.instruction("sub w3, w3, #48");                                       // subtract ASCII '0' to get digit value
    emitter.instruction("cmp w3, #7");                                            // accept only octal digits 0-7
    emitter.instruction("b.hi __rt_octdec_done");                                 // stop at the first non-octal character
    emitter.instruction("lsl x0, x0, #3");                                        // multiply accumulator by 8 (shift left 3 bits)
    emitter.instruction("add x0, x0, x3");                                        // add the new octal digit to the accumulator
    emitter.instruction("sub x2, x2, #1");                                        // decrement remaining byte count
    emitter.instruction("b __rt_octdec_loop");                                    // continue with the next character

    emitter.label("__rt_octdec_done");
    emitter.instruction("ret");                                                   // return result in x0 to caller
}

/// Emits the x86_64 Linux `__rt_octdec` runtime helper.
///
/// Identical parsing semantics to the ARM64 path but uses the elephc x86_64 string
/// convention: rax = string pointer, rdx = byte length.  Result is returned in rax.
fn emit_octdec_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: octdec ---");
    emitter.label_global("__rt_octdec");

    // -- initialize result accumulator --
    emitter.instruction("xor r8, r8");                                            // initialize result accumulator to zero
    emitter.instruction("test rdx, rdx");                                         // check whether the string is empty
    emitter.instruction("jz __rt_octdec_done_x86_64");                            // return 0 immediately for an empty string

    // -- parse octal digits: accumulator = accumulator * 8 + digit --
    emitter.label("__rt_octdec_loop_x86_64");
    emitter.instruction("test rdx, rdx");                                         // stop when all bytes are consumed
    emitter.instruction("jz __rt_octdec_done_x86_64");                            // exit loop when no bytes remain
    emitter.instruction("movzx rcx, BYTE PTR [rax]");                             // load next byte without sign-extension
    emitter.instruction("inc rax");                                               // advance the string pointer to the next byte
    emitter.instruction("sub ecx, 48");                                           // subtract ASCII '0' to get digit value
    emitter.instruction("cmp ecx, 7");                                            // accept only octal digits 0-7
    emitter.instruction("ja __rt_octdec_done_x86_64");                            // stop at the first non-octal character
    emitter.instruction("shl r8, 3");                                             // multiply accumulator by 8 (shift left 3 bits)
    emitter.instruction("add r8, rcx");                                           // add the new octal digit to the accumulator
    emitter.instruction("dec rdx");                                               // decrement remaining byte count
    emitter.instruction("jmp __rt_octdec_loop_x86_64");                           // continue with the next character

    emitter.label("__rt_octdec_done_x86_64");
    emitter.instruction("mov rax, r8");                                           // move result from accumulator register to rax
    emitter.instruction("ret");                                                   // return result in rax to caller
}
