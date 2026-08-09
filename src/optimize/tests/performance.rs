//! Purpose:
//! Architectural performance regressions for whole-program AST effect analysis.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness.
//!
//! Key details:
//! - These source-level invariants complement semantic optimizer tests without flaky timing thresholds.

/// Verifies the optimizer pipeline has a single effect-analysis entry instead
/// of recomputing the same whole-program fixed point independently per pass.
#[test]
fn callable_effect_analysis_is_not_recomputed_per_ast_pass() {
    let source = include_str!("../../optimize.rs");
    let calls = source
        .matches("compute_program_callable_effects(&program)")
        .count();

    assert!(
        calls <= 1,
        "the optimizer must share one whole-program effect analysis, found {calls} pass-local calls"
    );
}

/// Verifies effect analysis borrows callable bodies and updates summaries
/// incrementally instead of deep-cloning every body and complete map snapshot.
#[test]
fn callable_effect_analysis_avoids_deep_ast_and_map_snapshot_clones() {
    let source = include_str!("../../optimize.rs");

    for forbidden in [
        "body: body.clone()",
        "body: method.body.clone()",
        "let function_snapshot = function_effects.clone()",
        "let static_method_snapshot = static_method_effects.clone()",
        "let instance_method_snapshot = instance_method_effects.clone()",
        "Option<*const HashMap",
        "unsafe { ptr.as_ref() }",
    ] {
        assert!(
            !source.contains(forbidden),
            "whole-program effect analysis retains an avoidable clone: {forbidden}"
        );
    }
}
