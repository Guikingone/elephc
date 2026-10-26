//! Purpose:
//! Materializes a mutating heap's logical prefix before a clone reaches user code.
//!
//! Called from:
//! - The optional native clone post-copy hook while a receiver-bound scope is active.
//!
//! Key details:
//! - Copies logical count rather than the temporarily larger physical tail.
//! - Reads the retiring root through its scope, never through a dying physical cell.
//! - Clones keep native lock/corruption state but do not inherit the pending exception slot.

use super::*;
use crate::types::spl_heap_layout::{COUNT_OFFSET, ERROR_OFFSET, PRIORITIES_OFFSET, VALUES_OFFSET};

/// Emits canonical heap-prefix clone materialization and its owned packed-array builder.
pub(super) fn emit(e: &mut Emitter) {
    emit_clone_array(e);
    let value = result(e);
    let object = if arm64(e) { "x12" } else { "r8" };
    let temporary = if arm64(e) { "x10" } else { "r10" };
    e.label_global("__rt_heap_scope_clone_object");
    abi::emit_frame_prologue(e, 64);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 1), 8);
    abi::emit_call_label(e, "__rt_heap_scope_find");
    zero(e, value, "__rt_heap_scope_clone_done");
    abi::emit_load_from_address(e, temporary, value, QUEUE);
    abi::emit_store_to_sp(e, temporary, 16);
    abi::emit_load_temporary_stack_slot(e, object, 0);
    abi::emit_load_from_address(e, temporary, object, COUNT_OFFSET);
    abi::emit_store_to_sp(e, temporary, 24);
    copy_field(e, VALUES_OFFSET, false);
    abi::emit_load_temporary_stack_slot(e, temporary, 16);
    zero(e, temporary, "__rt_heap_scope_clone_error");
    copy_field(e, PRIORITIES_OFFSET, true);
    e.label("__rt_heap_scope_clone_error");
    abi::emit_load_temporary_stack_slot(e, object, 8);
    abi::emit_load_from_address(e, value, object, ERROR_OFFSET);
    abi::emit_load_int_immediate(e, temporary, 0);
    abi::emit_store_to_address(e, temporary, object, ERROR_OFFSET);
    abi::emit_call_label(e, "__rt_decref_mixed");
    e.label("__rt_heap_scope_clone_done");
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);
}

/// Publishes a newly owned logical array before releasing the clone's old shared array owner.
fn copy_field(e: &mut Emitter, offset: usize, priority: bool) {
    let value = result(e);
    let object = if arm64(e) { "x12" } else { "r8" };
    let replacement = if arm64(e) { "x11" } else { "r11" };
    arg(e, 0, 0);
    abi::emit_load_temporary_stack_slot(e, object, 8);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 1), object, offset);
    arg(e, 2, 24);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 3), i64::from(priority));
    abi::emit_call_label(e, "__rt_heap_scope_clone_array");
    abi::emit_store_to_sp(e, value, 32);
    abi::emit_load_temporary_stack_slot(e, object, 8);
    abi::emit_load_from_address(e, value, object, offset);
    abi::emit_load_temporary_stack_slot(e, replacement, 32);
    abi::emit_store_to_address(e, replacement, object, offset);
    abi::emit_call_label(e, "__rt_decref_array");
}

/// Copies exactly the initialized logical prefix, acquiring one Mixed cell per array element.
fn emit_clone_array(e: &mut Emitter) {
    let value = result(e);
    let array = if arm64(e) { "x11" } else { "r11" };
    let index = if arm64(e) { "x10" } else { "r10" };
    let count = if arm64(e) { "x12" } else { "r8" };
    e.label_global("__rt_heap_scope_clone_array");
    abi::emit_frame_prologue(e, 64);
    for argument in 0..4 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, argument), argument * 8);
    }
    abi::emit_load_temporary_stack_slot(e, abi::int_arg_reg_name(e.target, 0), 16);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 1), 8);
    abi::emit_call_label(e, "__rt_array_new");
    abi::emit_store_to_sp(e, value, 32);
    abi::emit_load_temporary_stack_slot(e, array, 8);
    zero(e, array, "__rt_heap_scope_clone_array_meta_done");
    ins(e, "ldur x10, [x11, #-8]", "mov r10, QWORD PTR [r11 - 8]");
    ins(e, "and x10, x10, #0x7f00", "and r10, 0x7f00");
    ins(e, "ldur x13, [x0, #-8]", "mov rdx, QWORD PTR [rax - 8]");
    ins(e, "orr x13, x13, x10", "or rdx, r10");
    ins(e, "stur x13, [x0, #-8]", "mov QWORD PTR [rax - 8], rdx");
    e.label("__rt_heap_scope_clone_array_meta_done");
    abi::emit_load_int_immediate(e, index, 0);
    abi::emit_store_to_sp(e, index, 40);
    e.label("__rt_heap_scope_clone_array_loop");
    abi::emit_load_temporary_stack_slot(e, index, 40);
    abi::emit_load_temporary_stack_slot(e, count, 16);
    ins(e, &format!("cmp {index}, {count}"), &format!("cmp {index}, {count}"));
    ins(e, "b.ge __rt_heap_scope_clone_array_done", "jge __rt_heap_scope_clone_array_done");
    arg(e, 0, 0);
    arg(e, 1, 8);
    arg(e, 2, 40);
    arg(e, 3, 24);
    abi::emit_call_label(e, "__rt_heap_scope_read");
    abi::emit_load_temporary_stack_slot(e, array, 32);
    abi::emit_load_temporary_stack_slot(e, index, 40);
    ins(e, "add x13, x11, #24", "lea rdx, [r11 + 24]");
    ins(e, "str x0, [x13, x10, lsl #3]", "mov QWORD PTR [rdx + r10 * 8], rax");
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_address(e, index, array, 0);
    abi::emit_store_to_sp(e, index, 40);
    abi::emit_jump(e, "__rt_heap_scope_clone_array_loop");
    e.label("__rt_heap_scope_clone_array_done");
    abi::emit_load_temporary_stack_slot(e, value, 32);
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);
}
