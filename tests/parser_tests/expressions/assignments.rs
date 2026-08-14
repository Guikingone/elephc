//! Purpose:
//! Integration or regression tests for parser AST coverage of assignment expressions, including compound assignment missing ops parse, array compound assignment, and effectful array compound assignment uses synthetic temporary.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP snippets are parsed and assertions inspect AST shape, precedence, or expected parse failures.

use super::*;

/// Verifies that compound assignment operators `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
/// parse correctly as `Assign` nodes where the value is a `BinaryOp` on the variable.
/// Each case checks the operator, lhs variable, and rhs integer literal match the expected AST shape.
#[test]
fn test_compound_assignment_missing_ops_parse() {
    let cases = [
        ("<?php $x **= 3;", BinOp::Pow),
        ("<?php $x &= 3;", BinOp::BitAnd),
        ("<?php $x |= 3;", BinOp::BitOr),
        ("<?php $x ^= 3;", BinOp::BitXor),
        ("<?php $x <<= 3;", BinOp::ShiftLeft),
        ("<?php $x >>= 3;", BinOp::ShiftRight),
    ];

    for (src, expected_op) in cases {
        let stmts = parse_source(src);
        match &stmts[0].kind {
            StmtKind::Assign { name, value } => {
                assert_eq!(name, "x");
                match &value.kind {
                    ExprKind::BinaryOp { left, op, right } => {
                        assert_eq!(left.kind, ExprKind::Variable("x".into()));
                        assert_eq!(op, &expected_op);
                        assert_eq!(right.kind, ExprKind::IntLiteral(3));
                    }
                    other => panic!("expected BinaryOp, got {:?}", other),
                }
            }
            other => panic!("expected Assign, got {:?}", other),
        }
    }
}

/// Verifies that direct reference assignment parses as a `RefAssign` statement.
#[test]
fn test_parse_reference_assignment() {
    let stmts = parse_source("<?php $b =& $a;");
    match &stmts[0].kind {
        StmtKind::RefAssign { target, source } => {
            assert_eq!(target, "b");
            assert!(matches!(&source.kind, ExprKind::Variable(name) if name == "a"));
        }
        other => panic!("expected RefAssign, got {:?}", other),
    }
}

/// Verifies that binding one declared property to another retains both lvalue expressions.
#[test]
fn test_parse_property_to_property_reference_assignment() {
    let stmts = parse_source("<?php $parser->refs = &$this->refs;");
    match &stmts[0].kind {
        StmtKind::PropertyRefAssign {
            object,
            property,
            source,
        } => {
            assert!(matches!(&object.kind, ExprKind::Variable(name) if name == "parser"));
            assert_eq!(property, "refs");
            assert!(matches!(
                &source.kind,
                ExprKind::PropertyAccess { object, property }
                    if matches!(&object.kind, ExprKind::This) && property == "refs"
            ));
        }
        other => panic!("expected PropertyRefAssign, got {:?}", other),
    }
}

/// Verifies that a reference bind used inside a comparison remains an assignment expression.
#[test]
fn test_parse_reference_assignment_expression() {
    let stmts = parse_source(
        "<?php if (null !== $exists = &self::$cache[$key]) {}",
    );
    let StmtKind::If { condition, .. } = &stmts[0].kind else {
        panic!("expected If statement");
    };
    let ExprKind::BinaryOp { right, .. } = &condition.kind else {
        panic!("expected comparison condition");
    };
    let ExprKind::Assignment {
        target,
        value,
        prelude,
        ..
    } = &right.kind
    else {
        panic!("expected reference assignment expression");
    };
    assert!(matches!(&target.kind, ExprKind::Variable(name) if name == "exists"));
    assert!(matches!(&value.kind, ExprKind::Variable(name) if name == "exists"));
    assert!(matches!(
        prelude.as_slice(),
        [Stmt {
            kind: StmtKind::RefAssign { target, source },
            ..
        }] if target == "exists" && matches!(source.kind, ExprKind::ArrayAccess { .. })
    ));
}

/// Verifies that postfix increment on a static property preserves the old value as its result.
#[test]
fn test_parse_static_property_post_increment_expression() {
    let stmts = parse_source("<?php if (!self::$level++) {}");
    let StmtKind::If { condition, .. } = &stmts[0].kind else {
        panic!("expected If statement");
    };
    let ExprKind::Not(inner) = &condition.kind else {
        panic!("expected logical negation");
    };
    let ExprKind::Assignment {
        target,
        result_target,
        prelude,
        ..
    } = &inner.kind
    else {
        panic!("expected postfix assignment expression");
    };
    assert!(matches!(target.kind, ExprKind::StaticPropertyAccess { .. }));
    assert!(result_target.is_some());
    assert!(matches!(
        prelude.as_slice(),
        [Stmt {
            kind: StmtKind::Assign { .. },
            ..
        }]
    ));
}

/// Verifies that a static-property element alias preserves both lvalue indexes in its AST node.
#[test]
fn test_parse_static_property_element_reference_assignment() {
    let stmts = parse_source("<?php C::$a[$target] = &C::$a[$source];");
    match &stmts[0].kind {
        StmtKind::StaticPropertyElementRefAssign {
            receiver,
            property,
            index,
            source,
        } => {
            assert!(matches!(receiver, StaticReceiver::Named(name) if name.as_str() == "C"));
            assert_eq!(property, "a");
            assert!(matches!(&index.kind, ExprKind::Variable(name) if name == "target"));
            assert!(matches!(&source.kind, ExprKind::ArrayAccess { .. }));
        }
        other => panic!("expected StaticPropertyElementRefAssign, got {:?}", other),
    }
}

/// Verifies that `<?php $items[0] += 3;` parses to an `ArrayAssign` (not a generic `Assign`).
/// Compound assignment on an array element must produce the correct AST shape.
#[test]
fn test_parse_array_compound_assignment() {
    let stmts = parse_source("<?php $items[0] += 3;");
    match &stmts[0].kind {
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => {
            assert_eq!(array, "items");
            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
            match &value.kind {
                ExprKind::BinaryOp { left, op, right } => {
                    assert_eq!(op, &BinOp::Add);
                    assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
                    match &left.kind {
                        ExprKind::ArrayAccess { array, index } => {
                            assert!(matches!(array.kind, ExprKind::Variable(ref name) if name == "items"));
                            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
                        }
                        other => panic!("Expected ArrayAccess lhs, got {:?}", other),
                    }
                }
                other => panic!("Expected BinaryOp value, got {:?}", other),
            }
        }
        other => panic!("Expected ArrayAssign, got {:?}", other),
    }
}

/// Verifies that array-element post-increment lowers to an `ArrayAssign` read-modify-write.
#[test]
fn test_parse_array_element_post_increment_assignment() {
    let stmts = parse_source("<?php $items[0]++;");
    match &stmts[0].kind {
        StmtKind::ArrayAssign {
            array,
            index,
            value,
        } => {
            assert_eq!(array, "items");
            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
            match &value.kind {
                ExprKind::BinaryOp { left, op, right } => {
                    assert_eq!(op, &BinOp::Add);
                    assert!(matches!(right.kind, ExprKind::IntLiteral(1)));
                    match &left.kind {
                        ExprKind::ArrayAccess { array, index } => {
                            assert!(matches!(array.kind, ExprKind::Variable(ref name) if name == "items"));
                            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
                        }
                        other => panic!("Expected ArrayAccess lhs, got {:?}", other),
                    }
                }
                other => panic!("Expected BinaryOp value, got {:?}", other),
            }
        }
        other => panic!("Expected ArrayAssign, got {:?}", other),
    }
}

/// Verifies that `<?php $items[idx()] += 3;` lowers to a `Synthetic` stmt containing
/// a temporary assignment for `idx()` and a subsequent `ArrayAssign`.
/// This protects against registering the call effect directly on the ArrayAssign.
#[test]
fn test_parse_effectful_array_compound_assignment_uses_synthetic_temporary() {
    let stmts = parse_source("<?php $items[idx()] += 3;");
    match &stmts[0].kind {
        StmtKind::Synthetic(stmts) => {
            assert_eq!(stmts.len(), 2);
            assert!(matches!(stmts[0].kind, StmtKind::Assign { .. }));
            assert!(matches!(stmts[1].kind, StmtKind::ArrayAssign { .. }));
        }
        other => panic!("Expected Synthetic lowering, got {:?}", other),
    }
}

/// Verifies that `<?php $data["a"][0]["b"] = "changed";` parses to a `NestedArrayAssign`
/// with the right-most key "b" as the index and the remainder as a nested `ArrayAccess` chain.
#[test]
fn test_parse_nested_array_assignment_target() {
    let stmts = parse_source("<?php $data[\"a\"][0][\"b\"] = \"changed\";");
    match &stmts[0].kind {
        StmtKind::NestedArrayAssign { target, value } => {
            assert!(matches!(value.kind, ExprKind::StringLiteral(ref text) if text == "changed"));
            match &target.kind {
                ExprKind::ArrayAccess { array, index } => {
                    assert!(matches!(index.kind, ExprKind::StringLiteral(ref key) if key == "b"));
                    assert!(matches!(array.kind, ExprKind::ArrayAccess { .. }));
                }
                other => panic!("Expected nested ArrayAccess target, got {:?}", other),
            }
        }
        other => panic!("Expected NestedArrayAssign, got {:?}", other),
    }
}

/// Verifies that a nested array element bound by reference retains the complete target chain and
/// marks the local source for reference-cell lowering.
#[test]
fn test_parse_nested_array_reference_assignment_target() {
    let stmts = parse_source("<?php $data[$prefix.'use']['$'.$key] = &$value;");
    match &stmts[0].kind {
        StmtKind::NestedArrayAssign { target, value } => {
            assert!(matches!(target.kind, ExprKind::ArrayAccess { .. }));
            assert!(matches!(
                value.kind,
                ExprKind::ArrayReference(ref source)
                    if matches!(source.kind, ExprKind::Variable(ref name) if name == "value")
            ));
        }
        other => panic!("Expected NestedArrayAssign, got {:?}", other),
    }
}

/// Verifies that nested append (`$items[0][] = 2`) lowers to a synthetic
/// read/append/write-back sequence instead of overwriting `$items[0]` directly.
#[test]
fn test_parse_nested_array_append_lowers_to_temp_push_writeback() {
    let stmts = parse_source("<?php $items[0][] = 2;");
    match &stmts[0].kind {
        StmtKind::Synthetic(stmts) => {
            assert_eq!(stmts.len(), 3);
            let temp = match &stmts[0].kind {
                StmtKind::Assign { name, value } => {
                    match &value.kind {
                        ExprKind::ArrayAccess { array, index } => {
                            assert!(matches!(array.kind, ExprKind::Variable(ref name) if name == "items"));
                            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
                        }
                        other => panic!("Expected temp read from ArrayAccess, got {:?}", other),
                    }
                    name.clone()
                }
                other => panic!("Expected temp Assign, got {:?}", other),
            };
            match &stmts[1].kind {
                StmtKind::ArrayPush { array, value } => {
                    assert_eq!(array, &temp);
                    assert!(matches!(value.kind, ExprKind::IntLiteral(2)));
                }
                other => panic!("Expected temp ArrayPush, got {:?}", other),
            }
            match &stmts[2].kind {
                StmtKind::ArrayAssign {
                    array,
                    index,
                    value,
                } => {
                    assert_eq!(array, "items");
                    assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
                    assert!(matches!(value.kind, ExprKind::Variable(ref name) if name == &temp));
                }
                other => panic!("Expected write-back ArrayAssign, got {:?}", other),
            }
        }
        other => panic!("Expected Synthetic lowering, got {:?}", other),
    }
}

/// Verifies that `<?php ?int $value = null;` parses to a `TypedAssign` with a nullable `?int`
/// type expression and a null initializer.
#[test]
fn test_parse_nullable_typed_assign() {
    let stmts = parse_source("<?php ?int $value = null;");
    match &stmts[0].kind {
        StmtKind::TypedAssign {
            type_expr,
            name,
            value,
        } => {
            assert_eq!(name, "value");
            assert_eq!(type_expr, &TypeExpr::Nullable(Box::new(TypeExpr::Int)));
            assert_eq!(value.kind, ExprKind::Null);
        }
        other => panic!("Expected typed assign, got {:?}", other),
    }
}

/// Verifies that `<?php int|string $value = 1;` parses to a `TypedAssign` with a union type
/// expression `int|string` and an integer initializer.
#[test]
fn test_parse_union_typed_assign() {
    let stmts = parse_source("<?php int|string $value = 1;");
    match &stmts[0].kind {
        StmtKind::TypedAssign {
            type_expr,
            name,
            value,
        } => {
            assert_eq!(name, "value");
            assert_eq!(type_expr, &TypeExpr::Union(vec![TypeExpr::Int, TypeExpr::Str]));
            assert_eq!(value.kind, ExprKind::IntLiteral(1));
        }
        other => panic!("Expected typed assign, got {:?}", other),
    }
}
