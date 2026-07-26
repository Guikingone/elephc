//! Purpose:
//! Emits the container-level runtime helpers that back `array_pop()` / `array_shift()`
//! on a boxed Mixed / union receiver: `__rt_mixed_box_raw`, `__rt_indexed_pop`, and
//! `__rt_hash_pop`. They remove-and-return an element IN PLACE, dispatching on the
//! runtime array kind (indexed vs associative hash) so the caller's array is truly
//! mutated (never a throwaway temp copy).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - The generated code from `crate::codegen::lower_inst::builtins::arrays::lower_array_pop`
//!   (dynamic Mixed path), which supplies the (already copy-on-write forced) container
//!   pointer and reboxes the returned container back into the by-ref slot.
//!
//! Key details:
//! - Ownership contract mirrors `__rt_array_unshift_grow`: the caller takes one extra
//!   `__rt_incref` on the unboxed container before calling, so the internal
//!   `__rt_*_ensure_unique` ALWAYS copy-on-write splits; the helper returns a uniquely
//!   owned container (refcount 1 = the caller's synthetic ownership) plus the removed
//!   element boxed as an owned Mixed cell.
//! - Indexed removal uses TRANSFER semantics: the removed slot's payload is adopted by
//!   the returned Mixed cell WITHOUT an extra retain, and the container's logical length
//!   is decremented so `__rt_array_free_deep` (which frees `[0, length)`) never touches
//!   it — leak-free and double-free-free, matching the `[0, length)` free bound.
//! - Hash removal uses INCREF semantics balanced by `__rt_hash_unset`: the tail value is
//!   boxed with a retain and `__rt_hash_unset` then releases the hash's own reference,
//!   so the net owner is the returned box while the entry is unlinked/tombstoned/counted.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_mixed_box_raw`, a minimal "box without retain" helper.
///
/// Allocates a 24-byte boxed Mixed cell `[tag][lo][hi]` and stores the supplied
/// `(tag, lo, hi)` verbatim, taking NO ownership (no incref, no string persist). It is
/// used to adopt a payload whose ownership is being transferred out of an indexed array
/// slot, so the boxed cell becomes the sole owner without inflating the refcount.
///
/// Input:  `x0`/`rax` = value tag, `x1`/`rdi` = value lo word, `x2`/`rsi` = value hi word.
/// Output: `x0`/`rax` = pointer to the freshly allocated boxed Mixed cell.
pub fn emit_mixed_box_raw(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_box_raw_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mixed_box_raw ---");
    emitter.label_global("__rt_mixed_box_raw");
    emitter.instruction("sub sp, sp, #48");                                     // reserve the box-construction frame plus saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the local frame
    emitter.instruction("str x0, [sp, #0]");                                    // save the runtime value tag across the allocation call
    emitter.instruction("stp x1, x2, [sp, #8]");                                // save the payload words that transfer into the boxed cell
    emitter.instruction("mov x0, #24");                                         // Mixed cells store a tag plus two payload words
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the boxed Mixed cell
    emitter.instruction("mov x9, #5");                                          // heap kind 5 = boxed Mixed cell
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the allocation as a Mixed cell in the uniform header
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload the saved runtime value tag
    emitter.instruction("str x10, [x0]");                                       // store the runtime value tag in the Mixed cell
    emitter.instruction("ldp x11, x12, [sp, #8]");                              // reload the transferred payload words
    emitter.instruction("stp x11, x12, [x0, #8]");                              // store the payload words in the Mixed cell
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // release the box-construction frame
    emitter.instruction("ret");                                                 // return the boxed Mixed cell pointer
}

/// Emits the x86_64 Linux variant of `__rt_mixed_box_raw` (SysV AMD64 ABI).
///
/// Input:  `rax` = value tag, `rdi` = value lo word, `rsi` = value hi word.
/// Output: `rax` = pointer to the freshly allocated boxed Mixed cell.
fn emit_mixed_box_raw_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_box_raw ---");
    emitter.label_global("__rt_mixed_box_raw");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before allocating the box
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for tag and payload words
    emitter.instruction("sub rsp, 32");                                         // reserve helper slots for tag, payload lo, and payload hi
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the runtime value tag across the allocation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the payload lo word across the allocation call
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // save the payload hi word across the allocation call
    emitter.instruction("mov rax, 24");                                         // Mixed cells store a tag plus two payload words
    emitter.instruction("call __rt_heap_alloc");                                // allocate the boxed Mixed cell
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // materialize the x86_64 Mixed heap kind word
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the allocation as a boxed Mixed cell
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the runtime value tag
    emitter.instruction("mov QWORD PTR [rax], r10");                            // store the runtime value tag in the Mixed cell
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the payload lo word
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // store the payload lo word in the Mixed cell
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the payload hi word
    emitter.instruction("mov QWORD PTR [rax + 16], r10");                       // store the payload hi word in the Mixed cell
    emitter.instruction("add rsp, 32");                                         // release the helper frame slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the boxed Mixed cell pointer
}

/// Emits `__rt_indexed_pop`, which removes and returns the LAST element of an indexed
/// array in place.
///
/// Copy-on-write splits the (caller-force-increffed) container, boxes the last element
/// as an owned Mixed cell using TRANSFER semantics (no retain), and decrements the
/// logical length so the container's deep free never revisits the removed slot.
///
/// Input:  `x0`/`rdi` = indexed-array pointer (caller holds one synthetic `__rt_incref`).
/// Output: `x0`/`rax` = the unique (copy-on-write split) container pointer,
///         `x1`/`rdx` = the removed element boxed as an owned Mixed cell (boxed null when empty).
pub fn emit_indexed_pop(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_indexed_pop_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: indexed_pop ---");
    emitter.label_global("__rt_indexed_pop");
    // Frame: [sp,#0]=container C, [sp,#8]=removed cell, saved fp/lr at [sp,#32].
    emitter.instruction("sub sp, sp, #48");                                     // reserve the pop frame plus saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the local frame
    emitter.instruction("bl __rt_array_ensure_unique");                         // copy-on-write split; x0 = uniquely owned container
    emitter.instruction("str x0, [sp, #0]");                                    // save the unique container pointer
    emitter.instruction("ldr x9, [x0]");                                        // x9 = current logical length
    emitter.instruction("cbz x9, __rt_indexed_pop_empty");                      // an empty array pops boxed null
    emitter.instruction("sub x9, x9, #1");                                      // x9 = index of the removed last element
    emitter.instruction("ldr x10, [x0, #-8]");                                  // load the packed indexed-array kind word
    emitter.instruction("lsr x10, x10, #8");                                    // move the packed value_type tag into the low bits
    emitter.instruction("and x10, x10, #0x7f");                                 // isolate the value_type without the persistent copy-on-write flag
    emitter.instruction("add x11, x0, #24");                                    // x11 = payload base (skip the 24-byte header)
    emitter.instruction("cmp x10, #1");                                         // string slots use a 16-byte pointer/length pair
    emitter.instruction("b.eq __rt_indexed_pop_string");                       // handle string payloads separately
    emitter.instruction("cmp x10, #7");                                         // slots already holding boxed Mixed cells
    emitter.instruction("b.eq __rt_indexed_pop_boxed");                        // adopt the stored Mixed cell directly
    // -- generic single-word payload (int/float/bool/null/array/hash/object/callable) --
    emitter.instruction("ldr x1, [x11, x9, lsl #3]");                           // load the removed single-word payload
    emitter.instruction("mov x2, #0");                                          // single-word payloads leave the hi word clear
    emitter.instruction("mov x0, x10");                                         // box with the array's runtime value_type tag
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the container to shrink its length
    emitter.instruction("str x9, [x12]");                                       // commit the decremented length before boxing (TRANSFER: drop slot ownership)
    emitter.instruction("bl __rt_mixed_box_raw");                               // adopt the payload into an owned Mixed cell without retaining
    emitter.instruction("str x0, [sp, #8]");                                    // save the removed boxed value
    emitter.instruction("b __rt_indexed_pop_ret");                              // converge on the shared return path
    emitter.label("__rt_indexed_pop_string");
    emitter.instruction("lsl x12, x9, #4");                                     // scale the removed index by the 16-byte string slot size
    emitter.instruction("add x12, x11, x12");                                   // x12 = removed string slot address
    emitter.instruction("ldr x1, [x12]");                                       // x1 = removed string pointer
    emitter.instruction("ldr x2, [x12, #8]");                                   // x2 = removed string length
    emitter.instruction("mov x0, #1");                                          // runtime value tag 1 = string
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the container to shrink its length
    emitter.instruction("str x9, [x12]");                                       // commit the decremented length (TRANSFER the string to the box)
    emitter.instruction("bl __rt_mixed_box_raw");                              // adopt the string pointer without re-persisting it
    emitter.instruction("str x0, [sp, #8]");                                    // save the removed boxed value
    emitter.instruction("b __rt_indexed_pop_ret");                              // converge on the shared return path
    emitter.label("__rt_indexed_pop_boxed");
    emitter.instruction("ldr x0, [x11, x9, lsl #3]");                           // load the stored boxed Mixed cell pointer
    emitter.instruction("cbz x0, __rt_indexed_pop_boxed_null");                 // a null slot pops boxed null
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the container to shrink its length
    emitter.instruction("str x9, [x12]");                                       // commit the decremented length (TRANSFER the cell to the caller)
    emitter.instruction("str x0, [sp, #8]");                                    // the stored cell IS the removed owned value
    emitter.instruction("b __rt_indexed_pop_ret");                             // converge on the shared return path
    emitter.label("__rt_indexed_pop_boxed_null");
    emitter.instruction("ldr x12, [sp, #0]");                                   // reload the container to shrink its length
    emitter.instruction("str x9, [x12]");                                       // still remove the null slot from the logical length
    emitter.instruction("mov x0, #8");                                          // runtime value tag 8 = null
    emitter.instruction("mov x1, #0");                                          // null payload lo word
    emitter.instruction("mov x2, #0");                                          // null payload hi word
    emitter.instruction("bl __rt_mixed_box_raw");                              // box PHP null for the removed slot
    emitter.instruction("str x0, [sp, #8]");                                    // save the removed boxed value
    emitter.instruction("b __rt_indexed_pop_ret");                             // converge on the shared return path
    emitter.label("__rt_indexed_pop_empty");
    emitter.instruction("mov x0, #8");                                          // runtime value tag 8 = null
    emitter.instruction("mov x1, #0");                                          // null payload lo word
    emitter.instruction("mov x2, #0");                                          // null payload hi word
    emitter.instruction("bl __rt_mixed_box_raw");                              // box PHP null for an empty array_pop
    emitter.instruction("str x0, [sp, #8]");                                    // save the removed boxed value
    emitter.label("__rt_indexed_pop_ret");
    emitter.instruction("ldr x1, [sp, #8]");                                    // x1 = removed boxed value
    emitter.instruction("ldr x0, [sp, #0]");                                    // x0 = unique container pointer
    emitter.instruction("ldp x29, x30, [sp, #32]");                            // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                    // release the pop frame
    emitter.instruction("ret");                                                // return (container, removed)
}

/// Emits the x86_64 Linux variant of `__rt_indexed_pop`.
///
/// Input:  `rdi` = indexed-array pointer (caller holds one synthetic `__rt_incref`).
/// Output: `rax` = unique container pointer, `rdx` = removed boxed Mixed cell.
fn emit_indexed_pop_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: indexed_pop ---");
    emitter.label_global("__rt_indexed_pop");
    // Frame: [rbp-8]=container C, [rbp-16]=removed cell.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve slots for the container and removed value
    emitter.instruction("call __rt_array_ensure_unique");                       // copy-on-write split; rax = uniquely owned container (rdi already set)
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the unique container pointer
    emitter.instruction("mov r9, QWORD PTR [rax]");                             // r9 = current logical length
    emitter.instruction("test r9, r9");                                         // an empty array pops boxed null
    emitter.instruction("jz __rt_indexed_pop_empty");                           // branch to the empty path
    emitter.instruction("sub r9, 1");                                           // r9 = index of the removed last element
    emitter.instruction("mov r10, QWORD PTR [rax - 8]");                        // load the packed indexed-array kind word
    emitter.instruction("shr r10, 8");                                          // move the packed value_type tag into the low bits
    emitter.instruction("and r10, 0x7f");                                       // isolate the value_type without the persistent copy-on-write flag
    emitter.instruction("lea r11, [rax + 24]");                                 // r11 = payload base (skip the 24-byte header)
    emitter.instruction("cmp r10, 1");                                          // string slots use a 16-byte pointer/length pair
    emitter.instruction("je __rt_indexed_pop_string");                          // handle string payloads separately
    emitter.instruction("cmp r10, 7");                                          // slots already holding boxed Mixed cells
    emitter.instruction("je __rt_indexed_pop_boxed");                           // adopt the stored Mixed cell directly
    // -- generic single-word payload --
    emitter.instruction("mov rdi, QWORD PTR [r11 + r9 * 8]");                   // load the removed single-word payload (box lo arg)
    emitter.instruction("xor rsi, rsi");                                        // single-word payloads leave the hi word clear
    emitter.instruction("mov rax, r10");                                        // box with the array's runtime value_type tag
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the container to shrink its length
    emitter.instruction("mov QWORD PTR [r8], r9");                              // commit the decremented length (TRANSFER: drop slot ownership)
    emitter.instruction("call __rt_mixed_box_raw");                             // adopt the payload into an owned Mixed cell without retaining
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the removed boxed value
    emitter.instruction("jmp __rt_indexed_pop_ret");                            // converge on the shared return path
    emitter.label("__rt_indexed_pop_string");
    emitter.instruction("mov rcx, r9");                                         // copy the removed index before scaling it
    emitter.instruction("shl rcx, 4");                                          // scale the removed index by the 16-byte string slot size
    emitter.instruction("lea rcx, [r11 + rcx]");                               // rcx = removed string slot address
    emitter.instruction("mov rdi, QWORD PTR [rcx]");                            // rdi = removed string pointer (box lo arg)
    emitter.instruction("mov rsi, QWORD PTR [rcx + 8]");                        // rsi = removed string length (box hi arg)
    emitter.instruction("mov rax, 1");                                          // runtime value tag 1 = string
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the container to shrink its length
    emitter.instruction("mov QWORD PTR [r8], r9");                              // commit the decremented length (TRANSFER the string to the box)
    emitter.instruction("call __rt_mixed_box_raw");                             // adopt the string pointer without re-persisting it
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the removed boxed value
    emitter.instruction("jmp __rt_indexed_pop_ret");                            // converge on the shared return path
    emitter.label("__rt_indexed_pop_boxed");
    emitter.instruction("mov rax, QWORD PTR [r11 + r9 * 8]");                   // load the stored boxed Mixed cell pointer
    emitter.instruction("test rax, rax");                                       // a null slot pops boxed null
    emitter.instruction("jz __rt_indexed_pop_boxed_null");                      // branch to the null path
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the container to shrink its length
    emitter.instruction("mov QWORD PTR [r8], r9");                              // commit the decremented length (TRANSFER the cell to the caller)
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // the stored cell IS the removed owned value
    emitter.instruction("jmp __rt_indexed_pop_ret");                            // converge on the shared return path
    emitter.label("__rt_indexed_pop_boxed_null");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the container to shrink its length
    emitter.instruction("mov QWORD PTR [r8], r9");                              // still remove the null slot from the logical length
    emitter.instruction("mov rax, 8");                                          // runtime value tag 8 = null
    emitter.instruction("xor edi, edi");                                        // null payload lo word
    emitter.instruction("xor esi, esi");                                        // null payload hi word
    emitter.instruction("call __rt_mixed_box_raw");                             // box PHP null for the removed slot
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the removed boxed value
    emitter.instruction("jmp __rt_indexed_pop_ret");                            // converge on the shared return path
    emitter.label("__rt_indexed_pop_empty");
    emitter.instruction("mov rax, 8");                                          // runtime value tag 8 = null
    emitter.instruction("xor edi, edi");                                        // null payload lo word
    emitter.instruction("xor esi, esi");                                        // null payload hi word
    emitter.instruction("call __rt_mixed_box_raw");                             // box PHP null for an empty array_pop
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the removed boxed value
    emitter.label("__rt_indexed_pop_ret");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // rdx = removed boxed value
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // rax = unique container pointer
    emitter.instruction("add rsp, 32");                                         // release the pop frame slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return (container, removed)
}

/// Emits `__rt_hash_pop`, which removes and returns the insertion-order TAIL entry of an
/// associative hash in place.
///
/// Copy-on-write splits the (caller-force-increffed) hash, boxes the tail value with a
/// retain, then delegates removal to `__rt_hash_unset` (which releases the hash's own
/// reference to the value, frees the key, unlinks the insertion-order chain, tombstones
/// the slot, and decrements the count) — so the returned box is the net owner.
///
/// Input:  `x0`/`rdi` = hash pointer (caller holds one synthetic `__rt_incref`).
/// Output: `x0`/`rax` = unique (copy-on-write split) hash pointer,
///         `x1`/`rdx` = removed value boxed as an owned Mixed cell (boxed null when empty).
pub fn emit_hash_pop(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_pop_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_pop ---");
    emitter.label_global("__rt_hash_pop");
    // Frame: [sp,#0]=hash C, [sp,#8]=key_lo, [sp,#16]=key_hi, [sp,#24]=removed, saved fp/lr at [sp,#32].
    emitter.instruction("sub sp, sp, #48");                                     // reserve the pop frame plus saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the local frame
    emitter.instruction("bl __rt_hash_ensure_unique");                          // copy-on-write split; x0 = uniquely owned hash
    emitter.instruction("str x0, [sp, #0]");                                    // save the unique hash pointer
    emitter.instruction("ldr x9, [x0, #32]");                                   // x9 = insertion-order tail slot index
    emitter.instruction("cmn x9, #1");                                          // is the hash empty (tail == -1)?
    emitter.instruction("b.eq __rt_hash_pop_empty");                            // an empty hash pops boxed null
    emitter.instruction("mov x10, #64");                                        // hash entry stride in bytes
    emitter.instruction("mul x11, x9, x10");                                    // byte offset of the tail slot
    emitter.instruction("add x11, x0, x11");                                    // advance from the hash base to the tail slot
    emitter.instruction("add x11, x11, #40");                                   // skip the 40-byte hash header to the tail entry
    emitter.instruction("ldr x12, [x11, #8]");                                  // x12 = tail key lo word
    emitter.instruction("ldr x13, [x11, #16]");                                 // x13 = tail key hi word (-1 marks an integer key)
    emitter.instruction("str x12, [sp, #8]");                                   // save the tail key lo word across helper calls
    emitter.instruction("str x13, [sp, #16]");                                  // save the tail key hi word across helper calls
    emitter.instruction("ldr x3, [x11, #24]");                                  // x3 = tail value lo word
    emitter.instruction("ldr x4, [x11, #32]");                                  // x4 = tail value hi word
    emitter.instruction("ldr x5, [x11, #40]");                                  // x5 = tail value runtime tag
    emitter.instruction("cmp x5, #7");                                          // is the entry already a boxed Mixed cell?
    emitter.instruction("b.eq __rt_hash_pop_boxed");                            // retain the stored Mixed cell rather than reboxing
    emitter.instruction("mov x0, x5");                                          // mixed_from_value tag argument
    emitter.instruction("mov x1, x3");                                          // mixed_from_value payload lo argument
    emitter.instruction("mov x2, x4");                                          // mixed_from_value payload hi argument
    emitter.instruction("bl __rt_mixed_from_value");                            // box the typed tail value with a retain
    emitter.instruction("b __rt_hash_pop_have_removed");                        // continue to the unset step
    emitter.label("__rt_hash_pop_boxed");
    emitter.instruction("mov x0, x3");                                          // move the stored Mixed cell into the retain register
    emitter.instruction("bl __rt_incref");                                      // retain it so the returned box owns a reference
    emitter.label("__rt_hash_pop_have_removed");
    emitter.instruction("str x0, [sp, #24]");                                   // save the removed boxed value
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the hash pointer for the unset
    emitter.instruction("ldr x1, [sp, #8]");                                    // reload the tail key lo word
    emitter.instruction("ldr x2, [sp, #16]");                                   // reload the tail key hi word
    emitter.instruction("bl __rt_hash_unset");                                  // release the hash's own value reference, unlink, tombstone, count--
    emitter.instruction("str x0, [sp, #0]");                                    // persist the (unchanged, already-unique) hash pointer
    emitter.instruction("b __rt_hash_pop_ret");                                 // converge on the shared return path
    emitter.label("__rt_hash_pop_empty");
    emitter.instruction("mov x0, #8");                                          // runtime value tag 8 = null
    emitter.instruction("mov x1, #0");                                          // null payload lo word
    emitter.instruction("mov x2, #0");                                          // null payload hi word
    emitter.instruction("bl __rt_mixed_box_raw");                              // box PHP null for an empty array_pop
    emitter.instruction("str x0, [sp, #24]");                                   // save the removed boxed value
    emitter.label("__rt_hash_pop_ret");
    emitter.instruction("ldr x1, [sp, #24]");                                   // x1 = removed boxed value
    emitter.instruction("ldr x0, [sp, #0]");                                    // x0 = unique hash pointer
    emitter.instruction("ldp x29, x30, [sp, #32]");                            // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                    // release the pop frame
    emitter.instruction("ret");                                                // return (hash, removed)
}

/// Emits the x86_64 Linux variant of `__rt_hash_pop`.
///
/// Input:  `rdi` = hash pointer (caller holds one synthetic `__rt_incref`).
/// Output: `rax` = unique hash pointer, `rdx` = removed boxed Mixed cell.
fn emit_hash_pop_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_pop ---");
    emitter.label_global("__rt_hash_pop");
    // Frame: [rbp-8]=hash C, [rbp-16]=key_lo, [rbp-24]=key_hi, [rbp-32]=removed.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 32");                                         // reserve slots for hash/key/removed state
    emitter.instruction("call __rt_hash_ensure_unique");                        // copy-on-write split; rax = uniquely owned hash (rdi already set)
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the unique hash pointer
    emitter.instruction("mov r9, QWORD PTR [rax + 32]");                        // r9 = insertion-order tail slot index
    emitter.instruction("cmp r9, -1");                                          // is the hash empty (tail == -1)?
    emitter.instruction("je __rt_hash_pop_empty");                              // an empty hash pops boxed null
    emitter.instruction("mov r10, r9");                                         // copy the tail slot index before scaling it
    emitter.instruction("shl r10, 6");                                          // convert the tail slot index into a 64-byte entry offset
    emitter.instruction("add r10, rax");                                        // advance from the hash base to the tail slot
    emitter.instruction("add r10, 40");                                         // skip the 40-byte hash header to the tail entry
    emitter.instruction("mov r11, QWORD PTR [r10 + 8]");                        // r11 = tail key lo word
    emitter.instruction("mov QWORD PTR [rbp - 16], r11");                       // save the tail key lo word across helper calls
    emitter.instruction("mov r11, QWORD PTR [r10 + 16]");                       // r11 = tail key hi word (-1 marks an integer key)
    emitter.instruction("mov QWORD PTR [rbp - 24], r11");                       // save the tail key hi word across helper calls
    emitter.instruction("mov rdi, QWORD PTR [r10 + 24]");                       // rdi = tail value lo word (mixed_from_value lo arg)
    emitter.instruction("mov rsi, QWORD PTR [r10 + 32]");                       // rsi = tail value hi word (mixed_from_value hi arg)
    emitter.instruction("mov r11, QWORD PTR [r10 + 40]");                       // r11 = tail value runtime tag
    emitter.instruction("cmp r11, 7");                                          // is the entry already a boxed Mixed cell?
    emitter.instruction("je __rt_hash_pop_boxed");                              // retain the stored Mixed cell rather than reboxing
    emitter.instruction("mov rax, r11");                                        // mixed_from_value tag argument
    emitter.instruction("call __rt_mixed_from_value");                          // box the typed tail value with a retain
    emitter.instruction("jmp __rt_hash_pop_have_removed");                      // continue to the unset step
    emitter.label("__rt_hash_pop_boxed");
    emitter.instruction("mov rax, rdi");                                        // move the stored Mixed cell into the retain register
    emitter.instruction("call __rt_incref");                                    // retain it so the returned box owns a reference
    emitter.label("__rt_hash_pop_have_removed");
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the removed boxed value
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the hash pointer for the unset
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // reload the tail key lo word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload the tail key hi word
    emitter.instruction("call __rt_hash_unset");                               // release the hash's own value reference, unlink, tombstone, count--
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // persist the (unchanged, already-unique) hash pointer
    emitter.instruction("jmp __rt_hash_pop_ret");                               // converge on the shared return path
    emitter.label("__rt_hash_pop_empty");
    emitter.instruction("mov rax, 8");                                          // runtime value tag 8 = null
    emitter.instruction("xor edi, edi");                                        // null payload lo word
    emitter.instruction("xor esi, esi");                                        // null payload hi word
    emitter.instruction("call __rt_mixed_box_raw");                             // box PHP null for an empty array_pop
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the removed boxed value
    emitter.label("__rt_hash_pop_ret");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // rdx = removed boxed value
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // rax = unique hash pointer
    emitter.instruction("add rsp, 32");                                         // release the pop frame slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return (hash, removed)
}
