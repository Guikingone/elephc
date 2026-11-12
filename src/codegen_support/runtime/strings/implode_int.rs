//! Purpose:
//! Emits the `__rt_implode_int` and `__rt_implode_float` runtime helpers (implode over int / float lists).
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits runtime helper for PHP `implode()` with integer array elements.
/// Dispatches to x86_64 variant; ARM64 uses the main implementation below.
/// ABI: x1/x2=glue_ptr/glue_len, x3=array_ptr → x1=result_ptr, x2=result_len.
/// Uses the shared concat buffer; updates `_concat_off` to reflect written bytes.
/// Side effects: writes to concat_buf, advances concat_off global.
pub fn emit_implode_int(emitter: &mut Emitter) {
    emit_implode_scalar(emitter, false);
}

/// Emits the runtime helper for PHP `implode()` with float array elements (`__rt_implode_float`).
/// Same ABI as `__rt_implode_int`; each element is converted with `__rt_ftoa`, PHP's
/// float-to-string conversion.
pub fn emit_implode_float(emitter: &mut Emitter) {
    emit_implode_scalar(emitter, true);
}

/// Emits `__rt_implode_int` or `__rt_implode_float`, which differ only in how one element is
/// loaded and converted to a string.
fn emit_implode_scalar(emitter: &mut Emitter, float: bool) {
    let p = if float { "__rt_implode_float" } else { "__rt_implode_int" };
    if emitter.target.arch == Arch::X86_64 {
        emit_implode_int_linux_x86_64(emitter, p, float);
        return;
    }

    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", p));
    emitter.label_global(p);

    // -- set up stack frame (80 bytes) --
    emitter.instruction("sub sp, sp, #80");                                     // allocate 80 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #64]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #64");                                    // establish new frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save glue string ptr and length
    emitter.instruction("str x3, [sp, #16]");                                   // save array pointer

    // -- get concat_buf write position --
    crate::codegen_support::abi::emit_symbol_address(emitter, "x6", "_concat_off");
    emitter.instruction("ldr x8, [x6]");                                        // load current write offset
    crate::codegen_support::abi::emit_symbol_address(emitter, "x7", "_concat_buf");
    emitter.instruction("add x9, x7, x8");                                      // compute destination pointer
    emitter.instruction("str x9, [sp, #24]");                                   // save result start pointer
    emitter.instruction("str x6, [sp, #32]");                                   // save offset variable address
    emitter.instruction("str x9, [sp, #40]");                                   // save current dest pointer

    // -- load array length and initialize index --
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload array pointer
    emitter.instruction("ldr x10, [x3]");                                       // load array element count
    emitter.instruction("str x10, [sp, #48]");                                  // save element count
    emitter.instruction("str xzr, [sp, #56]");                                  // initialize element index = 0

    // -- main loop: join elements with glue --
    emitter.label(&format!("{}_loop", p));
    emitter.instruction("ldr x11, [sp, #56]");                                  // load current element index
    emitter.instruction("ldr x10, [sp, #48]");                                  // load element count
    emitter.instruction("cmp x11, x10");                                        // check if all elements processed
    emitter.instruction(&format!("b.ge {}_done", p));                           // if done, finalize result

    // -- insert glue before element (skip for first element) --
    emitter.instruction(&format!("cbz x11, {}_elem", p));                       // skip glue before first element
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload glue ptr and length
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload current dest pointer
    emitter.instruction("mov x12, x2");                                         // copy glue length as counter
    emitter.label(&format!("{}_glue", p));
    emitter.instruction(&format!("cbz x12, {}_elem", p));                       // if no glue bytes remain, copy element
    emitter.instruction("ldrb w13, [x1], #1");                                  // load glue byte, advance glue ptr
    emitter.instruction("strb w13, [x9], #1");                                  // store to dest, advance dest ptr
    emitter.instruction("sub x12, x12, #1");                                    // decrement glue byte counter
    emitter.instruction(&format!("b {}_glue", p));                              // continue copying glue

    // -- convert current integer element to string via itoa --
    emitter.label(&format!("{}_elem", p));
    emitter.instruction("str x9, [sp, #40]");                                   // save updated dest pointer
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload array pointer
    emitter.instruction("ldr x11, [sp, #56]");                                  // reload current element index
    emitter.instruction("add x3, x3, #24");                                     // skip 24-byte array header to reach data
    if float {
        // `__rt_ftoa` writes at `_concat_off` and advances it by exactly the digits, so park
        // the cursor 32 bytes past the output cursor: the copy back then never overlaps.
        emitter.instruction("ldr x9, [sp, #40]");                               // reload the output cursor
        emitter.instruction("add x9, x9, #32");                                 // conversion scratch starts past the output
        crate::codegen_support::abi::emit_symbol_address(emitter, "x7", "_concat_buf");
        emitter.instruction("sub x9, x9, x7");                                  // as a concat-buffer offset
        emitter.instruction("ldr x6, [sp, #32]");                               // _concat_off address
        emitter.instruction("str x9, [x6]");                                    // point the conversion past the output
        emitter.instruction("ldr d0, [x3, x11, lsl #3]");                       // load float element at index (8 bytes each)
        emitter.instruction("bl __rt_ftoa");                                    // convert float to string → x1=ptr, x2=len
    } else {
        emitter.instruction("ldr x0, [x3, x11, lsl #3]");                       // load integer element at index (8 bytes each)
        emitter.instruction("bl __rt_itoa");                                    // convert integer to string → x1=ptr, x2=len
    }

    // -- copy itoa result bytes to output --
    emitter.instruction("ldr x9, [sp, #40]");                                   // reload dest pointer
    emitter.instruction("mov x12, x2");                                         // copy string length as counter
    emitter.label(&format!("{}_copy", p));
    emitter.instruction(&format!("cbz x12, {}_next", p));                       // if no bytes remain, move to next element
    emitter.instruction("ldrb w13, [x1], #1");                                  // load string byte, advance src ptr
    emitter.instruction("strb w13, [x9], #1");                                  // store to dest, advance dest ptr
    emitter.instruction("sub x12, x12, #1");                                    // decrement byte counter
    emitter.instruction(&format!("b {}_copy", p));                              // continue copying string

    // -- advance to next element --
    emitter.label(&format!("{}_next", p));
    emitter.instruction("str x9, [sp, #40]");                                   // save updated dest pointer
    emitter.instruction("ldr x11, [sp, #56]");                                  // reload element index
    emitter.instruction("add x11, x11, #1");                                    // increment element index
    emitter.instruction("str x11, [sp, #56]");                                  // save updated index
    emitter.instruction(&format!("b {}_loop", p));                              // process next element

    // -- finalize: compute result length and update concat_off --
    emitter.label(&format!("{}_done", p));
    emitter.instruction("ldr x9, [sp, #40]");                                   // load final dest pointer
    emitter.instruction("ldr x1, [sp, #24]");                                   // load result start pointer
    emitter.instruction("sub x2, x9, x1");                                      // result length = dest_end - dest_start
    emitter.instruction("ldr x6, [sp, #32]");                                   // load offset variable address
    if float {
        crate::codegen_support::abi::emit_symbol_address(emitter, "x7", "_concat_buf");
        emitter.instruction("sub x8, x9, x7");                                  // concat_off = end of the joined string
    } else {
        emitter.instruction("ldr x8, [x6]");                                    // load current concat_off
        emitter.instruction("add x8, x8, x2");                                  // advance offset by result length
    }
    emitter.instruction("str x8, [x6]");                                        // store updated concat_off

    // -- restore frame and return --
    emitter.instruction("ldp x29, x30, [sp, #64]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #80");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return to caller
}

/// Emits `__rt_implode_int` runtime helper for Linux x86_64.
/// ABI: rdi/rsi=glue_ptr/glue_len, rdx=array_ptr → rax=result_ptr, rdx=result_len.
/// Copies glue and each integer element (converted via `__rt_itoa`) into the shared concat buffer.
/// Advances `_concat_off` by the total bytes written.
/// Stack frame: 64 bytes of spill slots for glue, array, destination cursor, length, and loop index.
fn emit_implode_int_linux_x86_64(emitter: &mut Emitter, p: &str, float: bool) {
    emitter.blank();
    emitter.comment(&format!("--- runtime: {} ---", p));
    emitter.label_global(p);

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving integer-implode spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for glue, array, concat-buffer bookkeeping, and the loop cursor
    emitter.instruction("sub rsp, 64");                                         // reserve aligned spill slots for glue, array, concat destination, array length, and loop index
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the glue string pointer across integer conversion and copy helper calls
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve the glue string length across integer conversion and copy helper calls
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // preserve the indexed-array pointer across integer conversion and copy helper calls
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // load the current concat-buffer write offset before materializing the implode output start pointer
    crate::codegen_support::abi::emit_symbol_address(emitter, "r10", "_concat_buf");
    emitter.instruction("lea r10, [r10 + r9]");                                 // compute the current concat-buffer destination pointer for the integer implode output
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // preserve the implode result start pointer so the final string result can reference the copied bytes
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve the current concat-buffer destination cursor across glue emission and integer string copies
    emitter.instruction("mov r11, QWORD PTR [rdx]");                            // load the indexed-array logical length once before entering the integer implode loop
    emitter.instruction("mov QWORD PTR [rbp - 48], r11");                       // preserve the indexed-array logical length for the loop termination check
    emitter.instruction("mov QWORD PTR [rbp - 56], 0");                         // initialize the indexed-array loop cursor to the first integer element

    emitter.label(&format!("{}_loop", p));
    emitter.instruction("mov r11, QWORD PTR [rbp - 56]");                       // reload the current indexed-array loop cursor before deciding whether integer implode is complete
    emitter.instruction("cmp r11, QWORD PTR [rbp - 48]");                       // compare the current indexed-array loop cursor against the saved logical length
    emitter.instruction(&format!("jae {}_done", p));                            // stop once every indexed-array integer element has been copied into the concat buffer
    emitter.instruction("test r11, r11");                                       // check whether the current integer element is the first one in the indexed array
    emitter.instruction(&format!("jz {}_elem", p));                             // skip glue emission before converting the first integer element
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the glue string pointer before copying the separator bytes
    emitter.instruction("mov r9, QWORD PTR [rbp - 16]");                        // reload the glue string length before copying the separator bytes
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current concat-buffer destination cursor before copying the separator bytes

    emitter.label(&format!("{}_glue", p));
    emitter.instruction("test r9, r9");                                         // check whether every glue byte has already been copied into the concat buffer
    emitter.instruction(&format!("jz {}_glue_done", p));                        // continue with integer conversion once the glue string has been fully copied
    emitter.instruction("mov r11b, BYTE PTR [r8]");                             // load one byte from the glue string before advancing the source pointer
    emitter.instruction("mov BYTE PTR [r10], r11b");                            // store one separator byte into the concat buffer before advancing the destination pointer
    emitter.instruction("add r8, 1");                                           // advance the glue string source pointer after copying one separator byte
    emitter.instruction("add r10, 1");                                          // advance the concat-buffer destination pointer after storing one separator byte
    emitter.instruction("sub r9, 1");                                           // decrement the remaining glue byte count after copying one separator byte
    emitter.instruction(&format!("jmp {}_glue", p));                            // continue copying separator bytes until the glue string is exhausted

    emitter.label(&format!("{}_glue_done", p));
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve the concat-buffer destination cursor after copying the separator bytes

    emitter.label(&format!("{}_elem", p));
    emitter.instruction("mov r11, QWORD PTR [rbp - 56]");                       // reload the current indexed-array loop cursor before locating the next integer element slot
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the indexed-array pointer before addressing the current integer slot
    if float {
        // `__rt_ftoa` writes at `_concat_off` and advances it by exactly the digits, so park
        // the cursor 32 bytes past the output cursor: the copy back then never overlaps.
        emitter.instruction("mov r9, QWORD PTR [rbp - 40]");                    // reload the output cursor
        emitter.instruction("add r9, 32");                                      // conversion scratch starts past the output
        crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_buf");
        emitter.instruction("sub r9, r8");                                      // as a concat-buffer offset
        crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
        emitter.instruction("mov QWORD PTR [r8], r9");                          // point the conversion past the output
        emitter.instruction("movsd xmm0, QWORD PTR [r10 + r11 * 8 + 24]");      // load the current float element into the float-to-string helper input register
        emitter.instruction("call __rt_ftoa");                                  // convert the float element with PHP's float-to-string rules
    } else {
        emitter.instruction("mov rax, QWORD PTR [r10 + r11 * 8 + 24]");         // load the current indexed-array integer payload into the integer-to-string helper input register
        emitter.instruction("call __rt_itoa");                                  // convert the current indexed-array integer element into a concat-buffer-backed decimal string
    }
    emitter.instruction("mov r8, rax");                                         // preserve the decimal string pointer returned by the integer-to-string helper before copying bytes
    emitter.instruction("mov r9, rdx");                                         // preserve the decimal string length returned by the integer-to-string helper before copying bytes
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the current concat-buffer destination cursor before copying the converted decimal bytes

    emitter.label(&format!("{}_copy", p));
    emitter.instruction("test r9, r9");                                         // check whether every converted decimal byte has already been copied into the concat buffer
    emitter.instruction(&format!("jz {}_next", p));                             // advance to the next indexed-array integer once the current decimal string is fully copied
    emitter.instruction("mov r11b, BYTE PTR [r8]");                             // load one byte from the converted decimal string before advancing the source pointer
    emitter.instruction("mov BYTE PTR [r10], r11b");                            // store one byte from the converted decimal string into the concat buffer
    emitter.instruction("add r8, 1");                                           // advance the converted decimal string source pointer after copying one byte
    emitter.instruction("add r10, 1");                                          // advance the concat-buffer destination pointer after storing one byte
    emitter.instruction("sub r9, 1");                                           // decrement the remaining converted decimal byte count after copying one byte
    emitter.instruction(&format!("jmp {}_copy", p));                            // continue copying bytes from the converted decimal string until it is exhausted

    emitter.label(&format!("{}_next", p));
    emitter.instruction("mov QWORD PTR [rbp - 40], r10");                       // preserve the concat-buffer destination cursor after copying the current converted decimal string
    emitter.instruction("add QWORD PTR [rbp - 56], 1");                         // advance the indexed-array loop cursor to the next integer element
    emitter.instruction(&format!("jmp {}_loop", p));                            // continue joining converted integer elements into the concat buffer

    emitter.label(&format!("{}_done", p));
    emitter.instruction("mov r10, QWORD PTR [rbp - 40]");                       // reload the final concat-buffer destination cursor to compute the joined string length
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the implode result start pointer before computing the joined string length
    emitter.instruction("mov rdx, r10");                                        // copy the final concat-buffer destination cursor before subtracting the result start pointer
    emitter.instruction("sub rdx, rax");                                        // compute the joined string length as dest_end - dest_start
    crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_concat_off");
    if float {
        crate::codegen_support::abi::emit_symbol_address(emitter, "r9", "_concat_buf");
        emitter.instruction("sub r10, r9");                                     // concat_off = end of the joined string
        emitter.instruction("mov r9, r10");                                     // new concat-buffer offset
    } else {
        emitter.instruction("mov r9, QWORD PTR [r8]");                          // reload the current concat-buffer write offset after the integer-to-string helper scratch allocations
        emitter.instruction("add r9, rdx");                                     // advance the concat-buffer write offset by the joined string length that this integer implode call produced
    }
    emitter.instruction("mov QWORD PTR [r8], r9");                              // persist the updated concat-buffer write offset after writing the integer implode output bytes
    emitter.instruction("add rsp, 64");                                         // release the integer-implode spill slots before returning the joined string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the joined string
    emitter.instruction("ret");                                                 // return the joined string in the standard x86_64 string result registers
}