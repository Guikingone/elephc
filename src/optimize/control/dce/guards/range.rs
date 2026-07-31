//! Purpose:
//! Integer interval facts for DCE guard reasoning v2.
//! Records and discharges `$x <op> int` bounds under path-local GuardState.
//!
//! Called from:
//! - `crate::optimize::control::dce::guards::record::extend_guards`
//! - `crate::optimize::control::dce::guards::eval::known_condition_value_base`
//! - `crate::optimize::control::dce::switches` for impossible int cases
//!
//! Key details:
//! - Taken-true relational branches with an int literal establish or intersect ranges.
//! - False-branch inverse bounds are applied only when the variable already has an
//!   integer domain fact (exact int or an existing range), matching NaN conservatism
//!   for unconstrained float-typed values.
//! - Overflowing bound shifts (`>` at `i64::MAX`, `<` at `i64::MIN`) refuse to record.

use crate::parser::ast::{BinOp, Expr, ExprKind};

use super::super::state::{GuardLiteral, GuardState, IntInterval, RangeGuard};
use super::eval::known_exact_guard;

/// Matches `$name <op> int` or `int <op> $name` and returns a normalized
/// `(name, op, literal)` as if the variable were on the left.
pub(super) fn int_relational_guard(condition: &Expr) -> Option<(&str, BinOp, i64)> {
    let ExprKind::BinaryOp { left, op, right } = &condition.kind else {
        return None;
    };

    match (&left.kind, op, &right.kind) {
        (ExprKind::Variable(name), op, ExprKind::IntLiteral(n))
            if matches!(
                op,
                BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq
            ) =>
        {
            Some((name.as_str(), op.clone(), *n))
        }
        (ExprKind::IntLiteral(n), op, ExprKind::Variable(name))
            if matches!(
                op,
                BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq
            ) =>
        {
            let swapped = match op {
                BinOp::Lt => BinOp::Gt,
                BinOp::LtEq => BinOp::GtEq,
                BinOp::Gt => BinOp::Lt,
                BinOp::GtEq => BinOp::LtEq,
                _ => return None,
            };
            Some((name.as_str(), swapped, *n))
        }
        _ => None,
    }
}

/// Builds an inclusive integer interval contribution for a relational fact.
///
/// Returns `None` when the bound shift would overflow `i64` or the operator is
/// not a relational comparison.
pub(super) fn interval_from_relational(
    op: &BinOp,
    n: i64,
    branch_taken: bool,
) -> Option<IntInterval> {
    let effective_op = if branch_taken {
        op.clone()
    } else {
        match op {
            BinOp::Lt => BinOp::GtEq,
            BinOp::LtEq => BinOp::Gt,
            BinOp::Gt => BinOp::LtEq,
            BinOp::GtEq => BinOp::Lt,
            _ => return None,
        }
    };

    match effective_op {
        BinOp::Gt => {
            let lo = n.checked_add(1)?;
            Some(IntInterval {
                lo: Some(lo),
                hi: None,
            })
        }
        BinOp::GtEq => Some(IntInterval {
            lo: Some(n),
            hi: None,
        }),
        BinOp::Lt => {
            let hi = n.checked_sub(1)?;
            Some(IntInterval {
                lo: None,
                hi: Some(hi),
            })
        }
        BinOp::LtEq => Some(IntInterval {
            lo: None,
            hi: Some(n),
        }),
        _ => None,
    }
}

/// Returns the current interval for `name`, if any.
pub(in crate::optimize::control::dce) fn known_range_guard<'a>(
    guards: &'a GuardState,
    name: &str,
) -> Option<&'a IntInterval> {
    guards
        .range_guards
        .iter()
        .find(|known| known.name == name)
        .map(|known| &known.interval)
}

/// Returns whether `name` already carries an integer-domain fact that makes
/// false-branch relational inverses safe to apply.
fn has_integer_domain(guards: &GuardState, name: &str) -> bool {
    if known_range_guard(guards, name).is_some() {
        return true;
    }
    matches!(known_exact_guard(guards, name), Some(GuardLiteral::Int(_)))
}

/// Intersects `contrib` into the range fact for `name`, or installs it when absent.
///
/// An empty intersection leaves the previous fact untouched (the branch is already
/// contradictory; callers prune via `known_condition_value` rather than bottom facts).
pub(super) fn record_range_guard(guards: &mut GuardState, name: &str, contrib: IntInterval) {
    if let Some(existing) = guards
        .range_guards
        .iter_mut()
        .find(|known| known.name == name)
    {
        if let Some(next) = existing.interval.intersect(contrib) {
            existing.interval = next;
        }
        return;
    }

    guards.range_guards.push(RangeGuard {
        name: name.to_string(),
        interval: contrib,
    });
}

/// Couples an exact integer literal fact to a point range for `name`.
pub(super) fn record_exact_int_range(guards: &mut GuardState, name: &str, value: i64) {
    // Exact clears other facts for the name first; replace any prior range.
    guards.range_guards.retain(|known| known.name != name);
    guards.range_guards.push(RangeGuard {
        name: name.to_string(),
        interval: IntInterval::point(value),
    });
}

/// Records range contributions from a branch condition when applicable.
pub(super) fn extend_range_guards(guards: &mut GuardState, condition: &Expr, branch_taken: bool) {
    let Some((name, op, n)) = int_relational_guard(condition) else {
        return;
    };

    if !branch_taken && !has_integer_domain(guards, name) {
        // Unconstrained false-branch relational inverses are not total under NaN.
        return;
    }

    let Some(contrib) = interval_from_relational(&op, n, branch_taken) else {
        return;
    };
    record_range_guard(guards, name, contrib);
}

/// Evaluates whether every integer in `interval` agrees on `op` against `n`.
fn interval_entails_relational(interval: IntInterval, op: &BinOp, n: i64) -> Option<bool> {
    match (interval.lo, interval.hi, op) {
        (Some(lo), Some(hi), _) if lo > hi => None,
        (Some(lo), Some(hi), op) if lo == hi => Some(match op {
            BinOp::Lt => lo < n,
            BinOp::LtEq => lo <= n,
            BinOp::Gt => lo > n,
            BinOp::GtEq => lo >= n,
            _ => return None,
        }),
        // Fully bounded non-point: endpoints agreeing decides monotonic relations.
        (Some(lo), Some(hi), BinOp::Lt) => {
            if hi < n {
                Some(true)
            } else if lo >= n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), BinOp::LtEq) => {
            if hi <= n {
                Some(true)
            } else if lo > n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), BinOp::Gt) => {
            if lo > n {
                Some(true)
            } else if hi <= n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), BinOp::GtEq) => {
            if lo >= n {
                Some(true)
            } else if hi < n {
                Some(false)
            } else {
                None
            }
        }
        // Lower-bounded only.
        (Some(lo), None, BinOp::Gt) => {
            if lo > n {
                Some(true)
            } else {
                None
            }
        }
        (Some(lo), None, BinOp::GtEq) => {
            if lo >= n {
                Some(true)
            } else {
                None
            }
        }
        (Some(lo), None, BinOp::Lt) => {
            if lo >= n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), None, BinOp::LtEq) => {
            if lo > n {
                Some(false)
            } else {
                None
            }
        }
        // Upper-bounded only.
        (None, Some(hi), BinOp::Lt) => {
            if hi < n {
                Some(true)
            } else {
                None
            }
        }
        (None, Some(hi), BinOp::LtEq) => {
            if hi <= n {
                Some(true)
            } else {
                None
            }
        }
        (None, Some(hi), BinOp::Gt) => {
            if hi <= n {
                Some(false)
            } else {
                None
            }
        }
        (None, Some(hi), BinOp::GtEq) => {
            if hi < n {
                Some(false)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Proves a nested condition from integer range facts when every in-range int agrees.
///
/// Strict equality is only proved **false** when `n` lies outside the interval;
/// proving `===` true stays with `exact_guards` (float `0.0` vs int `0`).
pub(in crate::optimize::control::dce) fn known_from_range(
    guards: &GuardState,
    condition: &Expr,
) -> Option<bool> {
    if let Some((name, op, n)) = int_relational_guard(condition) {
        let interval = *known_range_guard(guards, name)?;
        return interval_entails_relational(interval, &op, n);
    }

    let ExprKind::BinaryOp { left, op, right } = &condition.kind else {
        return None;
    };

    let (name, compared, expects_equal) = match (&left.kind, op, &right.kind) {
        (ExprKind::Variable(name), BinOp::StrictEq, ExprKind::IntLiteral(n)) => {
            (name.as_str(), *n, true)
        }
        (ExprKind::IntLiteral(n), BinOp::StrictEq, ExprKind::Variable(name)) => {
            (name.as_str(), *n, true)
        }
        (ExprKind::Variable(name), BinOp::StrictNotEq, ExprKind::IntLiteral(n)) => {
            (name.as_str(), *n, false)
        }
        (ExprKind::IntLiteral(n), BinOp::StrictNotEq, ExprKind::Variable(name)) => {
            (name.as_str(), *n, false)
        }
        _ => return None,
    };

    let interval = *known_range_guard(guards, name)?;
    if interval.contains(compared) {
        // Inside the range: cannot prove === true from a range alone (float safety).
        return None;
    }

    // Outside the range: === is false, !== is true.
    Some(!expects_equal)
}
