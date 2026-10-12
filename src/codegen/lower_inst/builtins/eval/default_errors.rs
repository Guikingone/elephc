//! Purpose:
//! Publishes deferred native callable defaults to the eval signature ABI.
//!
//! Called from:
//! - Method and constructor metadata registration before dynamic eval executes.
//!
//! Key details:
//! - Deferred Errors are data, never callbacks that unwind through Rust frames.
//! - All supported targets use the shared six-argument registration ABI helpers.

use super::*;

/// Registers an Error-valued default, returning false for ordinary default trees.
pub(super) fn register_eval_native_param_default_error(
    ctx: &mut FunctionContext<'_>, context_offset: usize, key: (&str, usize),
    index: usize, default: Option<&Expr>, symbol: &str,
) -> bool {
    let Some(default) = default else { return false };
    let ExprKind::Throw(exception) = &default.kind else { return false };
    let ExprKind::NewObject { class_name, args } = &exception.kind else { return false };
    let [Expr { kind: ExprKind::StringLiteral(message), .. }] = args.as_slice() else { return false };
    if class_name.as_str() != "Error" { return false; }
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    load_eval_context_local_to_arg(ctx, context_offset, 0);
    for (index, label) in [(1, key.0), (4, message_label.as_str())] {
        abi::emit_symbol_address(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, index), label);
    }
    for (argument, value) in [(2, key.1), (3, index), (5, message_len)] {
        abi::emit_load_int_immediate(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, argument), value as i64);
    }
    let symbol = ctx.emitter.target.extern_symbol(symbol);
    abi::emit_call_label(ctx.emitter, &symbol);
    true
}
