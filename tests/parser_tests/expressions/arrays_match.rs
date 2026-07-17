//! Purpose:
//! Integration or regression tests for parser AST coverage of expression parsing, including string indexing uses array access AST, assoc array, and match.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP snippets are parsed and assertions inspect AST shape, precedence, or expected parse failures.

use super::*;

/// Verifies that `<?php echo $name[1];` parses as an `ArrayAccess` expression with an integer
/// index. String indexing in PHP uses the same `ArrayAccess` AST node as array indexing.
#[test]
fn test_parse_string_indexing_uses_array_access_ast() {
    let stmts = parse_source("<?php echo $name[1];");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Echo(expr) => match &expr.kind {
            ExprKind::ArrayAccess { array, index } => {
                assert_eq!(array.kind, ExprKind::Variable("name".into()));
                assert_eq!(index.kind, ExprKind::IntLiteral(1));
            }
            other => panic!("expected array access, got {:?}", other),
        },
        other => panic!("expected echo, got {:?}", other),
    }
}

/// Verifies that `<?php $m = ["a" => 1];` parses to an `Assign` with an `ArrayLiteralAssoc` value.
#[test]
fn test_parse_assoc_array() {
    let stmts = parse_source("<?php $m = [\"a\" => 1];");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        assert!(matches!(&value.kind, ExprKind::ArrayLiteralAssoc(_)));
    } else {
        panic!("expected Assign");
    }
}

/// Verifies that leading positional elements are preserved when a later array
/// entry uses an explicit key.
#[test]
fn test_parse_mixed_array_preserves_leading_positional_element() {
    let stmts = parse_source("<?php $m = [10, \"a\" => 1];");
    assert_eq!(stmts.len(), 1);
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::ArrayLiteralAssoc(items) = &value.kind else {
        panic!("expected ArrayLiteralAssoc");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].0.kind, ExprKind::IntLiteral(0));
    assert_eq!(items[0].1.kind, ExprKind::IntLiteral(10));
}

// --- Switch ---

/// Verifies that `<?php $x = match(1) { 1 => "a" };` parses to an `Assign` with a `Match`
/// expression. The `match` arm subject and single arm are preserved in the AST.
#[test]
fn test_parse_match() {
    let stmts = parse_source("<?php $x = match(1) { 1 => \"a\" };");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        assert!(matches!(&value.kind, ExprKind::Match { .. }));
    } else {
        panic!("expected Assign containing Match");
    }
}

/// Verifies that a trailing comma before `=>` in a match arm's pattern list
/// (`'a', 'b', => 1`) is treated as a separator, not a second pattern: the
/// resulting `Match` AST has the exact same arm pattern count/kinds, result
/// kind, and default presence as the equivalent non-trailing-comma form.
#[test]
fn test_parse_match_trailing_comma_arm_same_ast_as_without() {
    fn extract_match(stmts: &[Stmt]) -> (Vec<(Vec<Expr>, Expr)>, Option<Box<Expr>>) {
        let StmtKind::Assign { value, .. } = &stmts[0].kind else {
            panic!("expected Assign, got {:?}", stmts[0].kind);
        };
        let ExprKind::Match { arms, default, .. } = &value.kind else {
            panic!("expected Match, got {:?}", value.kind);
        };
        (arms.clone(), default.clone())
    }

    let with_trailing =
        parse_source("<?php $x = match($n) { 'a', 'b', => 1, default => 0 };");
    let without_trailing =
        parse_source("<?php $x = match($n) { 'a', 'b' => 1, default => 0 };");

    let (arms_a, default_a) = extract_match(&with_trailing);
    let (arms_b, default_b) = extract_match(&without_trailing);

    assert_eq!(arms_a.len(), 1);
    assert_eq!(arms_a.len(), arms_b.len());
    assert_eq!(arms_a[0].0.len(), 2, "trailing comma must not add a pattern");
    assert_eq!(arms_a[0].0.len(), arms_b[0].0.len());
    for (pattern_a, pattern_b) in arms_a[0].0.iter().zip(arms_b[0].0.iter()) {
        assert_eq!(pattern_a.kind, pattern_b.kind);
    }
    assert_eq!(arms_a[0].1.kind, arms_b[0].1.kind);
    assert_eq!(default_a.is_some(), default_b.is_some());
}

/// Verifies that standalone `match` expressions parse as expression statements.
#[test]
fn test_parse_standalone_match_expression_statement() {
    let stmts = parse_source("<?php match (1) { 1 => 2 }; echo 3;");
    assert_eq!(stmts.len(), 2);
    match &stmts[0].kind {
        StmtKind::ExprStmt(expr) => assert!(matches!(&expr.kind, ExprKind::Match { .. })),
        other => panic!("expected ExprStmt containing Match, got {:?}", other),
    }
}

// --- Long-form `array(...)` literal ---

/// Verifies that the long-form `array(1, 2, 3)` parses to the same `ArrayLiteral` node as the
/// short `[...]` form — it is the array-literal language construct, not a function call.
#[test]
fn test_parse_long_array_indexed() {
    let stmts = parse_source("<?php $a = array(1, 2, 3);");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        match &value.kind {
            ExprKind::ArrayLiteral(items) => assert_eq!(items.len(), 3),
            other => panic!("expected ArrayLiteral, got {:?}", other),
        }
    } else {
        panic!("expected Assign");
    }
}

/// Verifies that `array("a" => 1)` parses to an `ArrayLiteralAssoc`, matching the short keyed form.
#[test]
fn test_parse_long_array_assoc() {
    let stmts = parse_source("<?php $m = array(\"a\" => 1, \"b\" => 2);");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        assert!(matches!(&value.kind, ExprKind::ArrayLiteralAssoc(_)));
    } else {
        panic!("expected Assign");
    }
}

/// Verifies that an empty long-form `array()` parses to an empty `ArrayLiteral`.
#[test]
fn test_parse_long_array_empty() {
    let stmts = parse_source("<?php $a = array();");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        match &value.kind {
            ExprKind::ArrayLiteral(items) => assert!(items.is_empty()),
            other => panic!("expected empty ArrayLiteral, got {:?}", other),
        }
    } else {
        panic!("expected Assign");
    }
}

/// Verifies that the long form nests like the short form: `array("x" => array(1, 2))` yields an
/// `ArrayLiteralAssoc` whose value is itself an `ArrayLiteral`.
#[test]
fn test_parse_long_array_nested() {
    let stmts = parse_source("<?php $a = array(\"x\" => array(1, 2));");
    assert_eq!(stmts.len(), 1);
    if let StmtKind::Assign { value, .. } = &stmts[0].kind {
        match &value.kind {
            ExprKind::ArrayLiteralAssoc(items) => {
                assert_eq!(items.len(), 1);
                assert!(matches!(items[0].1.kind, ExprKind::ArrayLiteral(_)));
            }
            other => panic!("expected ArrayLiteralAssoc, got {:?}", other),
        }
    } else {
        panic!("expected Assign");
    }
}

// --- Foreach with key => value ---

// --- By-reference array-literal entries ---

/// Verifies that an array literal containing a by-reference entry desugars at parse time
/// into the hidden-temp prelude shape: `$arr = ['a' => 1, 's' => &$v]` becomes an
/// `Assignment` expression whose prelude holds the empty-literal init (`$tmp = []`), the
/// plain keyed assignment (`$tmp['a'] = 1`), and the committed keyed reference bind
/// (`$tmp['s'] = &$v`, `RefAssignToTarget` with `append == false`) IN SOURCE ORDER, and
/// whose value copies the temp into a distinct hidden yield local.
#[test]
fn test_parse_ref_entry_literal_desugars_to_prelude_temp() {
    let stmts = parse_source("<?php $arr = ['a' => 1, 's' => &$v];");
    assert_eq!(stmts.len(), 1);
    let StmtKind::Assign { name, value } = &stmts[0].kind else {
        panic!("expected Assign, got {:?}", stmts[0].kind);
    };
    assert_eq!(name, "arr");
    let ExprKind::Assignment {
        target,
        value,
        prelude,
        result_target,
        ..
    } = &value.kind
    else {
        panic!("expected Assignment desugar, got {:?}", value.kind);
    };
    assert!(result_target.is_none());
    let ExprKind::Variable(yield_name) = &target.kind else {
        panic!("expected Variable yield target");
    };
    assert!(yield_name.starts_with("__elephc_lit_yield_"));
    let ExprKind::Variable(temp_name) = &value.kind else {
        panic!("expected Variable temp value");
    };
    assert!(temp_name.starts_with("__elephc_lit_"));
    assert_eq!(prelude.len(), 3);
    // `$tmp = [];`
    let StmtKind::Assign { name, value } = &prelude[0].kind else {
        panic!("expected temp init Assign, got {:?}", prelude[0].kind);
    };
    assert_eq!(name, temp_name);
    assert!(matches!(&value.kind, ExprKind::ArrayLiteral(items) if items.is_empty()));
    // `$tmp['a'] = 1;`
    let StmtKind::ArrayAssign { array, index, value } = &prelude[1].kind else {
        panic!("expected keyed ArrayAssign, got {:?}", prelude[1].kind);
    };
    assert_eq!(array, temp_name);
    assert_eq!(index.kind, ExprKind::StringLiteral("a".into()));
    assert_eq!(value.kind, ExprKind::IntLiteral(1));
    // `$tmp['s'] = &$v;`
    let StmtKind::RefAssignToTarget { target, source, append } = &prelude[2].kind else {
        panic!("expected RefAssignToTarget, got {:?}", prelude[2].kind);
    };
    assert!(!*append);
    assert_eq!(source.kind, ExprKind::Variable("v".into()));
    let ExprKind::ArrayAccess { array, index } = &target.kind else {
        panic!("expected ArrayAccess ref target");
    };
    assert_eq!(array.kind, ExprKind::Variable(temp_name.clone()));
    assert_eq!(index.kind, ExprKind::StringLiteral("s".into()));
}

/// Verifies that a positional by-reference entry (`[&$v]`) desugars to the committed
/// append-reference statement form: `RefAssignToTarget` with the hidden temp as the
/// CONTAINER target and `append == true`, so PHP's runtime next-integer-key rule applies.
#[test]
fn test_parse_positional_ref_entry_desugars_to_append_ref() {
    let stmts = parse_source("<?php $arr = [&$v];");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::Assignment { prelude, .. } = &value.kind else {
        panic!("expected Assignment desugar, got {:?}", value.kind);
    };
    assert_eq!(prelude.len(), 2);
    let StmtKind::RefAssignToTarget { target, source, append } = &prelude[1].kind else {
        panic!("expected RefAssignToTarget, got {:?}", prelude[1].kind);
    };
    assert!(*append);
    assert_eq!(source.kind, ExprKind::Variable("v".into()));
    assert!(
        matches!(&target.kind, ExprKind::Variable(name) if name.starts_with("__elephc_lit_")),
        "expected hidden temp container target, got {:?}",
        target.kind
    );
}

/// Regression guard: array literals WITHOUT by-reference entries keep the exact
/// pre-desugar AST — a mixed positional/keyed literal still parses to a plain
/// `ArrayLiteralAssoc` with statically promoted integer keys (0 for the leading
/// positional, explicit "a", then 1 for the trailing positional), never an
/// `Assignment` with a prelude.
#[test]
fn test_parse_plain_literal_unchanged_by_ref_entry_support() {
    let stmts = parse_source("<?php $m = [10, \"a\" => 1, 20];");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::ArrayLiteralAssoc(items) = &value.kind else {
        panic!("expected plain ArrayLiteralAssoc, got {:?}", value.kind);
    };
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].0.kind, ExprKind::IntLiteral(0));
    assert_eq!(items[0].1.kind, ExprKind::IntLiteral(10));
    assert_eq!(items[1].0.kind, ExprKind::StringLiteral("a".into()));
    assert_eq!(items[1].1.kind, ExprKind::IntLiteral(1));
    assert_eq!(items[2].0.kind, ExprKind::IntLiteral(1));
    assert_eq!(items[2].1.kind, ExprKind::IntLiteral(20));
}

/// Verifies that the long-form `array('s' => &$v)` literal desugars through the same
/// hidden-temp prelude machinery as the short `[...]` form.
#[test]
fn test_parse_long_array_ref_entry_desugars() {
    let stmts = parse_source("<?php $arr = array(\"s\" => &$v);");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    assert!(
        matches!(&value.kind, ExprKind::Assignment { prelude, .. } if prelude.len() == 2),
        "expected Assignment desugar, got {:?}",
        value.kind
    );
}

/// Verifies the duplicate-key REPLACE guard: a plain KEYED entry appearing after a
/// by-reference entry gains an `unset($tmp[key]);` statement immediately before its
/// assignment (PHP literal construction replaces a colliding ref bucket instead of
/// writing through it), while entries and binds keep their source order.
#[test]
fn test_parse_ref_literal_keyed_after_ref_gains_unset_guard() {
    let stmts = parse_source("<?php $a = ['s' => &$v, 'b' => 2];");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::Assignment { prelude, .. } = &value.kind else {
        panic!("expected Assignment desugar, got {:?}", value.kind);
    };
    // init, ref bind, unset guard, keyed assign
    assert_eq!(prelude.len(), 4);
    assert!(matches!(&prelude[1].kind, StmtKind::RefAssignToTarget { .. }));
    let StmtKind::ExprStmt(guard) = &prelude[2].kind else {
        panic!("expected unset guard ExprStmt, got {:?}", prelude[2].kind);
    };
    let ExprKind::FunctionCall { name, args } = &guard.kind else {
        panic!("expected unset FunctionCall, got {:?}", guard.kind);
    };
    assert_eq!(name.as_str(), "unset");
    assert_eq!(args.len(), 1);
    let ExprKind::ArrayAccess { array, index } = &args[0].kind else {
        panic!("expected hidden-temp element unset target");
    };
    assert!(
        matches!(&array.kind, ExprKind::Variable(name) if name.starts_with("__elephc_lit_")),
        "expected hidden temp receiver, got {:?}",
        array.kind
    );
    assert_eq!(index.kind, ExprKind::StringLiteral("b".into()));
    let StmtKind::ArrayAssign { index, value, .. } = &prelude[3].kind else {
        panic!("expected keyed ArrayAssign, got {:?}", prelude[3].kind);
    };
    assert_eq!(index.kind, ExprKind::StringLiteral("b".into()));
    assert_eq!(value.kind, ExprKind::IntLiteral(2));
}

/// Verifies that a NON-literal duplicate-guard key is bound to a hidden key temporary
/// exactly once: the guard `unset` and the assignment both read `$__elephc_lit_key_*`,
/// so a side-effecting key expression cannot be evaluated twice.
#[test]
fn test_parse_ref_literal_dynamic_key_guard_binds_key_once() {
    let stmts = parse_source("<?php $a = ['s' => &$v, $k => 2];");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::Assignment { prelude, .. } = &value.kind else {
        panic!("expected Assignment desugar, got {:?}", value.kind);
    };
    // init, ref bind, key-temp bind, unset guard, keyed assign
    assert_eq!(prelude.len(), 5);
    let StmtKind::Assign { name: key_temp, value } = &prelude[2].kind else {
        panic!("expected key-temp Assign, got {:?}", prelude[2].kind);
    };
    assert!(key_temp.starts_with("__elephc_lit_key_"));
    assert_eq!(value.kind, ExprKind::Variable("k".into()));
    let StmtKind::ExprStmt(guard) = &prelude[3].kind else {
        panic!("expected unset guard ExprStmt, got {:?}", prelude[3].kind);
    };
    let ExprKind::FunctionCall { args, .. } = &guard.kind else {
        panic!("expected unset FunctionCall");
    };
    let ExprKind::ArrayAccess { index, .. } = &args[0].kind else {
        panic!("expected element unset target");
    };
    assert_eq!(index.kind, ExprKind::Variable(key_temp.clone()));
    let StmtKind::ArrayAssign { index, .. } = &prelude[4].kind else {
        panic!("expected keyed ArrayAssign, got {:?}", prelude[4].kind);
    };
    assert_eq!(index.kind, ExprKind::Variable(key_temp.clone()));
}

/// Regression guard: plain KEYED entries BEFORE the first by-reference entry gain no
/// duplicate guard (nothing to collide with), and positional appends never do — the
/// prelude stays exactly init + per-entry statements.
#[test]
fn test_parse_ref_literal_entries_before_ref_have_no_guard() {
    let stmts = parse_source("<?php $a = ['a' => 1, 's' => &$v, 2];");
    let StmtKind::Assign { value, .. } = &stmts[0].kind else {
        panic!("expected Assign");
    };
    let ExprKind::Assignment { prelude, .. } = &value.kind else {
        panic!("expected Assignment desugar, got {:?}", value.kind);
    };
    // init, keyed assign (no guard), ref bind, positional push (no guard)
    assert_eq!(prelude.len(), 4);
    assert!(matches!(&prelude[1].kind, StmtKind::ArrayAssign { .. }));
    assert!(matches!(&prelude[2].kind, StmtKind::RefAssignToTarget { .. }));
    assert!(matches!(&prelude[3].kind, StmtKind::ArrayPush { .. }));
}
