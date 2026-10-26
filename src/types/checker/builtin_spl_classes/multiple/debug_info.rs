//! Purpose:
//! Builds MultipleIterator's SplObjectStorage-compatible debug snapshot.
//!
//! Called from:
//! - MultipleIterator synthetic method registration.
//!
//! Key details:
//! - Zend reuses SplObjectStorage's private storage key and obj/inf pair schema.
//! - User properties survive; parallel implementation arrays and flags stay hidden.

use super::*;

/// Copies iterator/info pairs into an independent owned native debug view.
pub(super) fn body() -> Vec<Stmt> {
    let mut body = super::super::debug_info::member_snapshot("MultipleIterator", multiple_iterator_properties());
    body.extend([
        typed_assign_stmt("storage", array_type(), empty_array_expr()),
        assign_stmt("i", int_expr(0)),
        while_stmt(binary_expr(var_expr("i"), BinOp::Lt, count_expr(multiple_iterators_expr())), vec![
            array_push_stmt("storage", expr(ExprKind::ArrayLiteralAssoc(vec![
                (string_expr("obj"), multiple_iterator_at(var_expr("i"))),
                (string_expr("inf"), multiple_info_at(var_expr("i"))),
            ]))),
            increment_stmt("i"),
        ]),
        array_assign_stmt("debug", string_expr("\0SplObjectStorage\0storage"), var_expr("storage")),
        return_stmt(var_expr("debug")),
    ]);
    body
}
