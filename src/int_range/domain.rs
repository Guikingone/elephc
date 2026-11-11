//! Purpose:
//! Defines the abstract domain of the integer-range analysis: closed `i64` intervals for values
//! that are definitely integers, plus flow states mapping local variables to such intervals.
//!
//! Called from:
//! - `crate::int_range::analyzer` while interpreting function bodies.
//!
//! Key details:
//! - `AbsVal::Int(lo, hi)` claims "this value is an `int` in `[lo, hi]`"; `AbsVal::Top` claims
//!   nothing (the value may be any PHP value, including a float produced by an overflow).
//! - A `State` of `None` is unreachable code; inside a reachable state a variable that is absent
//!   from the map is `Top`, so forgetting a fact is always the sound direction.
//! - Interval arithmetic runs in `i128`, so detecting a result outside the `i64` range is exact.

use std::collections::BTreeMap;

/// Smallest PHP integer (`PHP_INT_MIN`).
pub(crate) const MIN: i64 = i64::MIN;
/// Largest PHP integer (`PHP_INT_MAX`).
pub(crate) const MAX: i64 = i64::MAX;

/// Abstract value of one expression or variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AbsVal {
    /// A definite `int` within the inclusive bounds.
    Int(i64, i64),
    /// Any PHP value (no claim).
    Top,
}

impl AbsVal {
    /// A definite `int` with no known bounds.
    pub(crate) const FULL_INT: AbsVal = AbsVal::Int(MIN, MAX);

    /// Returns a constant interval.
    pub(crate) fn exact(value: i64) -> AbsVal {
        AbsVal::Int(value, value)
    }

    /// Builds an interval from `i128` bounds, or `None` when either bound leaves the `i64` range.
    pub(crate) fn from_i128(lo: i128, hi: i128) -> Option<AbsVal> {
        if lo < MIN as i128 || hi > MAX as i128 || lo > hi {
            return None;
        }
        Some(AbsVal::Int(lo as i64, hi as i64))
    }

    /// Returns the bounds when the value is a definite int.
    pub(crate) fn bounds(self) -> Option<(i64, i64)> {
        match self {
            AbsVal::Int(lo, hi) => Some((lo, hi)),
            AbsVal::Top => None,
        }
    }

    /// Least upper bound of two values.
    pub(crate) fn join(self, other: AbsVal) -> AbsVal {
        match (self, other) {
            (AbsVal::Int(a, b), AbsVal::Int(c, d)) => AbsVal::Int(a.min(c), b.max(d)),
            _ => AbsVal::Top,
        }
    }
}

/// Joins two optional values where `None` stands for "not produced" (an unreachable branch).
pub(crate) fn join_opt(left: Option<AbsVal>, right: Option<AbsVal>) -> Option<AbsVal> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.join(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

/// Facts about the locals of one reachable program point.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Env {
    vars: BTreeMap<String, (i64, i64)>,
}

impl Env {
    /// Returns the abstract value of a local (absent means `Top`).
    pub(crate) fn get(&self, name: &str) -> AbsVal {
        match self.vars.get(name) {
            Some(&(lo, hi)) => AbsVal::Int(lo, hi),
            None => AbsVal::Top,
        }
    }

    /// Records the abstract value of a local.
    pub(crate) fn set(&mut self, name: &str, value: AbsVal) {
        match value {
            AbsVal::Int(lo, hi) => {
                self.vars.insert(name.to_string(), (lo, hi));
            }
            AbsVal::Top => {
                self.vars.remove(name);
            }
        }
    }

    /// Pointwise join: a variable keeps a fact only when both sides know it is an int.
    fn join(&self, other: &Env) -> Env {
        let mut vars = BTreeMap::new();
        for (name, &(lo, hi)) in &self.vars {
            if let Some(&(olo, ohi)) = other.vars.get(name) {
                vars.insert(name.clone(), (lo.min(olo), hi.max(ohi)));
            }
        }
        Env { vars }
    }

    /// Widening with thresholds: every bound that grew since `old` jumps to the next value of
    /// the sorted `thresholds` list (or to the matching `i64` extreme). The list is finite, so
    /// the number of loop iterations the fixpoint needs stays bounded.
    fn widen(old: &Env, new: &Env, thresholds: &[i64]) -> Env {
        let mut vars = BTreeMap::new();
        for (name, &(lo, hi)) in &new.vars {
            if let Some(&(olo, ohi)) = old.vars.get(name) {
                let lo = if lo < olo {
                    thresholds.iter().rev().copied().find(|t| *t <= lo).unwrap_or(MIN)
                } else {
                    lo
                };
                let hi = if hi > ohi {
                    thresholds.iter().copied().find(|t| *t >= hi).unwrap_or(MAX)
                } else {
                    hi
                };
                vars.insert(name.clone(), (lo, hi));
            }
        }
        Env { vars }
    }
}

/// Flow state: `None` is unreachable code.
pub(crate) type State = Option<Env>;

/// Joins two flow states.
pub(crate) fn join_states(left: &State, right: &State) -> State {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.join(right)),
        (Some(env), None) | (None, Some(env)) => Some(env.clone()),
        (None, None) => None,
    }
}

/// Widens a loop-head state with the newly joined state (`thresholds` must be sorted).
pub(crate) fn widen_states(old: &State, new: &State, thresholds: &[i64]) -> State {
    match (old, new) {
        (Some(old), Some(new)) => Some(Env::widen(old, new, thresholds)),
        (None, new) => new.clone(),
        (Some(old), None) => Some(old.clone()),
    }
}

/// Narrows a variable to `[lo, hi]` when it is already a definite int.
///
/// Returns the unreachable state when the intersection is empty. A `Top` variable is left
/// unchanged: a comparison cannot prove that a non-int operand is an int.
pub(crate) fn narrow(state: State, name: &str, lo: i64, hi: i64) -> State {
    let mut env = state?;
    if let AbsVal::Int(cur_lo, cur_hi) = env.get(name) {
        let new_lo = cur_lo.max(lo);
        let new_hi = cur_hi.min(hi);
        if new_lo > new_hi {
            return None;
        }
        env.set(name, AbsVal::Int(new_lo, new_hi));
    }
    Some(env)
}

/// Integer operations with an exact overflow verdict.
pub(crate) mod ops {
    use super::{AbsVal, MAX, MIN};

    /// Interval sum, or `None` when some pair of operands may overflow `i64`.
    pub(crate) fn add(l: (i64, i64), r: (i64, i64)) -> Option<AbsVal> {
        AbsVal::from_i128(l.0 as i128 + r.0 as i128, l.1 as i128 + r.1 as i128)
    }

    /// Interval difference, or `None` when some pair of operands may overflow `i64`.
    pub(crate) fn sub(l: (i64, i64), r: (i64, i64)) -> Option<AbsVal> {
        AbsVal::from_i128(l.0 as i128 - r.1 as i128, l.1 as i128 - r.0 as i128)
    }

    /// Interval product, or `None` when some pair of operands may overflow `i64`.
    pub(crate) fn mul(l: (i64, i64), r: (i64, i64)) -> Option<AbsVal> {
        let corners = [
            l.0 as i128 * r.0 as i128,
            l.0 as i128 * r.1 as i128,
            l.1 as i128 * r.0 as i128,
            l.1 as i128 * r.1 as i128,
        ];
        let lo = *corners.iter().min().expect("four corners");
        let hi = *corners.iter().max().expect("four corners");
        AbsVal::from_i128(lo, hi)
    }

    /// Result range of `%` (always an int, or a thrown `DivisionByZeroError`).
    ///
    /// PHP's remainder has the sign of the dividend and a magnitude strictly below the divisor's.
    pub(crate) fn rem(l: Option<(i64, i64)>, r: Option<(i64, i64)>) -> AbsVal {
        let Some((rlo, rhi)) = r else {
            return AbsVal::FULL_INT;
        };
        let max_abs = (rlo as i128).abs().max((rhi as i128).abs());
        if max_abs == 0 {
            // Always a division by zero: no value is ever produced.
            return AbsVal::FULL_INT;
        }
        let bound = (max_abs - 1).min(MAX as i128) as i64;
        match l {
            Some((lo, hi)) if lo >= 0 => AbsVal::Int(0, hi.min(bound)),
            Some((lo, hi)) if hi <= 0 => AbsVal::Int(lo.max(-bound), 0),
            _ => AbsVal::Int(-bound, bound),
        }
    }

    /// Result range of `&` when at least one operand is a definite int.
    pub(crate) fn bit_and(l: Option<(i64, i64)>, r: Option<(i64, i64)>) -> AbsVal {
        match (l, r) {
            (Some((llo, lhi)), Some((rlo, rhi))) if llo >= 0 && rlo >= 0 => {
                AbsVal::Int(0, lhi.min(rhi))
            }
            (Some((lo, hi)), _) | (_, Some((lo, hi))) if lo >= 0 => AbsVal::Int(0, hi),
            (Some(_), _) | (_, Some(_)) => AbsVal::FULL_INT,
            (None, None) => AbsVal::Top,
        }
    }

    /// Result range of `|` and `^` when at least one operand is a definite int.
    pub(crate) fn bit_or_xor(l: Option<(i64, i64)>, r: Option<(i64, i64)>) -> AbsVal {
        match (l, r) {
            (Some((llo, lhi)), Some((rlo, rhi))) if llo >= 0 && rlo >= 0 => {
                let max = lhi.max(rhi) as u64;
                let bits = 64 - max.leading_zeros();
                let hi = if bits >= 63 { MAX } else { ((1u64 << bits) - 1) as i64 };
                AbsVal::Int(0, hi)
            }
            (Some(_), _) | (_, Some(_)) => AbsVal::FULL_INT,
            (None, None) => AbsVal::Top,
        }
    }

    /// Result range of `<<` (always an int, or a thrown error).
    pub(crate) fn shl(l: Option<(i64, i64)>, r: Option<(i64, i64)>) -> AbsVal {
        if let (Some((lo, hi)), Some((k, k2))) = (l, r) {
            if k == k2 && (0..63).contains(&k) && lo >= 0 && hi <= (MAX >> k) {
                return AbsVal::Int(lo << k, hi << k);
            }
        }
        AbsVal::FULL_INT
    }

    /// Result range of `>>` (always an int, or a thrown error).
    pub(crate) fn shr(l: Option<(i64, i64)>, r: Option<(i64, i64)>) -> AbsVal {
        if let (Some((lo, hi)), Some((k1, k2))) = (l, r) {
            if k1 >= 0 {
                let k1 = k1.min(63) as u32;
                let k2 = k2.min(63) as u32;
                let (new_lo, new_hi) = if lo >= 0 {
                    (lo >> k2, hi >> k1)
                } else if hi < 0 {
                    (lo >> k1, hi >> k2)
                } else {
                    (lo >> k1, hi >> k1)
                };
                return AbsVal::Int(new_lo, new_hi);
            }
        }
        AbsVal::FULL_INT
    }

    /// Result of unary minus: an int unless `PHP_INT_MIN` may be negated.
    pub(crate) fn negate(v: Option<(i64, i64)>) -> AbsVal {
        match v {
            Some((lo, hi)) if lo > MIN => AbsVal::Int(-hi, -lo),
            _ => AbsVal::Top,
        }
    }

    /// Result of `~` on a definite int (`~x == -x - 1`, which never overflows).
    pub(crate) fn bit_not(v: Option<(i64, i64)>) -> AbsVal {
        match v {
            Some((lo, hi)) => AbsVal::Int(!hi, !lo),
            None => AbsVal::Top,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ops::*;
    use super::*;

    #[test]
    fn add_detects_overflow_exactly() {
        assert_eq!(add((0, 10), (1, 1)), Some(AbsVal::Int(1, 11)));
        assert_eq!(add((MAX - 1, MAX - 1), (1, 1)), Some(AbsVal::exact(MAX)));
        assert_eq!(add((MAX, MAX), (1, 1)), None);
        assert_eq!(sub((MIN, MIN), (1, 1)), None);
        assert_eq!(sub((MIN + 1, 0), (1, 1)), Some(AbsVal::Int(MIN, -1)));
    }

    #[test]
    fn mul_uses_all_corners() {
        assert_eq!(mul((-3, 2), (-5, 4)), Some(AbsVal::Int(-12, 15)));
        assert_eq!(mul((0, 1 << 31), (0, 3)), Some(AbsVal::Int(0, 3 << 31)));
        assert_eq!(mul((0, 1 << 32), (0, 1 << 31)), None);
        assert_eq!(mul((MIN, MIN), (-1, -1)), None);
    }

    #[test]
    fn bitwise_ranges() {
        assert_eq!(bit_and(Some((MIN, MAX)), Some((7, 7))), AbsVal::Int(0, 7));
        assert_eq!(bit_and(None, Some((0x3FFF_FFFF, 0x3FFF_FFFF))), AbsVal::Int(0, 0x3FFF_FFFF));
        assert_eq!(bit_and(None, None), AbsVal::Top);
        assert_eq!(bit_or_xor(Some((0, 5)), Some((0, 8))), AbsVal::Int(0, 15));
        assert_eq!(shl(Some((0, 3)), Some((4, 4))), AbsVal::Int(0, 48));
        assert_eq!(shl(Some((0, MAX)), Some((1, 1))), AbsVal::FULL_INT);
        assert_eq!(shr(Some((-16, 32)), Some((2, 2))), AbsVal::Int(-4, 8));
        assert_eq!(rem(Some((0, 100)), Some((7, 7))), AbsVal::Int(0, 6));
        assert_eq!(rem(None, Some((-7, -7))), AbsVal::Int(-6, 6));
        assert_eq!(negate(Some((MIN, 0))), AbsVal::Top);
        assert_eq!(bit_not(Some((0, 5))), AbsVal::Int(-6, -1));
    }

    #[test]
    fn widening_jumps_to_extremes() {
        let mut old = Env::default();
        old.set("i", AbsVal::Int(0, 1));
        let mut new = Env::default();
        new.set("i", AbsVal::Int(0, 2));
        new.set("j", AbsVal::exact(3));
        let widened = widen_states(&Some(old.clone()), &Some(new.clone()), &[]).expect("reachable");
        assert_eq!(widened.get("i"), AbsVal::Int(0, MAX));
        assert_eq!(widened.get("j"), AbsVal::Top);
        let widened = widen_states(&Some(old), &Some(new), &[-5, 10, 100]).expect("reachable");
        assert_eq!(widened.get("i"), AbsVal::Int(0, 10));
    }

    #[test]
    fn narrowing_to_empty_is_unreachable() {
        let mut env = Env::default();
        env.set("i", AbsVal::Int(0, 5));
        assert_eq!(narrow(Some(env.clone()), "i", 10, MAX), None);
        let narrowed = narrow(Some(env), "i", MIN, 3).expect("reachable");
        assert_eq!(narrowed.get("i"), AbsVal::Int(0, 3));
    }
}
