//! Purpose:
//! Regression tests pinning constant `switch` selection to PHP's loose (`==`) case comparison.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness.
//!
//! Key details:
//! - Every expected branch was confirmed by running the equivalent `switch` under `php -r` on
//!   PHP 8.4.20; the fixtures use `echo <int>` bodies so the selected branch is unambiguous.
//! - PHP 8 changed the number-versus-non-numeric-string rule, so `switch (0) { case "foo": }`
//!   must *not* match — the pass has to keep that.

use super::*;

/// Builds `switch (subject) { case pattern: echo 1; break; default: echo 2; }` and returns the
/// statements the pruner produced.
fn prune_switch(subject: Expr, pattern: Expr) -> Vec<Stmt> {
    prune_constant_control_flow(vec![Stmt::new(
        StmtKind::Switch {
            subject,
            cases: vec![(
                vec![pattern],
                vec![
                    Stmt::echo(Expr::int_lit(1)),
                    Stmt::new(StmtKind::Break(1), Span::dummy()),
                ],
            )],
            default: Some(vec![Stmt::echo(Expr::int_lit(2))]),
        },
        Span::dummy(),
    )])
}

/// Builds the literal named by the switch fixture table.
fn switch_operand(name: &str) -> Expr {
    match name {
        "null" => Expr::new(ExprKind::Null, Span::dummy()),
        "false" => Expr::new(ExprKind::BoolLiteral(false), Span::dummy()),
        "true" => Expr::new(ExprKind::BoolLiteral(true), Span::dummy()),
        "i0" => Expr::int_lit(0),
        "i1" => Expr::int_lit(1),
        "i2" => Expr::int_lit(2),
        "im1" => Expr::int_lit(-1),
        "f0" => Expr::float_lit(0.0),
        "f1_5" => Expr::float_lit(1.5),
        "f2" => Expr::float_lit(2.0),
        "s_empty" => Expr::string_lit(""),
        "s_1" => Expr::string_lit("1"),
        "s_1_5" => Expr::string_lit("1.5"),
        "s_a" => Expr::string_lit("a"),
        "s_abc" => Expr::string_lit("abc"),
        "s_ABC" => Expr::string_lit("ABC"),
        "s_foo" => Expr::string_lit("foo"),
        other => panic!("unknown switch operand {other}"),
    }
}

/// PHP 8.4 `switch (subject) { case pattern: ... }` outcomes: `true` means the case is selected.
#[rustfmt::skip]
const PHP_SWITCH_CASES: &[(&str, &str, bool)] = &[
    // B6: `switch (2) { case true: }` matched `true` against the integer `1`; PHP compares
    // with `==`, and `2 == true` is `(bool) 2`.
    ("i2", "true", true),
    ("im1", "true", true),
    ("s_a", "true", true),
    ("i0", "false", true),
    ("f0", "false", true),
    ("i0", "null", true),
    ("null", "false", true),
    ("null", "i0", true),
    ("null", "s_empty", true),
    ("s_empty", "null", true),
    ("s_1", "i1", true),
    ("f1_5", "s_1_5", true),
    ("f2", "i2", true),
    // PHP 8 string/number rules.
    ("i1", "s_abc", false),
    ("s_foo", "i0", false),
    ("i0", "s_foo", false),
    ("s_abc", "s_ABC", false),
    ("i0", "true", false),
    ("i2", "false", false),
];

/// Verifies constant `switch` selection matches PHP's `==` case comparison.
#[test]
fn test_prune_switch_case_uses_php_loose_equality() {
    for &(subject, pattern, selected) in PHP_SWITCH_CASES {
        let pruned = prune_switch(switch_operand(subject), switch_operand(pattern));
        let expected = if selected { 1 } else { 2 };
        assert_eq!(
            pruned,
            vec![Stmt::echo(Expr::int_lit(expected))],
            "switch ({subject}) {{ case {pattern}: }}"
        );
    }
}
