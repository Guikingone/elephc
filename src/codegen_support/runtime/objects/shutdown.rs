//! Purpose:
//! Retires normal-program handler roots after visiting remaining objects in PHP handle order.
//!
//! Called from:
//! - `crate::codegen::frame::emit_main_epilogue()` while the PHP main activation is still alive.
//! - `super::super::handler_state::emit_handler_state()` emits this entry with its dependencies.
//!
//! Key details:
//! - Registry roots remain available during the object-store destructor phase.
//! - Only the current receiver is retained; later entries are looked up again after callbacks.
//! - Destructor completion is shared with ordinary refcount and cycle-collector cleanup.
//! - Pending exceptions own their chains until all detached handler owners are retired.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};
use crate::codegen_support::runtime::exceptions;

const FRAME: usize = 96;
const HANDLE: usize = 8;
const RECEIVER: usize = 16;
const PENDING: usize = 24;
const FATAL: usize = 32;
const RETIRE: usize = 40;

/// Emits the owning shutdown boundary and its mutation-aware object and handler loops.
pub(crate) fn emit_handler_shutdown(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    emitter.blank();
    emitter.label_global("__rt_handlers_run_destructors");
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_jump(emitter, "__rt_handlers_shutdown_enter");
    emitter.label_global("__rt_handlers_shutdown");
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_jump(emitter, "__rt_handlers_shutdown_enter");
    emitter.label_global("__rt_handlers_shutdown_enter");
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, result, RETIRE);
    abi::emit_store_zero_to_local_slot(emitter, PENDING);
    abi::emit_store_zero_to_local_slot(emitter, FATAL);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_store_reg_to_symbol(emitter, result, "_obj_handle_shutdown", 0);
    emitter.label("__rt_handlers_shutdown_again");
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::store_at_offset(emitter, result, HANDLE);
    emitter.label("__rt_handlers_shutdown_object");
    abi::load_at_offset(emitter, result, HANDLE);
    abi::emit_cmp_reg_to_symbol(emitter, result, "_obj_handle_next");
    ins(emitter, "b.hs __rt_handlers_shutdown_registries", "jae __rt_handlers_shutdown_registries");
    abi::emit_call_label(emitter, "__rt_object_from_handle");
    abi::emit_branch_if_int_result_zero(emitter, "__rt_handlers_shutdown_next");
    abi::store_at_offset(emitter, result, RECEIVER);
    ins(emitter, "ldr x10, [x0, #-8]", "mov r10, QWORD PTR [rax - 8]");
    ins(emitter, "and x10, x10, #0xff", "and r10, 255");
    ins(emitter, "cmp x10, #4", "cmp r10, 4");
    ins(emitter, "b.ne __rt_handlers_shutdown_next", "jne __rt_handlers_shutdown_next");
    abi::load_at_offset(emitter, result, RECEIVER);
    ins(emitter, "ldr x10, [x0, #-8]", "mov r10, QWORD PTR [rax - 8]");
    ins(emitter, "tbnz x10, #14, __rt_handlers_shutdown_next", "test r10, 0x4000");
    if emitter.target.arch == Arch::X86_64 {
        emitter.instruction("jnz __rt_handlers_shutdown_next");                 // completed objects must not acquire another shutdown lease
    }
    abi::load_at_offset(emitter, result, FATAL);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_handlers_shutdown_call");
    abi::load_at_offset(emitter, result, RECEIVER);
    ins(emitter, "ldr x10, [x0, #-8]", "mov r10, QWORD PTR [rax - 8]");
    ins(emitter, "orr x10, x10, #0x4000", "or r10, 0x4000");
    ins(emitter, "str x10, [x0, #-8]", "mov QWORD PTR [rax - 8], r10");
    abi::emit_jump(emitter, "__rt_handlers_shutdown_next");
    emitter.label("__rt_handlers_shutdown_call");
    abi::load_at_offset(emitter, result, RECEIVER);
    abi::emit_call_label(emitter, "__rt_incref");
    abi::load_at_offset(emitter, result, RECEIVER);
    exceptions::emit_guarded_cleanup_call(
        emitter, "__rt_call_object_destructor", result, PENDING,
    );
    super::shutdown_exception::dispatch_pending(
        emitter, PENDING, FATAL, "__rt_handlers_shutdown_destructor_exception",
    );
    abi::load_at_offset(emitter, result, RECEIVER);
    ins(emitter, "ldr w10, [x0, #-12]", "mov r10d, DWORD PTR [rax - 12]");
    ins(emitter, "and w10, w10, #0x7fffffff", "and r10d, 0x7fffffff");
    ins(emitter, "str w10, [x0, #-12]", "mov DWORD PTR [rax - 12], r10d");
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_decref_any", result, PENDING);
    super::shutdown_exception::dispatch_pending(
        emitter, PENDING, FATAL, "__rt_handlers_shutdown_receiver_exception",
    );
    emitter.label("__rt_handlers_shutdown_next");
    abi::load_at_offset(emitter, result, HANDLE);
    ins(emitter, "add x0, x0, #1", "add rax, 1");
    abi::store_at_offset(emitter, result, HANDLE);
    abi::emit_jump(emitter, "__rt_handlers_shutdown_object");
    emitter.label("__rt_handlers_shutdown_registries");
    abi::load_at_offset(emitter, result, RETIRE);
    abi::emit_branch_if_int_result_nonzero(emitter, "__rt_handlers_shutdown_retire");
    abi::load_at_offset(emitter, result, FATAL);
    abi::emit_branch_if_int_result_nonzero(emitter, "__rt_handlers_shutdown_retire");
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_load_int_immediate(emitter, result, 0);
    abi::emit_return(emitter);
    emitter.label("__rt_handlers_shutdown_retire");
    emit_registry_cleanup(emitter);
    abi::emit_load_int_immediate(emitter, result, 0);
    exceptions::emit_guarded_cleanup_call(emitter, "__rt_gc_collect_cycles", result, PENDING);
    super::shutdown_exception::dispatch_pending(
        emitter, PENDING, FATAL, "__rt_handlers_shutdown_cycle_exception",
    );
    emit_finish(emitter);
    super::shutdown_exception::emit_callback(emitter);
}

/// Drains current and previous registrations, revisiting objects created by cleanup callbacks.
fn emit_registry_cleanup(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    for kind in ["error", "exception"] {
        let next = format!("__rt_handlers_shutdown_{kind}");
        emitter.label(&next);
        abi::emit_load_int_immediate(emitter, result, 0);
        exceptions::emit_guarded_cleanup_call(
            emitter, &format!("__rt_core_{kind}_handler_pop"), result, PENDING,
        );
        super::shutdown_exception::dispatch_pending(
            emitter, PENDING, FATAL, &format!("__rt_handlers_shutdown_{kind}_exception"),
        );
        abi::emit_load_symbol_to_reg(emitter, result, &format!("_php_{kind}_handler_stack"), 0);
        abi::emit_branch_if_int_result_nonzero(emitter, &next);
    }
    for kind in ["error", "exception"] {
        for suffix in ["stack", "value", "callable", "context", "context_release"] {
            abi::emit_load_symbol_to_reg(
                emitter, result, &format!("_php_{kind}_handler_{suffix}"), 0,
            );
            abi::emit_branch_if_int_result_nonzero(emitter, "__rt_handlers_shutdown_again");
        }
    }
}

/// Transfers any pending cleanup Throwable only after all sibling owners have been retired.
fn emit_finish(emitter: &mut Emitter) {
    let result = abi::int_result_reg(emitter);
    let older = match emitter.target.arch { Arch::AArch64 => "x1", Arch::X86_64 => "rdi" };
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_branch_if_int_result_zero(emitter, "__rt_handlers_shutdown_return");
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label("__rt_handlers_shutdown_return");
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

/// Emits one target-specific instruction in the shared ownership protocol.
fn ins(emitter: &mut Emitter, arm: &str, x86: &str) {
    let instruction = if emitter.target.arch == Arch::AArch64 { arm } else { x86 };
    emitter.instruction(instruction);                                           // preserve the shared shutdown ordering on each instruction set
}
