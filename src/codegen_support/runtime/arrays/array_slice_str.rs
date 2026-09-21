//! Purpose:
//! Emits `__rt_array_slice_str`, the `array_slice()` runtime helper for indexed arrays whose
//! payload slots are string `{pointer, length}` pairs.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - The EIR lowering of `array_slice()` in `crate::codegen::lower_inst::builtins::arrays`.
//!
//! Key details:
//! - Indexed string arrays use 16-byte slots (`__rt_array_new(n, 16)`), not the 8-byte slots the
//!   scalar and refcounted helpers move. `array_slice_runtime_helper` has always named this
//!   helper for a string element type, and until now nothing emitted it: any program that sliced
//!   an `array<string>` failed to LINK, with the symbol as the only clue.
//! - Each copied string is DUPLICATED through `__rt_str_persist`. Unlike `array_splice`, which
//!   removes its window and can transfer ownership, `array_slice` leaves the source array intact,
//!   so the source keeps owning its payloads and the result needs its own — an indexed string
//!   array frees every slot in `__rt_array_free_deep`, and sharing one would double-free it.
//! - The window is normalized by the shared `emit_slice_bounds` prologue, so the copy count is
//!   always within the source payload.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::arrays::slice_bounds::emit_slice_bounds;

/// Emits the `__rt_array_slice_str` runtime helper for the active target.
///
/// ## ARM64 ABI
/// - **Input**: `x0` = source indexed array, `x1` = `$offset`, `x2` = `$length`, `x3` = 1 when a
///   `$length` was supplied and 0 when it was omitted or `null`
/// - **Output**: `x0` = a freshly allocated 16-byte-slot indexed array holding the slice
///
/// ## x86_64 ABI
/// - **Input**: `rdi`, `rsi`, `rdx`, `rcx` with the same meaning
/// - **Output**: `rax` = the sliced indexed array
pub fn emit_array_slice_str(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_slice_str_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_slice_str ---");
    emitter.label_global("__rt_array_slice_str");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #64");                                     // reserve spill slots for the source, window and destination
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address across the persist calls
    emitter.instruction("add x29, sp, #48");                                    // establish a frame pointer for this helper
    emitter.instruction("str x0, [sp, #0]");                                    // preserve the source indexed-array pointer

    // -- normalize the requested slice window against PHP's offset/length rules --
    emit_slice_bounds(emitter, "__rt_array_slice_str");
    emitter.instruction("str x1, [sp, #16]");                                   // preserve the normalized slice offset
    emitter.instruction("str x2, [sp, #24]");                                   // preserve the clamped slice length

    // -- allocate the destination with STRING slots --
    emitter.instruction("mov x0, x2");                                          // capacity = clamped slice length
    emitter.instruction("mov x1, #16");                                         // 16-byte {pointer, length} payload slots
    emitter.instruction("bl __rt_array_new");                                   // allocate the destination indexed array
    emitter.instruction("str x0, [sp, #32]");                                   // preserve the destination pointer across the copy loop
    emitter.instruction("mov x6, #0");                                          // seed the copy index
    emitter.instruction("str x6, [sp, #40]");                                   // persist the copy index across the persist calls

    // -- copy each string slot, duplicating its payload --
    emitter.label("__rt_array_slice_str_copy");
    emitter.instruction("ldr x6, [sp, #40]");                                   // reload the copy index
    emitter.instruction("ldr x4, [sp, #24]");                                   // reload the clamped slice length
    emitter.instruction("cmp x6, x4");                                          // has every requested slot been copied?
    emitter.instruction("b.ge __rt_array_slice_str_done");                      // yes, publish the destination length
    emitter.instruction("ldr x5, [sp, #0]");                                    // reload the source indexed-array pointer
    emitter.instruction("add x5, x5, #24");                                     // compute the source string payload base address
    emitter.instruction("ldr x3, [sp, #16]");                                   // reload the normalized slice offset
    emitter.instruction("add x7, x3, x6");                                      // compute the source slot index for this element
    emitter.instruction("lsl x7, x7, #4");                                      // scale it by the 16-byte string slot size
    emitter.instruction("add x5, x5, x7");                                      // compute this element's source string slot address
    emitter.instruction("ldr x1, [x5]");                                        // load the borrowed source string pointer
    emitter.instruction("ldr x2, [x5, #8]");                                    // load the borrowed source string length
    emitter.instruction("bl __rt_str_persist");                                 // duplicate it so the slice owns its own bytes
    emitter.instruction("ldr x6, [sp, #40]");                                   // reload the copy index after the persist call
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the destination indexed-array pointer
    emitter.instruction("add x5, x0, #24");                                     // compute the destination string payload base address
    emitter.instruction("lsl x7, x6, #4");                                      // scale the destination slot index by the string slot size
    emitter.instruction("add x5, x5, x7");                                      // compute this element's destination string slot address
    emitter.instruction("str x1, [x5]");                                        // store the owned string pointer
    emitter.instruction("str x2, [x5, #8]");                                    // store the matching owned string length
    emitter.instruction("add x6, x6, #1");                                      // advance to the next element
    emitter.instruction("str x6, [sp, #40]");                                   // persist the updated copy index
    emitter.instruction("b __rt_array_slice_str_copy");                         // continue copying string slots

    // -- publish the destination length and return --
    emitter.label("__rt_array_slice_str_done");
    emitter.instruction("ldr x0, [sp, #32]");                                   // reload the destination indexed-array pointer
    emitter.instruction("ldr x9, [sp, #24]");                                   // reload the clamped slice length
    emitter.instruction("str x9, [x0]");                                        // publish it as the destination logical length
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // release the spill slots
    emitter.instruction("ret");                                                 // return the sliced indexed array
}

/// Emits the `__rt_array_slice_str` runtime helper for x86_64.
///
/// Same semantics as the ARM64 variant; only the System V register encoding differs.
fn emit_array_slice_str_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_slice_str ---");
    emitter.label_global("__rt_array_slice_str");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 48");                                         // reserve aligned spill slots for source, window, destination and index
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the source indexed-array pointer

    // -- normalize the requested slice window against PHP's offset/length rules --
    emit_slice_bounds(emitter, "__rt_array_slice_str");
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve the normalized slice offset
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // preserve the clamped slice length
    emitter.instruction("mov rdi, rdx");                                        // capacity = clamped slice length
    emitter.instruction("mov rsi, 16");                                         // 16-byte {pointer, length} payload slots
    emitter.instruction("call __rt_array_new");                                 // allocate the destination indexed array
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the destination pointer across the copy loop
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // seed the copy index

    emitter.label("__rt_array_slice_str_copy_x86");
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // reload the copy index
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // reload the clamped slice length
    emitter.instruction("cmp rcx, r9");                                         // has every requested slot been copied?
    emitter.instruction("jge __rt_array_slice_str_done_x86");                   // yes, publish the destination length
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer
    emitter.instruction("lea r10, [r10 + 24]");                                 // compute the source string payload base address
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // reload the normalized slice offset
    emitter.instruction("add r8, rcx");                                         // compute the source slot index for this element
    emitter.instruction("shl r8, 4");                                           // scale it by the 16-byte string slot size
    emitter.instruction("add r10, r8");                                         // compute this element's source string slot address
    emitter.instruction("mov rsi, QWORD PTR [r10]");                            // load the borrowed source string pointer
    emitter.instruction("mov rdx, QWORD PTR [r10 + 8]");                        // load the borrowed source string length
    emitter.instruction("call __rt_str_persist");                               // duplicate it so the slice owns its own bytes
    emitter.instruction("mov rcx, QWORD PTR [rbp - 40]");                       // reload the copy index after the persist call
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                       // reload the destination indexed-array pointer
    emitter.instruction("lea r11, [r11 + 24]");                                 // compute the destination string payload base address
    emitter.instruction("mov r8, rcx");                                         // copy the destination slot index
    emitter.instruction("shl r8, 4");                                           // scale it by the string slot size
    emitter.instruction("add r11, r8");                                         // compute this element's destination string slot address
    emitter.instruction("mov QWORD PTR [r11], rsi");                            // store the owned string pointer
    emitter.instruction("mov QWORD PTR [r11 + 8], rdx");                        // store the matching owned string length
    emitter.instruction("add rcx, 1");                                          // advance to the next element
    emitter.instruction("mov QWORD PTR [rbp - 40], rcx");                       // persist the updated copy index
    emitter.instruction("jmp __rt_array_slice_str_copy_x86");                   // continue copying string slots

    emitter.label("__rt_array_slice_str_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the destination indexed-array pointer
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // reload the clamped slice length
    emitter.instruction("mov QWORD PTR [rax], r9");                             // publish it as the destination logical length
    emitter.instruction("add rsp, 48");                                         // release the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the sliced indexed array in rax
}
