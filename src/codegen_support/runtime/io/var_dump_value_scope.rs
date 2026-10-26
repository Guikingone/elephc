//! Purpose:
//! Roots runtime debug values and restores formatting state across arbitrary PHP callbacks.
//!
//! Called from:
//! - Runtime value walkers and top-level var_dump lowering.
//!
//! Key details:
//! - Each recursive value owns its input lease and optional magic-debug view until rendering ends.
//! - Seen and indent state is restored before and after guarded child destruction.
//! - The implementation receives this context as an internal fourth argument.

use crate::codegen_support::{abi, callable_descriptor, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;

pub(super) const TAG: usize = 0;
pub(super) const LOW: usize = 8;
pub(super) const HIGH: usize = 16;
pub(super) const INPUT_OWNED: usize = 24;
pub(super) const DEBUG_OWNER: usize = 32;
pub(super) const DEBUG_CONTAINER: usize = 40;
pub(super) const SAVED_INDENT: usize = 48;
pub(super) const SAVED_SEEN: usize = 56;
pub(super) const PENDING: usize = 64;
pub(super) const TOP_LEVEL: usize = 72;
pub(super) const VIEW_STAGE: usize = 80;
pub(super) const ACTIVE_INDENT: usize = 88;
const CONTEXT: usize = 96;
const FRAME: usize = CONTEXT + 16;

/// Loads one field from the context pointer saved in the current helper's local slot.
pub(super) fn read_context(
    emitter: &mut Emitter, destination: &str, context_slot: usize, field: usize,
) {
    let base = abi::symbol_scratch_reg(emitter);
    abi::load_at_offset(emitter, base, context_slot);
    abi::emit_load_from_address(emitter, destination, base, field);
}

/// Stores one field without consuming any of the context's owned values.
pub(super) fn write_context(
    emitter: &mut Emitter, source: &str, context_slot: usize, field: usize,
) {
    let base = abi::symbol_scratch_reg(emitter);
    assert_ne!(source, base);
    abi::load_at_offset(emitter, base, context_slot);
    abi::emit_store_to_address(emitter, source, base, field);
}

/// Writes a scalar state flag into the active rendering context.
pub(super) fn context_literal(
    emitter: &mut Emitter, context_slot: usize, field: usize, value: i64,
) {
    let word = abi::secondary_scratch_reg(emitter);
    abi::emit_load_int_immediate(emitter, word, value);
    write_context(emitter, word, context_slot, field);
}

/// Emits borrowed-value and top-level entries sharing one guarded lifetime boundary.
pub fn emit_var_dump_value_scope(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let mode = abi::int_arg_reg_name(emitter.target, 3);
    emitter.blank();
    emitter.comment("--- runtime: scoped var_dump value ---");
    emitter.label_global("__rt_var_dump_value");
    abi::emit_load_int_immediate(emitter, mode, 0);
    abi::emit_jump(emitter, "__rt_var_dump_value_scope_enter");
    emitter.label_global("__rt_var_dump_value_top_level");
    abi::emit_load_int_immediate(emitter, mode, 1);
    abi::emit_jump(emitter, "__rt_var_dump_value_scope_enter");
    emitter.label_global("__rt_var_dump_value_scope_enter");
    abi::emit_frame_prologue(emitter, FRAME);
    for (index, field) in [(0, TAG), (1, LOW), (2, HIGH), (3, TOP_LEVEL)] {
        abi::store_at_offset(
            emitter, abi::int_arg_reg_name(emitter.target, index), CONTEXT - field,
        );
    }
    for field in [INPUT_OWNED, DEBUG_OWNER, DEBUG_CONTAINER, PENDING, VIEW_STAGE] {
        abi::emit_store_zero_to_local_slot(emitter, CONTEXT - field);
    }
    for (symbol, field) in [("_vd_indent", SAVED_INDENT), ("_vd_seen_n", SAVED_SEEN)] {
        abi::emit_load_symbol_to_reg(emitter, result, symbol, 0);
        abi::store_at_offset(emitter, result, CONTEXT - field);
    }
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_var_dump_value_scope_body", result, CONTEXT - PENDING,
    );
    restore_globals(emitter);
    for field in [DEBUG_CONTAINER, DEBUG_OWNER] {
        abi::load_at_offset(emitter, result, CONTEXT - field);
        exceptions::emit_guarded_cleanup_call(
            emitter, "__rt_decref_any", result, CONTEXT - PENDING,
        );
    }
    abi::load_at_offset(emitter, result, CONTEXT - INPUT_OWNED);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_value_scope_cleaned");
    abi::load_at_offset(emitter, result, CONTEXT - TAG);
    branch_tag(emitter, 10, "__rt_var_dump_value_scope_release_callable");
    abi::load_at_offset(emitter, result, CONTEXT - LOW);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_decref_any", result, CONTEXT - PENDING,
    );
    abi::emit_jump(emitter, "__rt_var_dump_value_scope_cleaned");
    emitter.label("__rt_var_dump_value_scope_release_callable");
    abi::load_at_offset(emitter, result, CONTEXT - LOW);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_callable_descriptor_release", result, CONTEXT - PENDING,
    );
    emitter.label("__rt_var_dump_value_scope_cleaned");
    restore_globals(emitter);
    abi::load_at_offset(emitter, result, CONTEXT - PENDING);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_value_scope_return");
    let older = match emitter.target.arch { Arch::AArch64 => "x1", Arch::X86_64 => "rdi" };
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, CONTEXT - PENDING);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_var_dump_value_scope_return");
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_return(emitter);
    emit_scope_body(emitter);
}

/// Restores the enclosing renderer after normal output, unwinding or destructor reentry.
fn restore_globals(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    for (symbol, field) in [("_vd_indent", SAVED_INDENT), ("_vd_seen_n", SAVED_SEEN)] {
        abi::load_at_offset(emitter, result, CONTEXT - field);
        abi::emit_store_reg_to_symbol(emitter, result, symbol, 0);
    }
}

/// Acquires a type-correct input lease, then delegates rendering through the saved triple.
fn emit_scope_body(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_var_dump_value_scope_body");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    read_context(emitter, result, 8, TAG);
    for tag in [1, 4, 5, 6, 7, 11] {
        branch_tag(emitter, tag, "__rt_var_dump_value_scope_acquire_any");
    }
    branch_tag(emitter, 10, "__rt_var_dump_value_scope_acquire_callable");
    abi::emit_jump(emitter, "__rt_var_dump_value_scope_invoke");
    emitter.label("__rt_var_dump_value_scope_acquire_any");
    read_context(emitter, result, 8, LOW);
    abi::emit_call_label(emitter, "__rt_incref");
    abi::emit_jump(emitter, "__rt_var_dump_value_scope_acquired");
    emitter.label("__rt_var_dump_value_scope_acquire_callable");
    read_context(emitter, result, 8, LOW);
    callable_descriptor::emit_retain_current_descriptor(emitter);
    emitter.label("__rt_var_dump_value_scope_acquired");
    context_literal(emitter, 8, INPUT_OWNED, 1);
    emitter.label("__rt_var_dump_value_scope_invoke");
    read_context(emitter, result, 8, TOP_LEVEL);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_var_dump_value_scope_nested");
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_store_reg_to_symbol(emitter, result, "_vd_indent", 0);
    emitter.label("__rt_var_dump_value_scope_nested");
    abi::emit_load_symbol_to_reg(emitter, result, "_vd_indent", 0);
    write_context(emitter, result, 8, ACTIVE_INDENT);
    for (index, field) in [(0, TAG), (1, LOW), (2, HIGH)] {
        read_context(emitter, abi::int_arg_reg_name(emitter.target, index), 8, field);
    }
    abi::load_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 3), 8);
    abi::emit_call_label(emitter, "__rt_var_dump_value_impl");
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
}

/// Compares a saved runtime tag without branching across target assembly atoms.
pub(super) fn branch_tag(emitter: &mut Emitter, tag: u8, label: &str) {
    let result = abi::int_result_reg(emitter);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {result}, #{tag}"));              // compare the boxed runtime tag
            emitter.instruction(&format!("b.eq {label}"));                      // select the matching lifetime or rendering path
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {result}, {tag}"));               // compare the boxed runtime tag
            emitter.instruction(&format!("je {label}"));                        // select the matching lifetime or rendering path
        }
    }
}
