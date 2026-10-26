//! Purpose:
//! Builds ArrayObject and ArrayIterator debug views from their native backing values.
//!
//! Called from:
//! - `super::spl_array_object_methods()` and `super::spl_array_iterator_methods()`.
//!
//! Key details:
//! - The storage row keeps the raw array/object identity without calling public overrides.
//! - Self-backed containers expose members alone, matching SPL_ARRAY_IS_SELF.

use crate::parser::ast::{BinOp, Stmt};

use super::super::common::*;

/// Copies user members and appends the canonical private storage row when appropriate.
pub(super) fn snapshot_body(class_name: &str) -> Vec<Stmt> {
    let properties = if class_name == "ArrayIterator" {
        super::array_iterator_properties()
    } else {
        super::array_object_properties()
    };
    let mut body = super::super::debug_info::member_snapshot(class_name, properties);
    body.push(if_stmt(
        binary_expr(super::storage_expr(), BinOp::StrictNotEq, this_expr()),
        vec![array_assign_stmt(
            "debug",
            string_expr(&format!("\0{class_name}\0storage")),
            super::storage_expr(),
        )],
        None,
    ));
    body.push(return_stmt(var_expr("debug")));
    body
}
