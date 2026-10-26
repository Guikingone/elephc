//! Purpose:
//! Builds public SPL heap readers, extractors and extraction-flag selection.
//!
//! Called from:
//! - Heap and priority queue synthetic method registration.
//!
//! Key details:
//! - Internal storage reads bypass user overrides of count/isEmpty/current.
//! - Priority extraction reads flags after comparison callbacks, matching Zend.

use super::*;

/// Returns the logical live count.
pub(super) fn heap_count_body() -> Vec<Stmt> { return_body(count_value()) }

/// Tests the logical live count.
pub(super) fn heap_is_empty_body() -> Vec<Stmt> { return_body(binary_expr(count_value(), BinOp::StrictEq, int_expr(0))) }

/// Returns the root after validating corruption and the nonempty precondition.
pub(super) fn heap_top_body() -> Vec<Stmt> {
    let mut body = vec![assert_state(false)]; body.extend(empty_error("Can't peek at an empty heap")); body.push(return_stmt(value_at(int_expr(0)))); body
}

/// Retains the root while removal restores the heap and releases the old array slot.
pub(super) fn heap_extract_body() -> Vec<Stmt> {
    let mut body = vec![assert_state(true)]; body.extend(empty_error("Can't extract from an empty heap")); body.extend([assign_stmt("value", value_at(int_expr(0))), expr_stmt(method_call(this_expr(), "__elephcHeapRemoveTop", Vec::new())), return_stmt(var_expr("value"))]); body
}

/// Reads the root directly, including from a corrupted heap as PHP's public current does.
pub(super) fn heap_current_body() -> Vec<Stmt> {
    vec![if_stmt(binary_expr(count_value(), BinOp::StrictEq, int_expr(0)), return_body(null_expr()), None), return_stmt(value_at(int_expr(0)))]
}

/// Returns count minus one, including minus one for an empty heap.
pub(super) fn heap_key_body() -> Vec<Stmt> { return_body(binary_expr(count_value(), BinOp::Sub, int_expr(1))) }

/// Validates mutation even when empty, then removes a nonempty root.
pub(super) fn heap_next_body() -> Vec<Stmt> {
    vec![assert_state(true), if_stmt(binary_expr(count_value(), BinOp::Gt, int_expr(0)), vec![expr_stmt(method_call(this_expr(), "__elephcHeapRemoveTop", Vec::new()))], None)]
}

/// Reports whether the native structure has elements, independently of corruption.
pub(super) fn heap_valid_body() -> Vec<Stmt> { return_body(binary_expr(count_value(), BinOp::Gt, int_expr(0))) }

/// Raises a native empty-heap exception without consulting overridable isEmpty.
fn empty_error(message: &str) -> Vec<Stmt> {
    vec![if_stmt(binary_expr(count_value(), BinOp::StrictEq, int_expr(0)), vec![throw_stmt(new_object_expr("RuntimeException", vec![string_expr(message)]))], None)]
}

/// Reads the masked extraction flags.
pub(super) fn priority_flags_expr() -> Expr { property_access(this_expr(), EXTRACT_FLAGS) }

/// Masks unsupported bits before storing and returning extraction flags.
pub(super) fn priority_set_extract_flags_body() -> Vec<Stmt> {
    vec![assign_stmt("flags", binary_expr(var_expr("flags"), BinOp::BitAnd, int_expr(3))), if_stmt(binary_expr(var_expr("flags"), BinOp::StrictEq, int_expr(0)), vec![throw_stmt(new_object_expr("RuntimeException", vec![string_expr("Must specify at least one extract flag")]))], None), property_assign_stmt(this_expr(), EXTRACT_FLAGS, var_expr("flags")), return_stmt(var_expr("flags"))]
}

/// Returns a queue pair with the currently selected extraction representation.
fn root_output() -> Expr { method_call(this_expr(), "__elephcHeapOutput", vec![value_at(int_expr(0)), rank_at(int_expr(0), true)]) }

/// Peeks at a nonempty consistent queue.
pub(super) fn priority_top_body() -> Vec<Stmt> {
    let mut body = vec![assert_state(false)]; body.extend(empty_error("Can't peek at an empty heap")); body.push(return_stmt(root_output())); body
}

/// Selects output after the sift because a comparison may change extraction flags.
pub(super) fn priority_extract_body() -> Vec<Stmt> {
    let mut body = vec![assert_state(true)]; body.extend(empty_error("Can't extract from an empty heap")); body.extend([assign_stmt("data", value_at(int_expr(0))), assign_stmt("priority", rank_at(int_expr(0), true)), expr_stmt(method_call(this_expr(), "__elephcHeapRemoveTop", Vec::new())), return_stmt(method_call(this_expr(), "__elephcHeapOutput", vec![var_expr("data"), var_expr("priority")]))]); body
}

/// Returns the current queue output, without the corruption validation used by top.
pub(super) fn priority_current_body() -> Vec<Stmt> {
    vec![if_stmt(binary_expr(count_value(), BinOp::StrictEq, int_expr(0)), return_body(null_expr()), None), return_stmt(root_output())]
}

/// Builds data, priority or both from retained arguments and the current mask.
pub(super) fn priority_output_body() -> Vec<Stmt> {
    vec![if_stmt(binary_expr(priority_flags_expr(), BinOp::StrictEq, int_expr(2)), return_body(var_expr("priority")), None), if_stmt(binary_expr(priority_flags_expr(), BinOp::StrictEq, int_expr(3)), return_body(expr(crate::parser::ast::ExprKind::ArrayLiteralAssoc(vec![(string_expr("data"), var_expr("data")), (string_expr("priority"), var_expr("priority"))]))), None), return_stmt(var_expr("data"))]
}
