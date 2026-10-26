//! Purpose:
//! Builds the PHP-visible SplObjectStorage debug snapshot from its native backing slots.
//!
//! Called from:
//! - `super::spl_object_storage_methods()` when declaring the injected `__debugInfo` body.
//!
//! Key details:
//! - PHP's outer array carries mangled member names and one private storage row.
//! - Fresh obj/inf entry arrays preserve value ownership and snapshot copy-on-write.
//! - The body reads backing slots directly and never invokes an overridden serialization hook.

use crate::parser::ast::{BinOp, ExprKind, Stmt, TypeExpr, Visibility};

use super::super::common::*;

/// Returns user members followed by PHP's private array of attached obj/inf pairs.
pub(super) fn debug_info_body() -> Vec<Stmt> {
    let mut body = vec![typed_assign_stmt(
        "debug",
        TypeExpr::AssocArray {
            key: Box::new(TypeExpr::Str),
            value: Box::new(mixed_type()),
        },
        function_call("get_mangled_object_vars", vec![this_expr()]),
    )];
    // Derive bookkeeping keys from the real backing declarations so a user member with
    // another visibility or a different declaring class keeps its own mangled name.
    for property in super::spl_object_storage_properties() {
        let key = if property.name.starts_with('\0') {
            property.name.clone()
        } else {
            match property.visibility {
                Visibility::Public => property.name.clone(),
                Visibility::Protected => format!("\0*\0{}", property.name),
                Visibility::Private => format!("\0SplObjectStorage\0{}", property.name),
            }
        };
        body.push(expr_stmt(function_call("unset", vec![
            array_access(var_expr("debug"), string_expr(&key)),
        ])));
    }
    body.extend([
        typed_assign_stmt("storage", array_type(), empty_array_expr()),
        assign_stmt("i", int_expr(0)),
        while_stmt(
            binary_expr(var_expr("i"), BinOp::Lt, count_expr(super::objects_expr())),
            vec![
                array_push_stmt("storage", expr(ExprKind::ArrayLiteralAssoc(vec![
                    (string_expr("obj"), array_access(super::objects_expr(), var_expr("i"))),
                    (string_expr("inf"), array_access(super::infos_expr(), var_expr("i"))),
                ]))),
                increment_stmt("i"),
            ],
        ),
        array_assign_stmt(
            "debug", string_expr("\0SplObjectStorage\0storage"), var_expr("storage"),
        ),
        return_stmt(var_expr("debug")),
    ]);
    body
}
