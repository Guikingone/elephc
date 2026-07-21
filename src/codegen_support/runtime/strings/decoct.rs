//! Purpose:
//! Emits the `__rt_decoct` runtime helper assembly for integer-to-octal-string conversion.
//! Implements PHP's `decoct()`: converts a 64-bit integer to its octal string representation.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - AArch64 input: x0=integer; output: x1=ptr, x2=len (written right-to-left into _concat_buf).
//! - x86_64 input: rax=integer; output: rax=ptr, rdx=len.
//! - Negative values are interpreted as unsigned 64-bit (matching PHP's behaviour).

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits the `__rt_decoct` runtime helper for integer-to-octal-string conversion.
///
/// Converts the input integer to its octal representation, writes digits right-to-left
/// into a 23-byte scratch area in `_concat_buf`, then returns a pointer/length pair.
///
/// # Input registers
/// - AArch64: x0 = integer value
/// - x86_64: rax = integer value
///
/// # Output registers
/// - AArch64: x1 = string pointer, x2 = string length
/// - x86_64: rax = string pointer, rdx = string length
pub fn emit_decoct(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_decoct_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: decoct ---");
    emitter.label_global("__rt_decoct");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #16");                                     // allocate 16 bytes for frame pointer and link register
    emitter.instruction("stp x29, x30, [sp]");                                  // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish new frame pointer

    // -- get concat_buf write position --
    abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load current offset into concat_buf
    abi::emit_symbol_address(emitter, "x7", "_concat_buf");
    emitter.instruction("add x9, x7, x8");                                      // compute write position: buf + offset
    emitter.instruction("add x9, x9, #22");                                     // advance to end of 23-byte scratch area (64-bit value needs at most 22 octal digits)

    // -- initialize counters and handle the zero special case --
    emitter.instruction("mov x10, #0");                                         // digit count = 0
    emitter.instruction("cbnz x0, __rt_decoct_loop");                           // if value != 0, start digit extraction loop
    emitter.instruction("mov w11, #48");                                        // ASCII '0'
    emitter.instruction("strb w11, [x9]");                                      // store '0' at current position
    emitter.instruction("sub x9, x9, #1");                                      // move write cursor left
    emitter.instruction("mov x10, #1");                                         // digit count = 1
    emitter.instruction("b __rt_decoct_done");                                  // skip digit extraction for zero

    // -- extract octal digits right-to-left --
    emitter.label("__rt_decoct_loop");
    emitter.instruction("cbz x0, __rt_decoct_done");                            // if value is 0, all digits extracted
    emitter.instruction("and x11, x0, #0x7");                                   // isolate the lowest 3-bit group (octal digit)
    emitter.instruction("add x11, x11, #48");                                   // map 0-7 to ASCII '0'-'7'
    emitter.instruction("strb w11, [x9]");                                      // store the octal digit at current position
    emitter.instruction("sub x9, x9, #1");                                      // move write cursor left (right-to-left)
    emitter.instruction("add x10, x10, #1");                                    // increment digit count
    emitter.instruction("lsr x0, x0, #3");                                      // shift value right by 3 bits for next octal digit
    emitter.instruction("b __rt_decoct_loop");                                  // continue extracting digits

    // -- finalize: update concat_buf offset and return ptr/len --
    emitter.label("__rt_decoct_done");
    emitter.instruction("add x8, x8, #23");                                     // advance concat_off by scratch area size
    emitter.instruction("str x8, [x6]");                                        // store updated offset back to _concat_off
    emitter.instruction("add x1, x9, #1");                                      // result ptr = one past last written position
    emitter.instruction("mov x2, x10");                                         // result length = digit count

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp]");                                  // restore frame pointer and return address
    emitter.instruction("add sp, sp, #16");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits the x86_64 Linux `__rt_decoct` runtime helper.
///
/// Input: rax = integer value. Output: rax = string pointer, rdx = string length.
fn emit_decoct_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: decoct ---");
    emitter.label_global("__rt_decoct");

    // -- set up stack frame --
    emitter.instruction("push rbp");                                            // save the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer

    // -- get concat_buf write position --
    abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current concat buffer offset
    abi::emit_symbol_address(emitter, "r10", "_concat_buf");
    emitter.instruction("add r10, r9");                                         // compute the current concat buffer write position
    emitter.instruction("add r10, 22");                                         // advance to end of 23-byte scratch area for right-to-left writes

    // -- initialize counter and handle the zero special case --
    emitter.instruction("xor ecx, ecx");                                        // digit count = 0
    emitter.instruction("test rax, rax");                                       // check whether the input is zero
    emitter.instruction("jne __rt_decoct_loop_x86_64");                         // non-zero -> start digit extraction
    emitter.instruction("mov BYTE PTR [r10], 48");                              // store ASCII '0' into the scratch area
    emitter.instruction("dec r10");                                             // move the write cursor left
    emitter.instruction("mov ecx, 1");                                          // digit count = 1 for the zero special case
    emitter.instruction("jmp __rt_decoct_done_x86_64");                         // skip digit extraction for zero

    // -- extract octal digits right-to-left --
    emitter.label("__rt_decoct_loop_x86_64");
    emitter.instruction("test rax, rax");                                       // check whether more digits remain
    emitter.instruction("je __rt_decoct_done_x86_64");                          // done when value reaches zero
    emitter.instruction("mov r11, rax");                                        // copy value for digit extraction
    emitter.instruction("and r11, 7");                                          // isolate the lowest 3-bit group (octal digit)
    emitter.instruction("add r11, 48");                                         // map 0-7 to ASCII '0'-'7'
    emitter.instruction("mov BYTE PTR [r10], r11b");                            // store the octal digit at the current scratch position
    emitter.instruction("dec r10");                                             // move the write cursor left for the next digit
    emitter.instruction("inc ecx");                                             // increment the output length
    emitter.instruction("shr rax, 3");                                          // shift value right by 3 bits for next octal digit
    emitter.instruction("jmp __rt_decoct_loop_x86_64");                         // continue extracting digits

    // -- finalize: update concat_buf offset and return ptr/len --
    emitter.label("__rt_decoct_done_x86_64");
    emitter.instruction("add r9, 23");                                          // advance concat_off by scratch area size
    emitter.instruction("mov QWORD PTR [r8], r9");                              // store the updated concat buffer offset
    emitter.instruction("lea rax, [r10 + 1]");                                  // return string pointer as one byte past the last decremented position
    emitter.instruction("mov rdx, rcx");                                        // return string length in rdx

    // -- restore frame and return --
    emitter.instruction("pop rbp");                                             // restore caller frame pointer
    emitter.instruction("ret");                                                 // return to caller with rax=ptr, rdx=len
}
