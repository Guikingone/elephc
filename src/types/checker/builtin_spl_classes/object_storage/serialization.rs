//! Purpose:
//! Builds PHP-compatible SplObjectStorage magic serialization and hydration bodies.
//!
//! Called from:
//! - The parent module's synthetic method registry.
//!
//! Key details:
//! - Native entry arrays and property hashes meet through the boxed PHP array contract.
//! - Hydration merges entries, validates in PHP order, and retains iterator position.

use super::*;

/// Keeps mixed packed/hash payloads boxed instead of selecting one homogeneous layout.
fn php_array_type() -> TypeExpr {
    TypeExpr::Union(vec![
        array_type(),
        TypeExpr::AssocArray { key: Box::new(mixed_type()), value: Box::new(mixed_type()) },
    ])
}

/// Returns PHP's pair of interleaved entries and mangled user-property data.
pub(super) fn serialize_array_body() -> Vec<Stmt> {
    vec![
        typed_assign_stmt("entries", array_type(), empty_array_expr()),
        assign_stmt("i", int_expr(0)),
        while_stmt(binary_expr(var_expr("i"), BinOp::Lt, count_expr(objects_expr())), vec![
            array_push_stmt("entries", object_at(var_expr("i"))),
            array_push_stmt("entries", info_at(var_expr("i"))),
            increment_stmt("i"),
        ]),
        typed_assign_stmt("data", php_array_type(), empty_array_expr()),
        array_push_stmt("data", var_expr("entries")),
        array_push_stmt("data", function_call("get_mangled_object_vars", vec![this_expr()])),
        return_stmt(var_expr("data")),
    ]
}

/// Restores entry identity/info and declared or dynamic user properties.
pub(super) fn unserialize_array_body() -> Vec<Stmt> {
    let malformed = || vec![throw_stmt(new_object_expr("UnexpectedValueException", vec![
        string_expr("Incomplete or ill-typed serialization data"),
    ]))];
    let mut body = vec![
        if_stmt(not_expr(binary_expr(
            function_call("array_key_exists", vec![int_expr(0), var_expr("data")]), BinOp::And,
            function_call("array_key_exists", vec![int_expr(1), var_expr("data")]),
        )), malformed(), None),
        if_stmt(not_expr(binary_expr(
            function_call("is_array", vec![array_access(var_expr("data"), int_expr(0))]), BinOp::And,
            function_call("is_array", vec![array_access(var_expr("data"), int_expr(1))]),
        )), malformed(), None),
    ];
    body.extend([
        assign_stmt("entries", array_access(var_expr("data"), int_expr(0))),
        assign_stmt("props", array_access(var_expr("data"), int_expr(1))),
        if_stmt(binary_expr(binary_expr(count_expr(var_expr("entries")), BinOp::Mod, int_expr(2)), BinOp::NotEq, int_expr(0)), vec![
            throw_stmt(new_object_expr("UnexpectedValueException", vec![string_expr("Odd number of elements")])),
        ], None),
        assign_stmt("i", int_expr(0)),
        while_stmt(binary_expr(var_expr("i"), BinOp::Lt, count_expr(var_expr("entries"))), {
            let mut entry = vec![
                assign_stmt("object", array_access(var_expr("entries"), var_expr("i"))),
                assign_stmt("info", array_access(var_expr("entries"), binary_expr(var_expr("i"), BinOp::Add, int_expr(1)))),
                if_stmt(not_expr(function_call("is_object", vec![var_expr("object")])), vec![
                    throw_stmt(new_object_expr("UnexpectedValueException", vec![string_expr("Non-object key")])),
                ], None),
            ];
            // Inline the storage operation: PHP hydration does not dispatch an override of attach().
            entry.push(assign_stmt("index", index_of_expr(var_expr("object"))));
            entry.push(if_stmt(binary_expr(var_expr("index"), BinOp::GtEq, int_expr(0)),
                vec![property_array_assign_stmt(this_expr(), INFOS_PROPERTY, var_expr("index"), var_expr("info"))],
                Some(vec![property_array_push_stmt(this_expr(), OBJECTS_PROPERTY, var_expr("object")),
                    property_array_push_stmt(this_expr(), INFOS_PROPERTY, var_expr("info"))])));
            entry.extend([
                assign_stmt("i", binary_expr(var_expr("i"), BinOp::Add, int_expr(2))),
            ]);
            entry
        }),
        foreach_stmt(var_expr("props"), Some("name"), "value", vec![
            expr_stmt(function_call("__elephc_object_restore_property", vec![
                this_expr(), var_expr("name"), var_expr("value"),
            ])),
        ]),
    ]);
    body
}
