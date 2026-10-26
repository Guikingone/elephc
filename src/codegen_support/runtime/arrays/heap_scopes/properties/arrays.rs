//! Purpose:
//! Captures container structure without PHP payload owners and materializes dying native property views.
//!
//! Called from:
//! - Native property scope capture, reads and retirement.
//!
//! Key details:
//! - One memo node per source container prevents cycles and repeated shared subgraphs.
//! - Entry snapshots retain only inert strings; object destructors keep their original timing.
//! - A zero-owner source array is copied into a fresh Mixed-element array only when observed.

use super::*;

/// Emits structural capture, acquired reads and raw graph retirement.
pub(super) fn emit(e: &mut Emitter) {
    capture(e);
    read(e);
    drop_graph(e);
}

/// Captures each reachable packed array once before any of its entries begins destruction.
fn capture(e: &mut Emitter) {
    let value = result(e);
    let a = if arm64(e) { "x10" } else { "r10" };
    let b = if arm64(e) { "x11" } else { "r11" };
    let c = if arm64(e) { "x12" } else { "r8" };
    e.label_global("__rt_native_array_capture");
    abi::emit_frame_prologue(e, 128);
    for i in 0..2 { abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, i), i * 8); }
    abi::emit_reg_move(e, value, abi::int_arg_reg_name(e.target, 0));
    abi::emit_load_from_address(e, a, value, 8);
    tag_is(e, a, 5, "__rt_native_array_capture_hash");
    tag_is(e, a, 4, "__rt_native_array_capture_array");
    abi::emit_jump(e, "__rt_native_array_capture_done");
    e.label("__rt_native_array_capture_hash");
    abi::emit_frame_restore(e, 128);
    abi::emit_jump(e, "__rt_native_hash_capture");
    e.label("__rt_native_array_capture_array");
    abi::emit_load_from_address(e, a, value, 16);
    zero(e, a, "__rt_native_array_capture_done");
    abi::emit_store_to_sp(e, a, 16);
    abi::emit_load_temporary_stack_slot(e, b, 8);
    abi::emit_load_from_address(e, value, b, 0);
    e.label("__rt_native_array_capture_find");
    zero(e, value, "__rt_native_array_capture_new");
    abi::emit_load_from_address(e, b, value, 8);
    equal(e, a, b, "__rt_native_array_capture_done");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_jump(e, "__rt_native_array_capture_find");
    e.label("__rt_native_array_capture_new");
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
    abi::emit_load_temporary_stack_slot(e, b, 24);
    abi::emit_store_to_address(e, b, value, 16);
    abi::emit_load_int_immediate(e, b, 4);
    abi::emit_store_to_address(e, b, value, 24);
    ins(e, "ldur x10, [x10, #-8]", "mov r10, QWORD PTR [r10 - 8]");
    ins(e, "lsr x10, x10, #8", "shr r10, 8");
    ins(e, "and x10, x10, #0x7f", "and r10, 0x7f");
    abi::emit_store_to_sp(e, a, 32);
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 48);
    e.label("__rt_native_array_capture_loop");
    abi::emit_load_temporary_stack_slot(e, a, 48);
    abi::emit_load_temporary_stack_slot(e, b, 24);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_native_array_capture_done", "jae __rt_native_array_capture_done");
    abi::emit_load_temporary_stack_slot(e, b, 16);
    abi::emit_load_temporary_stack_slot(e, c, 32);
    tag_is(e, c, 1, "__rt_native_array_capture_string");
    ins(e, "add x11, x11, #24", "add r11, 24");
    ins(e, "ldr x1, [x11, x10, lsl #3]", "mov rdi, QWORD PTR [r11 + r10 * 8]");
    abi::emit_load_int_immediate(e, if arm64(e) { "x2" } else { "rsi" }, 0);
    tag_is(e, c, 7, "__rt_native_array_capture_mixed");
    tag_is(e, c, 9, "__rt_native_array_capture_mixed");
    abi::emit_jump(e, "__rt_native_array_capture_box");
    e.label("__rt_native_array_capture_string");
    ins(e, "add x11, x11, #24", "add r11, 24");
    ins(e, "lsl x10, x10, #4", "shl r10, 4");
    ins(e, "add x11, x11, x10", "add r11, r10");
    ins(e, "ldp x1, x2, [x11]", "mov rdi, QWORD PTR [r11]");
    if !arm64(e) { ins(e, "", "mov rsi, QWORD PTR [r11 + 8]"); }
    e.label("__rt_native_array_capture_box");
    abi::emit_reg_move(e, value, c);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 56);
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), value);
    abi::emit_jump(e, "__rt_native_array_capture_snapshot");
    e.label("__rt_native_array_capture_mixed");
    // The loaded payload register is already the C first argument on x86; ARM uses x0.
    if arm64(e) { abi::emit_reg_move(e, "x0", "x1"); }
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 56);
    e.label("__rt_native_array_capture_snapshot");
    abi::emit_call_label(e, "__rt_heap_scope_capture");
    abi::emit_store_to_sp(e, value, 64);
    abi::emit_load_temporary_stack_slot(e, a, 40);
    abi::emit_load_temporary_stack_slot(e, b, 48);
    ins(e, "add x10, x10, x11, lsl #4", "shl r11, 4");
    if !arm64(e) { ins(e, "", "add r10, r11"); }
    abi::emit_store_to_address(e, value, a, 40);
    abi::emit_load_int_immediate(e, b, 0);
    abi::emit_store_to_address(e, b, a, 32);
    abi::emit_load_temporary_stack_slot(e, value, 56);
    zero(e, value, "__rt_native_array_capture_descend");
    abi::emit_call_label(e, "__rt_decref_mixed");
    e.label("__rt_native_array_capture_descend");
    arg(e, 0, 64);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_array_capture");
    abi::emit_load_temporary_stack_slot(e, a, 48);
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_sp(e, a, 48);
    abi::emit_jump(e, "__rt_native_array_capture_loop");
    e.label("__rt_native_array_capture_done");
    abi::emit_frame_restore(e, 128);
    abi::emit_return(e);
}

/// Reads a live payload normally, or builds an independent value from a retiring array's snapshots.
fn read(e: &mut Emitter) {
    let value = result(e);
    let a = if arm64(e) { "x10" } else { "r10" };
    let b = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_value_read");
    abi::emit_frame_prologue(e, 112);
    for i in 0..2 { abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, i), i * 8); }
    abi::emit_reg_move(e, value, abi::int_arg_reg_name(e.target, 0));
    abi::emit_load_from_address(e, a, value, 8);
    tag_is(e, a, 5, "__rt_native_value_read_array");
    tag_is(e, a, 4, "__rt_native_value_read_array");
    abi::emit_jump(e, "__rt_native_value_read_plain");
    e.label("__rt_native_value_read_array");
    abi::emit_load_from_address(e, b, value, 16);
    zero(e, b, "__rt_native_value_read_plain");
    ins(e, "ldur w10, [x11, #-12]", "mov r10d, DWORD PTR [r11 - 12]");
    zero(e, a, "__rt_native_value_read_find");
    abi::emit_jump(e, "__rt_native_value_read_plain");
    e.label("__rt_native_value_read_find");
    abi::emit_load_temporary_stack_slot(e, value, 8);
    e.label("__rt_native_value_read_find_loop");
    zero(e, value, "__rt_native_value_read_plain");
    abi::emit_load_from_address(e, a, value, 8);
    equal(e, a, b, "__rt_native_value_read_materialize");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_jump(e, "__rt_native_value_read_find_loop");
    e.label("__rt_native_value_read_materialize");
    abi::emit_store_to_sp(e, value, 16);
    abi::emit_load_from_address(e, a, value, 24);
    tag_is(e, a, 5, "__rt_native_value_read_hash");
    abi::emit_load_from_address(e, a, value, 16);
    abi::emit_store_to_sp(e, a, 24);
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), a);
    abi::emit_load_int_immediate(e, abi::int_arg_reg_name(e.target, 1), 8);
    abi::emit_call_label(e, "__rt_array_new");
    abi::emit_store_to_sp(e, value, 32);
    ins(e, "ldur x10, [x0, #-8]", "mov r10, QWORD PTR [rax - 8]");
    ins(e, "orr x10, x10, #0x700", "or r10, 0x700");
    ins(e, "stur x10, [x0, #-8]", "mov QWORD PTR [rax - 8], r10");
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 40);
    e.label("__rt_native_value_read_loop");
    abi::emit_load_temporary_stack_slot(e, a, 40);
    abi::emit_load_temporary_stack_slot(e, b, 24);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_native_value_read_box", "jae __rt_native_value_read_box");
    abi::emit_load_temporary_stack_slot(e, b, 16);
    ins(e, "add x11, x11, x10, lsl #4", "shl r10, 4");
    if !arm64(e) { ins(e, "", "add r11, r10"); }
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), b, 40);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_value_read");
    abi::emit_load_temporary_stack_slot(e, a, 32);
    abi::emit_load_temporary_stack_slot(e, b, 40);
    ins(e, "add x10, x10, #24", "add r10, 24");
    ins(e, "str x0, [x10, x11, lsl #3]", "mov QWORD PTR [r10 + r11 * 8], rax");
    ins(e, "add x11, x11, #1", "add r11, 1");
    abi::emit_store_to_sp(e, b, 40);
    abi::emit_load_temporary_stack_slot(e, a, 32);
    abi::emit_store_to_address(e, b, a, 0);
    abi::emit_jump(e, "__rt_native_value_read_loop");
    e.label("__rt_native_value_read_box");
    abi::emit_load_temporary_stack_slot(e, if arm64(e) { "x1" } else { "rdi" }, 32);
    abi::emit_load_int_immediate(e, value, 4);
    abi::emit_load_int_immediate(e, if arm64(e) { "x2" } else { "rsi" }, 0);
    abi::emit_call_label(e, "__rt_mixed_from_value");
    abi::emit_store_to_sp(e, value, 48);
    abi::emit_load_temporary_stack_slot(e, value, 32);
    abi::emit_call_label(e, "__rt_decref_array");
    abi::emit_load_temporary_stack_slot(e, value, 48);
    abi::emit_jump(e, "__rt_native_value_read_done");
    e.label("__rt_native_value_read_hash");
    arg(e, 0, 16);
    arg(e, 1, 8);
    abi::emit_call_label(e, "__rt_native_hash_materialize");
    abi::emit_jump(e, "__rt_native_value_read_done");
    e.label("__rt_native_value_read_plain");
    arg(e, 0, 0);
    abi::emit_call_label(e, "__rt_borrowed_value_read");
    e.label("__rt_native_value_read_done");
    abi::emit_frame_restore(e, 112);
    abi::emit_return(e);
}

/// Drops only raw memo nodes and inert snapshot stabilizers, once per captured array identity.
fn drop_graph(e: &mut Emitter) {
    let value = result(e);
    let a = if arm64(e) { "x10" } else { "r10" };
    let b = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_native_array_drop");
    abi::emit_frame_prologue(e, 64);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    e.label("__rt_native_array_drop_node");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    zero(e, value, "__rt_native_array_drop_done");
    abi::emit_load_from_address(e, a, value, 0);
    abi::emit_store_to_sp(e, a, 8);
    abi::emit_load_int_immediate(e, a, 0);
    abi::emit_store_to_sp(e, a, 16);
    e.label("__rt_native_array_drop_loop");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_load_from_address(e, b, value, 16);
    abi::emit_load_temporary_stack_slot(e, a, 16);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_native_array_drop_free", "jae __rt_native_array_drop_free");
    ins(e, "add x0, x0, x10, lsl #4", "shl r10, 4");
    if !arm64(e) { ins(e, "", "add rax, r10"); }
    abi::emit_load_from_address(e, a, value, 40);
    abi::emit_store_to_sp(e, a, 24);
    abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), value, 32);
    zero(e, abi::int_arg_reg_name(e.target, 0), "__rt_native_array_drop_value");
    abi::emit_call_label(e, "__rt_heap_scope_drop_capture");
    e.label("__rt_native_array_drop_value");
    arg(e, 0, 24);
    abi::emit_call_label(e, "__rt_heap_scope_drop_capture");
    abi::emit_load_temporary_stack_slot(e, a, 16);
    ins(e, "add x10, x10, #1", "add r10, 1");
    abi::emit_store_to_sp(e, a, 16);
    abi::emit_jump(e, "__rt_native_array_drop_loop");
    e.label("__rt_native_array_drop_free");
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_call_label(e, "__rt_heap_free");
    abi::emit_load_temporary_stack_slot(e, a, 8);
    abi::emit_store_to_sp(e, a, 0);
    abi::emit_jump(e, "__rt_native_array_drop_node");
    e.label("__rt_native_array_drop_done");
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);
}
