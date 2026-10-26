//! Purpose:
//! Builds the native SPL heap serialization pair from user members and logical heap storage.
//!
//! Called from:
//! - Synthetic SplHeap and SplPriorityQueue method registration.
//!
//! Key details:
//! - Reject corruption and active mutation before copying any members.
//! - Preserve physical heap order and queue data-before-priority entry order.
//! - The PHP array contract boxes packed and associative child arrays without exposing backing slots.

use super::*;

/// Encodes the native [properties, state] pair without dispatching user debug hooks.
pub(super) fn serialize_body(queue: bool) -> Vec<Stmt> {
    let owner = if queue { "SplPriorityQueue" } else { "SplHeap" };
    let mut body = vec![
        assert_state(false),
        if_stmt(not_expr(binary_expr(
            binary_expr(property_access(this_expr(), STATE), BinOp::BitAnd, int_expr(2)),
            BinOp::StrictEq, int_expr(0),
        )), vec![throw_stmt(new_object_expr("RuntimeException", vec![
            string_expr("Cannot serialize heap while it is being modified."),
        ]))], None),
    ];
    body.extend(super::super::debug_info::member_snapshot(owner, heap_properties(queue)));
    body.extend([
        typed_assign_stmt("entries", array_type(), empty_array_expr()),
        assign_stmt("i", int_expr(0)),
    ]);
    let entry = if queue {
        expr(crate::parser::ast::ExprKind::ArrayLiteralAssoc(vec![
            (string_expr("data"), value_at(var_expr("i"))),
            (string_expr("priority"), rank_at(var_expr("i"), true)),
        ]))
    } else {
        value_at(var_expr("i"))
    };
    body.push(while_stmt(binary_expr(var_expr("i"), BinOp::Lt, count_value()), vec![
        array_push_stmt("entries", entry), increment_stmt("i"),
    ]));
    body.extend([
        typed_assign_stmt("state", TypeExpr::AssocArray {
            key: Box::new(TypeExpr::Str), value: Box::new(mixed_type()),
        }, empty_assoc_array_expr()),
        array_assign_stmt("state", string_expr("flags"), if queue { priority_flags_expr() } else { int_expr(0) }),
        array_assign_stmt("state", string_expr("heap_elements"), var_expr("entries")),
        typed_assign_stmt("data", php_array_contract_type(), empty_array_expr()),
        array_push_stmt("data", var_expr("debug")),
        array_push_stmt("data", var_expr("state")),
        return_stmt(var_expr("data")),
    ]);
    body
}
