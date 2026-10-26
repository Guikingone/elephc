//! Purpose:
//! Finishes native SPL list cloning after the generic declared-property copy.
//!
//! Called from:
//! - The AArch64 and x86_64 boxed-object clone adapters.
//!
//! Key details:
//! - The parent-id table includes native classes and arbitrary descendants.
//! - Splitting consumes the clone's retained storage owner; element cells stay shared.
//! - A clone starts at physical node zero, including in LIFO mode, before __clone.

use super::*;
use crate::codegen_support::runtime::spl::{
    SPL_DLL_ITER_INDEX_OFFSET, SPL_DLL_STORAGE_OFFSET,
};

/// Uses the ARM64 clone adapter's saved class id and object pointer.
#[rustfmt::skip]
pub(super) fn emit_aarch64(emitter: &mut Emitter) {
    abi::emit_load_symbol_to_reg(emitter, "x10", "_spl_dll_class_id", 0);
    abi::emit_symbol_address(emitter, "x11", "_class_parent_ids");
    emitter.instruction("ldr x9, [sp, #56]");                                  // reload class id after ABI symbol loads may use scratch registers
    emitter.label("__rt_clone_spl_list_ancestor");
    emitter.instruction("cmp x9, #0");                                        // the negative parent sentinel ends the hierarchy
    emitter.instruction("b.lt __rt_clone_spl_list_done");                      // ordinary objects need no native-list finishing
    emitter.instruction("cmp x9, x10");                                       // does this class inherit SplDoublyLinkedList storage?
    emitter.instruction("b.eq __rt_clone_spl_list_storage");                   // native list nodes must be copied independently
    emitter.instruction("ldr x9, [x11, x9, lsl #3]");                         // advance through the dense parent-id table
    emitter.instruction("b __rt_clone_spl_list_ancestor");                     // test the next ancestor
    emitter.label("__rt_clone_spl_list_storage");
    emitter.instruction("ldr x9, [sp, #8]");                                  // recover the newly allocated clone
    abi::emit_load_from_address(emitter, "x0", "x9", SPL_DLL_STORAGE_OFFSET);
    abi::emit_call_label(emitter, "__rt_array_ensure_unique");
    emitter.instruction("ldr x9, [sp, #8]");                                  // restore the clone across the storage split
    abi::emit_store_to_address(emitter, "x0", "x9", SPL_DLL_STORAGE_OFFSET);
    abi::emit_store_zero_to_address(emitter, "x9", SPL_DLL_ITER_INDEX_OFFSET);
    emitter.label("__rt_clone_spl_list_done");
}

/// Uses the SysV clone adapter's saved class id and object pointer.
#[rustfmt::skip]
pub(super) fn emit_x86_64(emitter: &mut Emitter) {
    abi::emit_load_symbol_to_reg(emitter, "r11", "_spl_dll_class_id", 0);
    abi::emit_symbol_address(emitter, "r9", "_class_parent_ids");
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                      // reload class id after ABI symbol loads may use scratch registers
    emitter.label("__rt_clone_spl_list_ancestor_x86");
    emitter.instruction("cmp r10, 0");                                        // the negative parent sentinel ends the hierarchy
    emitter.instruction("jl __rt_clone_spl_list_done_x86");                    // ordinary objects need no native-list finishing
    emitter.instruction("cmp r10, r11");                                      // does this class inherit SplDoublyLinkedList storage?
    emitter.instruction("je __rt_clone_spl_list_storage_x86");                 // native list nodes must be copied independently
    emitter.instruction("mov r10, QWORD PTR [r9 + r10 * 8]");                 // advance through the dense parent-id table
    emitter.instruction("jmp __rt_clone_spl_list_ancestor_x86");               // test the next ancestor
    emitter.label("__rt_clone_spl_list_storage_x86");
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                      // recover the newly allocated clone
    abi::emit_load_from_address(emitter, "rdi", "r10", SPL_DLL_STORAGE_OFFSET);
    abi::emit_call_label(emitter, "__rt_array_ensure_unique");
    emitter.instruction("mov r10, QWORD PTR [rbp - 16]");                      // restore the clone across the storage split
    abi::emit_store_to_address(emitter, "rax", "r10", SPL_DLL_STORAGE_OFFSET);
    abi::emit_store_zero_to_address(emitter, "r10", SPL_DLL_ITER_INDEX_OFFSET);
    emitter.label("__rt_clone_spl_list_done_x86");
}
