//! Purpose:
//! Emits the `__rt_mixed_from_value`, `__rt_mixed_from_value_string` runtime helper assembly for mixed from value.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Mixed helpers use boxed tag/payload cells; tag constants and ownership rules are shared with type checking and codegen.
//! - Null or sentinel payloads presented with a container-shaped tag (4–6) are
//!   normalized to the canonical Mixed null tag before ownership or allocation.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;
use crate::codegen_support::sentinels::emit_branch_if_null_container;

/// Retains or persists a runtime value and boxes it into a Mixed cell.
///
/// Container-shaped tags with a zero or null-sentinel payload are normalized to
/// canonical PHP null before retain/allocation, preventing malformed boxes from
/// escaping into downstream consumers.
/// Input:  x0=value_tag, x1=value_lo, x2=value_hi
/// Output: x0=boxed mixed pointer
pub fn emit_mixed_from_value(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_from_value_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: mixed_from_value ---");
    emitter.label_global("__rt_mixed_from_value");

    emitter.instruction("sub sp, sp, #48");                                     // allocate stack frame for the incoming payload and boxed result
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // set up the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the runtime value tag across helper calls
    emitter.instruction("stp x1, x2, [sp, #8]");                                // save the incoming payload words across helper calls

    emitter.instruction("cmp x0, #4");                                          // do only container-shaped tags need null-payload normalization?
    emitter.instruction("b.lt __rt_mixed_from_value_dispatch");                 // scalar tags preserve every payload bit pattern
    emitter.instruction("cmp x0, #6");                                          // indexed arrays, hashes, and objects are the container-shaped tags
    emitter.instruction("b.gt __rt_mixed_from_value_dispatch");                 // nested Mixed/callable tags use ordinary pointer ownership
    emit_branch_if_null_container(
        emitter,
        "x1",
        "x9",
        "__rt_mixed_from_value_null_container",
    );
    emitter.label("__rt_mixed_from_value_dispatch");
    emitter.instruction("cmp x0, #1");                                          // does this mixed payload hold a string?
    emitter.instruction("b.eq __rt_mixed_from_value_string");                   // strings must be persisted for the boxed owner
    emitter.instruction("cmp x0, #4");                                          // does this mixed payload hold an indexed array?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // refcounted child pointers must be retained for the boxed owner
    emitter.instruction("cmp x0, #5");                                          // does this mixed payload hold an associative array?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // refcounted child pointers must be retained for the boxed owner
    emitter.instruction("cmp x0, #6");                                          // does this mixed payload hold an object?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // refcounted child pointers must be retained for the boxed owner
    emitter.instruction("cmp x0, #7");                                          // does this mixed payload hold another mixed cell?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // nested mixed cells must also be retained
    emitter.instruction("cmp x0, #10");                                         // does this mixed payload hold a callable descriptor?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // callable descriptors are retained for the boxed owner
    emitter.instruction("cmp x0, #11");                                         // does this mixed payload hold a reference cell?
    emitter.instruction("b.eq __rt_mixed_from_value_retain");                   // reference cells must be retained so the boxed Mixed owns its share of the cell
    emitter.instruction("b __rt_mixed_from_value_alloc");                       // scalars can be boxed without additional retention

    emitter.label("__rt_mixed_from_value_null_container");
    emitter.instruction("mov x9, #8");                                          // runtime tag 8 is the canonical boxed PHP null
    emitter.instruction("str x9, [sp, #0]");                                    // replace the container-shaped tag before allocation
    emitter.instruction("stp xzr, xzr, [sp, #8]");                              // canonical null has no payload words or ownership
    emitter.instruction("b __rt_mixed_from_value_alloc");                       // allocate the normalized Mixed null without retaining the sentinel

    emitter.label("__rt_mixed_from_value_string");
    emitter.instruction("bl __rt_str_persist");                                 // duplicate the string payload for the boxed owner
    emitter.instruction("stp x1, x2, [sp, #8]");                                // replace the saved payload with the owned string pointer and length
    emitter.instruction("b __rt_mixed_from_value_alloc");                       // continue with allocation after persisting the string

    emitter.label("__rt_mixed_from_value_retain");
    emitter.instruction("mov x0, x1");                                          // move the child heap pointer into the incref argument register
    emitter.instruction("bl __rt_incref");                                      // retain the shared child pointer for the boxed owner

    emitter.label("__rt_mixed_from_value_alloc");
    emitter.instruction("mov x0, #24");                                         // mixed cells store tag plus two payload words
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the mixed cell storage
    emitter.instruction("mov x9, #5");                                          // low byte 5 = mixed cell heap kind
    emitter.instruction("str x9, [x0, #-8]");                                   // install the mixed-cell heap kind in the uniform header
    emitter.instruction("ldr x10, [sp, #0]");                                   // reload the saved runtime value tag
    emitter.instruction("str x10, [x0]");                                       // store the runtime value tag at mixed[0]
    emitter.instruction("ldp x11, x12, [sp, #8]");                              // reload the normalized payload words
    emitter.instruction("stp x11, x12, [x0, #8]");                              // store the payload words at mixed[8] and mixed[16]
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate the stack frame
    emitter.instruction("ret");                                                 // return the boxed mixed pointer in x0
}

/// x86_64 Linux implementation of `__rt_mixed_from_value`.
/// Detects the runtime value tag in `rax`, applies ownership normalization (string persistence or refcount retention),
/// allocates a 24-byte mixed cell on the heap, and writes the tagged payload at offsets 0, 8, and 16.
/// Input:  rax=value_tag, rdi=value_lo, rsi=value_hi
/// Output: rax=boxed mixed pointer
/// Clobbers: r10 as temporary scratch during header stamping and payload installation.
fn emit_mixed_from_value_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_from_value ---");
    emitter.label_global("__rt_mixed_from_value");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before boxing the mixed payload
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the temporary payload spill
    emitter.instruction("sub rsp, 32");                                         // reserve local slots for tag, value_lo, value_hi, and future helper-preserved scratch state
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the runtime value tag across helper-driven ownership normalization
    emitter.instruction("mov QWORD PTR [rbp - 16], rdi");                       // save the low payload word across helper calls and the final heap allocation
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // save the high payload word across helper calls and the final heap allocation
    emitter.instruction("cmp rax, 4");                                          // do only container-shaped tags need null-payload normalization?
    emitter.instruction("jl __rt_mixed_from_value_dispatch");                   // scalar tags preserve every payload bit pattern
    emitter.instruction("cmp rax, 6");                                          // indexed arrays, hashes, and objects are the container-shaped tags
    emitter.instruction("jg __rt_mixed_from_value_dispatch");                   // nested Mixed/callable tags use ordinary pointer ownership
    emit_branch_if_null_container(
        emitter,
        "rdi",
        "r10",
        "__rt_mixed_from_value_null_container",
    );
    emitter.label("__rt_mixed_from_value_dispatch");
    emitter.instruction("cmp rax, 1");                                          // detect string payloads that need their own owned copy inside the mixed box
    emitter.instruction("je __rt_mixed_from_value_string");                     // strings must be persisted so the mixed cell owns a stable payload
    emitter.instruction("cmp rax, 4");                                          // detect indexed arrays that participate in refcounted ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain indexed arrays before storing them inside the mixed cell
    emitter.instruction("cmp rax, 5");                                          // detect associative arrays that participate in refcounted ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain associative arrays before storing them inside the mixed cell
    emitter.instruction("cmp rax, 6");                                          // detect objects that participate in refcounted ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain objects before storing them inside the mixed cell
    emitter.instruction("cmp rax, 7");                                          // detect nested mixed cells that participate in refcounted ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain nested mixed cells before storing them inside the parent mixed cell
    emitter.instruction("cmp rax, 10");                                         // detect callable descriptors that participate in callable ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain callable descriptors before storing them inside the mixed cell
    emitter.instruction("cmp rax, 11");                                         // detect reference cells that participate in refcounted ownership
    emitter.instruction("je __rt_mixed_from_value_retain");                     // retain reference cells so the boxed Mixed owns its share of the cell
    emitter.instruction("jmp __rt_mixed_from_value_alloc");                     // scalars can be boxed directly without additional ownership work

    emitter.label("__rt_mixed_from_value_null_container");
    emitter.instruction("mov QWORD PTR [rbp - 8], 8");                          // replace the container-shaped tag with canonical Mixed null
    emitter.instruction("mov QWORD PTR [rbp - 16], 0");                         // canonical null has no low payload or ownership
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // canonical null has no high payload
    emitter.instruction("jmp __rt_mixed_from_value_alloc");                     // allocate the normalized Mixed null without retaining the sentinel

    emitter.label("__rt_mixed_from_value_string");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // move the source string pointer into the x86_64 string helper input register
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // move the source string length into the paired x86_64 string helper register
    emitter.instruction("call __rt_str_persist");                               // duplicate the string payload so the new mixed owner receives heap-backed storage
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // replace the saved low payload word with the owned string pointer returned by str_persist
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // replace the saved high payload word with the owned string length returned by str_persist
    emitter.instruction("jmp __rt_mixed_from_value_alloc");                     // continue boxing once the string payload is safely owned

    emitter.label("__rt_mixed_from_value_retain");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // move the shared heap child into the x86_64 refcount helper input register
    emitter.instruction("call __rt_incref");                                    // retain the shared heap child for the new mixed owner

    emitter.label("__rt_mixed_from_value_alloc");
    emitter.instruction("mov rax, 24");                                         // mixed cells store tag plus two payload words in the owned heap allocation
    emitter.instruction("call __rt_heap_alloc");                                // allocate the mixed cell storage through the x86_64 heap wrapper
    emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // materialize the mixed-cell heap kind word with the x86_64 heap marker
    emitter.instruction("mov QWORD PTR [rax - 8], r10");                        // stamp the allocated payload as a mixed cell in the uniform heap header
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // reload the saved runtime value tag after helper-driven ownership normalization
    emitter.instruction("mov QWORD PTR [rax], r10");                            // store the runtime value tag at mixed[0]
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                       // reload the normalized low payload word after ownership helpers completed
    emitter.instruction("mov QWORD PTR [rax + 8], r10");                        // store the low payload word at mixed[8]
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // reload the normalized high payload word after ownership helpers completed
    emitter.instruction("mov QWORD PTR [rax + 16], r10");                       // store the high payload word at mixed[16]
    emitter.instruction("add rsp, 32");                                         // release the temporary payload spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning
    emitter.instruction("ret");                                                 // return the boxed mixed pointer in rax
}

/// Emits `__rt_mixed_box_scalar`: boxes an int, float or bool payload into a fresh Mixed cell.
///
/// The scalar tags need none of `__rt_mixed_from_value`'s ownership work (no string to
/// persist, no child to retain, no null normalization), so this skips its dispatch and frame.
/// Input: x0 / rax = runtime tag (0 int, 2 float, 3 bool); x1 / rdi = payload bits.
/// Output: x0 / rax = the Mixed cell.
pub fn emit_mixed_box_scalar(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_box_scalar ---");
    emitter.label_global("__rt_mixed_box_scalar");
    emit_mixed_box_scalar_fast_path(emitter);
    if emitter.target.arch == Arch::X86_64 {
        emitter.instruction("sub rsp, 24");                                     // spill slots (keeps the call aligned)
        emitter.instruction("mov QWORD PTR [rsp], rax");                        // save the runtime tag
        emitter.instruction("mov QWORD PTR [rsp + 8], rdi");                    // save the payload bits
        emitter.instruction("mov rax, 24");                                     // tag plus two payload words
        emitter.instruction("call __rt_heap_alloc");                            // allocate the cell
        emitter.instruction(&format!("mov r10, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // mixed-cell heap kind word
        emitter.instruction("mov QWORD PTR [rax - 8], r10");                    // stamp the cell kind
        emitter.instruction("mov r10, QWORD PTR [rsp]");                        // the runtime tag
        emitter.instruction("mov QWORD PTR [rax], r10");                        // mixed[0] = tag
        emitter.instruction("mov r10, QWORD PTR [rsp + 8]");                    // the payload
        emitter.instruction("mov QWORD PTR [rax + 8], r10");                    // mixed[8] = payload
        emitter.instruction("mov QWORD PTR [rax + 16], 0");                     // scalars use no second word
        emitter.instruction("add rsp, 24");                                     // release the spill slots
        emitter.instruction("ret");                                             // return the cell in rax
        return;
    }
    emitter.instruction("stp x29, x30, [sp, #-32]!");                           // save frame pointer and return address
    emitter.instruction("mov x29, sp");                                         // establish the frame
    emitter.instruction("stp x0, x1, [sp, #16]");                               // save the runtime tag and payload
    emitter.instruction("mov x0, #24");                                         // tag plus two payload words
    emitter.instruction("bl __rt_heap_alloc");                                  // allocate the cell
    emitter.instruction("mov x9, #5");                                          // low byte 5 = mixed cell heap kind
    emitter.instruction("str x9, [x0, #-8]");                                   // stamp the cell kind
    emitter.instruction("ldp x10, x11, [sp, #16]");                             // reload tag and payload
    emitter.instruction("stp x10, x11, [x0]");                                  // mixed[0] = tag, mixed[8] = payload
    emitter.instruction("str xzr, [x0, #16]");                                  // scalars use no second word
    emitter.instruction("ldp x29, x30, [sp], #32");                             // restore frame pointer and return address
    emitter.instruction("ret");                                                 // return the cell in x0
}

/// Emits `__rt_borrow_ring_park`: keeps a freshly boxed value produced by a *borrowed*
/// array read (a null miss or a by-reference marker) alive in a four-entry ring and releases
/// the box it evicts.
///
/// A borrowed read has no matching release, and its box is dead once its consumer (a mixed
/// arithmetic operation of the same block) has run; nested operations can keep at most three
/// such boxes live at once (`$a[9] + ($b[9] + $c[9])`), so a ring of four never evicts a live one.
/// Input/output: the box in x0 (AArch64) / rax (x86_64), returned unchanged.
pub fn emit_borrow_ring_park(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: borrow_ring_park ---");
    emitter.label_global("__rt_borrow_ring_park");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("stp x29, x30, [sp, #-32]!");                   // save frame pointer and return address
            emitter.instruction("mov x29, sp");                                 // anchor the helper frame
            emitter.instruction("str x0, [sp, #16]");                           // keep the parked box for the return
            crate::codegen_support::abi::emit_symbol_address(emitter, "x9", "_borrow_ring_idx");
            emitter.instruction("ldr x10, [x9]");                               // current ring slot
            emitter.instruction("add x11, x10, #1");                            // next ring slot
            emitter.instruction("and x11, x11, #3");                            // wrap within the four entries
            emitter.instruction("str x11, [x9]");                               // advance the ring cursor
            crate::codegen_support::abi::emit_symbol_address(emitter, "x12", "_borrow_ring");
            emitter.instruction("ldr x13, [x12, x10, lsl #3]");                 // the box parked four reads ago
            emitter.instruction("str x0, [x12, x10, lsl #3]");                  // park the new box
            emitter.instruction("mov x0, x13");                                 // release the evicted box (null is safe)
            emitter.instruction("bl __rt_decref_mixed");
            emitter.instruction("ldr x0, [sp, #16]");                           // return the parked box
            emitter.instruction("ldp x29, x30, [sp], #32");                     // restore frame pointer and return address
            emitter.instruction("ret");
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // save the caller frame pointer
            emitter.instruction("mov rbp, rsp");                                // anchor the helper frame
            emitter.instruction("sub rsp, 16");                                 // aligned slot for the parked box
            emitter.instruction("mov QWORD PTR [rbp - 8], rax");                // keep the parked box for the return
            crate::codegen_support::abi::emit_symbol_address(emitter, "r8", "_borrow_ring_idx");
            emitter.instruction("mov r9, QWORD PTR [r8]");                      // current ring slot
            emitter.instruction("lea r10, [r9 + 1]");                           // next ring slot
            emitter.instruction("and r10, 3");                                  // wrap within the four entries
            emitter.instruction("mov QWORD PTR [r8], r10");                     // advance the ring cursor
            crate::codegen_support::abi::emit_symbol_address(emitter, "r11", "_borrow_ring");
            emitter.instruction("mov rcx, QWORD PTR [r11 + r9 * 8]");           // the box parked four reads ago
            emitter.instruction("mov QWORD PTR [r11 + r9 * 8], rax");           // park the new box
            emitter.instruction("mov rax, rcx");                                // release the evicted box (null is safe)
            emitter.instruction("call __rt_decref_mixed");
            emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                // return the parked box
            emitter.instruction("leave");                                       // drop the helper frame
            emitter.instruction("ret");
        }
    }
}

/// Emits the allocation fast path of `__rt_mixed_box_scalar`: with heap debugging off, a box
/// is popped from the 24-byte size-class bin or carved from the bump tail, stamped, counted
/// exactly like `__rt_heap_alloc` does, filled and returned. Anything else (debug mode, a full
/// heap) falls through to the generic path that follows. Clobbers only registers
/// `__rt_heap_alloc` clobbers.
fn emit_mixed_box_scalar_fast_path(emitter: &mut Emitter) {
    use crate::codegen_support::abi::emit_symbol_address;
    match emitter.target.arch {
        Arch::AArch64 => {
            emit_symbol_address(emitter, "x9", "_heap_debug_enabled");
            emitter.instruction("ldr x9, [x9]");                                // heap-debug mode validates every allocation
            emitter.instruction("cbnz x9, __rt_mixed_box_scalar_slow");         // keep the generic path for it
            emit_symbol_address(emitter, "x10", "_heap_small_bins");
            emitter.instruction("ldr x11, [x10, #16]");                         // head of the 24-byte class bin
            emitter.instruction("cbz x11, __rt_mixed_box_scalar_bump");         // empty: carve from the bump tail
            emitter.instruction("ldr x12, [x11, #16]");                         // next cached block
            emitter.instruction("str x12, [x10, #16]");                         // unlink the reused block
            emitter.instruction("b __rt_mixed_box_scalar_stamp");
            emitter.label("__rt_mixed_box_scalar_bump");
            emit_symbol_address(emitter, "x10", "_heap_off");
            emitter.instruction("ldr x12, [x10]");                              // current bump offset
            emitter.instruction("add x13, x12, #40");                           // header plus the 24-byte cell
            emit_symbol_address(emitter, "x14", "_heap_max");
            emitter.instruction("ldr x14, [x14]");                              // heap capacity
            emitter.instruction("cmp x13, x14");                                // does the cell fit at the tail?
            emitter.instruction("b.hi __rt_mixed_box_scalar_slow");             // no: the generic allocator reuses or fails
            emitter.instruction("str x13, [x10]");                              // commit the bump
            emit_symbol_address(emitter, "x14", "_heap_buf");
            emitter.instruction("add x11, x14, x12");                           // header of the new block
            emitter.instruction("mov w12, #24");                                // payload size
            emitter.instruction("str w12, [x11]");
            emitter.label("__rt_mixed_box_scalar_stamp");
            emitter.instruction("mov w12, #1");                                 // refcount 1
            emitter.instruction("str w12, [x11, #4]");
            emitter.instruction("mov x12, #5");                                 // mixed-cell heap kind
            emitter.instruction("str x12, [x11, #8]");
            emit_symbol_address(emitter, "x12", "_gc_allocs");
            emitter.instruction("ldr x13, [x12]");                              // count the allocation
            emitter.instruction("add x13, x13, #1");
            emitter.instruction("str x13, [x12]");
            emitter.instruction("ldr w14, [x11]");                              // block payload size
            emitter.instruction("add x14, x14, #16");                           // plus its header
            emit_symbol_address(emitter, "x12", "_gc_live");
            emitter.instruction("ldr x13, [x12]");                              // live bytes
            emitter.instruction("add x13, x13, x14");
            emitter.instruction("str x13, [x12]");
            emit_symbol_address(emitter, "x12", "_gc_peak");
            emitter.instruction("ldr x15, [x12]");                              // peak live bytes
            emitter.instruction("cmp x13, x15");
            emitter.instruction("csel x15, x13, x15, hi");
            emitter.instruction("str x15, [x12]");
            emitter.instruction("add x15, x11, #16");                           // the cell
            emitter.instruction("stp x0, x1, [x15]");                           // mixed[0] = tag, mixed[8] = payload
            emitter.instruction("str xzr, [x15, #16]");                         // scalars use no second word
            emitter.instruction("mov x0, x15");                                 // return the cell
            emitter.instruction("ret");
            emitter.label("__rt_mixed_box_scalar_slow");
        }
        Arch::X86_64 => {
            emit_symbol_address(emitter, "r8", "_heap_debug_enabled");
            emitter.instruction("cmp QWORD PTR [r8], 0");                       // heap-debug mode validates every allocation
            emitter.instruction("jne __rt_mixed_box_scalar_slow");              // keep the generic path for it
            emit_symbol_address(emitter, "r9", "_heap_small_bins");
            emitter.instruction("mov r10, QWORD PTR [r9 + 16]");                // head of the 24-byte class bin
            emitter.instruction("test r10, r10");
            emitter.instruction("jz __rt_mixed_box_scalar_bump");               // empty: carve from the bump tail
            emitter.instruction("mov r11, QWORD PTR [r10 + 16]");               // next cached block
            emitter.instruction("mov QWORD PTR [r9 + 16], r11");                // unlink the reused block
            emitter.instruction("jmp __rt_mixed_box_scalar_stamp");
            emitter.label("__rt_mixed_box_scalar_bump");
            emit_symbol_address(emitter, "r9", "_heap_off");
            emitter.instruction("mov r10, QWORD PTR [r9]");                     // current bump offset
            emitter.instruction("lea rcx, [r10 + 40]");                         // header plus the 24-byte cell
            emit_symbol_address(emitter, "r8", "_heap_max");
            emitter.instruction("cmp rcx, QWORD PTR [r8]");                     // does the cell fit at the tail?
            emitter.instruction("ja __rt_mixed_box_scalar_slow");               // no: the generic allocator reuses or fails
            emitter.instruction("mov QWORD PTR [r9], rcx");                     // commit the bump
            emit_symbol_address(emitter, "r11", "_heap_buf");
            emitter.instruction("add r10, r11");                                // header of the new block
            emitter.instruction("mov DWORD PTR [r10], 24");                     // payload size
            emitter.label("__rt_mixed_box_scalar_stamp");
            emitter.instruction("mov DWORD PTR [r10 + 4], 1");                  // refcount 1
            emitter.instruction(&format!("mov r11, 0x{:x}", crate::codegen_support::sentinels::x86_64_heap_kind_word(5))); // mixed-cell heap kind word
            emitter.instruction("mov QWORD PTR [r10 + 8], r11");
            emit_symbol_address(emitter, "r8", "_gc_allocs");
            emitter.instruction("add QWORD PTR [r8], 1");                       // count the allocation
            emitter.instruction("mov r11d, DWORD PTR [r10]");                   // block payload size
            emitter.instruction("add r11, 16");                                 // plus its header
            emit_symbol_address(emitter, "r8", "_gc_live");
            emitter.instruction("add r11, QWORD PTR [r8]");                     // new live bytes
            emitter.instruction("mov QWORD PTR [r8], r11");
            emit_symbol_address(emitter, "r8", "_gc_peak");
            emitter.instruction("cmp r11, QWORD PTR [r8]");                     // a new peak?
            emitter.instruction("jbe __rt_mixed_box_scalar_counted");
            emitter.instruction("mov QWORD PTR [r8], r11");
            emitter.label("__rt_mixed_box_scalar_counted");
            emitter.instruction("mov QWORD PTR [r10 + 16], rax");               // mixed[0] = tag
            emitter.instruction("mov QWORD PTR [r10 + 24], rdi");               // mixed[8] = payload
            emitter.instruction("mov QWORD PTR [r10 + 32], 0");                 // scalars use no second word
            emitter.instruction("lea rax, [r10 + 16]");                         // return the cell
            emitter.instruction("ret");
            emitter.label("__rt_mixed_box_scalar_slow");
        }
    }
}
