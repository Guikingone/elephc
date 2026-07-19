//! Purpose:
//! Lowers a call site naming a curated late-bound undefined extension function (see
//! `crate::types::checker::builtins::late_bound`) to a catchable `\Error` throw with PHP's
//! exact "Call to undefined function X()" message, instead of an ordinary/builtin call.
//!
//! Called from:
//! - `crate::ir_lower::expr::mod::lower_function_call`, before argument materialization.
//!
//! Key details:
//! - `php -n` verified (side-effecting-argument probes): PHP resolves a function call at
//!   `INIT_FCALL`, BEFORE any argument `SEND_*` opcode runs, so an undefined-function call's
//!   arguments are never evaluated — not for a direct call, a dynamic `$fn()` call, a spread, or
//!   a named argument. This lowering therefore must NOT lower `call_expr`'s original argument
//!   expressions at all (no side effects, no hidden-temp materialization); the caller
//!   (`lower_function_call`) invokes this BEFORE it lowers/materializes any call operands.
//! - Reuses the exact same EIR shape a source-level `throw new \Error("...")` produces:
//!   `Op::ObjectNew` (via `super::lower_expr` on a synthetic `NewObject` AST node, so the
//!   "Error" builtin-Throwable-payload codegen path in
//!   `crate::codegen_ir::lower_inst::objects::lower_builtin_throwable_new` handles allocation on
//!   both targets unchanged) followed by `Op::ThrowException`, mirroring
//!   `crate::ir_lower::expr::mod::lower_throw_expr` (the lowering for PHP 8's `throw` expression)
//!   exactly, including its `lower_null` placeholder result — this call site's IR value is never
//!   actually consumed at runtime (the throw always fires), matching how throw-expressions are
//!   already deemed to still need "a value" structurally.
//! - The message is fully known at IR-lowering time (the callee name is a compile-time
//!   constant): no runtime string concatenation is needed, unlike the checked-downcast return
//!   type guard (`crate::ir_lower::stmt::return_type_guard`), which must read an actual runtime
//!   class name it cannot know until the mismatched value exists.

use crate::names::Name;
use crate::parser::ast::{Expr, ExprKind};

use super::super::context::{LoweredValue, LoweringContext};

/// Lowers `call_expr` (a call naming `canonical_name`) to a catchable `\Error` throw when
/// `canonical_name` is a curated late-bound undefined extension function. Returns `None` (no IR
/// emitted) for every other name, so the caller falls through to its ordinary
/// extern/user-function/builtin dispatch unchanged.
pub(super) fn lower_late_bound_undefined_call(
    ctx: &mut LoweringContext<'_, '_>,
    canonical_name: &str,
    call_expr: &Expr,
) -> Option<LoweredValue> {
    if !crate::types::checker::builtins::is_late_bound_undefined_function(canonical_name) {
        return None;
    }
    let span = call_expr.span;
    let message = format!("Call to undefined function {}()", canonical_name);
    let message_expr = Expr::new(ExprKind::StringLiteral(message), span);
    let new_error_expr = Expr::new(
        ExprKind::NewObject {
            class_name: Name::unqualified("Error"),
            args: vec![message_expr],
        },
        span,
    );
    let error_value = super::lower_expr(ctx, &new_error_expr);
    ctx.emit_void(
        crate::ir::Op::ThrowException,
        vec![error_value.value],
        None,
        crate::ir::Op::ThrowException.default_effects(),
        Some(span),
    );
    Some(super::lower_null(ctx, call_expr))
}
