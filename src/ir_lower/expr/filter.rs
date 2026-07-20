//! Purpose:
//! Lowers literal-dispatched `filter_var()` calls to a synthetic per-filter
//! builtin name that `crate::codegen_ir::lower_inst::builtins::filter` matches
//! on directly, so the EIR backend never has to re-derive the filter id/flags
//! from operand structure (which would depend on whether `--ir-opt` folded a
//! `FILTER_NULL_ON_FAILURE | FILTER_REQUIRE_SCALAR`-style flags expression).
//!
//! Called from:
//! - `crate::ir_lower::expr::lower_function_call()`.
//!
//! Key details:
//! - The checker (`crate::types::checker::builtins::system::check_builtin`)
//!   already rejects non-literal filter ids/flags and unsupported filters/flags
//!   before lowering ever runs, so this module can assume every call it sees
//!   resolves statically. It still returns `None` defensively (falling through
//!   to the generic builtin-call path, which surfaces a loud "unsupported
//!   builtin" EIR error) if that assumption is somehow violated.
//! - Only plain positional 1-3 argument calls are handled; a named/spread call
//!   falls through to the same defensive `None` path.

use crate::names::php_symbol_key;
use crate::parser::ast::{BinOp, Expr, ExprKind};
use crate::types::filter_constants::FILTER_INT_CONSTANTS;
use crate::types::PhpType;

use super::super::context::{LoweredValue, LoweringContext};

/// Lowers a literal-dispatched `filter_var()` call to its synthetic per-filter
/// builtin, or returns `None` to defer to the generic builtin-call path.
pub(super) fn lower_static_filter_var(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "filter_var" {
        return None;
    }
    if !(1..=3).contains(&args.len()) {
        return None;
    }
    if crate::types::call_args::has_named_args(args) || args.iter().any(super::is_spread_arg) {
        return None;
    }
    let filter_id = if args.len() >= 2 {
        static_int_value(&args[1])?
    } else {
        516 // FILTER_DEFAULT (== FILTER_UNSAFE_RAW)
    };
    let flags = if args.len() == 3 {
        static_int_value(&args[2])?
    } else {
        0
    };
    let null_on_failure = flags & 134_217_728 != 0;
    let synthetic_name = match (filter_id, null_on_failure) {
        // DEFAULT/UNSAFE_RAW only "fails" on array input (no REQUIRE_ARRAY/FORCE_ARRAY);
        // every scalar/null input always passes (php-verified), so the null_on_failure
        // variant only changes the array-input failure result (false vs null).
        (516, false) => "filter_var$default",
        (516, true) => "filter_var$default_nof",
        (257, false) => "filter_var$int",
        (257, true) => "filter_var$int_nof",
        (259, false) => "filter_var$float",
        (259, true) => "filter_var$float_nof",
        (258, false) => "filter_var$bool",
        (258, true) => "filter_var$bool_nof",
        // VALIDATE_IP: FILTER_FLAG_IPV4 (1048576) / FILTER_FLAG_IPV6 (2097152)
        // restrict the accepted family; both set or neither set accept either
        // family (php-verified — see `crate::types::filter_constants`'s module
        // doc for the exact matrix).
        (275, _) => {
            let ipv4 = flags & 1_048_576 != 0;
            let ipv6 = flags & 2_097_152 != 0;
            let family = match (ipv4, ipv6) {
                (true, false) => "ip4",
                (false, true) => "ip6",
                _ => "ip",
            };
            let value = super::lower_expr(ctx, &args[0]);
            let synthetic_name = if null_on_failure {
                format!("filter_var${family}_nof")
            } else {
                format!("filter_var${family}")
            };
            return Some(super::emit_builtin_call_value(
                ctx,
                &synthetic_name,
                vec![value.value],
                PhpType::Mixed,
                expr.span,
            ));
        }
        _ => return None,
    };
    let value = super::lower_expr(ctx, &args[0]);
    Some(super::emit_builtin_call_value(
        ctx,
        synthetic_name,
        vec![value.value],
        PhpType::Mixed,
        expr.span,
    ))
}

/// Attempts to evaluate an expression as a static integer at compile time
/// against the `ext/filter` constant table. Mirrors
/// `crate::types::checker::builtins::system`'s `filter_static_int_value`
/// (kept as a small, independently-testable duplicate since the checker and
/// ir_lower layers walk the same `&Expr` AST but do not share a helper module).
fn static_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::ConstRef(name) => FILTER_INT_CONSTANTS
            .iter()
            .find_map(|(constant, value)| (*constant == name.as_str()).then_some(*value)),
        ExprKind::Negate(inner) => static_int_value(inner).map(|value| -value),
        ExprKind::BinaryOp { left, op, right } => {
            let left = static_int_value(left)?;
            let right = static_int_value(right)?;
            match op {
                BinOp::BitAnd => Some(left & right),
                BinOp::BitOr => Some(left | right),
                BinOp::BitXor => Some(left ^ right),
                _ => None,
            }
        }
        _ => None,
    }
}
