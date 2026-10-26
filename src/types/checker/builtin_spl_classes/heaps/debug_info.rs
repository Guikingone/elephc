//! Purpose:
//! Builds SPL heap debug snapshots with Zend's private keys and physical array order.
//!
//! Called from:
//! - Synthetic SplHeap and SplPriorityQueue __debugInfo method registration.
//!
//! Key details:
//! - Copy only the logical live prefix, excluding temporary sift and tail slots.
//! - Preserve user members and keep the returned array independent through normal COW.

use super::*;

/// Copies user members and appends the three native fields without exposing backing slots.
pub(super) fn debug_info_body(queue: bool) -> Vec<Stmt> {
    let owner = if queue { "SplPriorityQueue" } else { "SplHeap" };
    let mut body = super::super::debug_info::member_snapshot(owner, heap_properties(queue));
    body.extend([typed_assign_stmt("heap", array_type(), empty_array_expr()), assign_stmt("i", int_expr(0))]);
    let value = if queue {
        expr(crate::parser::ast::ExprKind::ArrayLiteralAssoc(vec![(string_expr("data"), value_at(var_expr("i"))), (string_expr("priority"), rank_at(var_expr("i"), true))]))
    } else { value_at(var_expr("i")) };
    body.push(while_stmt(binary_expr(var_expr("i"), BinOp::Lt, count_value()), vec![array_push_stmt("heap", value), increment_stmt("i")]));
    body.extend([
        array_assign_stmt("debug", string_expr(&format!("\0{owner}\0flags")), if queue { priority_flags_expr() } else { int_expr(0) }),
        array_assign_stmt("debug", string_expr(&format!("\0{owner}\0isCorrupted")), corrupted_expr()),
        array_assign_stmt("debug", string_expr(&format!("\0{owner}\0heap")), var_expr("heap")),
        return_stmt(var_expr("debug")),
    ]);
    body
}
