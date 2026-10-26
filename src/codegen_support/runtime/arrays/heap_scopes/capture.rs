//! Purpose:
//! Captures heap root cells before their array owners are retired.
//!
//! Called from:
//! - The generated next mutation's compiler-only scope entry operation.
//!
//! Key details:
//! - Reads array<mixed> cells directly, avoiding a cloned temporary as the captured owner.
//! - String boxes may be retained independently because inert bytes have no PHP destructor.

use super::*;

/// Emits scoped capture and publishes a fully initialized receiver record.
pub(super) fn emit(e: &mut Emitter) {
    emit_capture(e);
    e.label_global("__rt_heap_scope_begin");
    abi::emit_frame_prologue(e, 80);
    for index in 0..4 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, index), index * 8);
    }
    let value = result(e);
    let node = if arm64(e) { "x4" } else { "r8" };
    let temporary = if arm64(e) { "x5" } else { "r9" };
    abi::emit_load_int_immediate(e, value, SIZE);
    abi::emit_call_label(e, "__rt_heap_alloc");
    abi::emit_store_to_sp(e, value, 32);
    abi::emit_load_int_immediate(e, temporary, 0);
    for offset in [0, DATA, PRIORITY, ACTIVE, DATA_VALID, PRIORITY_VALID] {
        abi::emit_store_to_address(e, temporary, value, offset);
    }
    abi::emit_load_temporary_stack_slot(e, temporary, 0);
    abi::emit_store_to_address(e, temporary, value, OBJECT);
    abi::emit_load_temporary_stack_slot(e, temporary, 24);
    abi::emit_store_to_address(e, temporary, value, QUEUE);
    capture_array_root(e, 8, DATA, DATA_VALID);
    abi::emit_load_temporary_stack_slot(e, temporary, 24);
    zero(e, temporary, "__rt_heap_scope_begin_publish");
    capture_array_root(e, 16, PRIORITY, PRIORITY_VALID);
    e.label("__rt_heap_scope_begin_publish");
    abi::emit_load_temporary_stack_slot(e, node, 32);
    abi::emit_load_int_immediate(e, temporary, 1);
    abi::emit_store_to_address(e, temporary, node, ACTIVE);
    abi::emit_load_symbol_to_reg(e, temporary, HEAD, 0);
    abi::emit_store_to_address(e, temporary, node, 0);
    let address = if arm64(e) { "x9" } else { "r10" };
    abi::emit_symbol_address(e, address, HEAD);
    abi::emit_store_to_address(e, node, address, 0);
    abi::emit_symbol_address(e, temporary, "__rt_heap_scope_clone_object");
    abi::emit_symbol_address(e, address, CLONE_HOOK);
    abi::emit_store_to_address(e, temporary, address, 0);
    abi::emit_reg_move(e, value, node);
    abi::emit_frame_restore(e, 80);
    abi::emit_return(e);
}

/// Reads the first packed Mixed cell without acquiring a payload owner.
fn capture_array_root(e: &mut Emitter, array_slot: usize, target: usize, valid: usize) {
    let value = result(e);
    let node = if arm64(e) { "x4" } else { "r8" };
    let temporary = if arm64(e) { "x5" } else { "r9" };
    let empty = format!("__rt_heap_scope_capture_empty_{target}");
    let ready = format!("__rt_heap_scope_capture_ready_{target}");
    abi::emit_load_temporary_stack_slot(e, value, array_slot);
    zero(e, value, &empty);
    abi::emit_load_from_address(e, temporary, value, 0);
    zero(e, temporary, &empty);
    abi::emit_load_from_address(e, value, value, 24);
    abi::emit_jump(e, &ready);
    e.label(&empty);
    abi::emit_load_int_immediate(e, value, 0);
    e.label(&ready);
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), value);
    abi::emit_call_label(e, "__rt_heap_scope_capture");
    abi::emit_load_temporary_stack_slot(e, node, 32);
    abi::emit_store_to_address(e, value, node, target);
    abi::emit_load_int_immediate(e, temporary, 1);
    abi::emit_store_to_address(e, temporary, node, valid);
}

/// Stabilizes scalar string bytes without retaining any destructor-bearing PHP payload.
fn emit_capture(e: &mut Emitter) {
    let value = result(e);
    let snapshot = if arm64(e) { "x10" } else { "r10" };
    let temporary = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_heap_scope_capture");
    abi::emit_frame_prologue(e, 32);
    abi::emit_call_label(e, "__rt_borrowed_value_snapshot");
    abi::emit_store_to_sp(e, value, 0);
    abi::emit_load_from_address(e, temporary, value, 8);
    tag_is(e, temporary, 1, "__rt_heap_scope_capture_string");
    abi::emit_jump(e, "__rt_heap_scope_capture_done");
    e.label("__rt_heap_scope_capture_string");
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), value);
    abi::emit_call_label(e, "__rt_borrowed_value_read");
    abi::emit_load_temporary_stack_slot(e, snapshot, 0);
    abi::emit_store_to_address(e, value, snapshot, 0);
    abi::emit_load_from_address(e, temporary, value, 8);
    abi::emit_store_to_address(e, temporary, snapshot, 16);
    abi::emit_load_from_address(e, temporary, value, 16);
    abi::emit_store_to_address(e, temporary, snapshot, 24);
    e.label("__rt_heap_scope_capture_done");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_frame_restore(e, 32);
    abi::emit_return(e);
}
