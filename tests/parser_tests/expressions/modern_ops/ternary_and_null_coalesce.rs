//! Purpose:
//! Integration or regression tests for parser AST coverage of expression modern PHP operators ternary and null coalesce, including short ternary expression, short ternary lower than symbolic or, and short ternary default accepts null coalesce.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP snippets are parsed and assertions inspect AST shape, precedence, or expected parse failures.

use super::*;

/// Verifies that `<?php echo $a ?: $b;` parses as a `ShortTernary` with `value=$a` and `default=$b`.
/// The short ternary is PHP's null-coalesce-style ternary (elvis operator).
#[test]
fn test_short_ternary_expression() {
    let stmts = parse_source("<?php echo $a ?: $b;");
    let expected = Stmt::echo(Expr::new(
        ExprKind::ShortTernary {
            value: Box::new(Expr::var("a")),
            default: Box::new(Expr::var("b")),
        },
        elephc::span::Span::dummy(),
    ));
    assert_eq!(stmts, vec![expected]);
}

/// Verifies short ternary has lower precedence than symbolic `||` (logical or).
/// Input: `<?php echo $a || $b ?: $c;` → parses as `($a || $b) ?: $c`.
/// The short ternary's value is the entire `||` expression, not just `$a`.
#[test]
fn test_short_ternary_lower_than_symbolic_or() {
    let stmts = parse_source("<?php echo $a || $b ?: $c;");
    let expected = Stmt::echo(Expr::new(
        ExprKind::ShortTernary {
            value: Box::new(Expr::binop(Expr::var("a"), BinOp::Or, Expr::var("b"))),
            default: Box::new(Expr::var("c")),
        },
        elephc::span::Span::dummy(),
    ));
    assert_eq!(stmts, vec![expected]);
}

/// Verifies short ternary's default branch accepts a null coalesce expression.
/// Input: `<?php echo $a ?: $b ?? $c;` → parses as `$a ?: ($b ?? $c)`.
/// The short ternary default slot wraps the full null coalesce subtree.
#[test]
fn test_short_ternary_default_accepts_null_coalesce() {
    let stmts = parse_source("<?php echo $a ?: $b ?? $c;");
    let expected = Stmt::echo(Expr::new(
        ExprKind::ShortTernary {
            value: Box::new(Expr::var("a")),
            default: Box::new(Expr::new(
                ExprKind::NullCoalesce {
                    value: Box::new(Expr::var("b")),
                    default: Box::new(Expr::var("c")),
                },
                elephc::span::Span::dummy(),
            )),
        },
        elephc::span::Span::dummy(),
    ));
    assert_eq!(stmts, vec![expected]);
}

/// Verifies short ternary can appear as the else branch of a full ternary.
/// Input: `<?php echo $a ? $b : $c ?: $d;` → the `?: $d` branch is a `ShortTernary`.
#[test]
fn test_short_ternary_can_nest_in_full_ternary_else_branch() {
    let stmts = parse_source("<?php echo $a ? $b : $c ?: $d;");
    match &stmts[0].kind {
        StmtKind::Echo(expr) => match &expr.kind {
            ExprKind::Ternary { else_expr, .. } => {
                assert!(matches!(else_expr.kind, ExprKind::ShortTernary { .. }));
            }
            other => panic!("expected Ternary, got {:?}", other),
        },
        other => panic!("expected Echo, got {:?}", other),
    }
}

/// Verifies null coalesce `??` parses as `ExprKind::NullCoalesce`.
/// Input: `<?php echo $x ?? 0;` → top-level echo wraps a single NullCoalesce.
#[test]
fn test_null_coalesce_parse() {
    let stmts = parse_source("<?php echo $x ?? 0;");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Echo(expr) = &stmts[0].kind {
        if let ExprKind::NullCoalesce { .. } = &expr.kind {
            // good
        } else {
            panic!("expected NullCoalesce, got {:?}", expr.kind);
        }
    } else {
        panic!("expected Echo");
    }
}

/// Verifies null coalesce assignment `??=` parses correctly.
/// Input: `<?php $x ??= 10;` → Assign with name=`x` and value=`NullCoalesce($x, 10)`.
#[test]
fn test_null_coalesce_assignment_parse() {
    let stmts = parse_source("<?php $x ??= 10;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            match &value.kind {
                ExprKind::NullCoalesce { value, default } => {
                    assert_eq!(value.kind, ExprKind::Variable("x".into()));
                    assert_eq!(default.kind, ExprKind::IntLiteral(10));
                }
                other => panic!("expected NullCoalesce, got {:?}", other),
            }
        }
        other => panic!("expected Assign, got {:?}", other),
    }
}

/// Verifies null coalesce assignment's RHS can itself be a null coalesce expression.
/// Input: `<?php $x ??= $fallback ?? 10;` → value is outer NullCoalesce($fallback, 10).
#[test]
fn test_null_coalesce_assignment_rhs_is_expression() {
    let stmts = parse_source("<?php $x ??= $fallback ?? 10;");
    match &stmts[0].kind {
        StmtKind::Assign { value, .. } => match &value.kind {
            ExprKind::NullCoalesce { default, .. } => {
                assert!(matches!(default.kind, ExprKind::NullCoalesce { .. }));
            }
            other => panic!("expected outer NullCoalesce, got {:?}", other),
        },
        other => panic!("expected Assign, got {:?}", other),
    }
}

// --- Expression-position array-element / append assignment in ternary branches ---

/// Verifies `$c ? $a[0] = 7 : $a[0] = 8;` parses as a `Ternary` whose branches are each
/// `ExprKind::Assignment` targeting `$a[0]`, with `result_target` set to the (replayable)
/// RHS literal and no prelude — the same shape `$a[0] = 7;` produces as a plain non-local
/// assignment EXPRESSION (as opposed to the statement-level `ArrayAssign` shortcut, which
/// this ternary shape cannot reach because the leading `?` makes the statement-level
/// postfix-assignment scanner bail to general expression parsing).
#[test]
fn test_ternary_branch_array_element_assignment() {
    let stmts = parse_source("<?php $c ? $a[0] = 7 : $a[0] = 8;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::ExprStmt(expr) => match &expr.kind {
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                assert_eq!(condition.kind, ExprKind::Variable("c".into()));
                assert_array_element_assignment(then_expr, "a", 0, 7);
                assert_array_element_assignment(else_expr, "a", 0, 8);
            }
            other => panic!("expected Ternary, got {:?}", other),
        },
        other => panic!("expected ExprStmt, got {:?}", other),
    }
}

/// Asserts `expr` is `ExprKind::Assignment` targeting `$array[index]` with an `IntLiteral`
/// RHS of `expected_value`, no prelude, and `result_target` equal to the RHS (the shape
/// `build_assignment_expression` produces for a replayable non-local `=` target).
fn assert_array_element_assignment(expr: &Expr, array: &str, index: i64, expected_value: i64) {
    match &expr.kind {
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            conditional_value_temp,
        } => {
            match &target.kind {
                ExprKind::ArrayAccess {
                    array: array_expr,
                    index: index_expr,
                } => {
                    assert_eq!(array_expr.kind, ExprKind::Variable(array.into()));
                    assert_eq!(index_expr.kind, ExprKind::IntLiteral(index));
                }
                other => panic!("expected ArrayAccess target, got {:?}", other),
            }
            assert_eq!(value.kind, ExprKind::IntLiteral(expected_value));
            assert!(prelude.is_empty());
            assert!(conditional_value_temp.is_none());
            match result_target {
                Some(result_target) => {
                    assert_eq!(result_target.kind, ExprKind::IntLiteral(expected_value));
                }
                None => panic!("expected a result_target"),
            }
        }
        other => panic!("expected Assignment, got {:?}", other),
    }
}

/// Verifies `$a[] = 5` in expression position (a ternary branch) desugars to
/// `ExprKind::Assignment` whose `prelude` contains the hidden-temp assign followed by the
/// same `StmtKind::ArrayPush` the bare statement form already lowers to, and whose yield is
/// a copy of that temp into a DISTINCT yield local — never a `$t = $t` self-assignment,
/// which would lower through `store_local`'s release-then-acquire on one slot and hand back
/// freed memory for a refcount-1 heap string (PHP: an assignment expression yields the
/// assigned value).
#[test]
fn test_ternary_branch_append_assignment_desugars_to_prelude_push() {
    let stmts = parse_source("<?php $x = true ? $a[] = 5 : 0;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            match &value.kind {
                ExprKind::Ternary { then_expr, .. } => match &then_expr.kind {
                    ExprKind::Assignment {
                        target,
                        value,
                        prelude,
                        ..
                    } => {
                        let ExprKind::Variable(yield_name) = &target.kind else {
                            panic!("expected Variable target, got {:?}", target.kind);
                        };
                        let ExprKind::Variable(temp_name) = &value.kind else {
                            panic!("expected Variable value, got {:?}", value.kind);
                        };
                        assert_ne!(
                            yield_name, temp_name,
                            "yield must copy into a distinct local, not self-assign the temp"
                        );
                        assert_eq!(prelude.len(), 2);
                        match &prelude[0].kind {
                            StmtKind::Assign { name, value } => {
                                assert_eq!(name, temp_name);
                                assert_eq!(value.kind, ExprKind::IntLiteral(5));
                            }
                            other => panic!("expected Assign temp prelude stmt, got {:?}", other),
                        }
                        match &prelude[1].kind {
                            StmtKind::ArrayPush { array, value } => {
                                assert_eq!(array, "a");
                                assert_eq!(value.kind, ExprKind::Variable(temp_name.clone()));
                            }
                            other => panic!("expected ArrayPush prelude stmt, got {:?}", other),
                        }
                    }
                    other => panic!("expected Assignment then_expr, got {:?}", other),
                },
                other => panic!("expected Ternary, got {:?}", other),
            }
        }
        other => panic!("expected Assign, got {:?}", other),
    }
}

/// Verifies the bare statement form `$a[] = 5;` (no ternary, no enclosing expression) is
/// unaffected by the expression-position append fix and still routes to the existing
/// statement-level `StmtKind::ArrayPush` shortcut, not the new `ExprKind::Assignment`
/// desugar — regression guard for the statement-level append path.
#[test]
fn test_bare_array_push_statement_still_routes_to_statement_form() {
    let stmts = parse_source("<?php $a[] = 5;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::ArrayPush { array, value } => {
            assert_eq!(array, "a");
            assert_eq!(value.kind, ExprKind::IntLiteral(5));
        }
        other => panic!("expected ArrayPush, got {:?}", other),
    }
}

// --- Spaceship operator ---

/// Verifies that `<?php echo 1 <=> 2;` parses as a `Spaceship` binary operation.
/// The spaceship operator returns -1, 0, or 1 for three-way comparison.
#[test]
fn test_spaceship_parse() {
    let stmts = parse_source("<?php echo 1 <=> 2;");
    let expected = Stmt::echo(Expr::binop(
        Expr::int_lit(1),
        BinOp::Spaceship,
        Expr::int_lit(2),
    ));
    assert_eq!(stmts, vec![expected]);
}

// --- Constants ---
