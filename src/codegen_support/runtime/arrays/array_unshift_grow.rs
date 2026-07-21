//! Purpose:
//! Emits the `__rt_array_unshift_grow` runtime helper: a capacity-aware,
//! COW-safe single-value prepend for indexed scalar (`Int`/`Bool`) arrays,
//! backing the EIR `array_unshift()` lowering.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - `crate::codegen::lower_inst::builtins::arrays::unshift` (the `array_unshift()`
//!   EIR lowering), once per variadic value, in REVERSE source order (so the
//!   first-listed value ends up first: prepending `$vN` then `...` then `$v0`
//!   last leaves `$v0` at index 0, matching PHP's `array_unshift($a, 1, 2)` →
//!   `[1, 2, ...old]`).
//!
//! Key details:
//! - DISTINCT from the legacy, frozen `__rt_array_unshift` (still emitted for
//!   `src/codegen/builtins/arrays/array_unshift.rs`, the legacy AST backend):
//!   that helper returns the new COUNT and never grows capacity (a pre-existing,
//!   out-of-scope memory-safety gap in the frozen legacy path). This helper
//!   returns the (possibly reallocated) ARRAY POINTER instead, so the EIR
//!   lowering can write the new pointer back into the by-ref `$array` slot —
//!   required for `$b =& $a; array_unshift($a, ...)` to keep `$b` in sync.
//! - Mirrors `__rt_array_push_int`'s COW (`__rt_array_ensure_unique`) and
//!   freshly-empty-array shape-specialization logic exactly, then grows
//!   (`__rt_array_grow`, which always yields `>= old_capacity + 1` free slots)
//!   before shifting every existing element one slot to the right and writing
//!   the new value into slot 0.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_array_unshift_grow` for the host target.
///
/// AArch64: input x0=array pointer, x1=value. Output: x0=array pointer (may
/// differ from the input if COW-split and/or grown).
/// x86_64: input rdi=array pointer, rsi=value. Output: rax=array pointer.
pub fn emit_array_unshift_grow(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_unshift_grow_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_unshift_grow ---");
    emitter.label_global("__rt_array_unshift_grow");

    // -- split shared arrays before mutating in place --
    emitter.instruction("sub sp, sp, #32");                                     // allocate 32 bytes on the stack
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish new frame pointer
    emitter.instruction("str x1, [sp, #0]");                                    // save the prepended value across ensure_unique
    emitter.instruction("bl __rt_array_ensure_unique");                         // split shared arrays before the prepend path mutates storage
    emitter.instruction("ldr x1, [sp, #0]");                                    // restore the prepended value after ensure_unique

    // -- specialize freshly empty arrays to pointer-sized scalar slots (mirrors __rt_array_push_int) --
    emitter.instruction("ldr x9, [x0]");                                        // x9 = current array length before first-write specialization
    emitter.instruction("cbnz x9, __rt_array_unshift_grow_shape_ready");        // existing arrays already have their element shape fixed
    emitter.instruction("mov x10, #8");                                         // scalar/refcounted slots are pointer-sized
    emitter.instruction("str x10, [x0, #16]");                                  // elem_size = 8 before any future grow copies live slots
    emitter.instruction("ldr x10, [x0, #-8]");                                  // load packed array metadata to clear stale placeholder tagging
    emitter.instruction("lsr x11, x10, #8");                                    // move the value_type tag into the low bits
    emitter.instruction("and x11, x11, #0x7f");                                 // ignore the persistent COW flag while checking the value_type
    emitter.instruction("cmp x11, #1");                                         // is this the empty-array string fallback tag?
    emitter.instruction("b.eq __rt_array_unshift_grow_shape_clear");            // clear the stale string fallback tag before the first int prepend
    emitter.instruction("cmp x11, #8");                                         // is this the empty-array never placeholder tag (e.g. `$dst = []`)?
    emitter.instruction("b.ne __rt_array_unshift_grow_shape_ready");            // preserve refcounted tags pre-stamped by array_push_refcounted
    emitter.label("__rt_array_unshift_grow_shape_clear");
    emitter.instruction("mov x12, #0x80ff");                                    // keep indexed-array kind and persistent COW metadata only
    emitter.instruction("and x10, x10, x12");                                   // clear the stale placeholder value_type tag for scalar int arrays
    emitter.instruction("str x10, [x0, #-8]");                                  // publish scalar int (value_type 0) metadata before the first prepend
    emitter.label("__rt_array_unshift_grow_shape_ready");

    // -- grow if the array is already at capacity (one __rt_array_grow call always yields room for one more element) --
    emitter.instruction("ldr x9, [x0]");                                        // x9 = current array length
    emitter.instruction("ldr x10, [x0, #8]");                                   // x10 = array capacity
    emitter.instruction("cmp x9, x10");                                         // is the array full?
    emitter.instruction("b.lt __rt_array_unshift_grow_shift");                  // room available: skip growth
    emitter.instruction("bl __rt_array_grow");                                  // double array capacity (min 8) → x0 = new array

    // -- shift every existing element one slot to the right, then insert at index 0 --
    emitter.label("__rt_array_unshift_grow_shift");
    emitter.instruction("ldr x9, [x0]");                                        // x9 = current array length (post-growth if applicable)
    emitter.instruction("add x10, x0, #24");                                    // x10 = base of the data region
    emitter.instruction("sub x11, x9, #1");                                     // x11 = src_index = length - 1 (last live element)

    emitter.label("__rt_array_unshift_grow_shift_loop");
    emitter.instruction("cmp x11, #0");                                         // has src_index gone negative?
    emitter.instruction("b.lt __rt_array_unshift_grow_insert");                 // shifting complete
    emitter.instruction("ldr x12, [x10, x11, lsl #3]");                         // x12 = data[src_index]
    emitter.instruction("add x13, x11, #1");                                    // x13 = dst_index = src_index + 1
    emitter.instruction("str x12, [x10, x13, lsl #3]");                         // data[dst_index] = data[src_index]
    emitter.instruction("sub x11, x11, #1");                                    // src_index -= 1
    emitter.instruction("b __rt_array_unshift_grow_shift_loop");                // continue shifting

    emitter.label("__rt_array_unshift_grow_insert");
    emitter.instruction("ldr x1, [sp, #0]");                                    // reload the prepended value (a growth call above may have clobbered x1)
    emitter.instruction("str x1, [x10]");                                       // data[0] = value
    emitter.instruction("ldr x9, [x0]");                                        // reload length (post-growth if applicable)
    emitter.instruction("add x9, x9, #1");                                      // length += 1
    emitter.instruction("str x9, [x0]");                                        // write updated length back to header

    // -- restore frame and return the (possibly reallocated) array pointer --
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate stack frame
    emitter.instruction("ret");                                                 // return with x0 = array pointer
}

/// Emits the Linux x86_64 `__rt_array_unshift_grow` variant.
fn emit_array_unshift_grow_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_unshift_grow ---");
    emitter.label_global("__rt_array_unshift_grow");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving spill slots
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve an aligned spill slot for the prepended value
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // preserve the prepended value across ensure_unique/grow
    emitter.instruction("call __rt_array_ensure_unique");                       // split shared arrays before the prepend path mutates storage (rdi in, rax out)

    // -- specialize freshly empty arrays to pointer-sized scalar slots (mirrors __rt_array_push_int) --
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // r10 = current array length before first-write specialization
    emitter.instruction("test r10, r10");
    emitter.instruction("jnz __rt_array_unshift_grow_shape_ready_x86_64");      // existing arrays already have their element shape fixed
    emitter.instruction("mov QWORD PTR [rax + 16], 8");                         // elem_size = 8 before any future grow copies live slots
    emitter.instruction("mov r11, QWORD PTR [rax - 8]");                        // load packed array metadata to clear stale placeholder tagging
    emitter.instruction("mov r8, r11");                                         // copy metadata before isolating the value_type tag
    emitter.instruction("shr r8, 8");                                           // move the value_type tag into the low bits
    emitter.instruction("and r8, 0x7f");                                        // ignore the persistent COW flag while checking the value_type
    emitter.instruction("cmp r8, 1");                                           // is this the empty-array string fallback tag?
    emitter.instruction("je __rt_array_unshift_grow_shape_clear_x86_64");       // clear the stale string fallback tag before the first int prepend
    emitter.instruction("cmp r8, 8");                                           // is this the empty-array never placeholder tag (e.g. `$dst = []`)?
    emitter.instruction("jne __rt_array_unshift_grow_shape_ready_x86_64");      // preserve refcounted tags pre-stamped by array_push_refcounted
    emitter.label("__rt_array_unshift_grow_shape_clear_x86_64");
    emitter.instruction("mov r8, 0xffffffff000080ff");                          // preserve heap marker, indexed-array kind, and persistent COW metadata
    emitter.instruction("and r11, r8");                                         // clear the stale placeholder value_type tag for scalar int arrays
    emitter.instruction("mov QWORD PTR [rax - 8], r11");                        // publish scalar int (value_type 0) metadata before the first prepend
    emitter.label("__rt_array_unshift_grow_shape_ready_x86_64");

    // -- grow if the array is already at capacity --
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // r10 = current array length
    emitter.instruction("mov r11, QWORD PTR [rax + 8]");                        // r11 = array capacity
    emitter.instruction("cmp r10, r11");                                        // is the array full?
    emitter.instruction("jl __rt_array_unshift_grow_shift_x86_64");             // room available: skip growth
    emitter.instruction("mov rdi, rax");                                        // pass the unique array pointer to the growth helper
    emitter.instruction("call __rt_array_grow");                                // double array capacity (min 8) → rax = new array

    // -- shift every existing element one slot to the right, then insert at index 0 --
    emitter.label("__rt_array_unshift_grow_shift_x86_64");
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // r10 = current array length (post-growth if applicable)
    emitter.instruction("lea r9, [rax + 24]");                                  // r9 = base of the data region
    emitter.instruction("mov rcx, r10");                                        // rcx = src_index, seeded from the length
    emitter.instruction("sub rcx, 1");                                          // rcx = length - 1 (last live element; -1 for an empty array)

    emitter.label("__rt_array_unshift_grow_shift_loop_x86_64");
    emitter.instruction("cmp rcx, 0");                                          // has src_index gone negative?
    emitter.instruction("jl __rt_array_unshift_grow_insert_x86_64");            // shifting complete
    emitter.instruction("mov r8, QWORD PTR [r9 + rcx * 8]");                    // r8 = data[src_index]
    emitter.instruction("lea rdx, [rcx + 1]");                                  // rdx = dst_index = src_index + 1
    emitter.instruction("mov QWORD PTR [r9 + rdx * 8], r8");                    // data[dst_index] = data[src_index]
    emitter.instruction("dec rcx");                                             // src_index -= 1
    emitter.instruction("jmp __rt_array_unshift_grow_shift_loop_x86_64");       // continue shifting

    emitter.label("__rt_array_unshift_grow_insert_x86_64");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the prepended value (a growth call above may have clobbered rsi)
    emitter.instruction("mov QWORD PTR [r9], r8");                              // data[0] = value
    emitter.instruction("mov r10, QWORD PTR [rax]");                            // reload length (post-growth if applicable)
    emitter.instruction("add r10, 1");                                          // length += 1
    emitter.instruction("mov QWORD PTR [rax], r10");                            // write updated length back to header

    emitter.instruction("add rsp, 16");                                         // release the prepended-value spill slot
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return with rax = array pointer
}
