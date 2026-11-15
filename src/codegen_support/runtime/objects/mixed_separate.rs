//! Purpose:
//! Emits `__rt_mixed_separate`, the separate-on-write helper for boxed Mixed
//! cells (PHP value semantics for shared `mixed` locals and array elements).
//!
//! Called from:
//! - `RuntimeCallTarget::MixedSeparate` lowering in `crate::codegen::lower_inst`
//!   (in-place writes through a boxed Mixed local: `$m[] = v`, `$m[k] = v`,
//!   nested write roots, `unset($m[k])`).
//! - `__rt_mixed_array_get_for_write` before returning a STORED child cell.
//!
//! Key details:
//! - Input: one owned reference to a boxed Mixed cell. Output: an owned
//!   reference to a cell that is safe to mutate in place.
//! - A cell whose refcount is <= 1 (or a null/non-heap pointer) is returned
//!   unchanged. A shared cell is cloned through `__rt_mixed_from_value` (which
//!   retains container payloads and persists strings) and the caller's share of
//!   the original cell is released; the original stays alive for its other
//!   owners, which keep observing the unmodified value.
//! - The cloned cell shares the container payload with the original; the
//!   array/hash payload itself is copy-on-write separated by the following
//!   write helper because its refcount is now > 1.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_mixed_separate` for the active target.
pub(crate) fn emit_mixed_separate(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_mixed_separate_x86_64(emitter);
        return;
    }
    emit_mixed_separate_aarch64(emitter);
}

/// Emits the AArch64 separate-on-write helper.
///
/// Input: `x0` = owned boxed Mixed cell. Output: `x0` = owned unshared cell.
fn emit_mixed_separate_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_separate ---");
    emitter.label_global("__rt_mixed_separate");

    emitter.instruction("cbz x0, __rt_mixed_separate_done");                    // null cells have nothing to separate
    abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("cmp x0, x9");                                          // is the cell below the managed heap?
    emitter.instruction("b.lo __rt_mixed_separate_done");                       // non-heap cells are never shared through refcounts
    abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // load the current heap offset
    emitter.instruction("add x10, x9, x10");                                    // compute the live heap end
    emitter.instruction("cmp x0, x10");                                         // is the cell at or beyond the live heap end?
    emitter.instruction("b.hs __rt_mixed_separate_done");                       // foreign pointers are returned unchanged
    emitter.instruction("ldr w9, [x0, #-12]");                                  // load the cell refcount from the uniform header
    emitter.instruction("cmp w9, #1");                                          // is the caller the only owner of the cell?
    emitter.instruction("b.ls __rt_mixed_separate_done");                       // unshared cells can be mutated in place

    emitter.instruction("sub sp, sp, #32");                                     // reserve slots for the shared cell, the clone, and fp/lr
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the shared cell for the release after cloning
    emitter.instruction("ldr x1, [x0, #8]");                                    // load the low payload word of the shared cell
    emitter.instruction("ldr x2, [x0, #16]");                                   // load the high payload word of the shared cell
    emitter.instruction("ldr x0, [x0]");                                        // load the runtime value tag of the shared cell
    emitter.instruction("bl __rt_mixed_from_value");                            // build a fresh cell retaining or persisting the payload
    emitter.instruction("str x0, [sp, #8]");                                    // save the fresh unshared cell across the release
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the shared cell
    emitter.instruction("bl __rt_decref_mixed");                                // drop the caller's share; other owners keep the original value
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the fresh unshared cell
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the helper frame
    emitter.label("__rt_mixed_separate_done");
    emitter.instruction("ret");                                                 // return the cell that is now safe to mutate in place
}

/// Emits the x86_64 separate-on-write helper.
///
/// Input: `rax` = owned boxed Mixed cell. Output: `rax` = owned unshared cell.
fn emit_mixed_separate_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: mixed_separate ---");
    emitter.label_global("__rt_mixed_separate");

    emitter.instruction("test rax, rax");                                       // null cells have nothing to separate
    emitter.instruction("jz __rt_mixed_separate_done");                         // return null cells unchanged
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea r10, [r10 + 16]");                                 // first valid payload begins after the initial heap header
    emitter.instruction("cmp rax, r10");                                        // is the cell below the managed heap payload window?
    emitter.instruction("jb __rt_mixed_separate_done");                         // non-heap cells are never shared through refcounts
    abi::emit_symbol_address(emitter, "r11", "_heap_off");
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // load the current heap bump extent
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("add r11, r10");                                        // compute the live heap end
    emitter.instruction("cmp rax, r11");                                        // is the cell at or beyond the live heap end?
    emitter.instruction("jae __rt_mixed_separate_done");                        // foreign pointers are returned unchanged
    emitter.instruction("mov r10, QWORD PTR [rax - 8]");                        // load the uniform heap kind word
    emitter.instruction("shr r10, 32");                                         // isolate the x86_64 heap marker
    emitter.instruction(&format!("cmp r10d, 0x{:x}", crate::codegen_support::sentinels::X86_64_HEAP_MAGIC_HI32)); // only elephc-owned heap cells carry refcounts
    emitter.instruction("jne __rt_mixed_separate_done");                        // foreign pointers are returned unchanged
    emitter.instruction("mov r10d, DWORD PTR [rax - 12]");                      // load the cell refcount from the uniform header
    emitter.instruction("cmp r10d, 1");                                         // is the caller the only owner of the cell?
    emitter.instruction("jbe __rt_mixed_separate_done");                        // unshared cells can be mutated in place

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame
    emitter.instruction("sub rsp, 16");                                         // reserve slots for the shared cell and the clone
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the shared cell for the release after cloning
    emitter.instruction("mov rdi, QWORD PTR [rax + 8]");                        // load the low payload word of the shared cell
    emitter.instruction("mov rsi, QWORD PTR [rax + 16]");                       // load the high payload word of the shared cell
    emitter.instruction("mov rax, QWORD PTR [rax]");                            // load the runtime value tag of the shared cell
    emitter.instruction("call __rt_mixed_from_value");                          // build a fresh cell retaining or persisting the payload
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the fresh unshared cell across the release
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the shared cell
    emitter.instruction("call __rt_decref_mixed");                              // drop the caller's share; other owners keep the original value
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the fresh unshared cell
    emitter.instruction("add rsp, 16");                                         // release the helper slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.label("__rt_mixed_separate_done");
    emitter.instruction("ret");                                                 // return the cell that is now safe to mutate in place
}
