//! Purpose:
//! Lowers unresolved direct function calls to PHP-compatible catchable `Error` throws.
//!
//! Called from:
//! - `crate::ir_lower::expr::function_calls::lower_function_call()` before argument lowering.
//!
//! Key details:
//! - PHP resolves an absent function before evaluating its arguments, so this path emits no
//!   argument side effects.
//! - The unreachable placeholder is boxed `Mixed` so downstream EIR remains well typed.

use crate::names::Name;
use crate::parser::ast::{Expr, ExprKind};

use super::super::context::{LoweredValue, LoweringContext};

/// Lowers one unresolved direct call to `throw new Error(...)`, or returns `None` otherwise.
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
    let null = super::lower_null(ctx, call_expr);
    Some(ctx.box_value_as_mixed(null, crate::types::PhpType::Mixed, Some(span)))
}
