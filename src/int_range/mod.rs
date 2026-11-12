//! Purpose:
//! Integer-range analysis that proves PHP integer `+`, `-`, `*`, `++` and `--` sites cannot
//! overflow, so they can be compiled as native machine arithmetic instead of the checked
//! "int, or float on overflow" form that boxes every result into a `Mixed` value.
//!
//! Called from:
//! - `crate::pipeline` (installs the facts before type checking and EIR lowering).
//! - `crate::types::checker` and `crate::ir_lower` (query [`is_proven`]).
//!
//! Key details:
//! - The analysis never changes PHP semantics: a site is only proven when every execution of it
//!   with integer operands stays inside the `i64` range, in which case PHP itself produces an
//!   `int` and the checked helper would never take its float path.
//! - Facts are keyed by `(Span, ArithKind)`. Every occurrence of a key in the program is
//!   analyzed and one unsafe occurrence makes the key unsafe, so span collisions (inlined
//!   includes, desugared nodes sharing a span, trait copies) only ever lose precision.
//! - Facts are installed per thread; code paths that never install them (unit tests, tools)
//!   see no proven site and keep the checked arithmetic.

mod analyzer;
mod domain;
mod scan;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::parser::ast::Program;
use crate::span::Span;

/// Kind of arithmetic site recorded by the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithKind {
    /// Binary `+`.
    Add,
    /// Binary `-`.
    Sub,
    /// Binary `*`.
    Mul,
    /// `++$x` / `$x++`.
    Inc,
    /// `--$x` / `$x--`.
    Dec,
}

/// Per-site verdicts: `true` when every analyzed occurrence is overflow-free.
pub(crate) type Facts = HashMap<(Span, ArithKind), bool>;

/// Proven-site table produced by [`analyze_program`].
#[derive(Debug, Default, Clone)]
pub struct OverflowFacts {
    facts: Facts,
}

impl OverflowFacts {
    /// Returns `true` when the site is proven overflow-free.
    pub fn is_proven(&self, span: Span, kind: ArithKind) -> bool {
        span.line != 0 && self.facts.get(&(span, kind)).copied().unwrap_or(false)
    }

    /// Keeps only the sites proven by both tables.
    pub fn intersect(mut self, other: &OverflowFacts) -> OverflowFacts {
        self.facts
            .retain(|key, proven| *proven && other.facts.get(key).copied().unwrap_or(false));
        self
    }
}

/// Analyzes a whole program. `web` disables the `$argc` seed (it is undefined under a web SAPI).
///
/// Parameter ranges of directly-called functions come from their call sites: each round
/// analyzes the program with the ranges assumed so far and joins the arguments observed at
/// reachable calls; the facts of a round are kept once the observations equal the assumption
/// (a fixpoint). Growing bounds are widened after a few rounds, and a run that does not
/// stabilize falls back to unknown parameters.
pub fn analyze_program(program: &Program, web: bool) -> OverflowFacts {
    let info = scan::ProgramInfo::collect(program);
    // Optimistic iteration: a tracked function starts unreachable and gains parameter ranges
    // from the reachable direct calls observed in the previous round. The facts of a round are
    // kept once every observed argument lies inside what that round assumed (a post-fixpoint),
    // which makes the assumption sound for every call the program can perform.
    let mut assumed: CallSiteRanges = HashMap::new();
    for round in 0..MAX_CALL_SITE_ROUNDS {
        let mut facts = Facts::new();
        let mut calls = analyzer::CallSites {
            assumed: assumed.clone(),
            optimistic: true,
            observed: HashMap::new(),
        };
        analyzer::Analyzer::analyze_script(&info, &mut facts, &mut calls, program, web);
        let observed = complete_observations(calls.observed);
        if subsumed(&observed, &assumed) {
            return OverflowFacts { facts };
        }
        let joined = join_call_sites(&assumed, &observed);
        assumed = if round >= WIDEN_CALL_SITES_AFTER {
            widen_call_sites(&assumed, joined)
        } else {
            joined
        };
    }
    // No fixpoint within the round budget: analyze with unknown parameters everywhere.
    let mut facts = Facts::new();
    let mut calls = analyzer::CallSites::default();
    analyzer::Analyzer::analyze_script(&info, &mut facts, &mut calls, program, web);
    OverflowFacts { facts }
}

/// Upper bound on call-site rounds before keeping the round-0 (no assumption) facts.
const MAX_CALL_SITE_ROUNDS: usize = 8;
/// Rounds after which growing parameter bounds jump to the `i64` extremes.
const WIDEN_CALL_SITES_AFTER: usize = 3;

/// Call-site parameter ranges keyed by function.
type CallSiteRanges = HashMap<String, Vec<domain::AbsVal>>;

/// Turns per-position observations into values (an unseen position is unknown).
fn complete_observations(observed: HashMap<String, Vec<Option<domain::AbsVal>>>) -> CallSiteRanges {
    observed
        .into_iter()
        .map(|(name, slots)| {
            let values = slots
                .into_iter()
                .map(|slot| slot.unwrap_or(domain::AbsVal::Top))
                .collect();
            (name, values)
        })
        .collect()
}

/// Returns `true` when every observed argument lies within the assumed parameter range.
fn subsumed(observed: &CallSiteRanges, assumed: &CallSiteRanges) -> bool {
    observed.iter().all(|(name, values)| {
        let Some(assumed_values) = assumed.get(name) else {
            return false;
        };
        values.iter().enumerate().all(|(index, value)| {
            match (assumed_values.get(index), value) {
                (Some(domain::AbsVal::Top), _) => true,
                (Some(domain::AbsVal::Int(alo, ahi)), domain::AbsVal::Int(lo, hi)) => alo <= lo && hi <= ahi,
                _ => false,
            }
        })
    })
}

/// Pointwise join of two call-site range tables.
fn join_call_sites(left: &CallSiteRanges, right: &CallSiteRanges) -> CallSiteRanges {
    let mut out = left.clone();
    for (name, values) in right {
        let entry = out.entry(name.clone()).or_insert_with(|| values.clone());
        for (index, value) in values.iter().enumerate() {
            if index < entry.len() {
                entry[index] = entry[index].join(*value);
            } else {
                entry.push(*value);
            }
        }
    }
    out
}

/// Widens parameter ranges: a bound that grew since the previous round becomes unbounded.
fn widen_call_sites(previous: &CallSiteRanges, joined: CallSiteRanges) -> CallSiteRanges {
    joined
        .into_iter()
        .map(|(name, values)| {
            let old = previous.get(&name);
            let widened = values
                .into_iter()
                .enumerate()
                .map(|(index, value)| match (old.and_then(|o| o.get(index)), value) {
                    (Some(domain::AbsVal::Int(olo, ohi)), domain::AbsVal::Int(lo, hi)) => {
                        let lo = if lo < *olo { domain::MIN } else { lo };
                        let hi = if hi > *ohi { domain::MAX } else { hi };
                        domain::AbsVal::Int(lo, hi)
                    }
                    (_, value) => value,
                })
                .collect();
            (name, widened)
        })
        .collect()
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<OverflowFacts>>> = const { RefCell::new(None) };
}

/// Installs the facts consulted by [`is_proven`] on the current thread.
pub fn install(facts: OverflowFacts) {
    ACTIVE.with(|active| *active.borrow_mut() = Some(Rc::new(facts)));
}

/// Returns `true` when the installed facts prove the site overflow-free.
pub fn is_proven(span: Span, kind: ArithKind) -> bool {
    ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|facts| facts.is_proven(span, kind))
    })
}

/// Returns `true` when the installed facts prove a binary `+`, `-` or `*` site overflow-free.
pub fn is_proven_binop(op: &crate::parser::ast::BinOp, span: Span) -> bool {
    use crate::parser::ast::BinOp;
    let kind = match op {
        BinOp::Add => ArithKind::Add,
        BinOp::Sub => ArithKind::Sub,
        BinOp::Mul => ArithKind::Mul,
        _ => return false,
    };
    is_proven(span, kind)
}

/// Returns `true` when the installed facts prove a `++`/`--` expression overflow-free.
pub fn is_proven_step(expr: &crate::parser::ast::Expr) -> bool {
    use crate::parser::ast::ExprKind;
    let kind = match &expr.kind {
        ExprKind::PreIncrement(_) | ExprKind::PostIncrement(_) => ArithKind::Inc,
        ExprKind::PreDecrement(_) | ExprKind::PostDecrement(_) => ArithKind::Dec,
        _ => return false,
    };
    is_proven(expr.span, kind)
}

#[cfg(test)]
mod tests;
