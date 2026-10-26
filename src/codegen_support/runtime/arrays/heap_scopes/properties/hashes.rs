//! Purpose:
//! Captures associative native property views before their buckets begin destruction.
//!
//! Called from:
//! - The native container snapshot dispatcher and acquired value reader.
//!
//! Key details:
//! - Memo entries store inert key snapshots and borrowed logical value snapshots in insertion order.
//! - No object, container or resource owner is retained by capture.
//! - Materialization consumes each acquired Mixed value through the ordinary hash builder ABI.

use super::*;

/// Emits hash capture and reconstruction using the shared container memo layout.
pub(super) fn emit(e: &mut Emitter) {
    capture(e);
    materialize(e);
}

/// Captures each reachable hash identity once, before recursive values can revisit it.
fn capture(e: &mut Emitter) {
    let value = result(e);
    let a = if arm64(e) { "x10" } else { "r10" };
    let b = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_hash_capture");
    abi::emit_frame_prologue(e, 160);
    for i in 0..2 { abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, i), i * 8); }
    abi::emit_reg_move(e, value, abi::int_arg_reg_name(e.target, 0));
    abi::emit_load_from_address(e, a, value, 16);
    zero(e, a, "__rt_native_hash_capture_done");
    abi::emit_store_to_sp(e, a, 16);
    abi::emit_load_temporary_stack_slot(e, b, 8);
    abi::emit_load_from_address(e, value, b, 0);
    e.label("__rt_native_hash_capture_find");
    zero(e, value, "__rt_native_hash_capture_new");
    abi::emit_load_from_address(e, b, value, 8);
    equal(e, a, b, "__rt_native_hash_capture_done");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_jump(e, "__rt_native_hash_capture_find");
    e.label("__rt_native_hash_capture_new");
    abi::emit_load_from_address(e, b, a, 0);
    abi::emit_store_to_sp(e, b, 24);
    ins(e, "lsl x0, x11, #4", "mov rax, r11");
    if !arm64(e) { ins(e, "", "shl rax, 4"); }
    ins(e, "add x0, x0, #32", "add rax, 32");
    abi::emit_call_label(e, "__rt_heap_alloc");
    abi::emit_store_to_sp(e, value, 40);
    abi::emit_load_temporary_stack_slot(e, a, 8);
    abi::emit_load_from_address(e, b, a, 0);
    abi::emit_store_to_address(e, b, value, 0);
    abi::emit_store_to_address(e, value, a, 0);
    abi::emit_load_temporary_stack_slot(e, a, 16);
    abi::emit_store_to_address(e, a, value, 8);
    abi::emit_load_temporary_stack_slot(e, a, 24);
    abi::emit_store_to_address(e, a, value, 16);
    abi::emit_load_int_immediate(e, a, 5);
    abi::emit_store_to_address(e, a, value, 24);
    abi::emit_load_int_immediate(e, a, 0);
    for offset in [32, 48] { abi::emit_store_to_sp(e, a, offset); }
    e.label("__rt_native_hash_capture_loop");
    abi::emit_load_temporary_stack_slot(e, a, 48);
    abi::emit_load_temporary_stack_slot(e, b, 24);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_native_hash_capture_done", "jae __rt_native_hash_capture_done");
    arg(e, 0, 16);
    arg(e, 1, 32);
    abi::emit_call_label(e, "__rt_hash_iter_next_value");
    abi::emit_store_to_sp(e, value, 32);
    let registers = if arm64(e) { ["x1", "x2", "x3", "x4", "x5"] } else { ["rdi", "rdx", "rcx", "r8", "r9"] };
    for (i, register) in registers.iter().enumerate() { abi::emit_store_to_sp(e, register, 72 + i * 8); }
    abi::emit_load_temporary_stack_slot(e, a, 80);
    ins(e, "cmn x10, #1", "cmp r10, -1");
    ins(e, "b.eq __rt_native_hash_capture_int_key", "je __rt_native_hash_capture_int_key");
    abi::emit_load_int_immediate(e, value, 1);
    abi::emit_jump(e, "__rt_native_hash_capture_key_box");
    e.label("__rt_native_hash_capture_int_key");
    abi::emit_load_int_immediate(e, value, 0);
    e.label("__rt_native_hash_capture_key_box");
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x1" } else { "rdi" }, 72);
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x2" } else { "rsi" }, 80);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 56);
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), value);
    abi::emit_call_label(e, "__rt_heap_scope_capture");
    abi::emit_store_to_sp(e, value, 112);
    abi::emit_load_temporary_stack_slot(e, value, 56);
    abi::emit_call_label(e, "__rt_decref_mixed");
    abi::emit_load_temporary_stack_slot(e, a, 104);
    tag_is(e, a, 7, "__rt_native_hash_capture_boxed");
    tag_is(e, a, 9, "__rt_native_hash_capture_boxed");
    abi::emit_reg_move(e, value, a);
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x1" } else { "rdi" }, 88);
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x2" } else { "rsi" }, 96);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 56);
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), value);
    abi::emit_jump(e, "__rt_native_hash_capture_snapshot");
    e.label("__rt_native_hash_capture_boxed");
    arg(e, 0, 88);
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 56);
    e.label("__rt_native_hash_capture_snapshot");
    abi::emit_call_label(e, "__rt_heap_scope_capture");
    abi::emit_store_to_sp(e, value, 64);
    abi::emit_load_temporary_stack_slot(e, a, 40);
    abi::emit_load_temporary_stack_slot(e, b, 48);
    ins(e, "add x10, x10, x11, lsl #4", "shl r11, 4");
    if !arm64(e) { ins(e, "", "add r10, r11"); }
    abi::emit_store_to_address(e, value, a, 40);
    abi::emit_load_temporary_stack_slot(e, b, 112);
    abi::emit_store_to_address(e, b, a, 32);
    abi::emit_load_temporary_stack_slot(e, value, 56);
    zero(e, value, "__rt_native_hash_capture_descend");
    abi::emit_call_label(e, "__rt_decref_mixed");
    e.label("__rt_native_hash_capture_descend");
    arg(e, 0, 64);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_array_capture");
    abi::emit_load_temporary_stack_slot(e, a, 48);
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_sp(e, a, 48);
    abi::emit_jump(e, "__rt_native_hash_capture_loop");
    e.label("__rt_native_hash_capture_done");
    abi::emit_frame_restore(e, 160);
    abi::emit_return(e);
}

/// Rebuilds a dying hash in insertion order without reading its already retiring buckets.
fn materialize(e: &mut Emitter) {
    let value = result(e);
    let a = if arm64(e) { "x10" } else { "r10" };
    let b = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_hash_materialize");
    abi::emit_frame_prologue(e, 96);
    for i in 0..2 { abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, i), i * 8); }
    abi::emit_load_from_address(e, a, abi::int_arg_reg_name(e.target, 0), 16);
    abi::emit_store_to_sp(e, a, 32);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 0), 8);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 1), 7);
    abi::emit_call_label(e, "__rt_hash_new");
    abi::emit_store_to_sp(e, value, 16);
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 24);
    e.label("__rt_native_hash_materialize_loop");
    abi::emit_load_temporary_stack_slot(e, a, 24);
    abi::emit_load_temporary_stack_slot(e, b, 32);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_native_hash_materialize_box", "jae __rt_native_hash_materialize_box");
    abi::emit_load_temporary_stack_slot(e, b, 0);
    ins(e, "add x11, x11, x10, lsl #4", "shl r10, 4");
    if !arm64(e) { ins(e, "", "add r11, r10"); }
    abi::emit_load_from_address(e, a, b, 32);
    abi::emit_store_to_sp(e, a, 48);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), b, 40);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_value_read");
    abi::emit_store_to_sp(e, value, 40);
    abi::emit_load_temporary_stack_slot(e, a, 48);
    abi::emit_load_from_address(e, b, a, 8);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 1), a, 16);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 2), a, 24);
    tag_is(e, b, 1, "__rt_native_hash_materialize_key_ready");
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 2), -1);
    e.label("__rt_native_hash_materialize_key_ready");
    arg(e, 0, 16);
    arg(e, 3, 40);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 4), 0);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 5), 7);
    abi::emit_call_label(e, "__rt_hash_set_value");
    abi::emit_store_to_sp(e, value, 16);
    abi::emit_load_temporary_stack_slot(e, a, 24);
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_sp(e, a, 24);
    abi::emit_jump(e, "__rt_native_hash_materialize_loop");
    e.label("__rt_native_hash_materialize_box");
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x1" } else { "rdi" }, 16);
    abi::emit_load_int_immediate(e, if arm64(e) { "x2" } else { "rsi" }, 0);
    abi::emit_load_int_immediate(e, value, 5);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 56);
    abi::emit_load_temporary_stack_slot(e, value, 16);
    abi::emit_call_label(e, "__rt_decref_hash");
    abi::emit_load_temporary_stack_slot(e, value, 56);
    abi::emit_frame_restore(e, 96);
    abi::emit_return(e);
}
