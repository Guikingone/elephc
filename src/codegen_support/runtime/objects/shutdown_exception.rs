//! Purpose:
//! Dispatches each shutdown destructor exception while registered PHP handlers are still alive.
//!
//! Called from:
//! - `super::shutdown` between individual destructor calls and handler-root retirement.
//!
//! Key details:
//! - The parent owns the raw Throwable, boxed arguments, callback result and partial raw array.
//! - Every owner is detached before fallible cleanup, including when the callback throws.
//! - A failed exception handler selects fatal shutdown instead of recursively invoking itself.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;

pub(super) const CONTEXT: usize = 80;
const THROWABLE: usize = 0;
const ARGUMENTS: usize = 8;
const RESULT: usize = 16;
const RAW_ARRAY: usize = 24;

/// Emits a dispatch at one parent-frame site, preserving its pending and fatal-state contracts.
pub(super) fn dispatch_pending(
    emitter: &mut Emitter, pending: usize, fatal: usize, prefix: &str,
) {
    let result = abi::int_result_reg(emitter);
    let done = format!("{prefix}_done");
    let failed = format!("{prefix}_fatal");
    abi::load_at_offset(emitter, result, pending);
    abi::emit_branch_if_int_result_zero(emitter, &done);
    abi::load_at_offset(emitter, result, fatal);
    abi::emit_branch_if_int_result_nonzero(emitter, &done);
    abi::emit_load_symbol_to_reg(emitter, result, "_php_exception_handler_callable", 0);
    abi::emit_branch_if_int_result_zero(emitter, &failed);
    abi::load_at_offset(emitter, result, pending);
    abi::emit_store_zero_to_local_slot(emitter, pending);
    abi::store_at_offset(emitter, result, CONTEXT - THROWABLE);
    for field in [ARGUMENTS, RESULT, RAW_ARRAY] {
        abi::emit_store_zero_to_local_slot(emitter, CONTEXT - field);
    }
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_shutdown_exception_callback", result, pending,
    );
    mark_pending_fatal(emitter, pending, fatal, &format!("{prefix}_callback_failed"));
    for (field, entry) in [
        (RESULT, "__rt_decref_mixed"), (ARGUMENTS, "__rt_decref_mixed"),
        (RAW_ARRAY, "__rt_decref_any"), (THROWABLE, "__rt_decref_any"),
    ] {
        abi::load_at_offset(emitter, result, CONTEXT - field);
        abi::emit_store_zero_to_local_slot(emitter, CONTEXT - field);
        exceptions::emit_guarded_cleanup_call(emitter, entry, result, pending);
        mark_pending_fatal(emitter, pending, fatal, &format!("{prefix}_cleanup_failed_{field}"));
    }
    abi::load_at_offset(emitter, result, pending);
    abi::emit_branch_if_int_result_zero(emitter, &done);
    emitter.label(&failed);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::store_at_offset(emitter, result, fatal);
    abi::emit_call_label(emitter, "__rt_shutdown_mark_destructed");
    emitter.label(&done);
}

/// Emits the guarded callback body; its parent owns all storage even if a native throw escapes.
pub(super) fn emit_callback(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let low = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    let high = if emitter.target.arch == Arch::AArch64 { "x2" } else { "rsi" };
    let word = abi::secondary_scratch_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_shutdown_exception_callback");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    for (index, value) in [(0, 1), (1, 8)] {
        abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, index), value);
    }
    abi::emit_call_label(emitter, "__rt_array_new");
    crate::codegen_support::emit_array_value_type_stamp(
        emitter, result, &crate::types::PhpType::Mixed,
    );
    write_field(emitter, result, RAW_ARRAY);
    abi::emit_load_int_immediate(emitter, result, 6);
    read_field(emitter, low, THROWABLE);
    abi::emit_load_int_immediate(emitter, high, 0);
    abi::emit_call_label(emitter, "__rt_mixed_from_value");
    read_field(emitter, word, RAW_ARRAY);
    abi::emit_store_to_address(emitter, result, word, 24);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_store_to_address(emitter, result, word, 0);
    abi::emit_load_int_immediate(emitter, result, 4);
    read_field(emitter, low, RAW_ARRAY);
    abi::emit_load_int_immediate(emitter, high, 0);
    abi::emit_call_label(emitter, "__rt_mixed_from_value");
    write_field(emitter, result, ARGUMENTS);
    read_field(emitter, result, RAW_ARRAY);
    clear_field(emitter, RAW_ARRAY);
    abi::emit_call_label(emitter, "__rt_decref_any");
    abi::emit_load_symbol_to_reg(emitter, result, "_php_exception_handler_callable", 0);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_shutdown_exception_callback_missing");
    read_field(emitter, abi::int_arg_reg_name(emitter.target, 1), ARGUMENTS);
    abi::emit_load_symbol_to_reg(
        emitter, abi::int_arg_reg_name(emitter.target, 0), "_php_exception_handler_callable", 0,
    );
    abi::emit_call_label(emitter, "__rt_exception_handler_invoke");
    write_field(emitter, result, RESULT);
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
    emitter.label("__rt_shutdown_exception_callback_missing");
    read_field(emitter, result, THROWABLE);
    clear_field(emitter, THROWABLE);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, 32);
    abi::emit_jump(emitter, "__rt_throw_current");
    emit_mark_destructed(emitter);
}

/// Reads one parent-owned field using the callback's borrowed context pointer.
fn read_field(emitter: &mut Emitter, destination: &str, field: usize) {
    let context = abi::symbol_scratch_reg(emitter);
    abi::load_at_offset(emitter, context, 8);
    abi::emit_load_from_address(emitter, destination, context, field);
}

/// Publishes a newly acquired owner before any subsequent runtime call can throw.
fn write_field(emitter: &mut Emitter, source: &str, field: usize) {
    let context = abi::symbol_scratch_reg(emitter);
    abi::load_at_offset(emitter, context, 8);
    abi::emit_store_to_address(emitter, source, context, field);
}

/// Detaches a parent-owned field before transferring or releasing its payload.
fn clear_field(emitter: &mut Emitter, field: usize) {
    let context = abi::symbol_scratch_reg(emitter);
    abi::load_at_offset(emitter, context, 8);
    abi::emit_store_zero_to_address(emitter, context, field);
}

/// Suppresses later callbacks as soon as a handler or its ignored return value fails.
fn mark_pending_fatal(emitter: &mut Emitter, pending: usize, fatal: usize, done: &str) {
    let result = abi::int_result_reg(emitter);
    abi::load_at_offset(emitter, result, pending);
    abi::emit_branch_if_int_result_zero(emitter, done);
    abi::load_at_offset(emitter, result, fatal);
    abi::emit_branch_if_int_result_nonzero(emitter, done);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::store_at_offset(emitter, result, fatal);
    abi::emit_call_label(emitter, "__rt_shutdown_mark_destructed");
    emitter.label(done);
}

/// Marks live object buckets after fatal shutdown without running another PHP destructor.
fn emit_mark_destructed(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let word = abi::secondary_scratch_reg(emitter);
    let tag = abi::symbol_scratch_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_shutdown_mark_destructed");
    abi::emit_frame_prologue(emitter, 32);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::store_at_offset(emitter, result, 8);
    emitter.label("__rt_shutdown_mark_loop");
    abi::load_at_offset(emitter, result, 8);
    abi::emit_cmp_reg_to_symbol(emitter, result, "_obj_handle_next");
    ins(emitter, "b.hs __rt_shutdown_mark_done", "jae __rt_shutdown_mark_done");
    abi::emit_call_label(emitter, "__rt_object_from_handle");
    abi::emit_branch_if_int_result_zero(emitter, "__rt_shutdown_mark_next");
    ins(emitter, "ldr x10, [x0, #-8]", "mov r10, QWORD PTR [rax - 8]");
    abi::emit_reg_move(emitter, tag, word);
    ins(emitter, "and x9, x9, #0xff", "and r11, 255");
    ins(emitter, "cmp x9, #4", "cmp r11, 4");
    ins(emitter, "b.ne __rt_shutdown_mark_next", "jne __rt_shutdown_mark_next");
    ins(emitter, "orr x10, x10, #0x4000", "or r10, 0x4000");
    ins(emitter, "str x10, [x0, #-8]", "mov QWORD PTR [rax - 8], r10");
    emitter.label("__rt_shutdown_mark_next");
    abi::load_at_offset(emitter, result, 8);
    ins(emitter, "add x0, x0, #1", "add rax, 1");
    abi::store_at_offset(emitter, result, 8);
    abi::emit_jump(emitter, "__rt_shutdown_mark_loop");
    emitter.label("__rt_shutdown_mark_done");
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
}

/// Selects equivalent instructions while keeping shutdown marking allocation-free.
fn ins(emitter: &mut Emitter, arm: &str, x86: &str) {
    let instruction = if emitter.target.arch == Arch::AArch64 { arm } else { x86 };
    emitter.instruction(instruction);                                           // update only the target's object-store completion state
}
