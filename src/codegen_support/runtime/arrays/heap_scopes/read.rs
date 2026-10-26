//! Purpose:
//! Selects a retiring logical root or an ordinary packed heap element.
//!
//! Called from:
//! - Compiler-generated current, top, rank and debug-view expressions.
//!
//! Key details:
//! - Does not evaluate a physical root cell while its owner is being destroyed.
//! - Invalidated payloads never become owners of reused allocator storage.

use super::*;

/// Emits an acquired Mixed read from a receiver, backing array, index and priority selector.
pub(super) fn emit(e: &mut Emitter) {
    let value = result(e);
    let node = if arm64(e) { "x12" } else { "r8" };
    let temporary = if arm64(e) { "x10" } else { "r10" };
    let tag = if arm64(e) { "x11" } else { "r11" };
    e.label_global("__rt_heap_scope_read");
    abi::emit_frame_prologue(e, 64);
    for index in 0..4 {
        abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, index), index * 8);
    }
    abi::emit_load_temporary_stack_slot(e, temporary, 16);
    zero(e, temporary, "__rt_heap_scope_read_find");
    abi::emit_jump(e, "__rt_heap_scope_read_physical");
    e.label("__rt_heap_scope_read_find");
    arg(e, 0, 0);
    abi::emit_call_label(e, "__rt_heap_scope_find");
    zero(e, value, "__rt_heap_scope_read_physical");
    abi::emit_reg_move(e, node, value);
    abi::emit_load_from_address(e, temporary, node, ACTIVE);
    zero(e, temporary, "__rt_heap_scope_read_physical");
    abi::emit_load_temporary_stack_slot(e, temporary, 24);
    zero(e, temporary, "__rt_heap_scope_read_data");
    abi::emit_load_from_address(e, value, node, PRIORITY);
    abi::emit_load_from_address(e, temporary, node, PRIORITY_VALID);
    abi::emit_jump(e, "__rt_heap_scope_read_capture");
    e.label("__rt_heap_scope_read_data");
    abi::emit_load_from_address(e, value, node, DATA);
    abi::emit_load_from_address(e, temporary, node, DATA_VALID);
    e.label("__rt_heap_scope_read_capture");
    zero(e, value, "__rt_heap_scope_read_null");
    zero(e, temporary, "__rt_heap_scope_read_null");
    abi::emit_store_to_sp(e, value, 40);
    abi::emit_load_from_address(e, tag, value, 8);
    for category in [4, 5, 10] { tag_is(e, tag, category, "__rt_heap_scope_read_container"); }
    tag_is(e, tag, 9, "__rt_heap_scope_read_resource");
    abi::emit_jump(e, "__rt_heap_scope_read_owned");
    e.label("__rt_heap_scope_read_container");
    abi::emit_load_from_address(e, temporary, value, 16);
    abi::emit_jump(e, "__rt_heap_scope_read_refcount");
    e.label("__rt_heap_scope_read_resource");
    abi::emit_load_from_address(e, temporary, value, 0);
    e.label("__rt_heap_scope_read_refcount");
    abi::emit_store_to_sp(e, temporary, 32);
    abi::emit_reg_move(e, value, temporary);
    abi::emit_call_label(e, "__rt_heap_kind");
    zero(e, value, "__rt_heap_scope_read_owned");
    abi::emit_load_temporary_stack_slot(e, temporary, 32);
    ins(e, "ldur w10, [x10, #-12]", "mov r10d, DWORD PTR [r10 - 12]");
    zero(e, temporary, "__rt_heap_scope_read_null");
    e.label("__rt_heap_scope_read_owned");
    arg(e, 0, 40);
    abi::emit_call_label(e, "__rt_borrowed_value_read");
    abi::emit_jump(e, "__rt_heap_scope_read_done");
    e.label("__rt_heap_scope_read_physical");
    abi::emit_load_temporary_stack_slot(e, node, 8);
    zero(e, node, "__rt_heap_scope_read_null");
    abi::emit_load_temporary_stack_slot(e, temporary, 16);
    abi::emit_load_from_address(e, tag, node, 0);
    ins(e, "cmp x10, x11", "cmp r10, r11");
    ins(e, "b.hs __rt_heap_scope_read_null", "jae __rt_heap_scope_read_null");
    ins(e, "add x12, x12, #24", "add r8, 24");
    ins(e, "ldr x0, [x12, x10, lsl #3]", "mov rax, QWORD PTR [r8 + r10 * 8]");
    abi::emit_call_label(e, "__rt_mixed_clone");
    abi::emit_jump(e, "__rt_heap_scope_read_done");
    e.label("__rt_heap_scope_read_null");
    abi::emit_load_int_immediate(e, value, 0);
    abi::emit_call_label(e, "__rt_mixed_clone");
    e.label("__rt_heap_scope_read_done");
    abi::emit_frame_restore(e, 64);
    abi::emit_return(e);
}
