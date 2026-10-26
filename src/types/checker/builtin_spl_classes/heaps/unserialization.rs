//! Purpose:
//! Builds the public native SPL heap loading protocol in pinned php-src validation order.
//!
//! Called from:
//! - SplHeap and SplPriorityQueue synthetic method registration.
//!
//! Key details:
//! - Properties are loaded before state validation, with pending destructor errors chained.
//! - Restoration appends through the native insertion body without dispatching insert overrides.
//! - Queue flags publish before element validation; regular heaps validate the write lock first.

use super::*;
use crate::names::{CompilerCall, Name};
use crate::parser::ast::{CatchClause, StmtKind};

/// Restores user properties and then appends the serialized physical heap entries.
pub(super) fn unserialize_body(queue: bool) -> Vec<Stmt> {
    let mut body = Vec::new();
    if !queue { body.push(assert_state(true)); }
    body.extend([
        require(binary_expr(count_expr(var_expr("data")), BinOp::StrictEq, int_expr(2))),
        require(has_key(int_expr(0), var_expr("data"))),
        assign_stmt("props", array_access(var_expr("data"), int_expr(0))),
        require(function_call("is_array", vec![var_expr("props")])),
        typed_assign_stmt("pendingPropertyError", mixed_type(), null_expr()),
        catching(vec![foreach_stmt(var_expr("props"), Some("propertyName"), "propertyValue", vec![
            typed_assign_stmt("propertyError", mixed_type(), compiler_call(
                CompilerCall::NativePropertyRestore,
                vec![this_expr(), function_call("strval", vec![var_expr("propertyName")]), var_expr("propertyValue")],
            )),
            if_stmt(instanceof_expr(var_expr("propertyError"), "Throwable"),
                accumulate("propertyError"), None),
        ])], accumulate("unserializeError")),
        if_stmt(instanceof_expr(var_expr("pendingPropertyError"), "Throwable"),
            invalid(Some("pendingPropertyError")), None),
        require(has_key(int_expr(1), var_expr("data"))),
        assign_stmt("state", array_access(var_expr("data"), int_expr(1))),
        require(function_call("is_array", vec![var_expr("state")])),
        require(has_key(string_expr("flags"), var_expr("state"))),
        assign_stmt("rawFlags", array_access(var_expr("state"), string_expr("flags"))),
        require(function_call("is_int", vec![var_expr("rawFlags")])),
        typed_assign_stmt("flags", TypeExpr::Int, function_call("intval", vec![var_expr("rawFlags")])),
    ]);
    if queue {
        body.extend([
            assign_stmt("flags", binary_expr(var_expr("flags"), BinOp::BitAnd, int_expr(3))),
            require(not_expr(binary_expr(var_expr("flags"), BinOp::StrictEq, int_expr(0)))),
            property_assign_stmt(this_expr(), EXTRACT_FLAGS, var_expr("flags")),
        ]);
    } else {
        body.push(require(binary_expr(var_expr("flags"), BinOp::StrictEq, int_expr(0))));
    }
    body.extend([
        require(has_key(string_expr("heap_elements"), var_expr("state"))),
        assign_stmt("elements", array_access(var_expr("state"), string_expr("heap_elements"))),
        require(function_call("is_array", vec![var_expr("elements")])),
    ]);
    let mut entry = if queue {
        vec![
            require(function_call("is_array", vec![var_expr("element")])),
            require(binary_expr(count_expr(var_expr("element")), BinOp::StrictEq, int_expr(2))),
            require(has_key(string_expr("data"), var_expr("element"))),
            require(has_key(string_expr("priority"), var_expr("element"))),
            assign_stmt("value", array_access(var_expr("element"), string_expr("data"))),
            assign_stmt("priority", array_access(var_expr("element"), string_expr("priority"))),
        ]
    } else { vec![assign_stmt("value", var_expr("element"))] };
    let mut insertion = insert_body(queue);
    // The public insert precondition and return are wrappers around the shared native sift.
    // Calling that wrapper would add an observable consistency check and dispatch surface.
    assert!(matches!(insertion.pop().map(|s| s.kind), Some(StmtKind::Return(_))));
    insertion.remove(0);
    entry.push(catching(insertion, invalid(Some("unserializeError"))));
    body.push(foreach_stmt(var_expr("elements"), None, "element", entry));
    body.push(assert_state(false));
    body
}

/// Tests existence independently from a null payload.
fn has_key(key: Expr, array: Expr) -> Expr { function_call("array_key_exists", vec![key, array]) }

/// Rejects malformed data without rolling back earlier native writes.
fn require(valid: Expr) -> Stmt { if_stmt(not_expr(valid), invalid(None), None) }

/// Preserves the newest pending Throwable and appends any previous property error.
fn accumulate(error: &str) -> Vec<Stmt> {
    vec![
        if_stmt(instanceof_expr(var_expr("pendingPropertyError"), "Throwable"), vec![
            expr_stmt(compiler_call(CompilerCall::ExceptionChainMixed,
                vec![var_expr(error), var_expr("pendingPropertyError")])),
        ], None),
        typed_assign_stmt("pendingPropertyError", mixed_type(), var_expr(error)),
    ]
}

/// Wraps native loading failures with the runtime receiver class and original pending exception.
fn invalid(previous: Option<&str>) -> Vec<Stmt> {
    let message = binary_expr(binary_expr(string_expr("Invalid serialization data for "), BinOp::Concat,
        function_call("get_class", vec![this_expr()])), BinOp::Concat, string_expr(" object"));
    let exception = new_object_expr("Exception", vec![message]);
    if let Some(previous) = previous {
        vec![
            typed_assign_stmt("invalidDataError", named_type("Exception"), exception),
            expr_stmt(compiler_call(CompilerCall::ExceptionChainMixed,
                vec![var_expr("invalidDataError"), var_expr(previous)])),
            throw_stmt(var_expr("invalidDataError")),
        ]
    } else { vec![throw_stmt(exception)] }
}

/// Captures native validation/destructor failure at the owning protocol boundary.
fn catching(try_body: Vec<Stmt>, body: Vec<Stmt>) -> Stmt {
    Stmt::new(StmtKind::Try { try_body,
        catches: vec![CatchClause { exception_types: vec![Name::unqualified("Throwable")],
            exception_type_args: Vec::new(), variable: Some("unserializeError".into()), body }],
        finally_body: None,
    }, crate::span::Span::dummy())
}

/// Attaches compiler authority without a PHP-visible function spelling.
fn compiler_call(operation: CompilerCall, args: Vec<Expr>) -> Expr {
    expr(crate::parser::ast::ExprKind::FunctionCall { name: Name::compiler_call(operation), args })
}
