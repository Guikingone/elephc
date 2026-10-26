//! Purpose:
//! Owns the native argument array built by AOT trigger_error across callback and cleanup exceptions.
//!
//! Called from:
//! - `super::emit_error_handler_invoke()` emits the owning boundary and unary adapter.
//! - `crate::codegen::lower_inst::core_builtins::handlers::lower_trigger_error()` transfers its array.
//!
//! Key details:
//! - The suspended-handler invoker borrows arguments; this outer boundary consumes their owner once.
//! - A normalized boxed result survives argument cleanup or is released before propagating a throw.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;

const FRAME: usize = 64;
const CONTEXT: usize = 40;
const ARGUMENTS: usize = 32;
const RESULT: usize = 24;
const PENDING: usize = 8;

/// Emits a consuming argument-array wrapper using the same exception boundary on both architectures.
pub(super) fn emit_owned_arguments(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_error_handler_invoke_owned_args");
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 0), CONTEXT);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 1), ARGUMENTS);
    abi::emit_store_zero_to_local_slot(emitter, RESULT);
    abi::emit_store_zero_to_local_slot(emitter, PENDING);
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_error_handler_owned_callback", result, PENDING,
    );
    abi::load_at_offset(emitter, result, ARGUMENTS);
    abi::emit_store_zero_to_local_slot(emitter, ARGUMENTS);
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_decref_mixed", result, PENDING);
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_error_handler_owned_return");
    abi::load_at_offset(emitter, result, RESULT);
    abi::emit_store_zero_to_local_slot(emitter, RESULT);
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_decref_mixed", result, PENDING);
    abi::load_at_offset(emitter, result, PENDING);
    let older = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_error_handler_owned_return");
    abi::load_at_offset(emitter, result, RESULT);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emit_callback(emitter);
}

/// Adapts the parent's owned context to the borrowed-argument suspended-handler invoker.
fn emit_callback(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let context = abi::secondary_scratch_reg(emitter);
    let arg0 = abi::int_arg_reg_name(emitter.target, 0);
    let arg1 = abi::int_arg_reg_name(emitter.target, 1);
    emitter.blank();
    emitter.label_global("__rt_error_handler_owned_callback");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    abi::emit_load_from_address(emitter, result, result, 0);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_error_handler_owned_missing");
    abi::load_at_offset(emitter, context, 8);
    abi::emit_load_from_address(emitter, arg0, context, 0);
    abi::emit_load_from_address(emitter, arg1, context, 8);
    abi::emit_call_label(emitter, "__rt_error_handler_invoke");
    abi::emit_jump(emitter, "__rt_error_handler_owned_stash");
    emitter.label("__rt_error_handler_owned_missing");
    let low = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    let high = if emitter.target.arch == Arch::AArch64 { "x2" } else { "rsi" };
    abi::emit_load_int_immediate(emitter, result, 3);
    abi::emit_load_int_immediate(emitter, low, 0);
    abi::emit_load_int_immediate(emitter, high, 0);
    abi::emit_call_label(emitter, "__rt_mixed_from_value");
    emitter.label("__rt_error_handler_owned_stash");
    abi::load_at_offset(emitter, context, 8);
    abi::emit_store_to_address(emitter, result, context, 16);
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
}
