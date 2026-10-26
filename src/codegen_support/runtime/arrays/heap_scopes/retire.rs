//! Purpose:
//! Publishes completed root writes and retires native heap mutation scopes.
//!
//! Called from:
//! - Generated sift writes, mutation finally blocks and final runtime cleanup.
//!
//! Key details:
//! - Unlinks before freeing any storage, including out-of-order fiber completion.
//! - Only inert captured string boxes are owned; other captured PHP values are never released.

use super::*;

/// Emits view publication, exact scope retirement and final abandoned-scope cleanup.
pub(super) fn emit(e: &mut Emitter) {
    emit_drop_capture(e);
    emit_end(e);
    e.label_global("__rt_heap_scope_root_written");
    abi::emit_frame_prologue(e, 16);
    abi::emit_call_label(e, "__rt_heap_scope_find");
    zero(e, result(e), "__rt_heap_scope_root_written_done");
    let temporary = if arm64(e) { "x10" } else { "r10" };
    abi::emit_load_int_immediate(e, temporary, 0);
    abi::emit_store_to_address(e, temporary, result(e), ACTIVE);
    e.label("__rt_heap_scope_root_written_done");
    abi::emit_frame_restore(e, 16);
    abi::emit_return(e);

    e.label_global("__rt_heap_scope_shutdown");
    abi::emit_frame_prologue(e, 16);
    e.label("__rt_heap_scope_shutdown_loop");
    abi::emit_load_symbol_to_reg(e, result(e), HEAD, 0);
    zero(e, result(e), "__rt_heap_scope_shutdown_done");
    abi::emit_reg_move(e, abi::int_arg_reg_name(e.target, 0), result(e));
    abi::emit_call_label(e, "__rt_heap_scope_end");
    abi::emit_jump(e, "__rt_heap_scope_shutdown_loop");
    e.label("__rt_heap_scope_shutdown_done");
    abi::emit_call_label(e, "__rt_native_property_shutdown");
    abi::emit_frame_restore(e, 16);
    abi::emit_return(e);
}

/// Releases a string stabilizer, then the raw logical-value snapshot itself.
fn emit_drop_capture(e: &mut Emitter) {
    let value = result(e);
    let temporary = if arm64(e) { "x10" } else { "r10" };
    e.label_global("__rt_heap_scope_drop_capture");
    abi::emit_frame_prologue(e, 32);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    abi::emit_reg_move(e, value, abi::int_arg_reg_name(e.target, 0));
    zero(e, value, "__rt_heap_scope_drop_capture_done");
    abi::emit_load_from_address(e, temporary, value, 8);
    tag_is(e, temporary, 1, "__rt_heap_scope_drop_capture_string");
    abi::emit_jump(e, "__rt_heap_scope_drop_capture_free");
    e.label("__rt_heap_scope_drop_capture_string");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_call_label(e, "__rt_decref_mixed");
    e.label("__rt_heap_scope_drop_capture_free");
    arg(e, 0, 0);
    abi::emit_call_label(e, "__rt_borrowed_value_free");
    e.label("__rt_heap_scope_drop_capture_done");
    abi::emit_frame_restore(e, 32);
    abi::emit_return(e);
}

/// Detaches exactly one live record without dereferencing an unrecognized token.
fn emit_end(e: &mut Emitter) {
    let value = result(e);
    let token = if arm64(e) { "x12" } else { "r8" };
    let link = if arm64(e) { "x9" } else { "r10" };
    let cursor = if arm64(e) { "x10" } else { "r11" };
    let next = if arm64(e) { "x11" } else { "rdx" };
    e.label_global("__rt_heap_scope_end");
    abi::emit_frame_prologue(e, 32);
    abi::emit_store_to_sp(e, abi::int_arg_reg_name(e.target, 0), 0);
    abi::emit_reg_move(e, token, abi::int_arg_reg_name(e.target, 0));
    zero(e, token, "__rt_heap_scope_end_done");
    abi::emit_symbol_address(e, link, HEAD);
    e.label("__rt_heap_scope_end_search");
    abi::emit_load_from_address(e, cursor, link, 0);
    zero(e, cursor, "__rt_heap_scope_end_done");
    equal(e, cursor, token, "__rt_heap_scope_end_found");
    abi::emit_reg_move(e, link, cursor);
    abi::emit_jump(e, "__rt_heap_scope_end_search");
    e.label("__rt_heap_scope_end_found");
    abi::emit_load_from_address(e, next, cursor, 0);
    abi::emit_store_to_address(e, next, link, 0);
    abi::emit_load_symbol_to_reg(e, value, HEAD, 0);
    zero(e, value, "__rt_heap_scope_end_clear_hook");
    abi::emit_jump(e, "__rt_heap_scope_end_drop");
    e.label("__rt_heap_scope_end_clear_hook");
    abi::emit_symbol_address(e, link, CLONE_HOOK);
    abi::emit_load_int_immediate(e, next, 0);
    abi::emit_store_to_address(e, next, link, 0);
    e.label("__rt_heap_scope_end_drop");
    for offset in [DATA, PRIORITY] {
        abi::emit_load_temporary_stack_slot(e, value, 0);
        abi::emit_load_from_address(e, abi::int_arg_reg_name(e.target, 0), value, offset);
        abi::emit_call_label(e, "__rt_heap_scope_drop_capture");
    }
    abi::emit_load_temporary_stack_slot(e, value, 0);
    abi::emit_call_label(e, "__rt_heap_free");
    e.label("__rt_heap_scope_end_done");
    abi::emit_frame_restore(e, 32);
    abi::emit_return(e);
}
