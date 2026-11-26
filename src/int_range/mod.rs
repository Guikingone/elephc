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
    /// User functions (symbol keys) the program can only enter through direct calls.
    direct_only: std::collections::HashSet<String>,
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
        self.direct_only.retain(|name| other.direct_only.contains(name));
        self
    }
}

/// Analyzes a whole program. `web` disables the `$argc` seed (it is undefined under a web SAPI).
///
/// Functions entered only through direct calls (`scan::ProgramInfo::seedable`) are analyzed in
/// rounds: round 0 assumes the full `int` range for every parameter, and each later round seeds
/// a parameter with the join of the argument ranges the previous round observed. Every round is
/// sound by induction (its seeds over-approximate every argument the previous round's sound
/// analysis could see), so the facts of the last round are kept.
pub fn analyze_program(program: &Program, web: bool) -> OverflowFacts {
    let mut info = scan::ProgramInfo::collect(program);
    let mut facts = Facts::new();
    analyzer::Analyzer::analyze_script(&info, &mut facts, program, web);
    for _ in 0..SEED_ROUNDS {
        if info.seedable.is_empty() {
            break;
        }
        let seeds = next_param_seeds(&info);
        if seeds == info.param_seeds {
            break;
        }
        info.param_seeds = seeds;
        info.observed_args.borrow_mut().clear();
        info.unmapped_calls.borrow_mut().clear();
        facts = Facts::new();
        analyzer::Analyzer::analyze_script(&info, &mut facts, program, web);
    }
    OverflowFacts { facts, direct_only: info.seedable.clone() }
}

/// Maximum number of seeding rounds after the first analysis (one per call-chain level).
const SEED_ROUNDS: usize = 6;

/// Turns the observed call-site ranges of the last round into parameter seeds.
fn next_param_seeds(info: &scan::ProgramInfo) -> HashMap<String, Vec<domain::AbsVal>> {
    let unmapped = info.unmapped_calls.borrow();
    info.observed_args
        .borrow()
        .iter()
        .filter(|(name, _)| !unmapped.contains(*name))
        .map(|(name, slots)| {
            let seeds = slots.iter().map(|slot| slot.unwrap_or(domain::AbsVal::Top)).collect();
            (name.clone(), seeds)
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

/// Returns `true` when the installed facts show that the user function `name` is only ever
/// entered through direct calls (no runtime-name dispatch can reach it), so a signature derived
/// from its call sites binds every call.
pub fn is_direct_only_function(name: &str) -> bool {
    let key = scan::symbol_key(name);
    ACTIVE.with(|active| {
        active.borrow().as_ref().is_some_and(|facts| facts.direct_only.contains(&key))
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

/// Returns the names of every direct function call in `body`, nested bodies included.
pub(crate) fn called_function_names(body: &[crate::parser::ast::Stmt]) -> Vec<crate::names::Name> {
    let mut names = Vec::new();
    scan::walk_stmts(body, true, &mut |node| {
        if let scan::Node::Expr(expr) = node {
            if let crate::parser::ast::ExprKind::FunctionCall { name, .. } = &expr.kind {
                names.push(name.clone());
            }
        }
    });
    names
}

#[cfg(test)]
mod tests;
