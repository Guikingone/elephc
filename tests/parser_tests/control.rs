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

/// Verifies a `for` with comma-separated init and update clauses parses, wrapping each multi-statement
/// clause in a `Synthetic` block (the init holds both assignments, the update holds both increments).
#[test]
fn test_for_comma_clauses_parse_to_synthetic() {
    let stmts = parse_source("<?php for ($i = 0, $j = 10; $i < 5; $i++, $j--) {}");
    let StmtKind::For { init, update, .. } = &stmts[0].kind else {
        panic!("expected For");
    };
    assert!(matches!(init.as_deref().map(|s| &s.kind), Some(StmtKind::Synthetic(stmts)) if stmts.len() == 2));
    assert!(matches!(update.as_deref().map(|s| &s.kind), Some(StmtKind::Synthetic(stmts)) if stmts.len() == 2));
}

/// Regression guard for the historical for-clause fast paths: `$i = 0`-style items must
/// stay dedicated `StmtKind::Assign` nodes (not expression-position `ExprKind::Assignment`)
/// and `$i++` / `$j--` items must stay inc/dec `ExprStmt`s, exactly as before arbitrary
/// expressions were allowed in for clauses.
#[test]
fn test_for_assignment_and_incdec_clause_ast_unchanged() {
    let stmts = parse_source("<?php for ($i = 0, $j = 10; $i < 5; $i++, $j--) {}");
    let StmtKind::For { init, update, .. } = &stmts[0].kind else {
        panic!("expected For");
    };
    let Some(StmtKind::Synthetic(init_stmts)) = init.as_deref().map(|s| &s.kind) else {
        panic!("expected Synthetic init");
    };
    assert!(matches!(&init_stmts[0].kind, StmtKind::Assign { name, .. } if name == "i"));
    assert!(matches!(&init_stmts[1].kind, StmtKind::Assign { name, .. } if name == "j"));
    let Some(StmtKind::Synthetic(update_stmts)) = update.as_deref().map(|s| &s.kind) else {
        panic!("expected Synthetic update");
    };
    assert!(matches!(
        &update_stmts[0].kind,
        StmtKind::ExprStmt(Expr { kind: ExprKind::PostIncrement(name), .. }) if name == "i"
    ));
    assert!(matches!(
        &update_stmts[1].kind,
        StmtKind::ExprStmt(Expr { kind: ExprKind::PostDecrement(name), .. }) if name == "j"
    ));
}

/// Verifies that arbitrary call expressions in `for` init/update clauses parse to
/// effect-only `ExprStmt` items, matching PHP's arbitrary-expression clause grammar
/// (`for (next($paths); null !== key($paths); next($paths))`, the Path.php:629 shape).
#[test]
fn test_for_call_expression_clauses_parse_to_expr_stmts() {
    let stmts =
        parse_source("<?php for (next($paths); null !== key($paths); next($paths)) {}");
    let StmtKind::For {
        init,
        condition,
        update,
        ..
    } = &stmts[0].kind
    else {
        panic!("expected For");
    };
    assert!(matches!(
        init.as_deref().map(|s| &s.kind),
        Some(StmtKind::ExprStmt(Expr { kind: ExprKind::FunctionCall { .. }, .. }))
    ));
    assert!(matches!(
        condition.as_ref().map(|c| &c.kind),
        Some(ExprKind::BinaryOp { .. })
    ));
    assert!(matches!(
        update.as_deref().map(|s| &s.kind),
        Some(StmtKind::ExprStmt(Expr { kind: ExprKind::FunctionCall { .. }, .. }))
    ));
}

/// Verifies a mixed comma list in a `for` init clause: an assignment fast-path item and a
/// call expression item share one `Synthetic` block in source order.
#[test]
fn test_for_mixed_assignment_and_call_clause_list_parses() {
    let stmts = parse_source("<?php for ($i = 0, log_(); $i < 2; log_(), $i++) {}");
    let StmtKind::For { init, update, .. } = &stmts[0].kind else {
        panic!("expected For");
    };
    let Some(StmtKind::Synthetic(init_stmts)) = init.as_deref().map(|s| &s.kind) else {
        panic!("expected Synthetic init");
    };
    assert!(matches!(&init_stmts[0].kind, StmtKind::Assign { name, .. } if name == "i"));
    assert!(matches!(
        &init_stmts[1].kind,
        StmtKind::ExprStmt(Expr { kind: ExprKind::FunctionCall { .. }, .. })
    ));
    let Some(StmtKind::Synthetic(update_stmts)) = update.as_deref().map(|s| &s.kind) else {
        panic!("expected Synthetic update");
    };
    assert!(matches!(
        &update_stmts[0].kind,
        StmtKind::ExprStmt(Expr { kind: ExprKind::FunctionCall { .. }, .. })
    ));
    assert!(matches!(
        &update_stmts[1].kind,
        StmtKind::ExprStmt(Expr { kind: ExprKind::PostIncrement(name), .. }) if name == "i"
    ));
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

/// Verifies `foreach ($a as [$x, $y]) {}` desugars to a `Foreach` whose synthetic
/// `value_var` is bound and whose body starts with a `ListUnpack` of `[$x, $y]`.
#[test]
fn test_parse_foreach_destructure_positional() {
    let stmts = parse_source("<?php foreach ($a as [$x, $y]) {}");
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
    assert!(value_var.starts_with("__elephc_foreach_destructure_"));
    assert!(!value_by_ref);
    assert!(matches!(
        body.first().map(|s| &s.kind),
        Some(StmtKind::ListUnpack { vars, .. }) if vars.len() == 2
    ));
}

/// Verifies `foreach ($a as $k => [$x, $y]) {}` keeps the key and desugars the value
/// pattern into a leading `ListUnpack`.
#[test]
fn test_parse_foreach_destructure_key_value() {
    let stmts = parse_source("<?php foreach ($a as $k => [$x, $y]) {}");
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
    assert!(value_var.starts_with("__elephc_foreach_destructure_"));
    assert!(matches!(
        body.first().map(|s| &s.kind),
        Some(StmtKind::ListUnpack { vars, .. }) if vars.len() == 2
    ));
}

/// Verifies a keyed foreach destructure pattern lowers to a `Synthetic` body prefix
/// (keyed entries cannot use the simple `ListUnpack` form).
#[test]
fn test_parse_foreach_destructure_keyed_pattern() {
    let stmts = parse_source("<?php foreach ($a as [\"id\" => $id]) {}");
    assert_eq!(stmts.len(), 1);
    let StmtKind::Foreach { body, .. } = &stmts[0].kind else {
        panic!("expected Foreach");
    };
    assert!(matches!(
        body.first().map(|s| &s.kind),
        Some(StmtKind::Synthetic(stmts)) if !stmts.is_empty()
    ));
}

/// Verifies a property KEY target (`foreach ($a as $this->k => $v)`) desugars to a hidden
/// `__elephc_fe_key_*` loop variable with a `PropertyAssign` store prepended as the first
/// body statement, leaving `value_var` as the plain variable.
#[test]
fn test_parse_foreach_property_key_target_desugars() {
    let stmts = parse_source("<?php foreach ($a as $this->k => $v) { echo 1; }");
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
    let key = key_var.as_deref().expect("expected desugared key variable");
    assert!(key.starts_with("__elephc_fe_key_"));
    assert_eq!(value_var, "v");
    assert!(!value_by_ref);
    assert_eq!(body.len(), 2);
    let StmtKind::PropertyAssign { object, property, value } = &body[0].kind else {
        panic!("expected prepended PropertyAssign, got {:?}", body[0].kind);
    };
    assert!(matches!(object.kind, ExprKind::This));
    assert_eq!(property, "k");
    assert!(matches!(&value.kind, ExprKind::Variable(name) if name == key));
    assert!(matches!(&body[1].kind, StmtKind::Echo(_)));
}

/// Verifies a property VALUE target (`foreach ($a as $q->v)`) desugars to a hidden
/// `__elephc_fe_val_*` loop variable with the `PropertyAssign` store prepended.
#[test]
fn test_parse_foreach_property_value_target_desugars() {
    let stmts = parse_source("<?php foreach ($a as $q->v) {}");
    let StmtKind::Foreach {
        key_var,
        value_var,
        body,
        ..
    } = &stmts[0].kind
    else {
        panic!("expected Foreach");
    };
    assert_eq!(key_var, &None);
    assert!(value_var.starts_with("__elephc_fe_val_"));
    assert_eq!(body.len(), 1);
    let StmtKind::PropertyAssign { property, value, .. } = &body[0].kind else {
        panic!("expected prepended PropertyAssign, got {:?}", body[0].kind);
    };
    assert_eq!(property, "v");
    assert!(matches!(&value.kind, ExprKind::Variable(name) if name == value_var));
}

/// Verifies an array-element VALUE target (`foreach ($a as $out["k"])`) desugars to the
/// same `ArrayAssign` statement shape the assignment parser produces for `$out["k"] = $v;`.
#[test]
fn test_parse_foreach_array_element_value_target_desugars() {
    let stmts = parse_source("<?php foreach ($a as $out[\"k\"]) {}");
    let StmtKind::Foreach { value_var, body, .. } = &stmts[0].kind else {
        panic!("expected Foreach");
    };
    assert!(value_var.starts_with("__elephc_fe_val_"));
    let StmtKind::ArrayAssign { array, index, value } = &body[0].kind else {
        panic!("expected prepended ArrayAssign, got {:?}", body[0].kind);
    };
    assert_eq!(array, "out");
    assert!(matches!(&index.kind, ExprKind::StringLiteral(s) if s == "k"));
    assert!(matches!(&value.kind, ExprKind::Variable(name) if name == value_var));
}

/// Verifies a static-property KEY target (`foreach ($a as R::$k => $v)`) desugars to the
/// same `StaticPropertyAssign` statement shape as `R::$k = $v;`.
#[test]
fn test_parse_foreach_static_property_key_target_desugars() {
    let stmts = parse_source("<?php foreach ($a as R::$k => $v) {}");
    let StmtKind::Foreach { key_var, body, .. } = &stmts[0].kind else {
        panic!("expected Foreach");
    };
    let key = key_var.as_deref().expect("expected desugared key variable");
    assert!(key.starts_with("__elephc_fe_key_"));
    let StmtKind::StaticPropertyAssign { receiver, property, value } = &body[0].kind else {
        panic!("expected prepended StaticPropertyAssign, got {:?}", body[0].kind);
    };
    assert!(matches!(receiver, StaticReceiver::Named(name) if name.as_str() == "R"));
    assert_eq!(property, "k");
    assert!(matches!(&value.kind, ExprKind::Variable(name) if name == key));
}

/// Verifies both positions desugared in the same loop prepend the VALUE store before the
/// KEY store, matching PHP's per-iteration assignment order (value first, key second).
#[test]
fn test_parse_foreach_both_lvalue_positions_value_store_first() {
    let stmts = parse_source("<?php foreach ($a as $t->k => $t->v) {}");
    let StmtKind::Foreach {
        key_var,
        value_var,
        body,
        ..
    } = &stmts[0].kind
    else {
        panic!("expected Foreach");
    };
    assert!(key_var.as_deref().is_some_and(|k| k.starts_with("__elephc_fe_key_")));
    assert!(value_var.starts_with("__elephc_fe_val_"));
    assert_eq!(body.len(), 2);
    assert!(matches!(&body[0].kind, StmtKind::PropertyAssign { property, .. } if property == "v"));
    assert!(matches!(&body[1].kind, StmtKind::PropertyAssign { property, .. } if property == "k"));
}

/// Regression guard: the plain `$k => $v` form keeps the exact historical AST — plain
/// names on the node, no hidden variables, and no prepended statements in the body.
#[test]
fn test_parse_foreach_plain_key_value_body_untouched() {
    let stmts = parse_source("<?php foreach ($a as $k => $v) { echo 1; }");
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
    assert_eq!(key_var, &Some("k".to_string()));
    assert_eq!(value_var, "v");
    assert!(!value_by_ref);
    assert_eq!(body.len(), 1);
    assert!(matches!(&body[0].kind, StmtKind::Echo(_)));
}

/// Verifies `goto target;` parses to a `Goto` statement carrying the label name.
#[test]
fn test_goto_parses() {
    let stmts = parse_source("<?php goto target;");
    assert!(matches!(&stmts[0].kind, StmtKind::Goto(name) if name == "target"));
}

/// Verifies a bare `name:` at statement position parses to a `Label` statement, distinct from a
/// constant-expression statement or a static `::` reference.
#[test]
fn test_label_parses() {
    let stmts = parse_source("<?php target: echo 1;");
    assert!(matches!(&stmts[0].kind, StmtKind::Label(name) if name == "target"));
    assert!(matches!(&stmts[1].kind, StmtKind::Echo(_)));
}

/// Verifies an `Identifier ::` reference is not misparsed as a label: `Foo::BAR;` stays an
/// expression statement because `::` lexes as one `DoubleColon` token, not `Identifier` + `Colon`.
#[test]
fn test_static_ref_is_not_label() {
    let stmts = parse_source("<?php Foo::BAR;");
    assert!(!matches!(&stmts[0].kind, StmtKind::Label(_)));
}

/// Verifies a `static $x;` with no initializer parses to a `StaticVar` whose init defaults to null.
#[test]
fn test_static_var_no_initializer_parses() {
    let stmts = parse_source("<?php static $x;");
    let StmtKind::StaticVar { name, init } = &stmts[0].kind else {
        panic!("expected StaticVar");
    };
    assert_eq!(name, "x");
    assert!(matches!(init.kind, ExprKind::Null));
}

/// Verifies a comma-separated `static $a = 1, $b;` declaration parses to a `Synthetic` block holding
/// one `StaticVar` per variable, preserving each initializer (the second defaults to null).
#[test]
fn test_static_var_comma_list_parses() {
    let stmts = parse_source("<?php static $a = 1, $b;");
    let StmtKind::Synthetic(decls) = &stmts[0].kind else {
        panic!("expected Synthetic block for multiple static vars");
    };
    assert_eq!(decls.len(), 2);
    assert!(matches!(&decls[0].kind, StmtKind::StaticVar { name, init }
        if name == "a" && matches!(init.kind, ExprKind::IntLiteral(1))));
    assert!(matches!(&decls[1].kind, StmtKind::StaticVar { name, init }
        if name == "b" && matches!(init.kind, ExprKind::Null)));
}

/// Verifies the dynamic first-class-callable form `$cb(...)` preserves closure creation as an
/// `__invoke` method target, allowing invokable object values to materialize a descriptor.
#[test]
fn test_dynamic_first_class_callable_parses_to_invoke_target() {
    let stmts = parse_source("<?php $x = $cb(...);");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected assignment");
    };
    assert!(matches!(
        &value.kind,
        ExprKind::FirstClassCallable(CallableTarget::Method { object, method })
            if method == "__invoke"
                && matches!(&object.kind, ExprKind::Variable(name) if name == "cb")
    ));
}
