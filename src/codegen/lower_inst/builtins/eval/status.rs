//! Purpose:
//! Handles eval bridge statuses, thrown values, and fatal diagnostics.
//!
//! Called from:
//! - The eval lowering facade and sibling eval support modules.
//!
//! Key details:
//! - Diagnostics and process exit remain target-aware and ABI-stable.

use super::*;

/// Emits a fatal diagnostic when the eval bridge reports any non-zero status.
pub(super) fn emit_eval_status_check(ctx: &mut FunctionContext<'_>) {
    let ok_label = ctx.next_label("eval_status_ok");
    let parse_error_label = ctx.next_label("eval_status_parse_error");
    let throwable_label = ctx.next_label("eval_status_throwable");
    let unsupported_label = ctx.next_label("eval_status_unsupported");
    abi::emit_branch_if_int_result_zero(ctx.emitter, &ok_label);
    emit_branch_if_eval_status(ctx, EVAL_STATUS_PARSE_ERROR, &parse_error_label);
    emit_branch_if_eval_status(ctx, EVAL_STATUS_UNCAUGHT_THROWABLE, &throwable_label);
    emit_branch_if_eval_status(ctx, EVAL_STATUS_UNSUPPORTED, &unsupported_label);
    emit_eval_bridge_fatal_exit(ctx, EVAL_STATUS_RUNTIME_FATAL);
    ctx.emitter.label(&parse_error_label);
    emit_eval_parse_error_exit(ctx);
    ctx.emitter.label(&throwable_label);
    emit_eval_throw_current(ctx);
    ctx.emitter.label(&unsupported_label);
    emit_eval_bridge_fatal_exit(ctx, EVAL_STATUS_UNSUPPORTED);
    ctx.emitter.label(&ok_label);
}

/// Branches to a label when the eval bridge returned a specific status code.
pub(super) fn emit_branch_if_eval_status(ctx: &mut FunctionContext<'_>, status: i64, label: &str) {
    let result_reg = abi::int_result_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, #{}", result_reg, status)); // compare the eval bridge status against the handled code
            ctx.emitter.instruction(&format!("b.eq {}", label));                // branch to the matching eval status handler
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, {}", result_reg, status)); // compare the eval bridge status against the handled code
            ctx.emitter.instruction(&format!("je {}", label));                  // branch to the matching eval status handler
        }
    }
}

/// Publishes an eval-thrown Throwable and enters the normal runtime unwinder.
pub(super) fn emit_eval_throw_current(ctx: &mut FunctionContext<'_>) {
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_RESULT_ERROR_OFFSET);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let object_reg = eval_mixed_unbox_low_payload_reg(ctx);
    abi::emit_store_reg_to_symbol(ctx.emitter, object_reg, "_exc_value", 0);
    abi::emit_call_label(ctx.emitter, "__rt_throw_current");
}

/// Returns the low payload register produced by `__rt_mixed_unbox` for eval status handling.
pub(super) fn eval_mixed_unbox_low_payload_reg(ctx: &FunctionContext<'_>) -> &'static str {
    match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdi",
    }
}

/// Exits after the bridge has already printed the PHP parse diagnostic.
///
/// The message names the file, the line and the failing token, none of which a constant in the
/// generated assembly can carry, so `libelephc-magician` prints it on standard output — where
/// `php -n` 8.5.6 prints it — before returning the parse-error status. Only the exit is left
/// here, and it uses PHP's own status for a parse error.
pub(super) fn emit_eval_parse_error_exit(ctx: &mut FunctionContext<'_>) {
    abi::emit_exit(ctx.emitter, EVAL_PARSE_ERROR_EXIT_STATUS);
}

/// Asks the bridge to print the fatal for one anonymous status, then exits the process.
///
/// The message names what the interpreter could not do, the file it was asked in and the line,
/// none of which a constant in the generated assembly can carry — the same reason the parse
/// diagnostic moved to `libelephc-magician`. `__elephc_eval_report_runtime_fatal` falls back to
/// the exact constant this used to emit when the interpreter recorded nothing, so a caller that
/// reads the diagnostic sees no less than before. Only the exit is left here, and it keeps the
/// status the bridge has always exited with.
pub(super) fn emit_eval_bridge_fatal_exit(ctx: &mut FunctionContext<'_>, status: i64) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("mov x0, #{status}")); // pass the ABI status the bridge should describe
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov edi, {status}")); // pass the ABI status the bridge should describe
        }
    }
    let symbol = ctx.emitter.target.extern_symbol("__elephc_eval_report_runtime_fatal");
    abi::emit_call_label(ctx.emitter, &symbol);
    abi::emit_exit(ctx.emitter, EVAL_RUNTIME_FATAL_EXIT_STATUS);
}
