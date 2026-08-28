//! Purpose:
//! Emits the `__rt_base64_encode`, `__rt_b64enc_loop` runtime helper assembly for base64 encode.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - Base64 helpers depend on fixed encode/decode tables and must report decoded pointer/length pairs consistently.

use crate::codegen_support::{emit::Emitter, platform::Arch};
use crate::codegen_support::abi;

/// Emits the `__rt_base64_encode` runtime helper for PHP's `base64_encode()`.
///
/// # Input (ARM64: x0/x1, x86_64: rax/rdx)
/// - x0/rax: source string pointer
/// - x1/rdx: source string byte length
///
/// # Output (ARM64: x0/x1, x86_64: rax/rdx)
/// - x0/rax: encoded string pointer in the concat buffer
/// - x1/rdx: encoded string byte length
///
/// # ABI details
/// - ARM64: appends to the shared concat buffer; advances `_concat_off` by the result length
/// - x86_64 Linux: same concat-buffer semantics using `_concat_buf` / `_concat_off`
/// - Uses `_b64_encode_tbl` lookup table for the base64 alphabet (A-Z, a-z, 0-9, +, /)
/// - Handles 0, 1, or 2 remainder bytes with `=` padding per RFC 4648
/// - Does not null-terminate the output
pub fn emit_base64_encode(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_base64_encode_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: base64_encode ---");
    emitter.label_global("__rt_base64_encode");

    // -- reserve the worst-case 4-chars-per-3-bytes result before writing anything --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed source string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the base64 encoder frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the source pointer and length across the reservation call
    emitter.instruction("adds x0, x2, x2");                                     // start the 4*ceil(len/3) upper bound from 2 * source length
    emitter.instruction("b.cs __rt_b64enc_size_overflow");                      // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("adds x0, x0, #4");                                     // add the padded final quantum so short inputs still fit the bound
    emitter.instruction("b.cs __rt_b64enc_size_overflow");                      // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the encoded result
    emitter.instruction("mov x9, x0");                                          // destination pointer
    emitter.instruction("mov x10, x0");                                         // save result start
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed source pointer and length
    emitter.instruction("mov x11, x2");                                         // remaining byte count

    // -- load base64 lookup table --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x15", "_b64_encode_tbl");

    // -- process 3 bytes at a time --
    emitter.label("__rt_b64enc_loop");
    emitter.instruction("cmp x11, #3");                                         // at least 3 bytes left?
    emitter.instruction("b.lt __rt_b64enc_remainder");                          // no -> handle remainder

    // -- load 3 source bytes --
    emitter.instruction("ldrb w12, [x1], #1");                                  // byte 0
    emitter.instruction("ldrb w13, [x1], #1");                                  // byte 1
    emitter.instruction("ldrb w14, [x1], #1");                                  // byte 2
    emitter.instruction("sub x11, x11, #3");                                    // consumed 3 bytes

    // -- encode char 0: top 6 bits of byte 0 --
    emitter.instruction("lsr w16, w12, #2");                                    // byte0 >> 2
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup table[index]
    emitter.instruction("strb w16, [x9], #1");                                  // write encoded char 0

    // -- encode char 1: bottom 2 of byte0 + top 4 of byte1 --
    emitter.instruction("and w16, w12, #0x3");                                  // byte0 & 0x3
    emitter.instruction("lsl w16, w16, #4");                                    // shift left 4
    emitter.instruction("lsr w17, w13, #4");                                    // byte1 >> 4
    emitter.instruction("orr w16, w16, w17");                                   // combine
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup table[index]
    emitter.instruction("strb w16, [x9], #1");                                  // write encoded char 1

    // -- encode char 2: bottom 4 of byte1 + top 2 of byte2 --
    emitter.instruction("and w16, w13, #0xf");                                  // byte1 & 0xf
    emitter.instruction("lsl w16, w16, #2");                                    // shift left 2
    emitter.instruction("lsr w17, w14, #6");                                    // byte2 >> 6
    emitter.instruction("orr w16, w16, w17");                                   // combine
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup table[index]
    emitter.instruction("strb w16, [x9], #1");                                  // write encoded char 2

    // -- encode char 3: bottom 6 of byte2 --
    emitter.instruction("and w16, w14, #0x3f");                                 // byte2 & 0x3f
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup table[index]
    emitter.instruction("strb w16, [x9], #1");                                  // write encoded char 3

    emitter.instruction("b __rt_b64enc_loop");                                  // next 3 bytes

    // -- handle remainder (0, 1, or 2 bytes left) --
    emitter.label("__rt_b64enc_remainder");
    emitter.instruction("cbz x11, __rt_b64enc_done");                           // 0 bytes left -> done

    emitter.instruction("cmp x11, #1");                                         // exactly 1 byte left?
    emitter.instruction("b.ne __rt_b64enc_rem2");                               // no -> 2 bytes

    // -- 1 byte remainder: 2 encoded chars + 2 padding --
    emitter.instruction("ldrb w12, [x1]");                                      // load last byte
    // char 0: top 6 bits
    emitter.instruction("lsr w16, w12, #2");                                    // byte0 >> 2
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup
    emitter.instruction("strb w16, [x9], #1");                                  // write char 0
    // char 1: bottom 2 bits << 4
    emitter.instruction("and w16, w12, #0x3");                                  // byte0 & 0x3
    emitter.instruction("lsl w16, w16, #4");                                    // shift left 4
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup
    emitter.instruction("strb w16, [x9], #1");                                  // write char 1
    // padding
    emitter.instruction("mov w16, #61");                                        // '=' padding char
    emitter.instruction("strb w16, [x9], #1");                                  // write '='
    emitter.instruction("strb w16, [x9], #1");                                  // write '='
    emitter.instruction("b __rt_b64enc_done");                                  // done

    // -- 2 byte remainder: 3 encoded chars + 1 padding --
    emitter.label("__rt_b64enc_rem2");
    emitter.instruction("ldrb w12, [x1]");                                      // load byte 0
    emitter.instruction("ldrb w13, [x1, #1]");                                  // load byte 1
    // char 0: top 6 bits of byte0
    emitter.instruction("lsr w16, w12, #2");                                    // byte0 >> 2
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup
    emitter.instruction("strb w16, [x9], #1");                                  // write char 0
    // char 1: bottom 2 of byte0 + top 4 of byte1
    emitter.instruction("and w16, w12, #0x3");                                  // byte0 & 0x3
    emitter.instruction("lsl w16, w16, #4");                                    // shift left 4
    emitter.instruction("lsr w17, w13, #4");                                    // byte1 >> 4
    emitter.instruction("orr w16, w16, w17");                                   // combine
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup
    emitter.instruction("strb w16, [x9], #1");                                  // write char 1
    // char 2: bottom 4 of byte1 << 2
    emitter.instruction("and w16, w13, #0xf");                                  // byte1 & 0xf
    emitter.instruction("lsl w16, w16, #2");                                    // shift left 2
    emitter.instruction("ldrb w16, [x15, x16]");                                // lookup
    emitter.instruction("strb w16, [x9], #1");                                  // write char 2
    // padding
    emitter.instruction("mov w16, #61");                                        // '=' padding char
    emitter.instruction("strb w16, [x9], #1");                                  // write '='

    emitter.label("__rt_b64enc_done");
    emitter.instruction("mov x1, x10");                                         // result pointer
    emitter.instruction("sub x2, x9, x10");                                     // result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the base64 encoder frame
    emitter.instruction("ret");                                                 // return

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_b64enc_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits the x86_64 Linux variant of `__rt_base64_encode`.
///
/// Uses the System V AMD64 ABI: source pointer in `rax`, length in `rdx`.
/// Output is returned in `rax` (pointer) and `rdx` (length) via the concat buffer.
/// Operates identically to the ARM64 variant but uses x86_64 registers and instructions.
/// Local labels are prefixed with `_linux_x86_64` to avoid collisions with the ARM64 path.
fn emit_base64_encode_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: base64_encode ---");
    emitter.label_global("__rt_base64_encode");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed source string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the source pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the source pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the source byte count across the reservation call
    emitter.instruction("mov rax, rdx");                                        // seed the encoded-size bound from the source byte count
    emitter.instruction("add rax, rax");                                        // start the 4*ceil(len/3) upper bound from 2 * source length
    emitter.instruction("jc __rt_b64enc_size_overflow_linux_x86_64");           // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("add rax, 4");                                          // add the padded final quantum so short inputs still fit the bound
    emitter.instruction("jc __rt_b64enc_size_overflow_linux_x86_64");           // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the encoded result
    emitter.instruction("mov r9, rax");                                         // compute the destination pointer at the reserved result start
    emitter.instruction("mov r10, r9");                                         // preserve the encoded string start pointer for the return value
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // copy the source byte count into a decrementing loop counter
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // copy the source pointer into a cursor register for byte-by-byte reads
    abi::emit_symbol_address(emitter, "r11", "_b64_encode_tbl");                // load the base64 lookup-table address for the encoding loop

    emitter.label("__rt_b64enc_loop_linux_x86_64");
    emitter.instruction("cmp rcx, 3");                                          // check whether at least one full 3-byte chunk remains
    emitter.instruction("jl __rt_b64enc_remainder_linux_x86_64");               // branch to the remainder path when fewer than 3 bytes remain

    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load source byte 0 and widen it for bit manipulation
    emitter.instruction("add rsi, 1");                                          // advance the source cursor past byte 0
    emitter.instruction("movzx edx, BYTE PTR [rsi]");                           // load source byte 1 and widen it for bit manipulation
    emitter.instruction("add rsi, 1");                                          // advance the source cursor past byte 1
    emitter.instruction("movzx r8d, BYTE PTR [rsi]");                           // load source byte 2 and widen it for bit manipulation
    emitter.instruction("add rsi, 1");                                          // advance the source cursor past byte 2
    emitter.instruction("sub rcx, 3");                                          // record that one full 3-byte chunk has been consumed

    emitter.instruction("mov edi, eax");                                        // copy byte 0 into a scratch register for char 0 encoding
    emitter.instruction("shr edi, 2");                                          // keep the top 6 bits of source byte 0
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the 6-bit group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded character 0 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 0

    emitter.instruction("mov edi, eax");                                        // reuse the integer scratch register while assembling encoded character 1
    emitter.instruction("mov edi, DWORD PTR [rsi - 3]");                        // reload source byte 0 from the just-consumed 3-byte chunk
    emitter.instruction("and edi, 3");                                          // keep the low 2 bits from source byte 0
    emitter.instruction("shl edi, 4");                                          // shift those 2 bits into the high half of the next 6-bit group
    emitter.instruction("mov eax, edx");                                        // copy source byte 1 into the integer scratch register for its upper nibble
    emitter.instruction("shr eax, 4");                                          // keep the top 4 bits from source byte 1
    emitter.instruction("or edi, eax");                                         // combine the carried byte-0 bits with the upper nibble from byte 1
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the second 6-bit group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded character 1 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 1

    emitter.instruction("mov edi, edx");                                        // seed the scratch register with source byte 1 while assembling encoded character 2
    emitter.instruction("and edi, 15");                                         // keep the low 4 bits from source byte 1
    emitter.instruction("shl edi, 2");                                          // shift those 4 bits into the high half of the next 6-bit group
    emitter.instruction("mov eax, r8d");                                        // copy source byte 2 into the integer scratch register for its upper bits
    emitter.instruction("shr eax, 6");                                          // keep the top 2 bits from source byte 2
    emitter.instruction("or edi, eax");                                         // combine the carried byte-1 bits with the upper bits from byte 2
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the third 6-bit group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded character 2 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 2

    emitter.instruction("mov edi, r8d");                                        // seed the scratch register with source byte 2 while assembling encoded character 3
    emitter.instruction("and edi, 63");                                         // keep the low 6 bits from source byte 2
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the final 6-bit group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded character 3 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 3
    emitter.instruction("jmp __rt_b64enc_loop_linux_x86_64");                   // continue encoding subsequent 3-byte chunks

    emitter.label("__rt_b64enc_remainder_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once no remainder bytes remain after the main loop
    emitter.instruction("je __rt_b64enc_done_linux_x86_64");                    // skip the remainder path when the input length was an exact multiple of 3
    emitter.instruction("cmp rcx, 1");                                          // check whether exactly one source byte remains
    emitter.instruction("jne __rt_b64enc_rem2_linux_x86_64");                   // branch to the two-byte remainder path when two bytes remain

    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load the final source byte for the 1-byte remainder case
    emitter.instruction("mov edi, eax");                                        // copy the remaining byte into a scratch register for char 0 encoding
    emitter.instruction("shr edi, 2");                                          // keep the top 6 bits from the remaining source byte
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the first remainder group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded remainder character 0 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 0
    emitter.instruction("movzx edi, BYTE PTR [rsi]");                           // reload the remaining source byte while assembling encoded remainder character 1
    emitter.instruction("and edi, 3");                                          // keep the low 2 bits from the remaining source byte
    emitter.instruction("shl edi, 4");                                          // shift those 2 bits into the high half of the next 6-bit group
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the second remainder group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded remainder character 1 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 1
    emitter.instruction("mov BYTE PTR [r9], 61");                               // append the first '=' padding byte for the 1-byte remainder case
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after the first padding byte
    emitter.instruction("mov BYTE PTR [r9], 61");                               // append the second '=' padding byte for the 1-byte remainder case
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after the second padding byte
    emitter.instruction("jmp __rt_b64enc_done_linux_x86_64");                   // finish after handling the 1-byte remainder case

    emitter.label("__rt_b64enc_rem2_linux_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load remainder source byte 0 for the 2-byte remainder case
    emitter.instruction("movzx edx, BYTE PTR [rsi + 1]");                       // load remainder source byte 1 for the 2-byte remainder case
    emitter.instruction("mov edi, eax");                                        // copy remainder byte 0 into a scratch register for char 0 encoding
    emitter.instruction("shr edi, 2");                                          // keep the top 6 bits from remainder byte 0
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the first remainder group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded remainder character 0 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 0
    emitter.instruction("movzx edi, BYTE PTR [rsi]");                           // reload remainder byte 0 while assembling encoded remainder character 1
    emitter.instruction("and edi, 3");                                          // keep the low 2 bits from remainder byte 0
    emitter.instruction("shl edi, 4");                                          // shift those 2 bits into the high half of the next 6-bit group
    emitter.instruction("mov eax, edx");                                        // copy remainder byte 1 into the integer scratch register for its upper nibble
    emitter.instruction("shr eax, 4");                                          // keep the top 4 bits from remainder byte 1
    emitter.instruction("or edi, eax");                                         // combine the carried byte-0 bits with the upper nibble from remainder byte 1
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the second remainder group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded remainder character 1 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 1
    emitter.instruction("mov edi, edx");                                        // seed the scratch register with remainder byte 1 while assembling encoded remainder character 2
    emitter.instruction("and edi, 15");                                         // keep the low 4 bits from remainder byte 1
    emitter.instruction("shl edi, 2");                                          // shift those 4 bits into the high half of the next 6-bit group
    emitter.instruction("movzx eax, BYTE PTR [r11 + rdi]");                     // map the third remainder group through the base64 alphabet
    emitter.instruction("mov BYTE PTR [r9], al");                               // write encoded remainder character 2 to the destination buffer
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing character 2
    emitter.instruction("mov BYTE PTR [r9], 61");                               // append the single '=' padding byte for the 2-byte remainder case
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after the padding byte

    emitter.label("__rt_b64enc_done_linux_x86_64");
    emitter.instruction("mov rax, r10");                                        // return the encoded string start pointer in the standard x86_64 string result register
    emitter.instruction("mov rdx, r9");                                         // copy the destination cursor into the length scratch register
    emitter.instruction("sub rdx, r10");                                        // compute the encoded string length from the written byte count
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the base64 encoder spill slots before returning the encoded string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the encoded string
    emitter.instruction("ret");                                                 // return the encoded string through the standard x86_64 string result registers

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_b64enc_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
