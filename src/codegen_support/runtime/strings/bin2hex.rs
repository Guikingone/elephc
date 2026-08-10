//! Purpose:
//! Emits the `__rt_bin2hex`, `__rt_bin2hex_loop` runtime helper assembly for bin2hex.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - The `2 * len` result is reserved through `__rt_concat_reserve` before the first store, so
//!   inputs whose hexadecimal expansion exceeds the 64 KiB concat scratch fall back to heap storage.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_bin2hex` runtime helper for the `bin2hex` builtin.
/// Dispatches to target-specific implementations. On ARM64, uses x1/x2 for input
/// string pointer/length and returns result pointer/length in x1/x2. On x86_64 Linux,
/// uses rax/rdx for string result, rsi for source pointer, rdx for source length.
/// Both variants reserve the exact `2 * len` result through `__rt_concat_reserve`
/// (concat scratch while it fits, owned heap storage otherwise) and finish through
/// `__rt_concat_publish`, which advances `_concat_off` only for scratch-backed results.
pub fn emit_bin2hex(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_bin2hex_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: bin2hex ---");
    emitter.label_global("__rt_bin2hex");

    // -- reserve the exact 2-bytes-per-input-byte result before writing anything --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed source string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the bin2hex helper frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the source pointer and length across the reservation call
    emitter.instruction("adds x0, x2, x2");                                     // compute the hexadecimal result size as 2 * source length
    emitter.instruction("b.cs __rt_bin2hex_size_overflow");                     // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the hexadecimal result
    emitter.instruction("mov x9, x0");                                          // destination cursor
    emitter.instruction("mov x10, x0");                                         // save result start
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed source pointer and length
    emitter.instruction("mov x11, x2");                                         // remaining count

    emitter.label("__rt_bin2hex_loop");
    emitter.instruction("cbz x11, __rt_bin2hex_done");                          // done if no bytes left
    emitter.instruction("ldrb w12, [x1], #1");                                  // load byte, advance source
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining
    // -- high nibble --
    emitter.instruction("lsr w13, w12, #4");                                    // extract high 4 bits
    emitter.instruction("cmp w13, #10");                                        // >= 10?
    emitter.instruction("b.ge __rt_bin2hex_hi_af");                             // yes → use a-f
    emitter.instruction("add w13, w13, #48");                                   // convert 0-9 to '0'-'9'
    emitter.instruction("b __rt_bin2hex_hi_store");                             // store
    emitter.label("__rt_bin2hex_hi_af");
    emitter.instruction("add w13, w13, #87");                                   // convert 10-15 to 'a'-'f'
    emitter.label("__rt_bin2hex_hi_store");
    emitter.instruction("strb w13, [x9], #1");                                  // write high nibble hex char
    // -- low nibble --
    emitter.instruction("and w13, w12, #0xf");                                  // extract low 4 bits
    emitter.instruction("cmp w13, #10");                                        // >= 10?
    emitter.instruction("b.ge __rt_bin2hex_lo_af");                             // yes → use a-f
    emitter.instruction("add w13, w13, #48");                                   // convert 0-9 to '0'-'9'
    emitter.instruction("b __rt_bin2hex_lo_store");                             // store
    emitter.label("__rt_bin2hex_lo_af");
    emitter.instruction("add w13, w13, #87");                                   // convert 10-15 to 'a'-'f'
    emitter.label("__rt_bin2hex_lo_store");
    emitter.instruction("strb w13, [x9], #1");                                  // write low nibble hex char
    emitter.instruction("b __rt_bin2hex_loop");                                 // next byte

    emitter.label("__rt_bin2hex_done");
    emitter.instruction("mov x1, x10");                                         // result pointer
    emitter.instruction("sub x2, x9, x10");                                     // result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the bin2hex helper frame
    emitter.instruction("ret");                                                 // return

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_bin2hex_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits `__rt_bin2hex` for x86_64 Linux using the System V ABI.
fn emit_bin2hex_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: bin2hex ---");
    emitter.label_global("__rt_bin2hex");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed source string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the source pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the source pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the source byte count across the reservation call
    emitter.instruction("mov rax, rdx");                                        // seed the result size from the source byte count
    emitter.instruction("add rax, rax");                                        // compute the hexadecimal result size as 2 * source length
    emitter.instruction("jc __rt_bin2hex_size_overflow_linux_x86_64");          // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the hexadecimal result
    emitter.instruction("mov r9, rax");                                         // compute the destination pointer at the reserved result start
    emitter.instruction("mov r10, r9");                                         // preserve the hexadecimal string start pointer for the return value
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // copy the source byte count into a decrementing loop counter
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // copy the source pointer into a cursor register for byte-by-byte reads

    emitter.label("__rt_bin2hex_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every source byte has been converted to two hex characters
    emitter.instruction("je __rt_bin2hex_done_linux_x86_64");                   // finish when the binary source string has been fully consumed
    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load the next source byte and widen it for nibble extraction
    emitter.instruction("add rsi, 1");                                          // advance the source cursor after consuming one byte
    emitter.instruction("sub rcx, 1");                                          // record that one source byte has been consumed

    emitter.instruction("mov edx, eax");                                        // copy the source byte into a scratch register for the high nibble
    emitter.instruction("shr edx, 4");                                          // isolate the high nibble from the source byte
    emitter.instruction("cmp edx, 10");                                         // decide whether the high nibble maps to 0-9 or a-f
    emitter.instruction("jge __rt_bin2hex_hi_af_linux_x86_64");                 // branch when the high nibble must be rendered as a-f
    emitter.instruction("add edx, 48");                                         // map high nibble 0-9 to ASCII '0'-'9'
    emitter.instruction("jmp __rt_bin2hex_hi_store_linux_x86_64");              // skip the a-f conversion once the numeric digit is ready
    emitter.label("__rt_bin2hex_hi_af_linux_x86_64");
    emitter.instruction("add edx, 87");                                         // map high nibble 10-15 to ASCII 'a'-'f'
    emitter.label("__rt_bin2hex_hi_store_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r9], dl");                               // write the high nibble as the first hexadecimal character
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing the first hex character

    emitter.instruction("mov edx, eax");                                        // copy the source byte into a scratch register for the low nibble
    emitter.instruction("and edx, 15");                                         // isolate the low nibble from the source byte
    emitter.instruction("cmp edx, 10");                                         // decide whether the low nibble maps to 0-9 or a-f
    emitter.instruction("jge __rt_bin2hex_lo_af_linux_x86_64");                 // branch when the low nibble must be rendered as a-f
    emitter.instruction("add edx, 48");                                         // map low nibble 0-9 to ASCII '0'-'9'
    emitter.instruction("jmp __rt_bin2hex_lo_store_linux_x86_64");              // skip the a-f conversion once the numeric digit is ready
    emitter.label("__rt_bin2hex_lo_af_linux_x86_64");
    emitter.instruction("add edx, 87");                                         // map low nibble 10-15 to ASCII 'a'-'f'
    emitter.label("__rt_bin2hex_lo_store_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r9], dl");                               // write the low nibble as the second hexadecimal character
    emitter.instruction("add r9, 1");                                           // advance the destination cursor after writing the second hex character
    emitter.instruction("jmp __rt_bin2hex_loop_linux_x86_64");                  // continue converting subsequent source bytes

    emitter.label("__rt_bin2hex_done_linux_x86_64");
    emitter.instruction("mov rax, r10");                                        // return the hexadecimal string start pointer in the standard x86_64 string result register
    emitter.instruction("mov rdx, r9");                                         // copy the destination cursor into the length scratch register
    emitter.instruction("sub rdx, r10");                                        // compute the hexadecimal string length from the written byte count
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the bin2hex spill slots before returning the hexadecimal string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the hexadecimal string
    emitter.instruction("ret");                                                 // return the hexadecimal string through the standard x86_64 string result registers

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_bin2hex_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
