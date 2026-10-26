//! Purpose:
//! Owns compiler-private SPL heap state and Zend comparison exception bookkeeping.
//!
//! Called from:
//! - Heap class metadata, mutation builders and debug snapshot generation.
//!
//! Key details:
//! - Live count is independent of temporary sift slots; callbacks observe Zend's count.
//! - Comparison exceptions are held until the mutation has finished its required bookkeeping.

use super::*;
use crate::parser::ast::{CastType, CatchClause, StmtKind};
use crate::names::Name;

pub(super) use crate::types::spl_heap_layout::{VALUES, PRIORITIES, COUNT, STATE, ERROR, EXTRACT_FLAGS};

/// Declares private storage with defaults that also work when a user constructor omits parent calls.
pub(super) fn heap_properties(queue: bool) -> Vec<ClassProperty> {
    let mut properties = vec![
        storage_property_default(VALUES, array_type(), empty_array_expr()),
        storage_property_default(COUNT, TypeExpr::Int, int_expr(0)),
        storage_property_default(STATE, TypeExpr::Int, int_expr(0)),
        storage_property_default(ERROR, mixed_type(), null_expr()),
    ];
    if queue {
        properties.push(storage_property_default(PRIORITIES, array_type(), empty_array_expr()));
        properties.push(storage_property_default(EXTRACT_FLAGS, TypeExpr::Int, int_expr(1)));
    }
    properties
}

/// Initializes the same state for an explicitly invoked synthetic constructor.
pub(super) fn construct_body(queue: bool) -> Vec<Stmt> {
    let mut body = vec![
        property_assign_stmt(this_expr(), VALUES, empty_array_expr()),
        property_assign_stmt(this_expr(), COUNT, int_expr(0)),
        property_assign_stmt(this_expr(), STATE, int_expr(0)),
        property_assign_stmt(this_expr(), ERROR, null_expr()),
    ];
    if queue {
        body.push(property_assign_stmt(this_expr(), PRIORITIES, empty_array_expr()));
        body.push(property_assign_stmt(this_expr(), EXTRACT_FLAGS, int_expr(1)));
    }
    body
}

/// Reads the logical live element count rather than the temporary array capacity.
pub(super) fn count_value() -> Expr { property_access(this_expr(), COUNT) }

/// Tests the corrupted bit independently from the write lock and queue extraction flags.
pub(super) fn corrupted_expr() -> Expr {
    not_expr(binary_expr(binary_expr(property_access(this_expr(), STATE), BinOp::BitAnd, int_expr(1)), BinOp::StrictEq, int_expr(0)))
}

/// Invokes native consistency checks at the public mutating or peek entry point.
pub(super) fn assert_state(write: bool) -> Stmt {
    expr_stmt(method_call(this_expr(), "__elephcHeapAssert", vec![bool_expr(write)]))
}

/// Implements Zend's corruption check followed by the optional write-lock check.
pub(super) fn consistency_body() -> Vec<Stmt> {
    vec![
        if_stmt(corrupted_expr(), vec![throw_stmt(new_object_expr("RuntimeException", vec![string_expr("Heap is corrupted, heap properties are no longer ensured.")]))], None),
        if_stmt(binary_expr(var_expr("write"), BinOp::And, not_expr(binary_expr(binary_expr(property_access(this_expr(), STATE), BinOp::BitAnd, int_expr(2)), BinOp::StrictEq, int_expr(0)))), vec![throw_stmt(new_object_expr("RuntimeException", vec![string_expr("Heap cannot be changed when it is already being modified.")]))], None),
    ]
}

/// Sets the write lock without changing an existing corruption bit.
pub(super) fn lock_stmt() -> Stmt {
    property_assign_stmt(this_expr(), STATE, binary_expr(property_access(this_expr(), STATE), BinOp::BitOr, int_expr(2)))
}

/// Calls the virtual comparison once and captures its exception like Zend's pending exception slot.
pub(super) fn compare_body() -> Vec<Stmt> {
    vec![
        if_stmt(not_expr(binary_expr(property_access(this_expr(), ERROR), BinOp::StrictEq, null_expr())), return_body(int_expr(0)), None),
        Stmt::new(StmtKind::Try {
            try_body: return_body(cast_expr(CastType::Int, method_call(this_expr(), "compare", vec![var_expr("left"), var_expr("right")]))),
            catches: vec![CatchClause {
                exception_types: vec![Name::unqualified("Throwable")],
                exception_type_args: Vec::new(),
                variable: Some("caught".to_string()),
                body: vec![property_assign_stmt(this_expr(), ERROR, var_expr("caught")), return_stmt(int_expr(0))],
            }],
            finally_body: None,
        }, crate::span::Span::dummy()),
    ]
}

/// Releases the lock, marks corruption and rethrows only after the sift has finished.
pub(super) fn finish_body() -> Vec<Stmt> {
    vec![
        property_assign_stmt(this_expr(), STATE, binary_expr(property_access(this_expr(), STATE), BinOp::BitAnd, int_expr(-3))),
        if_stmt(not_expr(binary_expr(property_access(this_expr(), ERROR), BinOp::StrictEq, null_expr())), vec![
            property_assign_stmt(this_expr(), STATE, binary_expr(property_access(this_expr(), STATE), BinOp::BitOr, int_expr(1))),
            assign_stmt("error", property_access(this_expr(), ERROR)),
            property_assign_stmt(this_expr(), ERROR, null_expr()),
            throw_stmt(var_expr("error")),
        ], None),
    ]
}

/// Clears corruption without rebuilding or reordering the heap, preserving an active write lock.
pub(super) fn recover_body() -> Vec<Stmt> {
    vec![property_assign_stmt(this_expr(), STATE, binary_expr(property_access(this_expr(), STATE), BinOp::BitAnd, int_expr(-2))), return_stmt(bool_expr(true))]
}

/// Compares two values through the exception-aware private hook.
pub(super) fn compare_call(left: Expr, right: Expr) -> Expr {
    method_call(this_expr(), "__elephcHeapCompare", vec![left, right])
}

/// Terminates a generated sift loop without introducing another observable callback.
pub(super) fn break_stmt() -> Stmt { Stmt::new(StmtKind::Break(1), crate::span::Span::dummy()) }
