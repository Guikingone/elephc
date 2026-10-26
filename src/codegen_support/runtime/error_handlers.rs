//! Purpose:
//! Invokes PHP error and exception handlers with exception-safe suspension of their registrations.
//!
//! Called from:
//! - AOT trigger lowering, the eval error-handler bridge, and shutdown exception dispatch.
//!
//! Key details:
//! - A native cleanup activation restores suspended ownership during unwinding.
//! - Callback-installed replacements survive; otherwise old owners are transferred back once.
//! - The PHP return value is destroyed before restoration; only its exact-false classification escapes.
//! - Discarding owners finishes sibling cleanup before propagating any destructor throw.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch, RuntimeFeatures};
use crate::codegen_support::runtime::exceptions;

mod owned_args;
mod exception_stack;

/// Lists the four movable owners shared by error and exception registrations.
fn saved_fields(kind: &str) -> [(usize, String); 4] {
    [(32, "value"), (40, "callable"), (48, "context"), (56, "context_release")]
        .map(|(offset, suffix)| (offset, format!("_php_{kind}_handler_{suffix}")))
}

/// Emits both C-ABI invocation wrappers with their matching unwind cleanup callbacks.
pub(super) fn emit_error_handler_invoke(emitter: &mut Emitter, features: RuntimeFeatures) {
    emit_invoke(emitter, "error", false);
    emit_restore(emitter, "error");
    if features.handler_state || features.eval_bridge || features.web {
        emit_invoke(emitter, "exception", features.eval_bridge);
        emit_restore(emitter, "exception");
        exception_stack::emit_snapshot_release(emitter);
    }
    owned_args::emit_owned_arguments(emitter);
}

/// Moves the active registration into a published callback activation until invocation finishes.
fn emit_invoke(emitter: &mut Emitter, kind: &str, eval_bridge: bool) {
    let arm = emitter.target.arch == Arch::AArch64;
    let stack = if arm { "sp" } else { "rsp" };
    let scratch = abi::secondary_scratch_reg(emitter);
    let result = abi::int_result_reg(emitter);
    let arg0 = abi::int_arg_reg_name(emitter.target, 0);
    let arg1 = abi::int_arg_reg_name(emitter.target, 1);
    let restore = format!("__rt_{kind}_handler_restore");
    emitter.blank();
    emitter.label_global(&format!("__rt_{kind}_handler_invoke"));
    emit_frame_enter(emitter, 128);
    abi::emit_store_to_address(emitter, arg1, stack, 88);
    abi::emit_store_zero_to_address(emitter, stack, 64);
    abi::emit_store_zero_to_address(emitter, stack, 72);
    abi::emit_store_zero_to_address(emitter, stack, 80);
    for (offset, symbol) in saved_fields(kind) {
        abi::emit_load_symbol_to_reg(emitter, scratch, &symbol, 0);
        abi::emit_store_to_address(emitter, scratch, stack, offset);
        abi::emit_store_zero_to_symbol(emitter, &symbol, 0);
    }
    abi::emit_load_symbol_to_reg(emitter, scratch, "_exc_call_frame_top", 0);
    abi::emit_store_to_address(emitter, scratch, stack, 0);
    abi::emit_symbol_address(emitter, scratch, &restore);
    abi::emit_store_to_address(emitter, scratch, stack, 8);
    abi::emit_reg_move(emitter, scratch, stack);
    abi::emit_store_to_address(emitter, scratch, stack, 16);
    abi::emit_store_reg_to_symbol(emitter, scratch, "_exc_call_frame_top", 0);
    abi::emit_load_int_immediate(emitter, scratch, 0);
    abi::emit_store_to_address(emitter, scratch, stack, 24);
    if kind == "exception" {
        exception_stack::emit_suspend_copy(emitter, stack, eval_bridge);
    }
    abi::emit_load_from_address(emitter, arg0, stack, 40);
    abi::emit_load_from_address(emitter, arg1, stack, 88);
    abi::emit_load_int_immediate(emitter, abi::int_arg_reg_name(emitter.target, 2), -1);
    abi::emit_load_from_address(
        emitter, scratch, arg0,
        crate::codegen_support::callable_descriptor::CALLABLE_DESC_INVOKER_OFFSET,
    );
    abi::emit_call_reg(emitter, scratch);
    abi::emit_store_to_address(emitter, result, stack, 72);
    if kind == "exception" {
        abi::emit_reg_move(emitter, arg0, stack);
        abi::emit_call_label(emitter, "__rt_exception_handler_release_snapshot");
    }
    abi::emit_load_from_address(emitter, result, stack, 72);
    emit_normalize_result(emitter, kind, stack);
    abi::emit_load_int_immediate(emitter, scratch, if kind == "exception" { 3 } else { 1 });
    abi::emit_store_to_address(emitter, scratch, stack, 80);
    abi::emit_reg_move(emitter, arg0, stack);
    abi::emit_call_label(emitter, &restore);
    abi::emit_load_from_address(emitter, scratch, stack, 0);
    abi::emit_store_reg_to_symbol(emitter, scratch, "_exc_call_frame_top", 0);
    abi::emit_load_from_address(emitter, result, stack, 64);
    emit_frame_leave(emitter, 128);
}

/// Transfers suspended owners back or discards each owner once while preserving callback replacements.
fn emit_restore(emitter: &mut Emitter, kind: &str) {
    const FRAME: usize = 64;
    const RECORD: usize = 8;
    const PENDING: usize = 16;
    const CONTEXT: usize = 24;
    let result = abi::int_result_reg(emitter);
    let record = abi::secondary_scratch_reg(emitter);
    let discard = format!("__rt_{kind}_handler_discard");
    let context_done = format!("__rt_{kind}_handler_context_done");
    let done = format!("__rt_{kind}_handler_restore_done");
    let owners = format!("__rt_{kind}_handler_restore_owners");
    let finish = format!("__rt_{kind}_handler_restore_finish");
    emitter.blank();
    emitter.label_global(&format!("__rt_{kind}_handler_restore"));
    abi::emit_frame_prologue(emitter, FRAME);
    abi::store_at_offset(emitter, abi::int_arg_reg_name(emitter.target, 0), RECORD);
    abi::emit_store_zero_to_local_slot(emitter, PENDING);
    abi::load_at_offset(emitter, record, RECORD);
    abi::emit_load_from_address(emitter, result, record, 80);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("and x0, x0, #1");                              // only the normal-return bit transfers the normalized result to its caller
        }
        Arch::X86_64 => {
            emitter.instruction("and rax, 1");                                  // stack-publication state does not imply normal callback completion
        }
    }
    abi::emit_branch_if_int_result_nonzero(emitter, &owners);
    if kind == "exception" {
        abi::emit_call_label(emitter, "__rt_shutdown_mark_destructed");
    }
    release_result_owners(emitter, RECORD, PENDING);
    emitter.label(&owners);
    if kind == "exception" {
        let not_published = "__rt_exception_handler_stack_not_published";
        abi::load_at_offset(emitter, record, RECORD);
        abi::emit_load_from_address(emitter, result, record, 80);
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction("and x0, x0, #2");                          // distinguish an allocation failure before the stack copy became visible
            }
            Arch::X86_64 => {
                emitter.instruction("and rax, 2");                              // distinguish an allocation failure before the stack copy became visible
            }
        }
        abi::emit_branch_if_int_result_zero(emitter, not_published);
        abi::emit_load_symbol_to_reg(emitter, result, "_php_exception_handler_callable", 0);
        abi::emit_branch_if_int_result_nonzero(emitter, &discard);
        abi::emit_load_int_immediate(emitter, result, 0);
        exceptions::emit_guarded_cleanup_call(
            emitter, "__rt_core_exception_handler_pop", result, PENDING,
        );
        abi::emit_jump(emitter, &discard);
        emitter.label(not_published);
    }
    abi::emit_load_symbol_to_reg(emitter, result, &format!("_php_{kind}_handler_callable"), 0);
    abi::emit_branch_if_int_result_nonzero(emitter, &discard);
    for (offset, symbol) in saved_fields(kind) {
        abi::load_at_offset(emitter, record, RECORD);
        abi::emit_load_from_address(emitter, result, record, offset);
        abi::emit_store_zero_to_address(emitter, record, offset);
        abi::emit_store_reg_to_symbol(emitter, result, &symbol, 0);
    }
    abi::emit_jump(emitter, &finish);
    emitter.label(&discard);
    exception_stack::release_registration(emitter, RECORD, PENDING, CONTEXT, &context_done);
    emitter.label(&finish);
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_branch_if_int_result_zero(emitter, &done);
    // Normalized return storage has no caller when restoration itself throws.
    release_result_owners(emitter, RECORD, PENDING);
    abi::load_at_offset(emitter, result, PENDING);
    let older = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    abi::emit_load_symbol_to_reg(emitter, older, "_exc_value", 0);
    abi::emit_store_zero_to_symbol(emitter, "_exc_value", 0);
    abi::emit_call_label(emitter, "__rt_exception_chain");
    abi::load_at_offset(emitter, result, PENDING);
    abi::emit_store_reg_to_symbol(emitter, result, "_exc_value", 0);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_jump(emitter, "__rt_throw_current");
    emitter.label(&done);
    abi::emit_frame_restore(emitter, FRAME);
    abi::emit_return(emitter);
}

/// Destroys the PHP return value while the old handler remains suspended, preserving exact false.
fn emit_normalize_result(emitter: &mut Emitter, kind: &str, stack: &str) {
    let result = abi::int_result_reg(emitter);
    let low = if emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    let high = if emitter.target.arch == Arch::AArch64 { "x2" } else { "rsi" };
    let handled = format!("__rt_{kind}_handler_handled");
    let unhandled = format!("__rt_{kind}_handler_unhandled");
    let classified = format!("__rt_{kind}_handler_classified");
    abi::emit_call_label(emitter, "__rt_mixed_unbox");
    if emitter.target.arch == Arch::AArch64 {
        emitter.instruction("cmp x0, #3");                                      // only the exact PHP boolean false falls through to default reporting
        emitter.instruction(&format!("b.ne {handled}"));                        // every other PHP return value handles the diagnostic
        emitter.instruction(&format!("cbz x1, {unhandled}"));                   // preserve false independently of cleanup callbacks
    } else {
        emitter.instruction("cmp rax, 3");                                      // only the exact PHP boolean false falls through to default reporting
        emitter.instruction(&format!("jne {handled}"));                         // every other PHP return value handles the diagnostic
        emitter.instruction("test rdi, rdi");                                   // inspect the unboxed boolean payload
        emitter.instruction(&format!("jz {unhandled}"));                        // preserve false independently of cleanup callbacks
    }
    emitter.label(&handled);
    abi::emit_load_int_immediate(emitter, result, 1);
    abi::emit_jump(emitter, &classified);
    emitter.label(&unhandled);
    abi::emit_load_int_immediate(emitter, result, 0);
    emitter.label(&classified);
    abi::emit_store_to_address(emitter, result, stack, 88);
    abi::emit_load_from_address(emitter, result, stack, 72);
    abi::emit_store_zero_to_address(emitter, stack, 72);
    abi::emit_call_label(emitter, "__rt_decref_mixed");
    abi::emit_load_from_address(emitter, low, stack, 88);
    abi::emit_load_int_immediate(emitter, high, 0);
    abi::emit_load_int_immediate(emitter, result, 3);
    abi::emit_call_label(emitter, "__rt_mixed_from_value");
    abi::emit_store_to_address(emitter, result, stack, 64);
}

/// Detaches result owners before cleanup can reenter the suspended-handler activation.
fn release_result_owners(emitter: &mut Emitter, record_slot: usize, pending: usize) {
    let result = abi::int_result_reg(emitter);
    let record = abi::secondary_scratch_reg(emitter);
    for offset in [72, 64] {
        abi::load_at_offset(emitter, record, record_slot);
        abi::emit_load_from_address(emitter, result, record, offset);
        abi::emit_store_zero_to_address(emitter, record, offset);
        exceptions::emit_guarded_cleanup_call(emitter, "__rt_decref_mixed", result, pending);
    }
}

/// Creates aligned storage with standard frame linkage on every supported target.
fn emit_frame_enter(emitter: &mut Emitter, size: usize) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("sub sp, sp, #{size}"));               // reserve aligned helper storage
            emitter.instruction(&format!("stp x29, x30, [sp, #{}]", size - 16));// preserve caller frame linkage
            emitter.instruction(&format!("add x29, sp, #{}", size - 16));       // establish the helper frame pointer
        }
        Arch::X86_64 => {
            emitter.instruction("push rbp");                                    // preserve caller linkage and align nested calls
            emitter.instruction("mov rbp, rsp");                                // establish stable helper frame linkage
            emitter.instruction(&format!("sub rsp, {size}"));                   // reserve aligned helper storage
        }
    }
}

/// Releases helper storage and returns without changing the result register.
fn emit_frame_leave(emitter: &mut Emitter, size: usize) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("ldp x29, x30, [sp, #{}]", size - 16));// restore caller frame linkage
            emitter.instruction(&format!("add sp, sp, #{size}"));               // release aligned helper storage
        }
        Arch::X86_64 => {
            emitter.instruction("leave");                                       // restore the caller frame and stack pointer
        }
    }
    emitter.instruction("ret");                                                 // return the callback result or finish unwind cleanup
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{AppleVariant, Platform, Target};

    /// Every target publishes a cleanup activation and restores suspended owners.
    #[test]
    fn error_handler_invocation_has_unwind_cleanup_on_every_target() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new_apple(Arch::AArch64, AppleVariant::IOS),
            Target::new_apple(Arch::AArch64, AppleVariant::IOSSimulator),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            let mut features = RuntimeFeatures::none();
            features.handler_state = true;
            emit_error_handler_invoke(&mut emitter, features);
            let asm = emitter.output();
            for symbol in [
                "__rt_error_handler_invoke:", "__rt_error_handler_restore:",
                "_exc_call_frame_top", "__rt_callable_descriptor_release",
                "_php_error_handler_context_release",
            ] {
                assert!(asm.contains(symbol), "missing {symbol} for {target:?}");
            }
        }
    }
}
