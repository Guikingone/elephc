//! Purpose:
//! Synthesizes Zend-order binary heap insertion and root removal for SPL containers.
//!
//! Called from:
//! - Heap metadata method registration through `super`.
//!
//! Key details:
//! - Call comparisons with the same operand order as php-src, including equal priorities.
//! - Keep queue data and priority indices aligned and complete state changes before rethrowing.

use super::*;
use crate::names::{CompilerCall, Name};
use crate::parser::ast::{CatchClause, StmtKind};

/// Reads a value from the private heap array.
pub(super) fn value_at(index: Expr) -> Expr {
    compiler_call(CompilerCall::HeapScopeRead, vec![this_expr(), property_access(this_expr(), VALUES), index, bool_expr(false)])
}

/// Reads a queue priority, or the heap value itself for an ordinary heap.
pub(super) fn rank_at(index: Expr, queue: bool) -> Expr {
    if queue {
        compiler_call(CompilerCall::HeapScopeRead, vec![this_expr(), property_access(this_expr(), PRIORITIES), index, bool_expr(true)])
    } else { value_at(index) }
}

/// Copies one logical element, preserving both parts of a priority queue pair.
fn copy_element(to: Expr, from: Expr, queue: bool) -> Vec<Stmt> {
    let mut body = vec![property_array_assign_stmt(this_expr(), VALUES, to.clone(), value_at(from.clone()))];
    if queue { body.push(property_array_assign_stmt(this_expr(), PRIORITIES, to.clone(), rank_at(from, true))); }
    body.push(publish_root_write(to));
    body
}

/// Writes the held element into its final position after a sift.
fn place_element(index: Expr, value: &str, priority: &str, queue: bool) -> Vec<Stmt> {
    let mut body = vec![property_array_assign_stmt(this_expr(), VALUES, index.clone(), var_expr(value))];
    if queue { body.push(property_array_assign_stmt(this_expr(), PRIORITIES, index.clone(), var_expr(priority))); }
    body.push(publish_root_write(index));
    body
}

/// Sifts an inserted element upward, retaining Zend's parent/incoming comparison order.
pub(super) fn insert_body(queue: bool) -> Vec<Stmt> {
    let mut loop_body = vec![
        assign_stmt("parent", binary_expr(binary_expr(var_expr("pos"), BinOp::Sub, int_expr(1)), BinOp::ShiftRight, int_expr(1))),
        if_stmt(not_expr(binary_expr(compare_call(rank_at(var_expr("parent"), queue), var_expr(if queue { "priority" } else { "value" })), BinOp::Lt, int_expr(0))), vec![break_stmt()], None),
    ];
    loop_body.extend(copy_element(var_expr("pos"), var_expr("parent"), queue));
    loop_body.push(assign_stmt("pos", var_expr("parent")));
    let mut body = vec![assert_state(true), lock_stmt(), assign_stmt("pos", count_value()), assign_stmt("parent", int_expr(0)), while_stmt(binary_expr(var_expr("pos"), BinOp::Gt, int_expr(0)), loop_body), property_assign_stmt(this_expr(), COUNT, binary_expr(count_value(), BinOp::Add, int_expr(1)))];
    body.extend(place_element(var_expr("pos"), "value", "priority", queue));
    body.push(expr_stmt(method_call(this_expr(), "__elephcHeapFinish", Vec::new())));
    body.push(return_stmt(bool_expr(true)));
    body
}

/// Sifts the former tail down into the root hole, then removes the extra owned tail slot.
pub(super) fn remove_top_body(queue: bool) -> Vec<Stmt> {
    let mut sift = vec![
        assign_stmt("child", binary_expr(binary_expr(var_expr("parent"), BinOp::Mul, int_expr(2)), BinOp::Add, int_expr(1))),
        if_stmt(binary_expr(not_expr(binary_expr(var_expr("child"), BinOp::StrictEq, count_value())), BinOp::And,
            binary_expr(compare_call(rank_at(binary_expr(var_expr("child"), BinOp::Add, int_expr(1)), queue), rank_at(var_expr("child"), queue)), BinOp::Gt, int_expr(0))), vec![increment_stmt("child")], None),
        if_stmt(not_expr(binary_expr(compare_call(var_expr(if queue { "bottomPriority" } else { "bottom" }), rank_at(var_expr("child"), queue)), BinOp::Lt, int_expr(0))), vec![break_stmt()], None),
    ];
    sift.extend(copy_element(var_expr("parent"), var_expr("child"), queue));
    sift.push(assign_stmt("parent", var_expr("child")));
    let scope = crate::names::generated_local_name("__elephc_heap_retirement");
    let mut mutation = vec![retire_root_slot(VALUES)];
    if queue { mutation.push(retire_root_slot(PRIORITIES)); }
    mutation.extend([
        assign_stmt("limit", binary_expr(binary_expr(count_value(), BinOp::Sub, int_expr(1)), BinOp::ShiftRight, int_expr(1))),
        property_assign_stmt(this_expr(), COUNT, binary_expr(count_value(), BinOp::Sub, int_expr(1))),
    ]);
    let mut nonempty = vec![assign_stmt("bottom", value_at(count_value()))];
    if queue { nonempty.push(assign_stmt("bottomPriority", rank_at(count_value(), true))); }
    nonempty.extend([
        assign_stmt("parent", int_expr(0)), assign_stmt("child", int_expr(0)),
        while_stmt(binary_expr(var_expr("parent"), BinOp::Lt, var_expr("limit")), sift),
        if_stmt(not_expr(binary_expr(var_expr("parent"), BinOp::StrictEq, count_value())),
            place_element(var_expr("parent"), "bottom", "bottomPriority", queue), None),
    ]);
    mutation.push(if_stmt(binary_expr(count_value(), BinOp::Gt, int_expr(0)), nonempty, None));
    mutation.push(expr_stmt(function_call("array_pop", vec![property_access(this_expr(), VALUES)])));
    if queue { mutation.push(expr_stmt(function_call("array_pop", vec![property_access(this_expr(), PRIORITIES)]))); }
    mutation.push(expr_stmt(method_call(this_expr(), "__elephcHeapFinish", Vec::new())));
    vec![
        assert_state(true), lock_stmt(),
        typed_assign_stmt(&scope, TypeExpr::Int, compiler_call(CompilerCall::HeapScopeBegin, vec![
            this_expr(), property_access(this_expr(), VALUES),
            property_access(this_expr(), if queue { PRIORITIES } else { VALUES }), bool_expr(queue),
        ])),
        Stmt::new(StmtKind::Try { try_body: mutation, catches: Vec::new(),
            finally_body: Some(vec![expr_stmt(compiler_call(CompilerCall::HeapScopeEnd, vec![var_expr(&scope)]))]),
        }, crate::span::Span::dummy()),
    ]
}

/// Emits a typed internal operation without declaring a PHP function or method.
fn compiler_call(operation: CompilerCall, args: Vec<Expr>) -> Expr {
    expr(crate::parser::ast::ExprKind::FunctionCall { name: Name::compiler_call(operation), args })
}

/// Publishes a root only after both data and priority writes have completed.
fn publish_root_write(index: Expr) -> Stmt {
    if_stmt(binary_expr(index, BinOp::StrictEq, int_expr(0)),
        vec![expr_stmt(compiler_call(CompilerCall::HeapScopeRootWritten, vec![this_expr()]))], None)
}

/// Releases one original root owner while preserving the old logical count and pending error.
fn retire_root_slot(field: &str) -> Stmt {
    Stmt::new(StmtKind::Try {
        try_body: vec![expr_stmt(compiler_call(CompilerCall::HeapScopeReleaseRoot,
            vec![this_expr(), bool_expr(field == PRIORITIES)]))],
        catches: vec![CatchClause { exception_types: vec![Name::unqualified("Throwable")],
            exception_type_args: Vec::new(), variable: Some("retiredError".to_string()),
            body: vec![
                if_stmt(not_expr(binary_expr(property_access(this_expr(), ERROR), BinOp::StrictEq, null_expr())),
                    vec![expr_stmt(compiler_call(CompilerCall::ExceptionChainMixed,
                        vec![var_expr("retiredError"), property_access(this_expr(), ERROR)]))], None),
                property_assign_stmt(this_expr(), ERROR, var_expr("retiredError")),
            ],
        }], finally_body: None,
    }, crate::span::Span::dummy())
}
