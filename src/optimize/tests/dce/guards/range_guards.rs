//! Purpose:
//! Regression tests for integer range guard reasoning in AST DCE.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness
//!
//! Key details:
//! - Covers transitive relational bounds, strict-int contradictions outside a
//!   range, switch case pruning, overflow refusal, and write invalidation.

use super::*;

/// Verifies a taken-true `$x > 10` range proves nested `$x > 5` and prunes the dead else.
#[test]
fn test_eliminate_dead_code_prunes_nested_if_from_transitive_range_guard() {
    let program = vec![Stmt::new(
        StmtKind::FunctionDecl {
            name: "main".into(),
            params: Vec::new(),
            param_attributes: Vec::new(),
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: None,
            by_ref_return: false,
            body: vec![Stmt::new(
                StmtKind::If {
                    condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(10)),
                    then_body: vec![Stmt::new(
                        StmtKind::If {
                            condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(5)),
                            then_body: vec![Stmt::echo(Expr::int_lit(7))],
                            elseif_clauses: Vec::new(),
                            else_body: Some(vec![Stmt::echo(Expr::int_lit(8))]),
                        },
                        Span::dummy(),
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: Some(vec![Stmt::echo(Expr::int_lit(9))]),
                },
                Span::dummy(),
            )],
        },
        Span::dummy(),
    )];

    let eliminated = eliminate_dead_code(program);

    let StmtKind::FunctionDecl { body, .. } = &eliminated[0].kind else {
        panic!("expected function");
    };
    let StmtKind::If { then_body, .. } = &body[0].kind else {
        panic!("expected if");
    };
    assert_eq!(then_body, &vec![Stmt::echo(Expr::int_lit(7))]);
}

/// Verifies intersecting `$x >= 0` and `$x <= 0` proves `$x === 1` false.
#[test]
fn test_eliminate_dead_code_prunes_strict_int_outside_intersected_range() {
    let program = vec![Stmt::new(
        StmtKind::FunctionDecl {
            name: "main".into(),
            params: Vec::new(),
            param_attributes: Vec::new(),
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: None,
            by_ref_return: false,
            body: vec![Stmt::new(
                StmtKind::If {
                    condition: Expr::binop(Expr::var("x"), BinOp::GtEq, Expr::int_lit(0)),
                    then_body: vec![Stmt::new(
                        StmtKind::If {
                            condition: Expr::binop(Expr::var("x"), BinOp::LtEq, Expr::int_lit(0)),
                            then_body: vec![Stmt::new(
                                StmtKind::If {
                                    condition: Expr::binop(
                                        Expr::var("x"),
                                        BinOp::StrictEq,
                                        Expr::int_lit(1),
                                    ),
                                    then_body: vec![Stmt::echo(Expr::int_lit(8))],
                                    elseif_clauses: Vec::new(),
                                    else_body: Some(vec![Stmt::echo(Expr::int_lit(7))]),
                                },
                                Span::dummy(),
                            )],
                            elseif_clauses: Vec::new(),
                            else_body: Some(vec![Stmt::echo(Expr::int_lit(9))]),
                        },
                        Span::dummy(),
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: Some(vec![Stmt::echo(Expr::int_lit(10))]),
                },
                Span::dummy(),
            )],
        },
        Span::dummy(),
    )];

    let eliminated = eliminate_dead_code(program);

    let StmtKind::FunctionDecl { body, .. } = &eliminated[0].kind else {
        panic!("expected function");
    };
    let StmtKind::If { then_body, .. } = &body[0].kind else {
        panic!("expected outer if");
    };
    let StmtKind::If { then_body, .. } = &then_body[0].kind else {
        panic!("expected middle if");
    };
    assert_eq!(then_body, &vec![Stmt::echo(Expr::int_lit(7))]);
}

/// Verifies an outer `$x > 5` range drops impossible `case 0:` labels.
#[test]
fn test_eliminate_dead_code_drops_switch_int_cases_outside_range() {
    let program = vec![Stmt::new(
        StmtKind::FunctionDecl {
            name: "main".into(),
            params: Vec::new(),
            param_attributes: Vec::new(),
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: None,
            by_ref_return: false,
            body: vec![Stmt::new(
                StmtKind::If {
                    condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(5)),
                    then_body: vec![Stmt::new(
                        StmtKind::Switch {
                            subject: Expr::var("x"),
                            cases: vec![
                                (vec![Expr::int_lit(0)], vec![Stmt::echo(Expr::int_lit(7))]),
                                (vec![Expr::int_lit(6)], vec![Stmt::echo(Expr::int_lit(8))]),
                            ],
                            default: Some(vec![Stmt::echo(Expr::int_lit(9))]),
                        },
                        Span::dummy(),
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                Span::dummy(),
            )],
        },
        Span::dummy(),
    )];

    let eliminated = eliminate_dead_code(program);

    let StmtKind::FunctionDecl { body, .. } = &eliminated[0].kind else {
        panic!("expected function");
    };
    let StmtKind::If { then_body, .. } = &body[0].kind else {
        panic!("expected if");
    };
    let StmtKind::Switch { cases, default, .. } = &then_body[0].kind else {
        panic!("expected switch");
    };
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].0, vec![Expr::int_lit(6)]);
    assert_eq!(cases[0].1, vec![Stmt::echo(Expr::int_lit(8))]);
    assert_eq!(default, &Some(vec![Stmt::echo(Expr::int_lit(9))]));
}

/// Verifies `$x > i64::MAX` does not wrap into a bogus lower bound that prunes live code.
#[test]
fn test_eliminate_dead_code_refuses_overflowing_range_bound() {
    let program = vec![Stmt::new(
        StmtKind::FunctionDecl {
            name: "main".into(),
            params: Vec::new(),
            param_attributes: Vec::new(),
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: None,
            by_ref_return: false,
            body: vec![Stmt::new(
                StmtKind::If {
                    condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(i64::MAX)),
                    then_body: vec![Stmt::new(
                        StmtKind::If {
                            condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(5)),
                            then_body: vec![Stmt::echo(Expr::int_lit(7))],
                            elseif_clauses: Vec::new(),
                            else_body: Some(vec![Stmt::echo(Expr::int_lit(8))]),
                        },
                        Span::dummy(),
                    )],
                    elseif_clauses: Vec::new(),
                    else_body: Some(vec![Stmt::echo(Expr::int_lit(9))]),
                },
                Span::dummy(),
            )],
        },
        Span::dummy(),
    )];

    let eliminated = eliminate_dead_code(program);

    let StmtKind::FunctionDecl { body, .. } = &eliminated[0].kind else {
        panic!("expected function");
    };
    let StmtKind::If { then_body, .. } = &body[0].kind else {
        panic!("expected if");
    };
    // Without a recorded range, the nested if must survive intact.
    let StmtKind::If {
        then_body: nested_then,
        else_body: nested_else,
        ..
    } = &then_body[0].kind
    else {
        panic!("expected nested if to remain");
    };
    assert_eq!(nested_then, &vec![Stmt::echo(Expr::int_lit(7))]);
    assert_eq!(nested_else, &Some(vec![Stmt::echo(Expr::int_lit(8))]));
}

/// Verifies assigning to the guarded variable clears its range fact.
#[test]
fn test_eliminate_dead_code_invalidates_range_guard_on_write() {
    let program = vec![Stmt::new(
        StmtKind::FunctionDecl {
            name: "main".into(),
            params: Vec::new(),
            param_attributes: Vec::new(),
            variadic: None,
            variadic_by_ref: false,
            variadic_type: None,
            return_type: None,
            by_ref_return: false,
            body: vec![Stmt::new(
                StmtKind::If {
                    condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(10)),
                    then_body: vec![
                        Stmt::new(
                            StmtKind::Assign {
                                name: "x".into(),
                                value: Expr::int_lit(0),
                            },
                            Span::dummy(),
                        ),
                        Stmt::new(
                            StmtKind::If {
                                condition: Expr::binop(Expr::var("x"), BinOp::Gt, Expr::int_lit(5)),
                                then_body: vec![Stmt::echo(Expr::int_lit(7))],
                                elseif_clauses: Vec::new(),
                                else_body: Some(vec![Stmt::echo(Expr::int_lit(8))]),
                            },
                            Span::dummy(),
                        ),
                    ],
                    elseif_clauses: Vec::new(),
                    else_body: None,
                },
                Span::dummy(),
            )],
        },
        Span::dummy(),
    )];

    let eliminated = eliminate_dead_code(program);

    let StmtKind::FunctionDecl { body, .. } = &eliminated[0].kind else {
        panic!("expected function");
    };
    let StmtKind::If { then_body, .. } = &body[0].kind else {
        panic!("expected if");
    };
    assert_eq!(then_body.len(), 2);
    let StmtKind::If {
        then_body: nested_then,
        else_body: nested_else,
        ..
    } = &then_body[1].kind
    else {
        panic!("expected nested if to remain after invalidation");
    };
    assert_eq!(nested_then, &vec![Stmt::echo(Expr::int_lit(7))]);
    assert_eq!(nested_else, &Some(vec![Stmt::echo(Expr::int_lit(8))]));
}
