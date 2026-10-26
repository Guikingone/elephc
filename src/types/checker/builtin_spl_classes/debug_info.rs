//! Purpose:
//! Builds owned SPL debug snapshots while hiding synthetic backing properties.
//!
//! Called from:
//! - Storage and filesystem builtin class method builders.
//!
//! Key details:
//! - Visibility keys come from the actual declarations, preserving unrelated user members.
//! - The caller appends PHP's computed private fields after the member snapshot.

use crate::parser::ast::{ClassProperty, Stmt, TypeExpr, Visibility};

use super::common::*;

/// Copies initialized members and removes only the supplied implementation slots.
pub(super) fn member_snapshot(class_name: &str, properties: Vec<ClassProperty>) -> Vec<Stmt> {
    let mut body = vec![typed_assign_stmt(
        "debug",
        TypeExpr::AssocArray {
            key: Box::new(TypeExpr::Str),
            value: Box::new(mixed_type()),
        },
        function_call("get_mangled_object_vars", vec![this_expr()]),
    )];
    for property in properties {
        let key = if property.name.starts_with('\0') {
            property.name
        } else {
            match property.visibility {
                Visibility::Public => property.name,
                Visibility::Protected => format!("\0*\0{}", property.name),
                Visibility::Private => format!("\0{class_name}\0{}", property.name),
            }
        };
        body.push(expr_stmt(function_call("unset", vec![
            array_access(var_expr("debug"), string_expr(&key)),
        ])));
    }
    body
}
