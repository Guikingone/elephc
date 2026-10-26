//! Purpose:
//! Isolates print_r capture state and owns the complete rendered result.
//! Keeps the source argument alive through output and restores outer captures on unwind.
//!
//! Called from:
//! - The EIR PrintR backend for direct calls and callable adapters.
//!
//! Key details:
//! - Input is one owned Mixed cell, return flag, result representation and native address format.
//! - Both echo and return modes build a full string before exposing output.
//! - Receiver/view callbacks use a bounded pending-exception slot in this scope.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;
use super::var_dump_value_scope::read_context;
use super::print_r_php_call::emit_guarded_php_call;

const INPUT: usize = 0;
const RETURN: usize = 8;
const KIND: usize = 16;
const OUTPUT: usize = 24;
const LENGTH: usize = 32;
const SAVED_MODE: usize = 40;
const SAVED_BUFFER: usize = 48;
const SAVED_CAPACITY: usize = 56;
const SAVED_LENGTH: usize = 64;
const SAVED_SEEN: usize = 72;
const SAVED_INDENT: usize = 80;
const PREVIOUS: usize = 88;
const PENDING: usize = 96;
const FORMAT: usize = 104;
const CONTEXT: usize = 112;
const FRAME: usize = CONTEXT + 16;

/// Emits one guarded capture boundary with scalar, string and Mixed return representations.
pub fn emit_print_r_scope(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_print_r_scoped");
    abi::emit_frame_prologue(emitter, FRAME);
    for (index, field) in [(0, INPUT), (1, RETURN), (2, KIND), (3, FORMAT)] {
        abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, index), CONTEXT - field);
    }
    for field in [OUTPUT, LENGTH, PENDING] {
        abi::emit_store_zero_to_local_slot(emitter, CONTEXT - field);
    }
    for (symbol, field) in saved_globals() {
        abi::emit_load_symbol_to_reg(emitter, result, symbol, 0);
        abi::store_at_offset(emitter, result, CONTEXT - field);
    }
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    abi::emit_store_reg_to_symbol(emitter, result, "_print_r_scope", 0);
    for symbol in ["_print_r_buf", "_print_r_cap", "_print_r_off"] {
        abi::emit_store_zero_to_symbol(emitter, symbol, 0);
    }
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_store_reg_to_symbol(emitter, result, "_print_r_mode", 0);
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_print_r_scope_body", result, CONTEXT - PENDING);
    abi::emit_call_label(emitter, "__rt_pr_finish");
    let (pointer, length) = abi::string_result_regs(emitter);
    abi::store_at_offset(emitter, pointer, CONTEXT - OUTPUT);
    abi::store_at_offset(emitter, length, CONTEXT - LENGTH);
    restore_walk_state(emitter);
    abi::load_at_offset(emitter, result, CONTEXT - RETURN);
    abi::emit_branch_if_int_result_nonzero(emitter, "__rt_print_r_scope_after_flush");
    abi::emit_frame_slot_address(emitter, result, CONTEXT);
    emit_guarded_php_call(emitter, "__rt_print_r_scope_flush", result, CONTEXT - PENDING);
    emitter.label("__rt_print_r_scope_after_flush");
    abi::load_at_offset(emitter, result, CONTEXT - INPUT);
    abi::emit_store_zero_to_local_slot(emitter, CONTEXT - INPUT);
    emit_guarded_php_call(emitter, "__rt_decref_any", result, CONTEXT - PENDING);
    for (symbol, field) in saved_globals() {
        abi::load_at_offset(emitter, result, CONTEXT - field);
        abi::emit_store_reg_to_symbol(emitter, result, symbol, 0);
    }
    abi::load_at_offset(emitter, result, CONTEXT - PENDING);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_print_r_scope_result");
    release_output(emitter);
    abi::load_at_offset(emitter, result, CONTEXT - PENDING);
    let older = match emitter.target.arch { Arch::AArch64 => "x1", Arch::X86_64 => "rdi" };
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, CONTEXT - PENDING);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_print_r_scope_result");
    abi::load_at_offset(emitter, result, CONTEXT - KIND);
    branch_equal(emitter, 1, "__rt_print_r_scope_string");
    branch_equal(emitter, 2, "__rt_print_r_scope_mixed");
    release_output(emitter);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emitter.label("__rt_print_r_scope_string");
    abi::load_at_offset(emitter, pointer, CONTEXT - OUTPUT);
    abi::load_at_offset(emitter, length, CONTEXT - LENGTH);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emitter.label("__rt_print_r_scope_mixed");
    abi::load_at_offset(emitter, result, CONTEXT - RETURN);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_print_r_scope_box_true");
    abi::load_at_offset(emitter, pointer, CONTEXT - OUTPUT);
    abi::load_at_offset(emitter, length, CONTEXT - LENGTH);
    crate::codegen_support::value_boxing::emit_box_current_value_as_mixed(emitter, &crate::types::PhpType::Str);
    abi::emit_jump(emitter, "__rt_print_r_scope_boxed");
    emitter.label("__rt_print_r_scope_box_true");
    abi::emit_load_int_immediate(emitter, result, 1);
    crate::codegen_support::value_boxing::emit_box_current_value_as_mixed(emitter, &crate::types::PhpType::Bool);
    emitter.label("__rt_print_r_scope_boxed");
    abi::store_at_offset(emitter, result, CONTEXT - INPUT);
    release_output(emitter);
    abi::load_at_offset(emitter, result, CONTEXT - INPUT);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
    emit_body(emitter);
    emit_flush(emitter);
    emit_record_exception(emitter);
}

/// Names the enclosing capture and walk state owned by this activation.
fn saved_globals() -> [(&'static str, usize); 7] {
    [("_print_r_mode", SAVED_MODE), ("_print_r_buf", SAVED_BUFFER),
     ("_print_r_cap", SAVED_CAPACITY), ("_print_r_off", SAVED_LENGTH),
     ("_vd_seen_n", SAVED_SEEN), ("_vd_indent", SAVED_INDENT), ("_print_r_scope", PREVIOUS)]
}

/// Restores formatting before output handlers or argument destructors can reenter PHP.
fn restore_walk_state(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    for (symbol, field) in [("_vd_seen_n", SAVED_SEEN), ("_vd_indent", SAVED_INDENT)] {
        abi::load_at_offset(emitter, result, CONTEXT - field);
        abi::emit_store_reg_to_symbol(emitter, result, symbol, 0);
    }
}

/// Retires the captured string after its final use, without touching any source owner.
fn release_output(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    abi::load_at_offset(emitter, result, CONTEXT - OUTPUT);
    abi::emit_store_zero_to_local_slot(emitter, CONTEXT - OUTPUT);
    abi::emit_call_label(emitter, "__rt_decref_any");
}

/// Renders an argument while its original boxed owner remains in the outer scope.
fn emit_body(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_print_r_scope_body");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    read_context(emitter, result, 8, FORMAT);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_print_r_scope_body_value");
    read_context(emitter, result, 8, INPUT);
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    let low = match emitter.target.arch { Arch::AArch64 => "x1", Arch::X86_64 => "rdi" };
    abi::emit_reg_move(emitter, result, low);
    abi::emit_call_label(emitter, "__rt_ptoa");
    abi::emit_reg_move(emitter, abi::int_arg_reg_name(emitter.target, 1), abi::string_result_regs(emitter).0);
    abi::emit_call_label(emitter, "__rt_pr_write");
    abi::emit_jump(emitter, "__rt_print_r_scope_body_done");
    emitter.label("__rt_print_r_scope_body_value");
    read_context(emitter, abi::int_arg_reg_name(emitter.target, 1), 8, INPUT);
    abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 0), 7);
    for index in [2, 3] {
        abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, index), 0);
    }
    abi::emit_call_label(emitter, "__rt_print_r_value");
    emitter.label("__rt_print_r_scope_body_done");
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
}

/// Flushes the complete string through output buffering while capture mode is suspended.
fn emit_flush(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_print_r_scope_flush");
    abi::emit_frame_prologue(emitter, 32);
    abi::store_at_offset(emitter, result, 8);
    read_context(emitter, abi::int_arg_reg_name(emitter.target, 0), 8, OUTPUT);
    read_context(emitter, abi::int_arg_reg_name(emitter.target, 1), 8, LENGTH);
    abi::emit_call_label(emitter, "__rt_stdout_write");
    abi::emit_frame_restore(emitter, 32);
    abi::emit_return(emitter);
}

/// Selects the requested physical result without reading capture globals after callbacks.
fn branch_equal(emitter: &mut Emitter, value: i64, label: &str) {
    let result = abi::int_result_reg(emitter);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("cmp {result}, #{value}"));            // compare the requested return representation
            emitter.instruction(&format!("b.eq {label}"));                      // dispatch to the corresponding result ABI
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("cmp {result}, {value}"));             // compare the requested return representation
            emitter.instruction(&format!("je {label}"));                        // dispatch to the corresponding result ABI
        }
    }
}

/// Transfers a completed child cleanup exception to the enclosing buffered render.
fn emit_record_exception(e: &mut Emitter) {
    let r=abi::int_result_reg(e);let base=abi::secondary_scratch_reg(e);
    e.label_global("__rt_print_r_record_exception");
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_record_exception_empty");
    abi::emit_frame_prologue(e,32);abi::store_at_offset(e,r,8);
    abi::emit_load_symbol_to_reg(e,base,"_print_r_scope",0);
    abi::store_at_offset(e,base,16);
    abi::emit_reg_move(e,r,base);
    abi::emit_branch_if_int_result_zero(e,"__rt_print_r_record_exception_unscoped");
    abi::load_at_offset(e,r,8);
    let older=if e.target.arch==Arch::AArch64 {"x1"}else{"rdi"};
    abi::emit_load_from_address(e,older,base,PENDING);
    abi::emit_store_to_address(e,r,base,PENDING);
    abi::emit_call_label(e,"__rt_exception_chain");
    abi::emit_frame_restore(e,32);abi::emit_return(e);
    e.label("__rt_print_r_record_exception_unscoped");
    abi::load_at_offset(e,r,8);abi::emit_store_reg_to_symbol(e,r,"_exc_value",0);
    abi::emit_frame_restore(e,32);abi::emit_jump(e,"__rt_throw_current");
    e.label("__rt_print_r_record_exception_empty");abi::emit_return(e);
}
