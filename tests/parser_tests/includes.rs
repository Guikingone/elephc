//! Purpose:
//! Integration or regression tests for parser AST coverage of includes, including word logical typed assignment rhs requires parentheses, include parses, and require parses.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP snippets cover successful AST shapes plus malformed syntax that must fail during parsing.

use super::*;

/// Verifies that `<?php int $x = true or false;` fails to parse because the RHS of a
/// typed assignment requires parentheses — the `or` keyword has lower precedence than
/// the `=` sign, which would incorrectly parse as `(int $x = true) or false`.
#[test]
fn test_word_logical_typed_assignment_rhs_requires_parentheses() {
    assert!(parse_fails("<?php int $x = true or false;"));
}

/// Verifies that `<?php include 'file.php';` parses to an `Include` with path StringLiteral
/// "file.php", once=false, required=false.
#[test]
fn test_include_parses() {
    let stmts = parse_source("<?php include 'file.php';");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Include {
        path,
        once,
        required,
    } = &stmts[0].kind
    {
        assert_path_string_literal(path, "file.php");
        assert!(!once);
        assert!(!required);
    } else {
        panic!("expected Include");
    }
}

/// Verifies that `<?php @include 'file.php';` parses with error suppression applied to the include.
#[test]
fn test_error_suppressed_include_parses() {
    let stmts = parse_source("<?php @include 'file.php';");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Include {
        path,
        once,
        required,
    } = &stmts[0].kind
    {
        assert_path_string_literal(path, "file.php");
        assert!(!once);
        assert!(!required);
    } else {
        panic!("expected Include");
    }
}

/// Verifies that `<?php require 'file.php';` parses with required=true, once=false.
#[test]
fn test_require_parses() {
    let stmts = parse_source("<?php require 'file.php';");
    if let StmtKind::Include {
        path,
        once,
        required,
    } = &stmts[0].kind
    {
        assert_path_string_literal(path, "file.php");
        assert!(!once);
        assert!(required);
    } else {
        panic!("expected Include (require)");
    }
}

/// Verifies that `<?php include_once 'file.php';` parses with once=true, required=false.
#[test]
fn test_include_once_parses() {
    let stmts = parse_source("<?php include_once 'file.php';");
    if let StmtKind::Include { once, required, .. } = &stmts[0].kind {
        assert!(once);
        assert!(!required);
    } else {
        panic!("expected Include (include_once)");
    }
}

/// Verifies that `<?php require_once 'file.php';` parses with once=true, required=true.
#[test]
fn test_require_once_parses() {
    let stmts = parse_source("<?php require_once 'file.php';");
    if let StmtKind::Include { once, required, .. } = &stmts[0].kind {
        assert!(once);
        assert!(required);
    } else {
        panic!("expected Include (require_once)");
    }
}

/// Verifies that `<?php include('file.php');` (parenthesized path) parses to an `Include`
/// with a string literal path. Parenthesized include paths are valid PHP.
#[test]
fn test_include_with_parens_parses() {
    let stmts = parse_source("<?php include('file.php');");
    if let StmtKind::Include { path, .. } = &stmts[0].kind {
        assert_path_string_literal(path, "file.php");
    } else {
        panic!("expected Include");
    }
}

/// Verifies that `<?php require __DIR__ . '/lib/x.php';` parses with a binary concatenation
/// of `__DIR__` magic constant and a string literal as the include path.
#[test]
fn test_require_with_dunder_dir_concat_parses() {
    let stmts = parse_source("<?php require __DIR__ . '/lib/x.php';");
    if let StmtKind::Include { path, .. } = &stmts[0].kind {
        match &path.kind {
            ExprKind::BinaryOp { left, op: BinOp::Concat, right } => {
                assert_eq!(left.kind, ExprKind::MagicConstant(MagicConstant::Dir));
                assert_eq!(right.kind, ExprKind::StringLiteral("/lib/x.php".to_string()));
            }
            other => panic!("expected BinaryOp(Concat) path, got {:?}", other),
        }
    } else {
        panic!("expected Include");
    }
}

/// Verifies that `<?php require BASE . '/x.php';` parses with a binary concatenation of
/// a constant reference and a string literal as the include path.
#[test]
fn test_require_with_const_ref_parses() {
    let stmts = parse_source("<?php require BASE . '/x.php';");
    if let StmtKind::Include { path, .. } = &stmts[0].kind {
        match &path.kind {
            ExprKind::BinaryOp { left, op: BinOp::Concat, right } => {
                match &left.kind {
                    ExprKind::ConstRef(name) => assert_eq!(name.as_str(), "BASE"),
                    other => panic!("expected ConstRef left, got {:?}", other),
                }
                assert_eq!(right.kind, ExprKind::StringLiteral("/x.php".to_string()));
            }
            other => panic!("expected BinaryOp(Concat) path, got {:?}", other),
        }
    } else {
        panic!("expected Include");
    }
}

// --- Exponentiation ---

/// Asserts that `expr` is the transient `IncludeValue` marker the parser emits for an
/// expression-position include, returning its `(once, required)` flags for the caller to check.
/// The resolver later expands this marker into caller-scope statements.
fn assert_include_value(expr: &ExprKind) -> (bool, bool) {
    match expr {
        ExprKind::IncludeValue { once, required, .. } => (*once, *required),
        other => panic!("Expected IncludeValue, got {:?}", other),
    }
}

/// Verifies that `return require X;` parses to `return <IncludeValue>`, carrying the include's
/// once/required flags for the resolver to expand into the caller's scope.
#[test]
fn test_return_require_parses_as_include_value() {
    let stmts = parse_source("<?php function f() { return require 'helper.php'; }");
    let body = match &stmts[0].kind {
        StmtKind::FunctionDecl { body, .. } => body,
        other => panic!("Expected FunctionDecl, got {:?}", other),
    };
    match &body[0].kind {
        StmtKind::Return(Some(value)) => {
            assert_eq!(assert_include_value(&value.kind), (false, true));
        }
        other => panic!("Expected Return, got {:?}", other),
    }
}

/// Verifies that `$x = require_once X;` assigns the `IncludeValue` marker so the resolver can
/// expand it, and that the once/required flags are carried through.
#[test]
fn test_assign_require_parses_as_include_value() {
    let stmts = parse_source("<?php $x = require_once 'helper.php';");
    match &stmts[0].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(assert_include_value(&value.kind), (true, true));
        }
        other => panic!("Expected Assign, got {:?}", other),
    }
}

/// Verifies that `require`/`include` (with optional `_once`) now parse as a general expression
/// operand — not just the direct RHS of `$var = ...` or `return ...` (which were already
/// special-cased by `compound.rs` and the return-statement parser before the general prefix
/// expression parser learned the include keywords). Here `require` is the right operand of `+`,
/// nested two levels deep (`BinaryOp` inside the assignment value), which only the general
/// prefix parser (`crate::parser::expr::prefix::parse_prefix`) can reach. Cross-checked with
/// `php`: `$y = 10 + (require 'five.php');` evaluates to `15` when `five.php` returns `5`.
#[test]
fn test_require_as_binary_operand_parses_as_include_value() {
    let stmts = parse_source("<?php $y = 10 + (require 'five.php');");
    match &stmts[0].kind {
        StmtKind::Assign { name, value } => {
            assert_eq!(name, "y");
            match &value.kind {
                ExprKind::BinaryOp { left, op, right } => {
                    assert_eq!(op, &BinOp::Add);
                    assert_eq!(left.kind, ExprKind::IntLiteral(10));
                    assert_eq!(assert_include_value(&right.kind), (false, true));
                }
                other => panic!("Expected BinaryOp value, got {:?}", other),
            }
        }
        other => panic!("Expected Assign, got {:?}", other),
    }
}

/// Verifies that `require` parses as the RHS of `??=` on a *static property* target
/// (`self::$v ??= require F;`), mirroring the real-world shape that motivated this feature:
/// `self::$tableZero ??= require __DIR__.'/Resources/data/wcswidth_table_zero.php';` in
/// symfony/string's `AbstractUnicodeString`. A static-property compound assignment is not the
/// `StmtKind::Assign`-to-simple-variable shape that `compound.rs` special-cases, so this exercises
/// the general prefix parser exclusively. The lowered shape is
/// `StmtKind::StaticPropertyAssign { value: NullCoalesce { value: <self::$v>, default: IncludeValue } }`,
/// matching PHP's `??=` desugaring (`target = target ?? rhs`) applied to the parsed target/rhs.
#[test]
fn test_require_as_static_property_null_coalesce_rhs_parses_as_include_value() {
    let stmts = parse_source(
        "<?php class L { public static ?int $v = null; public static function g(): int { self::$v ??= require 'd.php'; return self::$v; } }",
    );
    let StmtKind::ClassDecl { methods, .. } = &stmts[0].kind else {
        panic!("Expected ClassDecl, got {:?}", &stmts[0].kind);
    };
    let body = &methods[0].body;
    match &body[0].kind {
        StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value,
        } => {
            assert_eq!(receiver, &StaticReceiver::Self_);
            assert_eq!(property, "v");
            match &value.kind {
                ExprKind::NullCoalesce { value, default } => {
                    assert_eq!(
                        value.kind,
                        ExprKind::StaticPropertyAccess {
                            receiver: StaticReceiver::Self_,
                            property: "v".to_string(),
                        }
                    );
                    assert_eq!(assert_include_value(&default.kind), (false, true));
                }
                other => panic!("Expected NullCoalesce value, got {:?}", other),
            }
        }
        other => panic!("Expected StaticPropertyAssign, got {:?}", other),
    }
}
