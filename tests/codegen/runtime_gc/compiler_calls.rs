//! Purpose:
//! Exercises compiler-only calls through AST rewrites, checking, EIR and native execution.
//!
//! Called from:
//! - Focused runtime GC integration tests.
//!
//! Key details:
//! - The original PHP program is an identity oracle; only its trusted AST is transformed.
//! - Calls have no PHP declaration or name-based capability and must leave no heap owners.

use crate::support::*;
use elephc::names::{CompilerCall, Name};
use elephc::parser::ast::{Expr, ExprKind, Program, StmtKind};

/// Replaces the identity function's local operations with a complete borrowed snapshot scope.
fn inject_snapshot_scope(program: &mut Program) {
    let mut changed = 0;
    for statement in program {
        let StmtKind::FunctionDecl { name, body, .. } = &mut statement.kind else { continue; };
        if name != "roundtrip" { continue; }
        assert_eq!(body.len(), 4);
        for (index, target, variable) in [
            (0, CompilerCall::BorrowedValueSnapshot, "value"),
            (1, CompilerCall::BorrowedValueRead, "scope"),
        ] {
            let StmtKind::Assign { value, .. } = &mut body[index].kind else { panic!("identity assignment missing"); };
            value.kind = ExprKind::FunctionCall { name: Name::compiler_call(target),
                args: vec![Expr::new(ExprKind::Variable(variable.to_string()), value.span)] };
        }
        let StmtKind::ExprStmt(expression) = &mut body[2].kind else { panic!("identity unset missing"); };
        expression.kind = ExprKind::FunctionCall { name: Name::compiler_call(CompilerCall::BorrowedValueFree),
            args: vec![Expr::new(ExprKind::Variable("scope".to_string()), expression.span)] };
        changed += 1;
    }
    assert_eq!(changed, 1);
}

/// Compiler capabilities survive the real frontend while ordinary PHP strings remain unbound.
#[test]
fn test_compiler_runtime_calls_keep_values_and_private_identity() {
    let source = r#"<?php
namespace CompilerBridge;
function roundtrip(mixed $value): mixed {
    $scope = 0;
    $result = $value;
    unset($scope);
    return $result;
}
echo roundtrip(7), '|';
echo roundtrip(false) === false ? 'false|' : 'bad|';
echo bin2hex(roundtrip("a\0b")), '|';
$array = roundtrip(['x' => 3]); echo $array['x'], '|';
$object = new \stdClass(); echo roundtrip($object) === $object ? 'object|' : 'bad|';
$stream = fopen('php://temp', 'w+'); echo roundtrip($stream) === $stream ? 'resource|' : 'bad|';
fclose($stream);
echo is_object(roundtrip(new \stdClass())) ? 'temporary|' : 'bad|';
$temporary = roundtrip(fopen('php://temp', 'w+'));
echo is_resource($temporary) ? 'owned-resource|' : 'bad|';
fclose($temporary);
echo function_exists('__compiler@BorrowedValueSnapshot') ? 'bad' : 'private';
"#;
    let run = || with_fixture_ast_transform(inject_snapshot_scope, || {
        let output = compile_and_run_with_heap_debug(source);
        assert!(output.success, "{}\n{}", output.stdout, output.stderr);
        assert_eq!(output.stdout, "7|false|610062|3|object|resource|temporary|owned-resource|private", "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    });
    run();
    without_ir_opt(run);
}
