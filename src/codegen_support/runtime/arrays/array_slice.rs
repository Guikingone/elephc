//! Purpose:
//! Emits the `__rt_array_slice` runtime helper assembly for array slice.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Array helpers operate on runtime array headers and element cells; mutations must respect capacity and COW contracts.
//! - The slice window is normalized by the shared `slice_bounds` prologue, so the copy loop always
//!   runs over a non-negative element count that lies inside the source payload.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::runtime::arrays::slice_bounds::emit_slice_bounds;

/// Emits the `__rt_array_slice` runtime helper for ARM64.
/// Extracts a contiguous slice from an integer array, returning a new array.
///
/// # ABI (ARM64)
/// - Input: x0 = source array pointer, x1 = `$offset`, x2 = `$length`,
///   x3 = 1 when `$length` was supplied, 0 when it was omitted or `null`
/// - Output: x0 = pointer to newly allocated sliced array
///
/// # Behavior
/// - Offset/length normalization is delegated to `emit_slice_bounds`, which applies PHP's rules:
///   negative offsets count from the end, an omitted length runs to the end, and a negative length
///   stops that many elements before the end (clamped to an empty result).
/// - Calls `__rt_array_new` to allocate the result array
pub fn emit_array_slice(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_slice_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_slice ---");
    emitter.label_global("__rt_array_slice");

    // -- set up stack frame --
    emitter.instruction("sub sp, sp, #64");                                     // allocate 64 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #48]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #48");                                    // set up new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save source array pointer

    // -- normalize the requested slice window against PHP's offset/length rules --
    emit_slice_bounds(emitter, "__rt_array_slice");
    emitter.instruction("str x1, [sp, #16]");                                   // save computed offset
    emitter.instruction("str x2, [sp, #24]");                                   // save computed slice length

    // -- create new array --
    emitter.instruction("mov x0, x2");                                          // x0 = capacity = slice length
    emitter.instruction("mov x1, #8");                                          // x1 = elem_size = 8 (integers)
    emitter.instruction("bl __rt_array_new");                                   // allocate new array
    emitter.instruction("str x0, [sp, #32]");                                   // save new array pointer

    // -- copy slice elements --
    emitter.instruction("ldr x1, [sp, #0]");                                    // x1 = source array pointer
    emitter.instruction("add x2, x1, #24");                                     // x2 = source data base
    emitter.instruction("ldr x3, [sp, #16]");                                   // x3 = offset
    emitter.instruction("ldr x4, [sp, #24]");                                   // x4 = slice length
    emitter.instruction("add x5, x0, #24");                                     // x5 = dest data base
    emitter.instruction("mov x6, #0");                                          // x6 = i = 0

    emitter.label("__rt_array_slice_copy");
    emitter.instruction("cmp x6, x4");                                          // compare i with slice length
    emitter.instruction("b.ge __rt_array_slice_done");                          // if done, finish up
    emitter.instruction("add x7, x3, x6");                                      // x7 = offset + i (source index)
    emitter.instruction("ldr x8, [x2, x7, lsl #3]");                            // x8 = source[offset + i]
    emitter.instruction("str x8, [x5, x6, lsl #3]");                            // dest[i] = source[offset + i]
    emitter.instruction("add x6, x6, #1");                                      // i += 1
    emitter.instruction("b __rt_array_slice_copy");                             // continue loop

    // -- set length and return --
    emitter.label("__rt_array_slice_done");
    emitter.instruction("ldr x0, [sp, #32]");                                   // x0 = new array pointer
    emitter.instruction("ldr x9, [sp, #24]");                                   // x9 = slice length
    emitter.instruction("str x9, [x0]");                                        // set new array length
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #64");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return with x0 = sliced array
}

/// Emits the `__rt_array_slice` runtime helper for x86_64 Linux.
///
/// Same slice semantics as the ARM64 variant; only the System V register encoding differs:
/// `rdi` = source array pointer, `rsi` = `$offset`, `rdx` = `$length`, `rcx` = 1 when a `$length`
/// was supplied and 0 when it was omitted or `null`; the sliced array is returned in `rax`.
fn emit_array_slice_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_slice ---");
    emitter.label_global("__rt_array_slice");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving scalar slice spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the source indexed-array pointer, computed offset, slice length, and result pointer
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the scalar slice bookkeeping while keeping nested constructor calls 16-byte aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the source indexed-array pointer across slice-length normalization and result-array construction

    // -- normalize the requested slice window against PHP's offset/length rules --
    emit_slice_bounds(emitter, "__rt_array_slice");
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // preserve the normalized slice offset across the destination indexed-array constructor call
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // preserve the clamped slice length across the destination indexed-array constructor call
    emitter.instruction("mov rdi, rdx");                                        // pass the clamped slice length as the destination indexed-array capacity to the shared constructor
    emitter.instruction("mov rsi, 8");                                          // request 8-byte scalar payload slots for the destination indexed array
    emitter.instruction("call __rt_array_new");                                 // allocate the destination indexed array through the shared x86_64 constructor
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the destination indexed-array pointer across the scalar slice copy loop
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the source indexed-array pointer after the constructor clobbered caller-saved registers
    emitter.instruction("lea r10, [r10 + 24]");                                 // compute the first scalar payload slot address in the source indexed array
    emitter.instruction("mov r11, QWORD PTR [rbp - 32]");                       // reload the destination indexed-array pointer before seeding the slice-copy loop
    emitter.instruction("lea r11, [r11 + 24]");                                 // compute the first scalar payload slot address in the destination indexed array
    emitter.instruction("mov r8, QWORD PTR [rbp - 16]");                        // reload the normalized slice offset before seeding the source index for the copy loop
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // reload the clamped slice length before testing whether there is any scalar payload to copy
    emitter.instruction("xor ecx, ecx");                                        // initialize the destination slice index to the first scalar payload slot in the destination indexed array

    emitter.label("__rt_array_slice_copy_x86");
    emitter.instruction("cmp rcx, r9");                                         // compare the destination slice index against the clamped slice length
    emitter.instruction("jge __rt_array_slice_done_x86");                       // finish once every requested scalar payload has been copied into the destination indexed array
    emitter.instruction("mov rax, QWORD PTR [r10 + r8 * 8]");                   // load the current scalar payload from the normalized source slice position
    emitter.instruction("mov QWORD PTR [r11 + rcx * 8], rax");                  // store that scalar payload into the next destination indexed-array slot
    emitter.instruction("add r8, 1");                                           // advance the normalized source slice index to the next scalar payload slot
    emitter.instruction("add rcx, 1");                                          // advance the destination slice index after copying one scalar payload
    emitter.instruction("jmp __rt_array_slice_copy_x86");                       // continue copying until the destination indexed array holds the full scalar slice

    emitter.label("__rt_array_slice_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // reload the destination indexed-array pointer before publishing its logical length
    emitter.instruction("mov r9, QWORD PTR [rbp - 24]");                        // reload the clamped slice length so the destination indexed-array header can report the copied payload count
    emitter.instruction("mov QWORD PTR [rax], r9");                             // store the clamped slice length as the destination indexed-array logical length
    emitter.instruction("add rsp, 32");                                         // release the scalar slice spill slots before returning to the caller
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the scalar slice helper completes
    emitter.instruction("ret");                                                 // return the destination indexed-array pointer in rax
}
