//! Purpose:
//! Cross-variable relational and strict-equality atoms for DCE guard reasoning v2.
//!
//! Called from:
//! - `crate::optimize::control::dce::guards::record::extend_guards`
//! - `crate::optimize::control::dce::guards::eval::known_condition_value_base`
//!
//! Key details:
//! - Loose `==` / `!=` stay on structural `condition_guards` only (no coercion theorems).
//! - Exact int facts on one side eagerly strengthen the other side's integer range.
//! - False-branch complements for relational ops between variables follow the same
//!   NaN policy as condition complements: only recorded when already in an integer
//!   domain or when the op is strict equality (total).

use crate::parser::ast::{BinOp, Expr, ExprKind};

use super::super::state::{
    GuardLiteral, GuardState, IntInterval, RelOp, RelSide, RelationalGuard,
};
use super::eval::known_exact_guard;
use super::range::{interval_from_relational, known_range_guard, record_range_guard};

/// Converts a tracked `BinOp` into a `RelOp`.
fn rel_op_from_binop(op: &BinOp) -> Option<RelOp> {
    match op {
        BinOp::Lt => Some(RelOp::Lt),
        BinOp::LtEq => Some(RelOp::Le),
        BinOp::Gt => Some(RelOp::Gt),
        BinOp::GtEq => Some(RelOp::Ge),
        BinOp::StrictEq => Some(RelOp::StrictEq),
        BinOp::StrictNotEq => Some(RelOp::StrictNotEq),
        _ => None,
    }
}

/// Converts a `RelOp` back to a `BinOp` for interval helpers.
fn binop_from_rel_op(op: RelOp) -> BinOp {
    match op {
        RelOp::Lt => BinOp::Lt,
        RelOp::Le => BinOp::LtEq,
        RelOp::Gt => BinOp::Gt,
        RelOp::Ge => BinOp::GtEq,
        RelOp::StrictEq => BinOp::StrictEq,
        RelOp::StrictNotEq => BinOp::StrictNotEq,
    }
}

/// Returns the operand-swapped operator (`$x > $y` ↔ `$y < $x`).
fn swap_rel(op: RelOp) -> RelOp {
    match op {
        RelOp::Lt => RelOp::Gt,
        RelOp::Le => RelOp::Ge,
        RelOp::Gt => RelOp::Lt,
        RelOp::Ge => RelOp::Le,
        RelOp::StrictEq => RelOp::StrictEq,
        RelOp::StrictNotEq => RelOp::StrictNotEq,
    }
}

/// Returns the logical inverse operator when it is total for the atom's sides.
fn inverse_rel(op: RelOp) -> RelOp {
    match op {
        RelOp::Lt => RelOp::Ge,
        RelOp::Le => RelOp::Gt,
        RelOp::Gt => RelOp::Le,
        RelOp::Ge => RelOp::Lt,
        RelOp::StrictEq => RelOp::StrictNotEq,
        RelOp::StrictNotEq => RelOp::StrictEq,
    }
}

/// Parses a relational / strict-equality atom with `Var|Int` sides.
pub(super) fn relational_atom(condition: &Expr) -> Option<(RelSide, RelOp, RelSide)> {
    let ExprKind::BinaryOp { left, op, right } = &condition.kind else {
        return None;
    };
    let rel_op = rel_op_from_binop(op)?;
    let left_side = rel_side(left)?;
    let right_side = rel_side(right)?;
    // Require at least one variable; int-int atoms are constant-foldable elsewhere.
    if matches!((&left_side, &right_side), (RelSide::Int(_), RelSide::Int(_))) {
        return None;
    }
    Some((left_side, rel_op, right_side))
}

/// Parses one atom side as a variable or int literal.
fn rel_side(expr: &Expr) -> Option<RelSide> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(RelSide::Var(name.clone())),
        ExprKind::IntLiteral(n) => Some(RelSide::Int(*n)),
        _ => None,
    }
}

/// Returns whether a relational false-branch complement is safe to record.
fn relational_inverse_is_safe(guards: &GuardState, left: &RelSide, op: RelOp, right: &RelSide) -> bool {
    match op {
        RelOp::StrictEq | RelOp::StrictNotEq => true,
        RelOp::Lt | RelOp::Le | RelOp::Gt | RelOp::Ge => {
            side_has_integer_domain(guards, left) || side_has_integer_domain(guards, right)
        }
    }
}

/// Returns whether a side is an int literal or a variable with an int-domain fact.
fn side_has_integer_domain(guards: &GuardState, side: &RelSide) -> bool {
    match side {
        RelSide::Int(_) => true,
        RelSide::Var(name) => {
            known_range_guard(guards, name).is_some()
                || matches!(known_exact_guard(guards, name), Some(GuardLiteral::Int(_)))
        }
    }
}

/// Upserts a relational atom (deduped by left/op/right, polarity updated).
fn upsert_relational_guard(
    guards: &mut GuardState,
    left: RelSide,
    op: RelOp,
    right: RelSide,
    holds: bool,
) {
    if let Some(existing) = guards.relational_guards.iter_mut().find(|known| {
        known.left == left && known.op == op && known.right == right
    }) {
        existing.holds = holds;
        return;
    }
    guards.relational_guards.push(RelationalGuard {
        left,
        op,
        right,
        holds,
    });
}

/// Records a relational atom and its safe complement / swapped forms.
pub(super) fn record_relational_guard(
    guards: &mut GuardState,
    left: RelSide,
    op: RelOp,
    right: RelSide,
    holds: bool,
) {
    upsert_relational_guard(guards, left.clone(), op, right.clone(), holds);
    upsert_relational_guard(guards, right.clone(), swap_rel(op), left.clone(), holds);

    if holds || relational_inverse_is_safe(guards, &left, op, &right) {
        let inv = inverse_rel(op);
        upsert_relational_guard(guards, left.clone(), inv, right.clone(), !holds);
        upsert_relational_guard(guards, right, swap_rel(inv), left, !holds);
    }
}

/// Looks up an exact int for a relational side when available.
fn side_exact_int(guards: &GuardState, side: &RelSide) -> Option<i64> {
    match side {
        RelSide::Int(n) => Some(*n),
        RelSide::Var(name) => match known_exact_guard(guards, name) {
            Some(GuardLiteral::Int(n)) => Some(*n),
            _ => {
                let interval = known_range_guard(guards, name)?;
                match (interval.lo, interval.hi) {
                    (Some(lo), Some(hi)) if lo == hi => Some(lo),
                    _ => None,
                }
            }
        },
    }
}

/// When one side of a holding relational atom is a concrete int, strengthen the
/// other variable's integer range (the multi-variable → range bridge).
fn strengthen_range_from_relational(
    guards: &mut GuardState,
    left: &RelSide,
    op: RelOp,
    right: &RelSide,
    holds: bool,
) {
    if !holds {
        // Use the inverse polarity for false atoms when safe.
        if !relational_inverse_is_safe(guards, left, op, right) {
            return;
        }
        strengthen_range_from_relational(guards, left, inverse_rel(op), right, true);
        return;
    }

    match (left, right) {
        (RelSide::Var(name), _) => {
            if let Some(n) = side_exact_int(guards, right) {
                if matches!(op, RelOp::StrictEq) {
                    record_range_guard(guards, name, IntInterval::point(n));
                } else if let Some(contrib) =
                    interval_from_relational(&binop_from_rel_op(op), n, true)
                {
                    record_range_guard(guards, name, contrib);
                }
            }
        }
        (_, RelSide::Var(name)) => {
            if let Some(n) = side_exact_int(guards, left) {
                let swapped = swap_rel(op);
                if matches!(swapped, RelOp::StrictEq) {
                    record_range_guard(guards, name, IntInterval::point(n));
                } else if let Some(contrib) =
                    interval_from_relational(&binop_from_rel_op(swapped), n, true)
                {
                    record_range_guard(guards, name, contrib);
                }
            }
        }
        _ => {}
    }
}

/// Records relational atoms from a branch condition and derives range strengthenings.
pub(super) fn extend_relational_guards(
    guards: &mut GuardState,
    condition: &Expr,
    branch_taken: bool,
) {
    let Some((left, op, right)) = relational_atom(condition) else {
        return;
    };

    // Var-vs-int relational atoms are also handled by the range domain; still
    // record the atom so swapped / complementary queries hit structurally.
    if !branch_taken && matches!(op, RelOp::Lt | RelOp::Le | RelOp::Gt | RelOp::Ge) {
        if !relational_inverse_is_safe(guards, &left, op, &right) {
            // Still record the false polarity of the atom itself (not the inverse).
            upsert_relational_guard(guards, left.clone(), op, right.clone(), false);
            upsert_relational_guard(guards, right, swap_rel(op), left, false);
            return;
        }
    }

    record_relational_guard(guards, left.clone(), op, right.clone(), branch_taken);
    strengthen_range_from_relational(guards, &left, op, &right, branch_taken);
}

/// Structural lookup of a recorded relational atom, including swapped forms.
fn lookup_relational(
    guards: &GuardState,
    left: &RelSide,
    op: RelOp,
    right: &RelSide,
) -> Option<bool> {
    for known in &guards.relational_guards {
        if known.left == *left && known.op == op && known.right == *right {
            return Some(known.holds);
        }
        if known.left == *right && known.op == swap_rel(op) && known.right == *left {
            return Some(known.holds);
        }
    }
    None
}

/// Proves a condition from relational atoms, with exact/range substitution into
/// the opposite side when one operand is a concrete int.
pub(in crate::optimize::control::dce) fn known_from_relational(
    guards: &GuardState,
    condition: &Expr,
) -> Option<bool> {
    let (left, op, right) = relational_atom(condition)?;

    if let Some(value) = lookup_relational(guards, &left, op, &right) {
        return Some(value);
    }

    // Substitute concrete ints and discharge through range-style var/int atoms
    // already recorded, or through known_from_range after rewriting.
    match (&left, &right) {
        (RelSide::Var(name), RelSide::Var(other)) => {
            if let Some(n) = side_exact_int(guards, &RelSide::Var(other.clone())) {
                return known_from_substituted_var_int(guards, name, op, n);
            }
            if let Some(n) = side_exact_int(guards, &RelSide::Var(name.clone())) {
                return known_from_substituted_var_int(guards, other, swap_rel(op), n);
            }
            None
        }
        (RelSide::Var(name), RelSide::Int(n)) => known_from_substituted_var_int(guards, name, op, *n),
        (RelSide::Int(n), RelSide::Var(name)) => {
            known_from_substituted_var_int(guards, name, swap_rel(op), *n)
        }
        _ => None,
    }
}

/// Discharges a var/int relational or strict-equality after substitution.
fn known_from_substituted_var_int(
    guards: &GuardState,
    name: &str,
    op: RelOp,
    n: i64,
) -> Option<bool> {
    // Prefer an already-recorded atom.
    if let Some(value) = lookup_relational(
        guards,
        &RelSide::Var(name.to_string()),
        op,
        &RelSide::Int(n),
    ) {
        return Some(value);
    }

    let Some(interval) = known_range_guard(guards, name).copied() else {
        // Exact int on the variable itself.
        return match (known_exact_guard(guards, name), op) {
            (Some(GuardLiteral::Int(known)), RelOp::StrictEq) => Some(*known == n),
            (Some(GuardLiteral::Int(known)), RelOp::StrictNotEq) => Some(*known != n),
            (Some(GuardLiteral::Int(known)), RelOp::Lt) => Some(*known < n),
            (Some(GuardLiteral::Int(known)), RelOp::Le) => Some(*known <= n),
            (Some(GuardLiteral::Int(known)), RelOp::Gt) => Some(*known > n),
            (Some(GuardLiteral::Int(known)), RelOp::Ge) => Some(*known >= n),
            _ => None,
        };
    };

    match op {
        RelOp::StrictEq => {
            if interval.contains(n) {
                None
            } else {
                Some(false)
            }
        }
        RelOp::StrictNotEq => {
            if interval.contains(n) {
                None
            } else {
                Some(true)
            }
        }
        RelOp::Lt | RelOp::Le | RelOp::Gt | RelOp::Ge => {
            interval_entails_for_rel(interval, op, n)
        }
    }
}

/// Interval entailment for `RelOp` against an int literal (mirrors range.rs).
fn interval_entails_for_rel(interval: IntInterval, op: RelOp, n: i64) -> Option<bool> {
    match (interval.lo, interval.hi, op) {
        (Some(lo), Some(hi), _) if lo > hi => None,
        (Some(lo), Some(hi), op) if lo == hi => Some(match op {
            RelOp::Lt => lo < n,
            RelOp::Le => lo <= n,
            RelOp::Gt => lo > n,
            RelOp::Ge => lo >= n,
            _ => return None,
        }),
        (Some(lo), Some(hi), RelOp::Lt) => {
            if hi < n {
                Some(true)
            } else if lo >= n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), RelOp::Le) => {
            if hi <= n {
                Some(true)
            } else if lo > n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), RelOp::Gt) => {
            if lo > n {
                Some(true)
            } else if hi <= n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), Some(hi), RelOp::Ge) => {
            if lo >= n {
                Some(true)
            } else if hi < n {
                Some(false)
            } else {
                None
            }
        }
        (Some(lo), None, RelOp::Gt) if lo > n => Some(true),
        (Some(lo), None, RelOp::Ge) if lo >= n => Some(true),
        (Some(lo), None, RelOp::Lt) if lo >= n => Some(false),
        (Some(lo), None, RelOp::Le) if lo > n => Some(false),
        (None, Some(hi), RelOp::Lt) if hi < n => Some(true),
        (None, Some(hi), RelOp::Le) if hi <= n => Some(true),
        (None, Some(hi), RelOp::Gt) if hi <= n => Some(false),
        (None, Some(hi), RelOp::Ge) if hi < n => Some(false),
        _ => None,
    }
}
