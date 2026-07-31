//! Purpose:
//! Handles DCE state cases.
//! Preserves observable effects while removing unreachable tails, redundant branches, or dead writes.
//!
//! Called from:
//! - `crate::optimize::control::dce`
//!
//! Key details:
//! - The pass must remain conservative around throws, finally blocks, switch fallthrough, method calls, and variable writes.

use crate::parser::ast::Expr;

#[derive(Clone, Copy)]
/// Indicates where a dead-code tail should sink: either into the next statement
/// (`FallsThrough`) or out of a surrounding construct (`Breaks`).
pub(super) enum TailSinkTarget {
    FallsThrough,
    Breaks,
}

#[derive(Clone, Default)]
/// Tracks guard conditions discovered during DCE analysis.
///
/// `truthy_vars` / `falsy_vars` — variables known to be true/false at this point.
/// `bool_true_vars` / `bool_false_vars` — boolean-typed variables confirmed true/false.
/// `exact_guards` — variable = literal constraints currently active.
/// `excluded_guards` — variable = literal constraints ruled out at this point.
/// `condition_guards` — complex expression conditions mapped to a known boolean value
/// and the set of variables they constrain.
/// `range_guards` — integer interval facts from relational int-literal branches.
/// `relational_guards` — cross-variable (or var/int) relational and strict-equality atoms.
pub(super) struct GuardState {
    pub(super) truthy_vars: Vec<String>,
    pub(super) falsy_vars: Vec<String>,
    pub(super) bool_true_vars: Vec<String>,
    pub(super) bool_false_vars: Vec<String>,
    pub(super) exact_guards: Vec<ExactGuard>,
    pub(super) excluded_guards: Vec<ExactGuard>,
    pub(super) condition_guards: Vec<ConditionGuard>,
    pub(super) range_guards: Vec<RangeGuard>,
    pub(super) relational_guards: Vec<RelationalGuard>,
}

#[derive(Clone, PartialEq, Eq)]
/// Records an exact constraint that a variable holds a specific literal value.
/// `name` is the variable name; `value` is the known literal.
pub(super) struct ExactGuard {
    pub(super) name: String,
    pub(super) value: GuardLiteral,
}

#[derive(Clone)]
/// Records a condition expression that evaluates to a known boolean value at this point.
/// `condition` is the expression; `value` is the known result; `names` are the variables
/// whose guard state is affected by this condition.
pub(super) struct ConditionGuard {
    pub(super) condition: Expr,
    pub(super) value: bool,
    pub(super) names: Vec<String>,
}

#[derive(Clone, PartialEq, Eq)]
/// The set of literal values a guard can constrain a variable to.
/// Used in `ExactGuard` to record variable = value constraints.
pub(super) enum GuardLiteral {
    Bool(bool),
    Null,
    Int(i64),
    Float(u64),
    String(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Inclusive integer bounds; `None` means ±∞ on that side.
pub(super) struct IntInterval {
    pub(super) lo: Option<i64>,
    pub(super) hi: Option<i64>,
}

impl IntInterval {
    /// Returns a degenerate point interval `[n, n]`.
    pub(super) fn point(n: i64) -> Self {
        Self {
            lo: Some(n),
            hi: Some(n),
        }
    }

    /// Returns whether `n` lies inside this inclusive interval.
    pub(super) fn contains(self, n: i64) -> bool {
        if let Some(lo) = self.lo {
            if n < lo {
                return false;
            }
        }
        if let Some(hi) = self.hi {
            if n > hi {
                return false;
            }
        }
        true
    }

    /// Intersects two intervals. Returns `None` when the result is empty.
    pub(super) fn intersect(self, other: Self) -> Option<Self> {
        let lo = match (self.lo, other.lo) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let hi = match (self.hi, other.hi) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        if let (Some(lo), Some(hi)) = (lo, hi) {
            if lo > hi {
                return None;
            }
        }
        Some(Self { lo, hi })
    }
}

#[derive(Clone, PartialEq, Eq)]
/// Integer range fact for a single variable under the current path.
pub(super) struct RangeGuard {
    pub(super) name: String,
    pub(super) interval: IntInterval,
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// One side of a relational / strict-equality atom: a variable or an int literal.
pub(super) enum RelSide {
    Var(String),
    Int(i64),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Relational and strict-equality operators tracked as first-class guard atoms.
pub(super) enum RelOp {
    Lt,
    Le,
    Gt,
    Ge,
    StrictEq,
    StrictNotEq,
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Cross-variable (or var/int) relational fact with recorded polarity.
pub(super) struct RelationalGuard {
    pub(super) left: RelSide,
    pub(super) op: RelOp,
    pub(super) right: RelSide,
    pub(super) holds: bool,
}
