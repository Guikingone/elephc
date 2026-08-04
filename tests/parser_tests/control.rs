//! Purpose:
//! Integration or regression tests for parser AST coverage of control, including if parses, if else parses, and if elseif else parses.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP snippets are parsed and assertions inspect AST shape, precedence, or expected parse failures.

use super::*;

/// Verifies that `<?php if (1 == 1) { echo "yes"; }` parses to an `If` statement.
#[test]
fn test_if_parses() {
    let stmts = parse_source("<?php if (1 == 1) { echo \"yes\"; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0].kind, StmtKind::If { .. }));
}

/// Verifies that `<?php if (1) { echo "a"; } else { echo "b"; }` parses to an `If` with `else_body` present.
#[test]
fn test_if_else_parses() {
    let stmts = parse_source("<?php if (1) { echo \"a\"; } else { echo \"b\"; }");
    if let StmtKind::If { else_body, .. } = &stmts[0].kind {
        assert!(else_body.is_some());
    } else {
        panic!("expected If");
    }
}

/// Verifies that `<?php if (1) { echo "a"; } elseif (2) { echo "b"; } else { echo "c"; }`
/// parses to an `If` with one `elseif_clause` and an `else_body`.
#[test]
fn test_if_elseif_else_parses() {
    let stmts = parse_source(
        "<?php if (1) { echo \"a\"; } elseif (2) { echo \"b\"; } else { echo \"c\"; }",
    );
    if let StmtKind::If {
        elseif_clauses,
        else_body,
        ..
    } = &stmts[0].kind
    {
        assert_eq!(elseif_clauses.len(), 1);
        assert!(else_body.is_some());
    } else {
        panic!("expected If");
    }
}

/// Verifies that `<?php while (1) { echo "loop"; }` parses to a `While` statement.
#[test]
fn test_while_parses() {
    let stmts = parse_source("<?php while (1) { echo \"loop\"; }");
    assert!(matches!(&stmts[0].kind, StmtKind::While { .. }));
}

/// Verifies that `<?php do { echo "loop"; } while (1);` parses to a `DoWhile` statement.
#[test]
fn test_do_while_parses() {
    let stmts = parse_source("<?php do { echo \"loop\"; } while (1);");
    assert!(matches!(&stmts[0].kind, StmtKind::DoWhile { .. }));
}

/// Verifies that `<?php for ($i = 0; $i < 10; $i++) { echo $i; }` parses to a `For` statement.
#[test]
fn test_for_parses() {
    let stmts = parse_source("<?php for ($i = 0; $i < 10; $i++) { echo $i; }");
    assert!(matches!(&stmts[0].kind, StmtKind::For { .. }));
}

/// Verifies that `<?php while (1) { break; }` parses with the `Break(1)` statement nested
/// inside `While`. The argument 1 means break one level.
#[test]
fn test_break_parses() {
    let stmts = parse_source("<?php while (1) { break; }");
    if let StmtKind::While { body, .. } = &stmts[0].kind {
        assert!(matches!(&body[0].kind, StmtKind::Break(1)));
    }
}

/// Verifies that `<?php while (1) { while (1) { break 2; } }` parses with `Break(2)` at depth 2.
/// The numeric argument must be preserved correctly across nesting levels.
#[test]
fn test_multilevel_break_parses() {
    let stmts = parse_source("<?php while (1) { while (1) { break 2; } }");
    if let StmtKind::While { body, .. } = &stmts[0].kind {
        if let StmtKind::While { body, .. } = &body[0].kind {
            assert!(matches!(&body[0].kind, StmtKind::Break(2)));
        } else {
            panic!("expected nested While");
        }
    } else {
        panic!("expected While");
    }
}

/// Verifies that `<?php while (1) { continue; }` parses with `Continue(1)` inside `While`.
#[test]
fn test_continue_parses() {
    let stmts = parse_source("<?php while (1) { continue; }");
    if let StmtKind::While { body, .. } = &stmts[0].kind {
        assert!(matches!(&body[0].kind, StmtKind::Continue(1)));
    }
}

/// Verifies that `<?php while (1) { while (1) { continue (2); } }` parses with `Continue(2)`
/// at depth 2. The parenthesized form of the level argument must be accepted.
#[test]
fn test_multilevel_continue_parses() {
    let stmts = parse_source("<?php while (1) { while (1) { continue (2); } }");
    if let StmtKind::While { body, .. } = &stmts[0].kind {
        if let StmtKind::While { body, .. } = &body[0].kind {
            assert!(matches!(&body[0].kind, StmtKind::Continue(2)));
        } else {
            panic!("expected nested While");
        }
    } else {
        panic!("expected While");
    }
}

// --- Functions ---

/// Verifies that `<?php switch ($x) { case 1: echo "one"; break; default: echo "other"; }`
/// parses to a `Switch` statement with a default case.
#[test]
fn test_parse_switch() {
    let stmts =
        parse_source("<?php switch ($x) { case 1: echo \"one\"; break; default: echo \"other\"; }");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0].kind, StmtKind::Switch { .. }));
}

// --- Match ---

/// Verifies that `<?php foreach ($a as $k => $v) {}` parses with `key_var = Some("k")`,
/// `value_var = "v"`, and `value_by_ref = false`.
#[test]
fn test_parse_foreach_key_value() {
    let stmts = parse_source("<?php foreach ($a as $k => $v) {}");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Foreach {
        key_var,
        value_var,
        value_by_ref,
        ..
    } = &stmts[0].kind
    {
        assert_eq!(key_var, &Some("k".to_string()));
        assert_eq!(value_var, "v");
        assert!(!value_by_ref);
    } else {
        panic!("expected Foreach");
    }
}

/// Verifies that `<?php foreach ($a as $value) {}` parses with no key variable,
/// `value_var = "value"`, and `value_by_ref = false`.
#[test]
fn test_parse_foreach_value_only() {
    let stmts = parse_source("<?php foreach ($a as $value) {}");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Foreach {
        key_var,
        value_var,
        value_by_ref,
        ..
    } = &stmts[0].kind
    {
        assert_eq!(key_var, &None);
        assert_eq!(value_var, "value");
        assert!(!value_by_ref);
    } else {
        panic!("expected Foreach");
    }
}

/// Verifies that `<?php foreach ($a as &$value) {}` parses with no key variable,
/// `value_var = "value"`, and `value_by_ref = true`.
#[test]
fn test_parse_foreach_value_by_ref() {
    let stmts = parse_source("<?php foreach ($a as &$value) {}");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Foreach {
        key_var,
        value_var,
        value_by_ref,
        ..
    } = &stmts[0].kind
    {
        assert_eq!(key_var, &None);
        assert_eq!(value_var, "value");
        assert!(value_by_ref);
    } else {
        panic!("expected Foreach");
    }
}

/// Verifies that `<?php foreach ($a as $key => &$value) {}` parses with key_var = Some("key"),
/// `value_var = "value"`, and `value_by_ref = true`.
#[test]
fn test_parse_foreach_key_value_by_ref() {
    let stmts = parse_source("<?php foreach ($a as $key => &$value) {}");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Foreach {
        key_var,
        value_var,
        value_by_ref,
        ..
    } = &stmts[0].kind
    {
        assert_eq!(key_var, &Some("key".to_string()));
        assert_eq!(value_var, "value");
        assert!(value_by_ref);
    } else {
        panic!("expected Foreach");
    }
}

/// Verifies `foreach ($m as [$a, $b])` desugars to a loop over a hidden value variable whose
/// body starts with the same unpack statement `[$a, $b] = $tmp;` produces.
#[test]
fn test_parse_foreach_value_destructuring_desugars_to_hidden_temp() {
    let stmts = parse_source("<?php foreach ($m as [$a, $b]) { echo $a; }");
    assert_eq!(stmts.len(), 1);
    let StmtKind::Foreach {
        key_var,
        value_var,
        value_by_ref,
        body,
        ..
    } = &stmts[0].kind
    else {
        panic!("expected Foreach");
    };
    assert_eq!(key_var, &None);
    assert!(value_var.starts_with("__elephc_foreach_"));
    assert!(!value_by_ref);
    assert_eq!(body.len(), 2);
    let StmtKind::ListUnpack { vars, value } = &body[0].kind else {
        panic!("expected the unpack statement first in the body");
    };
    assert_eq!(vars, &vec!["a".to_string(), "b".to_string()]);
    assert_eq!(value.kind, ExprKind::Variable(value_var.clone()));
}

/// Verifies the `$key => [pattern]` form keeps the real key variable and only replaces the
/// value target with the hidden temporary.
#[test]
fn test_parse_foreach_key_with_value_destructuring() {
    let stmts = parse_source("<?php foreach ($m as $k => [$a, $b]) {}");
    assert_eq!(stmts.len(), 1);
    let StmtKind::Foreach {
        key_var,
        value_var,
        body,
        ..
    } = &stmts[0].kind
    else {
        panic!("expected Foreach");
    };
    assert_eq!(key_var, &Some("k".to_string()));
    assert!(value_var.starts_with("__elephc_foreach_"));
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0].kind, StmtKind::ListUnpack { .. }));
}

/// Verifies a reference to a whole destructuring pattern is rejected: PHP allows `&` on the
/// targets inside the pattern, never on the pattern itself.
#[test]
fn test_parse_foreach_reference_to_pattern_is_rejected() {
    assert!(parse_fails("<?php foreach ($m as &[$a, $b]) {}"));
    assert!(parse_fails("<?php foreach ($m as $k => &[$a, $b]) {}"));
}
