//! Purpose:
//! Integration-style unit fixtures for optimizer passes over hand-built ASTs.
//! Provides shared imports and submodules for fold, propagate, prune, DCE, effects, and normalization tests.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness
//!
//! Key details:
//! - Tests assert AST rewrites directly, so spans and statement ordering are part of the expected behavior.

use super::*;
use crate::names::Name;
use crate::parser::ast::{ClassProperty, StaticReceiver, Visibility};
use crate::span::Span;

mod effects;
mod propagate;
mod fold;
mod prune;
mod dce;
mod control;
mod normalize;
mod performance;

/// Runs DCE over a hand-built AST that no checker ever saw.
///
/// Shadows [`super::eliminate_dead_code`] for the test tree only. These fixtures build their ASTs
/// by hand, so there are no `CheckResult` local-binding decisions to keep singular and the empty
/// set is the honest argument — while the real callers keep having to pass
/// `CheckResult::local_binding_decision_spans()` explicitly, which is what stops a production
/// caller from quietly losing the tail-sinking guard.
fn eliminate_dead_code(program: Program) -> Program {
    super::eliminate_dead_code(program, HashSet::new())
}
