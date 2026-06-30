//! Purpose:
//! Type-checks function declarations, calls, and function-like control-flow contracts.
//! Connects call resolution, shared argument validation, return analysis, and inferred function metadata.
//!
//! Called from:
//! - `crate::types::checker::driver::functions`
//! - `crate::types::checker::inference`
//!
//! Key details:
//! - User functions, builtins, externs, and callable aliases must share the same argument semantics.

mod call_validation;
mod resolution;
mod returns;

use crate::parser::ast::{Expr, ExprKind};

/// Returns true when `arg` is an acceptable lvalue for a by-reference argument slot.
///
/// Accepts a plain `$var` (the established path) and, additively, a non-nullsafe
/// instance property fetch (`$this->prop`, `$obj->prop`). EIR lowering routes the
/// property form through a hidden copy-in/copy-out temp so the existing
/// plain-variable by-reference machinery is reused unchanged. Nullsafe property
/// access, dynamic property access (`$obj->$name`), static properties (`Cls::$p`),
/// array elements (`$arr[$k]`), and non-lvalues stay rejected (future work).
pub(crate) fn is_by_ref_lvalue(arg: &Expr) -> bool {
    matches!(arg.kind, ExprKind::Variable(_)) || is_by_ref_property_arg(arg)
}

/// Returns true when `arg` is a non-nullsafe instance property fetch eligible for
/// by-reference copy-in/copy-out lowering.
///
/// Only `ExprKind::PropertyAccess` (a statically named instance property) qualifies;
/// the receiver may be any expression (typically `$this` or `$obj`). The caller must
/// skip `require_boxed_by_ref_storage` for these args because the object's declared
/// property storage must never be widened — the hidden temp carries the wide storage.
pub(crate) fn is_by_ref_property_arg(arg: &Expr) -> bool {
    matches!(arg.kind, ExprKind::PropertyAccess { .. })
}
