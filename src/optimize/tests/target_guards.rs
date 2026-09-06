//! Purpose:
//! Tests pre-check propagation of target-availability guard names and its safety boundaries.
//!
//! Called from:
//! - `cargo test -p elephc --lib target_guard` through Rust's test harness.
//!
//! Key details:
//! - Parser fixtures exercise the same target fold for all five supported output targets.

use super::*;

/// Parses and resolves a fixture before running only the pre-check target-folding pass.
fn fold_source(source: &str, target: &str) -> Program {
    let tokens = crate::lexer::tokenize(source).unwrap();
    let program = crate::parser::parse(&tokens).unwrap();
    let program = crate::name_resolver::resolve(program).unwrap();
    fold_constants_for_target(program, Target::parse(target).unwrap())
}

/// Resolves variable guards against the output target, independently of the host machine.
#[test]
fn test_target_guard_variable_uses_selected_target() {
    for (target, expected) in [
        ("linux-x86_64", "yes"),
        ("linux-aarch64", "yes"),
        ("macos-aarch64", "no"),
        ("ios-arm64", "no"),
        ("ios-sim-arm64", "no"),
    ] {
        let program = fold_source("<?php $name = 'pcntl_getcpu';
            if (function_exists($name)) { echo 'yes'; } else { echo 'no'; }", target);
        assert!(matches!(&program.last().unwrap().kind,
            StmtKind::Echo(Expr { kind: ExprKind::StringLiteral(value), .. }) if value == expected),
            "{target}: {program:?}");
    }
}

/// Keeps ordinary reads intact while resolving nested guards from namespace constants and aliases.
#[test]
fn test_target_guard_string_facts_preserve_checker_shape() {
    let program = fold_source("<?php namespace Demo;
        const PREFIX = 'pcntl_'; $name = PREFIX . 'getqos_class'; echo $name;
        if ($argc > 0) {
            if (!function_exists($name)) { echo 'fallback'; }
        }", "linux-x86_64");
    assert!(matches!(&program[2].kind, StmtKind::Echo(Expr { kind: ExprKind::Variable(_), .. })));
    let StmtKind::If { then_body, .. } = &program[3].kind else { panic!("{program:?}") };
    assert!(matches!(&then_body[0].kind,
        StmtKind::Echo(Expr { kind: ExprKind::StringLiteral(value), .. }) if value == "fallback"));
}

/// Never substitutes a stale value across calls, branches, references, or condition-side writes.
#[test]
fn test_target_guard_invalidates_unknown_writes_and_aliases() {
    for source in [
        "<?php $name = 'pcntl_getqos_class'; change($name);
            if (function_exists($name)) { echo 'yes'; }",
        "<?php function change() { global $name; $name = 'strtoupper'; }
            $name = 'pcntl_getqos_class'; change();
            if (function_exists($name)) { echo 'yes'; }",
        "<?php $name = 'pcntl_getqos_class'; if ($argc > 1) { $name = 'strtoupper'; }
            if (function_exists($name)) { echo 'yes'; }",
        "<?php $alias = &$name; $name = 'pcntl_getqos_class'; $alias = 'strtoupper';
            if (function_exists($name)) { echo 'yes'; }",
        "<?php $name = 'pcntl_getqos_class';
            if (($name = 'strtoupper') && function_exists($name)) { echo 'yes'; }",
        "<?php $name = 'pcntl_getqos_class'; $callback = function () use (&$name) {};
            $name = 'pcntl_getqos_class'; $callback();
            if (function_exists($name)) { echo 'yes'; }",
        "<?php $name = 'pcntl_getqos_class'; unset($name);
            if (function_exists($name)) { echo 'yes'; }",
    ] {
        let program = fold_source(source, "linux-x86_64");
        let StmtKind::If { condition, .. } = &program.last().unwrap().kind
            else { panic!("Guard was incorrectly removed: {source}: {program:?}") };
        assert!(!matches!(condition.kind, ExprKind::BoolLiteral(_)), "{source}");
    }
}

/// Excludes reference parameters inside a callable without poisoning same-named outer locals.
#[test]
fn test_target_guard_callable_reference_scope() {
    let program = fold_source("<?php
        function probe(&$name, &$alias) {
            $name = 'pcntl_getqos_class'; $alias = 'strtoupper';
            if (function_exists($name)) { echo 'yes'; }
        }
        $name = 'pcntl_getqos_class';
        if (function_exists($name)) { echo 'yes'; } else { echo 'no'; }", "linux-x86_64");
    let StmtKind::FunctionDecl { body, .. } = &program[0].kind else { panic!("{program:?}") };
    assert!(matches!(&body.last().unwrap().kind, StmtKind::If { .. }));
    assert!(matches!(&program.last().unwrap().kind,
        StmtKind::Echo(Expr { kind: ExprKind::StringLiteral(value), .. }) if value == "no"));
}
