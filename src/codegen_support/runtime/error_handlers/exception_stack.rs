//! Purpose:
//! Copies an invoked exception registration onto PHP's previous-handler stack and leases its call frame.
//!
//! Called from:
//! - `super::emit_invoke()` before the exception callback and immediately after its return.
//! - `super::emit_restore()` to retire suspended registration snapshots safely.
//!
//! Key details:
//! - Stack copies own independent boxed values, descriptors and optional eval-context references.
//! - The invocation snapshot ends before ignored PHP return-value destruction.
//! - Opaque eval contexts use their existing C retain/free ABI, never native heap refcount operations.

use crate::codegen_support::{abi, callable_descriptor, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;

/// Copies the suspended tuple into a linked stack node, then records successful publication.
pub(super) fn emit_suspend_copy(emitter: &mut Emitter, stack: &str, eval_bridge: bool) {
    let result = abi::int_result_reg(emitter);
    let node = abi::secondary_scratch_reg(emitter);
    abi::emit_load_int_immediate(emitter, result, 56);
    abi::emit_call_label(emitter, "__rt_heap_alloc");
    abi::emit_store_to_address(emitter, result, stack, 96);
    abi::emit_load_symbol_to_reg(emitter, result, "_php_exception_handler_stack", 0);
    abi::emit_load_from_address(emitter, node, stack, 96);
    abi::emit_store_to_address(emitter, result, node, 0);
    for (source, destination) in [(32, 8), (40, 16)] {
        abi::emit_load_from_address(emitter, result, stack, source);
        if source == 32 {
            abi::emit_call_label(emitter, "__rt_incref");
        } else {
            callable_descriptor::emit_retain_current_descriptor(emitter);
        }
        abi::emit_load_from_address(emitter, node, stack, 96);
        abi::emit_store_to_address(emitter, result, node, destination);
    }
    if eval_bridge {
        abi::emit_load_from_address(emitter, result, stack, 48);
        abi::emit_branch_if_int_result_zero(emitter, "__rt_exception_handler_no_context_copy");
        abi::emit_reg_move(emitter, abi::int_arg_reg_name(emitter.target, 0), result);
        emitter.bl_c("__elephc_eval_context_retain");                           // clone the opaque context through its own ABI before publishing the stack copy
        emitter.label("__rt_exception_handler_no_context_copy");
    }
    for (source, destination) in [(48, 24), (56, 32)] {
        abi::emit_load_from_address(emitter, result, stack, source);
        abi::emit_load_from_address(emitter, node, stack, 96);
        abi::emit_store_to_address(emitter, result, node, destination);
    }
    abi::emit_load_from_address(emitter, result, stack, 96);
    abi::emit_store_reg_to_symbol(emitter, result, "_php_exception_handler_stack", 0);
    abi::emit_store_zero_to_address(emitter, stack, 96);
    abi::emit_load_int_immediate(emitter, result, 2);
    abi::emit_store_to_address(emitter, result, stack, 80);
}

/// Retires each tuple owner before callbacks can reenter the surrounding cleanup activation.
pub(super) fn release_registration(
    emitter: &mut Emitter, record_slot: usize, pending: usize, context_slot: usize, done: &str,
) {
    let result = abi::int_result_reg(emitter);
    let record = abi::secondary_scratch_reg(emitter);
    for (offset, entry) in [(32, "__rt_decref_mixed"), (40, "__rt_callable_descriptor_release")] {
        abi::load_at_offset(emitter, record, record_slot);
        abi::emit_load_from_address(emitter, result, record, offset);
        abi::emit_store_zero_to_address(emitter, record, offset);
        exceptions::emit_guarded_cleanup_call(emitter, entry, result, pending);
    }
    abi::load_at_offset(emitter, record, record_slot);
    abi::emit_load_from_address(emitter, result, record, 48);
    abi::emit_store_zero_to_address(emitter, record, 48);
    abi::store_at_offset(emitter, result, context_slot);
    abi::emit_load_from_address(emitter, result, record, 56);
    abi::emit_store_zero_to_address(emitter, record, 56);
    abi::emit_branch_if_int_result_zero(emitter, done);
    abi::emit_reg_move(emitter, abi::int_arg_reg_name(emitter.target, 0), result);
    abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 1), context_slot);
    abi::emit_frame_slot_address(emitter, abi::int_arg_reg_name(emitter.target, 2), pending);
    abi::emit_call_label(emitter, "__rt_cleanup_invoke");
    emitter.label(done);
}

/// Emits the call-duration lease release without restoring the exception-handler stack early.
pub(super) fn emit_snapshot_release(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_exception_handler_release_snapshot");
    abi::emit_frame_prologue(emitter, 64);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 0), 8);
    abi::emit_store_zero_to_local_slot(emitter, 16);
    release_registration(emitter, 8, 16, 24, "__rt_exception_snapshot_context_done");
    abi::load_at_offset(emitter, result, 16);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_exception_snapshot_return");
    let older = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, 16);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, 64);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_exception_snapshot_return");
    abi::emit_frame_restore(emitter, 64);
    abi::emit_return(emitter);
}
