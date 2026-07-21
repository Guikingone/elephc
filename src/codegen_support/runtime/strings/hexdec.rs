//! Purpose:
//! Emits the `__rt_hexdec` runtime helper assembly for hexdec.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_hexdec` runtime helper for PHP `hexdec()`.
///
/// Parses the hexadecimal digits of the input string into a 64-bit integer,
/// ignoring any non-hexadecimal bytes (matching PHP). Overflow wraps modulo
/// 2^64 rather than promoting to float.
///
/// Register contract (ARM64):
/// - Input: x1 = string ptr, x2 = string len
/// - Output: x0 = parsed integer value
///
/// Register contract (x86_64 System V):
/// - Input: rdi = string ptr, rsi = string len
/// - Output: rax = parsed integer value
pub fn emit_hexdec(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hexdec_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hexdec ---");
    emitter.label_global("__rt_hexdec");

    emitter.instruction("mov x0, #0");                                          // accumulated integer result = 0
    emitter.instruction("mov x5, #0");                                          // byte scan index = 0
    emitter.label("__rt_hexdec_loop");
    emitter.instruction("cmp x5, x2");                                          // have we consumed every input byte?
    emitter.instruction("b.ge __rt_hexdec_done");                               // yes -> return the accumulated value
    emitter.instruction("ldrb w6, [x1, x5]");                                   // load the current input byte
    emitter.instruction("sub w7, w6, #48");                                     // compute byte - '0' to test for a decimal digit
    emitter.instruction("cmp w7, #9");                                          // is the byte in the range '0'..'9'?
    emitter.instruction("b.ls __rt_hexdec_digit");                              // yes -> use the 0..9 digit value directly
    emitter.instruction("orr w7, w6, #0x20");                                   // fold ASCII letters to lowercase
    emitter.instruction("sub w7, w7, #97");                                     // compute byte - 'a' to test for a hex letter
    emitter.instruction("cmp w7, #5");                                          // is the byte in the range 'a'..'f'?
    emitter.instruction("b.hi __rt_hexdec_skip");                               // no -> ignore this non-hexadecimal byte
    emitter.instruction("add w7, w7, #10");                                     // map 'a'..'f' to the digit values 10..15
    emitter.label("__rt_hexdec_digit");
    emitter.instruction("lsl x0, x0, #4");                                      // shift the accumulated value left by one hex digit
    emitter.instruction("add x0, x0, w7, uxtw");                                // add the current hexadecimal digit value
    emitter.label("__rt_hexdec_skip");
    emitter.instruction("add x5, x5, #1");                                      // advance to the next input byte
    emitter.instruction("b __rt_hexdec_loop");                                  // continue parsing the input string
    emitter.label("__rt_hexdec_done");
    emitter.instruction("ret");                                                 // return the parsed integer in x0
}

/// Emits the x86_64 Linux implementation of `__rt_hexdec`.
///
/// Input: rdi = string ptr, rsi = string len. Output: rax = parsed integer value.
fn emit_hexdec_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hexdec ---");
    emitter.label_global("__rt_hexdec");

    emitter.instruction("xor eax, eax");                                        // accumulated integer result = 0
    emitter.instruction("xor r8d, r8d");                                        // byte scan index = 0
    emitter.label("__rt_hexdec_loop_x86_64");
    emitter.instruction("cmp r8, rsi");                                         // have we consumed every input byte?
    emitter.instruction("jge __rt_hexdec_done_x86_64");                         // yes -> return the accumulated value
    emitter.instruction("movzx r9d, BYTE PTR [rdi + r8]");                      // load the current input byte
    emitter.instruction("lea r10d, [r9 - 48]");                                 // compute byte - '0' to test for a decimal digit
    emitter.instruction("cmp r10d, 9");                                         // is the byte in the range '0'..'9'?
    emitter.instruction("jbe __rt_hexdec_digit_x86_64");                        // yes -> use the 0..9 digit value directly
    emitter.instruction("mov r10d, r9d");                                       // reload the raw byte to test for a hex letter
    emitter.instruction("or r10d, 0x20");                                       // fold ASCII letters to lowercase
    emitter.instruction("sub r10d, 97");                                        // compute byte - 'a' to test for a hex letter
    emitter.instruction("cmp r10d, 5");                                         // is the byte in the range 'a'..'f'?
    emitter.instruction("ja __rt_hexdec_skip_x86_64");                          // no -> ignore this non-hexadecimal byte
    emitter.instruction("add r10d, 10");                                        // map 'a'..'f' to the digit values 10..15
    emitter.label("__rt_hexdec_digit_x86_64");
    emitter.instruction("shl rax, 4");                                          // shift the accumulated value left by one hex digit
    emitter.instruction("add rax, r10");                                        // add the current hexadecimal digit value
    emitter.label("__rt_hexdec_skip_x86_64");
    emitter.instruction("add r8, 1");                                           // advance to the next input byte
    emitter.instruction("jmp __rt_hexdec_loop_x86_64");                         // continue parsing the input string
    emitter.label("__rt_hexdec_done_x86_64");
    emitter.instruction("ret");                                                 // return the parsed integer in rax
}
