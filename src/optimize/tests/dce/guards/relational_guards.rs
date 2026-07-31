//! Purpose:
//! Regression tests for cross-variable relational guard atoms in AST DCE.
//!
//! Called from:
//! - `crate::optimize::tests` through Rust's test harness
//!
//! Key details:
//! - Covers `$x === $y` complements, exact/range substitution into `$y > $x`,
//!   and write invalidation of relational facts.

use super::*;

/// Verifies `$x === $y` prunes a nested `$x !== $y` then-branch.
#[test]
fn test_eliminate_dead_code_prunes_nested_if_from_cross_var_strict_eq() {
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
                    condition: Expr::binop(Expr::var("x"), BinOp::StrictEq, Expr::var("y")),
                    then_body: vec![Stmt::new(
                        StmtKind::If {
                            condition: Expr::binop(
                                Expr::var("x"),
                                BinOp::StrictNotEq,
                                Expr::var("y"),
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

/// Verifies `$x === 3` plus `$y > $x` strengthens `$y` so `$y <= 3` is pruned.
#[test]
fn test_eliminate_dead_code_prunes_nested_if_from_relational_exact_substitution() {
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
                    condition: Expr::binop(Expr::var("x"), BinOp::StrictEq, Expr::int_lit(3)),
                    then_body: vec![Stmt::new(
                        StmtKind::If {
                            condition: Expr::binop(Expr::var("y"), BinOp::Gt, Expr::var("x")),
                            then_body: vec![Stmt::new(
                                StmtKind::If {
                                    condition: Expr::binop(
                                        Expr::var("y"),
                                        BinOp::LtEq,
                                        Expr::int_lit(3),
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

/// Verifies writing `$x` clears relational atoms that mention it.
#[test]
fn test_eliminate_dead_code_invalidates_relational_guard_on_write() {
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
                    condition: Expr::binop(Expr::var("x"), BinOp::StrictEq, Expr::var("y")),
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
                                condition: Expr::binop(
                                    Expr::var("x"),
                                    BinOp::StrictNotEq,
                                    Expr::var("y"),
                                ),
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
