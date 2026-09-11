//! Purpose:
//! Regression tests for optimizer normalize loop-shell behavior over parser AST fixtures.
//! Documents the v2 canonicalizations: `for` without update to `while`, leading break guards
//! folded into the loop test, trailing `continue` removal, and unconditional-loop rotation.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness
//!
//! Key details:
//! - Fixtures are intentionally small and structural; expected AST equality captures observable optimizer semantics.

use super::*;

/// Builds a `true` literal for loop tests.
fn bool_true() -> Expr {
    Expr::new(ExprKind::BoolLiteral(true), Span::dummy())
}

/// Builds `!expr`.
fn not(expr: Expr) -> Expr {
    Expr::new(ExprKind::Not(Box::new(expr)), Span::dummy())
}

/// Builds `if (condition) { break; }` with an optional `else` body.
fn break_guard(condition: Expr, else_body: Option<Vec<Stmt>>) -> Stmt {
    Stmt::new(
        StmtKind::If {
            condition,
            then_body: vec![Stmt::new(StmtKind::Break(1), Span::dummy())],
            elseif_clauses: Vec::new(),
            else_body,
        },
        Span::dummy(),
    )
}

/// `for ($i = 0; $i < $n;) { echo 1; }` hoists the init clause and becomes a `while` loop,
/// because `continue` reaches the test directly in both forms.
#[test]
fn test_normalize_control_flow_rewrites_for_without_update_to_while() {
    let condition = Expr::binop(Expr::var("i"), BinOp::Lt, Expr::var("n"));
    let program = vec![Stmt::new(
        StmtKind::For {
            init: Some(Box::new(Stmt::assign("i", Expr::int_lit(0)))),
            condition: Some(condition.clone()),
            update: None,
            body: vec![Stmt::echo(Expr::int_lit(1))],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![
            Stmt::assign("i", Expr::int_lit(0)),
            Stmt::new(
                StmtKind::While {
                    condition,
                    body: vec![Stmt::echo(Expr::int_lit(1))],
                },
                Span::dummy(),
            ),
        ]
    );
}

/// `for (;;) { if ($x) { break; } echo 1; }` becomes `while (!$x) { echo 1; }`: the missing
/// test is `true`, and the leading break guard supplies the real loop test.
#[test]
fn test_normalize_control_flow_folds_leading_break_guard_of_endless_for_into_while_test() {
    let program = vec![Stmt::new(
        StmtKind::For {
            init: None,
            condition: None,
            update: None,
            body: vec![break_guard(Expr::var("x"), None), Stmt::echo(Expr::int_lit(1))],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: not(Expr::var("x")),
                body: vec![Stmt::echo(Expr::int_lit(1))],
            },
            Span::dummy(),
        )]
    );
}

/// `while ($a) { if ($b) { break; } else { echo 1; } echo 2; }` becomes
/// `while ($a && !$b) { echo 1; echo 2; }`: the guard's `else` ran whenever the loop stayed,
/// so it leads the remaining body.
#[test]
fn test_normalize_control_flow_folds_leading_break_guard_with_else_into_while_test() {
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: Expr::var("a"),
            body: vec![
                break_guard(Expr::var("b"), Some(vec![Stmt::echo(Expr::int_lit(1))])),
                Stmt::echo(Expr::int_lit(2)),
            ],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: Expr::binop(Expr::var("a"), BinOp::And, not(Expr::var("b"))),
                body: vec![Stmt::echo(Expr::int_lit(1)), Stmt::echo(Expr::int_lit(2))],
            },
            Span::dummy(),
        )]
    );
}

/// `for ($i = 0; $i < 3; $i++) { if ($x) { break; } echo 1; }` keeps its `for` shape because of
/// the update clause, but the guard still folds into the test: `$i < 3 && !$x`.
#[test]
fn test_normalize_control_flow_folds_leading_break_guard_into_for_test() {
    let init = Some(Box::new(Stmt::assign("i", Expr::int_lit(0))));
    let update = Some(Box::new(Stmt::new(
        StmtKind::ExprStmt(Expr::new(
            ExprKind::PostIncrement("i".into()),
            Span::dummy(),
        )),
        Span::dummy(),
    )));
    let test = Expr::binop(Expr::var("i"), BinOp::Lt, Expr::int_lit(3));
    let program = vec![Stmt::new(
        StmtKind::For {
            init: init.clone(),
            condition: Some(test.clone()),
            update: update.clone(),
            body: vec![break_guard(Expr::var("x"), None), Stmt::echo(Expr::int_lit(1))],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::For {
                init,
                condition: Some(Expr::binop(test, BinOp::And, not(Expr::var("x")))),
                update,
                body: vec![Stmt::echo(Expr::int_lit(1))],
            },
            Span::dummy(),
        )]
    );
}

/// `while (true) { echo 1; if ($x) { break; } }` rotates into `do { echo 1; } while (!$x)`.
#[test]
fn test_normalize_control_flow_rotates_endless_loop_with_trailing_break_guard_into_do_while() {
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: bool_true(),
            body: vec![Stmt::echo(Expr::int_lit(1)), break_guard(Expr::var("x"), None)],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::DoWhile {
                body: vec![Stmt::echo(Expr::int_lit(1))],
                condition: not(Expr::var("x")),
            },
            Span::dummy(),
        )]
    );
}

/// A `continue` that targets the loop skips the trailing guard, so the loop is not rotated:
/// `while (true) { if ($y) { continue; } echo 1; if ($x) { break; } }` keeps its shape.
#[test]
fn test_normalize_control_flow_keeps_endless_loop_whose_body_continues_before_trailing_guard() {
    let body = vec![
        Stmt::new(
            StmtKind::If {
                condition: Expr::var("y"),
                then_body: vec![Stmt::new(StmtKind::Continue(1), Span::dummy())],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            Span::dummy(),
        ),
        Stmt::echo(Expr::int_lit(1)),
        break_guard(Expr::var("x"), None),
    ];
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: bool_true(),
            body: body.clone(),
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: bool_true(),
                body,
            },
            Span::dummy(),
        )]
    );
}

/// A `continue 2` inside a nested loop targets the outer loop, so the outer loop is not
/// rotated either; a `continue` inside the nested loop that targets only the inner loop is fine.
#[test]
fn test_normalize_control_flow_counts_continue_depth_through_nested_loops_before_rotating() {
    let inner_continue_outer = Stmt::new(
        StmtKind::Foreach {
            array: Expr::var("xs"),
            key_var: None,
            value_var: "v".into(),
            value_by_ref: false,
            body: vec![Stmt::new(StmtKind::Continue(2), Span::dummy())],
        },
        Span::dummy(),
    );
    let blocked = vec![Stmt::new(
        StmtKind::While {
            condition: bool_true(),
            body: vec![inner_continue_outer.clone(), break_guard(Expr::var("x"), None)],
        },
        Span::dummy(),
    )];
    assert_eq!(
        normalize_control_flow(blocked),
        vec![Stmt::new(
            StmtKind::While {
                condition: bool_true(),
                body: vec![inner_continue_outer, break_guard(Expr::var("x"), None)],
            },
            Span::dummy(),
        )]
    );

    let inner_continue_inner = Stmt::new(
        StmtKind::Foreach {
            array: Expr::var("xs"),
            key_var: None,
            value_var: "v".into(),
            value_by_ref: false,
            body: vec![
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::var("v"),
                        then_body: vec![Stmt::new(StmtKind::Continue(1), Span::dummy())],
                        elseif_clauses: Vec::new(),
                        else_body: None,
                    },
                    Span::dummy(),
                ),
                Stmt::echo(Expr::var("v")),
            ],
        },
        Span::dummy(),
    );
    let allowed = vec![Stmt::new(
        StmtKind::While {
            condition: bool_true(),
            body: vec![inner_continue_inner.clone(), break_guard(Expr::var("x"), None)],
        },
        Span::dummy(),
    )];
    assert_eq!(
        normalize_control_flow(allowed),
        vec![Stmt::new(
            StmtKind::DoWhile {
                body: vec![inner_continue_inner],
                condition: not(Expr::var("x")),
            },
            Span::dummy(),
        )]
    );
}

/// `do { echo 1; } while (true)` becomes `while (true) { echo 1; }`.
#[test]
fn test_normalize_control_flow_rewrites_do_while_true_to_while_true() {
    let program = vec![Stmt::new(
        StmtKind::DoWhile {
            body: vec![Stmt::echo(Expr::int_lit(1))],
            condition: bool_true(),
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: bool_true(),
                body: vec![Stmt::echo(Expr::int_lit(1))],
            },
            Span::dummy(),
        )]
    );
}

/// `while ($a) { echo 1; if ($b) { echo 2; continue; } }` drops the trailing `continue`, and the
/// emptied-nothing `if` shell keeps its remaining body: `while ($a) { echo 1; if ($b) { echo 2; } }`.
#[test]
fn test_normalize_control_flow_strips_trailing_continue_through_if_branches() {
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: Expr::var("a"),
            body: vec![
                Stmt::echo(Expr::int_lit(1)),
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::var("b"),
                        then_body: vec![
                            Stmt::echo(Expr::int_lit(2)),
                            Stmt::new(StmtKind::Continue(1), Span::dummy()),
                        ],
                        elseif_clauses: Vec::new(),
                        else_body: None,
                    },
                    Span::dummy(),
                ),
            ],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: Expr::var("a"),
                body: vec![
                    Stmt::echo(Expr::int_lit(1)),
                    Stmt::new(
                        StmtKind::If {
                            condition: Expr::var("b"),
                            then_body: vec![Stmt::echo(Expr::int_lit(2))],
                            elseif_clauses: Vec::new(),
                            else_body: None,
                        },
                        Span::dummy(),
                    ),
                ],
            },
            Span::dummy(),
        )]
    );
}

/// A trailing `if ($b) { continue; }` whose only job was the `continue` collapses into its
/// condition when that condition is observable, and disappears entirely when it is pure:
/// `while ($a) { echo 1; if ($b) { continue; } }` becomes `while ($a) { echo 1; }`.
#[test]
fn test_normalize_control_flow_drops_trailing_continue_only_guard() {
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: Expr::var("a"),
            body: vec![
                Stmt::echo(Expr::int_lit(1)),
                Stmt::new(
                    StmtKind::If {
                        condition: Expr::var("b"),
                        then_body: vec![Stmt::new(StmtKind::Continue(1), Span::dummy())],
                        elseif_clauses: Vec::new(),
                        else_body: None,
                    },
                    Span::dummy(),
                ),
            ],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: Expr::var("a"),
                body: vec![Stmt::echo(Expr::int_lit(1))],
            },
            Span::dummy(),
        )]
    );
}

/// A `continue` that is not on the tail path is never touched:
/// `while ($a) { if ($b) { continue; } echo 1; }` keeps the guard.
#[test]
fn test_normalize_control_flow_keeps_non_trailing_continue() {
    let body = vec![
        Stmt::new(
            StmtKind::If {
                condition: Expr::var("b"),
                then_body: vec![Stmt::new(StmtKind::Continue(1), Span::dummy())],
                elseif_clauses: Vec::new(),
                else_body: None,
            },
            Span::dummy(),
        ),
        Stmt::echo(Expr::int_lit(1)),
    ];
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: Expr::var("a"),
            body: body.clone(),
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: Expr::var("a"),
                body,
            },
            Span::dummy(),
        )]
    );
}

/// A trailing `continue` inside a nested `switch` targets the `switch`, not the loop, so the
/// walk never descends into it: `while ($a) { switch ($x) { case 1: continue; } }` is kept.
#[test]
fn test_normalize_control_flow_keeps_trailing_continue_inside_switch() {
    let switch = Stmt::new(
        StmtKind::Switch {
            subject: Expr::var("x"),
            cases: vec![
                (
                    vec![Expr::int_lit(1)],
                    vec![
                        Stmt::echo(Expr::int_lit(1)),
                        Stmt::new(StmtKind::Continue(1), Span::dummy()),
                    ],
                ),
                (vec![Expr::int_lit(2)], vec![Stmt::echo(Expr::int_lit(2))]),
            ],
            default: None,
        },
        Span::dummy(),
    );
    let program = vec![Stmt::new(
        StmtKind::While {
            condition: Expr::var("a"),
            body: vec![switch.clone()],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::While {
                condition: Expr::var("a"),
                body: vec![switch],
            },
            Span::dummy(),
        )]
    );
}

/// `foreach ($xs as $v) { echo $v; continue; }` drops the trailing `continue` like any other
/// loop body.
#[test]
fn test_normalize_control_flow_strips_trailing_continue_from_foreach_body() {
    let program = vec![Stmt::new(
        StmtKind::Foreach {
            array: Expr::var("xs"),
            key_var: None,
            value_var: "v".into(),
            value_by_ref: false,
            body: vec![
                Stmt::echo(Expr::var("v")),
                Stmt::new(StmtKind::Continue(1), Span::dummy()),
            ],
        },
        Span::dummy(),
    )];

    let normalized = normalize_control_flow(program);

    assert_eq!(
        normalized,
        vec![Stmt::new(
            StmtKind::Foreach {
                array: Expr::var("xs"),
                key_var: None,
                value_var: "v".into(),
                value_by_ref: false,
                body: vec![Stmt::echo(Expr::var("v"))],
            },
            Span::dummy(),
        )]
    );
}
