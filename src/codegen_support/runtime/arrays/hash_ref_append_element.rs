//! Purpose:
//! Emits `__rt_hash_ref_append_element`: append an EXISTING kind-6 reference cell as a new element at
//! a hash's next automatic integer key (`$a[] = &$var`, `$a[$k][] = &$var`) and return the
//! (possibly relocated) hash. The cell is `$var`'s persistent reference cell, shared across every bind.
//! Keeps PHP array/hash storage, heap ownership, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Backs the `HashRefAppendElement` EIR op. Composes `__rt_ref_cell_incref` (the new element takes an
//!   owning share of the shared cell) + `__rt_hash_append` (append the cell at PHP's next automatic
//!   integer key with value-tag 11, Reference). It does NOT allocate a cell — the cell was already
//!   materialized by the `LocalRefEnsure` get-or-promote of `$var`, so binding a fresh cell each time
//!   would give divergent aliases (Zend keeps ONE cell per referenced variable, shared across binds).
//! - Retain FIRST, then append: the cell must not be freed between the incref and the store.
//! - `__rt_hash_append` may grow / copy-on-write split the table; the relocated pointer is returned so
//!   the caller stores it back to the array local.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits the `__rt_hash_ref_append_element` runtime helper.
///
/// # ABI
/// - ARM64: in `x0` = hash pointer, `x1` = kind-6 reference cell pointer; out `x0` =
///   possibly-relocated hash pointer.
/// - x86_64: in `rdi` = hash pointer, `rsi` = kind-6 reference cell pointer; out `rax` =
///   possibly-relocated hash pointer.
pub fn emit_hash_ref_append_element(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_ref_append_element_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_ref_append_element ---");
    emitter.label_global("__rt_hash_ref_append_element");

    // -- frame: [0]=hash [8]=cell --
    emitter.instruction("sub sp, sp, #32");                                     // allocate a frame for the hash and cell spills
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #16");                                    // set up the new frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the hash pointer across helper calls
    emitter.instruction("str x1, [sp, #8]");                                    // save the reference cell pointer across helper calls

    // -- retain the shared cell FIRST so the appended element owns its share --
    emitter.instruction("mov x0, x1");                                          // pass the reference cell to the retain helper
    emitter.instruction("bl __rt_ref_cell_incref");                             // the new element takes a new owning share of the shared cell

    // -- append the reference cell at the hash's next automatic integer key --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the hash pointer
    emitter.instruction("ldr x1, [sp, #8]");                                    // value_lo = the shared reference cell pointer
    emitter.instruction("mov x2, xzr");                                         // value_hi = 0 for a single-word reference value
    emitter.instruction("mov x3, #11");                                         // value_tag = 11 (Reference)
    emitter.instruction("bl __rt_hash_append");                                 // x0 = possibly-relocated hash after appending the cell

    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // deallocate the frame
    emitter.instruction("ret");                                                 // return (x0 = relocated hash)
}

/// Emits the x86_64 Linux variant of `__rt_hash_ref_append_element`.
fn emit_hash_ref_append_element_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_ref_append_element ---");
    emitter.label_global("__rt_hash_ref_append_element");

    // -- frame: [-8]=hash [-16]=cell --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before spilling helper state
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve slots for the hash and cell spills
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the hash pointer across helper calls
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the reference cell pointer across helper calls

    // -- retain the shared cell FIRST so the appended element owns its share --
    emitter.instruction("mov rax, rsi");                                        // pass the reference cell to the retain helper (rax ABI)
    emitter.instruction("call __rt_ref_cell_incref");                           // the new element takes a new owning share of the shared cell

    // -- append the reference cell at the hash's next automatic integer key --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // value_lo = the shared reference cell pointer
    emitter.instruction("xor edx, edx");                                        // value_hi = 0 for a single-word reference value
    emitter.instruction("mov ecx, 11");                                         // value_tag = 11 (Reference)
    emitter.instruction("call __rt_hash_append");                               // rax = possibly-relocated hash after appending the cell

    emitter.instruction("add rsp, 16");                                         // release the spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return (rax = relocated hash)
}
