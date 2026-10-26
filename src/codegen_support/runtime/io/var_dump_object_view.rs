//! Purpose:
//! Renders one object using a single native magic-debug snapshot when available.
//!
//! Called from:
//! - The object arm of the scoped var_dump value implementation.
//!
//! Key details:
//! - The enclosing value scope owns the receiver, returned Mixed cell and snapshot container.
//! - The caller has already pushed the object's recursion marker.
//! - Null deprecations are guarded so their exceptions propagate only after the empty view closes.

use crate::codegen_support::{abi, emit::Emitter};
use crate::codegen_support::runtime::exceptions;
use super::var_dump_value_scope::{
    branch_tag, context_literal, read_context, write_context,
    ACTIVE_INDENT, DEBUG_CONTAINER, DEBUG_OWNER, INPUT_OWNED, VIEW_STAGE,
};

const FRAME: usize = 80;
const OBJECT: usize = 8;
const SCOPE: usize = 16;
const KIND: usize = 24;
const DATA: usize = 32;
const CLASS: usize = 40;
const HANDLE: usize = 48;
const PENDING: usize = 56;

/// Emits header/body rendering while transferring snapshot ownership to the enclosing scope.
pub fn emit_var_dump_object_view(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let first = abi::int_arg_reg_name(emitter.target, 0);
    let second = abi::int_arg_reg_name(emitter.target, 1);
    let low = match emitter.target.arch {
        crate::codegen_support::platform::Arch::AArch64 => "x1",
        crate::codegen_support::platform::Arch::X86_64 => "rdi",
    };
    emitter.blank();
    emitter.comment("--- runtime: var_dump_object_view ---");
    emitter.label_global("__rt_var_dump_object_view");
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, first, OBJECT);
    abi::store_at_offset(emitter, second, SCOPE);
    abi::emit_store_zero_to_local_slot(emitter, PENDING);
    abi::load_at_offset(emitter, result, OBJECT);
    abi::emit_load_from_address(emitter, result, result, 0);
    abi::store_at_offset(emitter, result, CLASS);
    abi::load_at_offset(emitter, result, OBJECT);
    abi::emit_call_label(emitter, "__rt_object_handle_of");
    abi::store_at_offset(emitter, result, HANDLE);
    abi::emit_call_label(emitter, "__rt_vd_pad");
    context_literal(emitter, SCOPE, VIEW_STAGE, 1);
    abi::load_at_offset(emitter, first, OBJECT);
    abi::emit_call_label(emitter, "__rt_object_debug_info");
    write_context(emitter, result, SCOPE, DEBUG_OWNER);
    context_literal(emitter, SCOPE, VIEW_STAGE, 2);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_object_view_declared");
    context_literal(emitter, SCOPE, INPUT_OWNED, 0);
    read_context(emitter, result, SCOPE, super::var_dump_value_scope::LOW);
    // The original input may be a Mixed owner, not the unboxed receiver itself.
    abi::emit_call_label(emitter, "__rt_decref_any");
    read_context(emitter, result, SCOPE, DEBUG_OWNER);
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    branch_tag(emitter, 4, "__rt_var_dump_object_view_array");
    branch_tag(emitter, 5, "__rt_var_dump_object_view_array");
    branch_tag(emitter, 8, "__rt_var_dump_object_view_null");
    abi::emit_jump(emitter, "__rt_debug_info_invalid_result");
    emitter.label("__rt_var_dump_object_view_array");
    abi::store_at_offset(emitter, result, KIND);
    abi::store_at_offset(emitter, low, DATA);
    abi::emit_reg_move(emitter, result, low);
    abi::emit_call_label(emitter, "__rt_incref");
    abi::load_at_offset(emitter, result, DATA);
    write_context(emitter, result, SCOPE, DEBUG_CONTAINER);
    release_result_cell(emitter);
    abi::emit_jump(emitter, "__rt_var_dump_object_view_open");
    emitter.label("__rt_var_dump_object_view_null");
    abi::emit_load_int_immediate(emitter, result, 8);
    abi::store_at_offset(emitter, result, KIND);
    abi::emit_store_zero_to_local_slot(emitter, DATA);
    release_result_cell(emitter);
    abi::load_at_offset(emitter, result, CLASS);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_debug_info_warn_null", result, PENDING,
    );
    emitter.label("__rt_var_dump_object_view_open");
    read_context(emitter, result, SCOPE, ACTIVE_INDENT);
    abi::emit_store_reg_to_symbol(emitter, result, "_vd_indent", 0);
    let count = abi::int_arg_reg_name(emitter.target, 2);
    abi::load_at_offset(emitter, result, DATA);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_object_view_empty_count");
    abi::emit_load_from_address(emitter, count, result, 0);
    abi::emit_jump(emitter, "__rt_var_dump_object_view_counted");
    emitter.label("__rt_var_dump_object_view_empty_count");
    abi::emit_load_int_immediate(emitter, count, 0);
    emitter.label("__rt_var_dump_object_view_counted");
    abi::load_at_offset(emitter, first, CLASS);
    abi::load_at_offset(emitter, second, HANDLE);
    abi::emit_call_label(emitter, "__rt_var_dump_open_object_info");
    abi::emit_call_label(emitter, "__rt_vd_indent_push");
    abi::load_at_offset(emitter, result, KIND);
    branch_tag(emitter, 4, "__rt_var_dump_object_view_indexed");
    branch_tag(emitter, 5, "__rt_var_dump_object_view_hash");
    abi::emit_jump(emitter, "__rt_var_dump_object_view_close");
    emitter.label("__rt_var_dump_object_view_indexed");
    abi::load_at_offset(emitter, first, DATA);
    abi::emit_call_label(emitter, "__rt_var_dump_indexed");
    abi::emit_jump(emitter, "__rt_var_dump_object_view_close");
    emitter.label("__rt_var_dump_object_view_hash");
    abi::load_at_offset(emitter, first, DATA);
    abi::emit_call_label(emitter, "__rt_var_dump_debug_hash");
    abi::emit_jump(emitter, "__rt_var_dump_object_view_close");
    emitter.label("__rt_var_dump_object_view_declared");
    read_context(emitter, result, SCOPE, ACTIVE_INDENT);
    abi::emit_store_reg_to_symbol(emitter, result, "_vd_indent", 0);
    abi::load_at_offset(emitter, first, OBJECT);
    abi::emit_call_label(emitter, "__rt_vd_obj_count");
    abi::emit_reg_move(emitter, abi::int_arg_reg_name(emitter.target, 2), result);
    abi::load_at_offset(emitter, first, CLASS);
    abi::load_at_offset(emitter, second, HANDLE);
    abi::emit_call_label(emitter, "__rt_var_dump_open_object_info");
    abi::emit_call_label(emitter, "__rt_vd_indent_push");
    abi::load_at_offset(emitter, first, OBJECT);
    abi::emit_call_label(emitter, "__rt_var_dump_object");
    emitter.label("__rt_var_dump_object_view_close");
    for field in [DEBUG_CONTAINER, DEBUG_OWNER] {
        read_context(emitter, result, SCOPE, field);
        context_literal(emitter, SCOPE, field, 0);
        exceptions::emit_guarded_cleanup_call(
            emitter, "__rt_decref_any", result, PENDING,
        );
    }
    abi::load_at_offset(emitter, result, PENDING);
    write_context(emitter, result, SCOPE, super::var_dump_value_scope::PENDING);
    read_context(emitter, result, SCOPE, ACTIVE_INDENT);
    abi::emit_store_reg_to_symbol(emitter, result, "_vd_indent", 0);
    abi::emit_call_label(emitter, "__rt_var_dump_close_container");
    read_context(emitter, result, SCOPE, super::var_dump_value_scope::PENDING);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_object_view_return");
    context_literal(emitter, SCOPE, super::var_dump_value_scope::PENDING, 0);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_var_dump_object_view_return");
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

/// Transfers the logical array owner out of any reference/result cell before walking its snapshot.
fn release_result_cell(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    read_context(emitter, result, SCOPE, DEBUG_OWNER);
    context_literal(emitter, SCOPE, DEBUG_OWNER, 0);
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_decref_any", result, PENDING);
}
