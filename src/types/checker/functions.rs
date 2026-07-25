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

pub(crate) use returns::ReturnInfo;

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
