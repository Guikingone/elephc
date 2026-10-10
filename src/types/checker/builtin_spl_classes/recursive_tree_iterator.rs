//! Purpose:
//! Injects `RecursiveTreeIterator` metadata and its prefix/postfix synthetic method bodies.
//! Builds the ASCII tree prefix php renders in front of each visited entry.
//!
//! Called from:
//! - `super::inject_builtin_spl_classes()`.
//!
//! Key details:
//! - The constructor wraps the source in a `RecursiveCachingIterator` so `getPrefix()` can ask
//!   each active level whether it has a next sibling (`hasNext()`), matching php's traversal.
//! - `getSubIterator()` is read as `mixed`, because the active sub-iterator is a caching iterator
//!   whose `hasNext()` is not part of the `RecursiveIterator` interface php declares it as.

use std::collections::HashMap;

use crate::parser::ast::{
    BinOp, CastType, ClassConst, ClassMethod, ClassProperty, Expr, ExprKind, StaticReceiver, Stmt,
    TypeExpr,
};
use crate::types::traits::FlattenedClass;

use super::common::*;

/// Builds `$condition ? $then_expr : $else_expr`.
fn ternary(condition: Expr, then_expr: Expr, else_expr: Expr) -> Expr {
    expr(ExprKind::Ternary {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
    })
}

/// Builds `parent::$method($args)`.
fn parent_call(method: &str, args: Vec<Expr>) -> Expr {
    expr(ExprKind::StaticMethodCall {
        receiver: StaticReceiver::Parent,
        method: method.to_string(),
        args,
    })
}

/// The prefix part slots php exposes as `RecursiveTreeIterator::PREFIX_*`.
const PREFIX_PART_COUNT: i64 = 6;

/// The `RecursiveTreeIterator::BYPASS_CURRENT` flag bit.
const BYPASS_CURRENT: i64 = 4;

/// The `RecursiveTreeIterator::BYPASS_KEY` flag bit.
const BYPASS_KEY: i64 = 8;

/// Inserts `RecursiveTreeIterator` into the builtin class registry.
pub(super) fn insert_class(class_map: &mut HashMap<String, FlattenedClass>) {
    class_map.insert(
        "RecursiveTreeIterator".to_string(),
        FlattenedClass {
            name: "RecursiveTreeIterator".to_string(),
            span: crate::span::Span::dummy(),
            extends: Some("RecursiveIteratorIterator".to_string()),
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: tree_iterator_properties(),
            methods: tree_iterator_methods(),
            attributes: Vec::new(),
            constants: tree_iterator_constants(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );
}

/// Builds `RecursiveTreeIterator` storage properties.
fn tree_iterator_properties() -> Vec<ClassProperty> {
    vec![
        storage_property("prefix", array_type()),
        storage_property("postfix", TypeExpr::Str),
        // The tree's own BYPASS_* flags. Kept separate from the inherited traversal `flags`
        // slot so the subclass owns a declared per-instance property the backend can write.
        storage_property("treeFlags", TypeExpr::Int),
    ]
}

/// Builds `RecursiveTreeIterator` constants.
fn tree_iterator_constants() -> Vec<ClassConst> {
    vec![
        class_const("BYPASS_CURRENT", BYPASS_CURRENT),
        class_const("BYPASS_KEY", BYPASS_KEY),
        class_const("PREFIX_LEFT", 0),
        class_const("PREFIX_MID_HAS_NEXT", 1),
        class_const("PREFIX_MID_LAST", 2),
        class_const("PREFIX_END_HAS_NEXT", 3),
        class_const("PREFIX_END_LAST", 4),
        class_const("PREFIX_RIGHT", 5),
    ]
}

/// Builds `RecursiveTreeIterator` methods.
fn tree_iterator_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body(
            "__construct",
            vec![
                // php declares this `Traversable`. It is typed `mixed` here so an
                // `IteratorAggregate` source can be unwrapped through `getIterator()` without the
                // checker refusing the call, and the caching wrapper below still enforces the
                // `RecursiveIterator` contract.
                param("iterator", mixed_type()),
                param_default("flags", TypeExpr::Int, int_expr(BYPASS_KEY)),
                param_default("cachingIteratorFlags", TypeExpr::Int, int_expr(16)),
                param_default("mode", TypeExpr::Int, int_expr(1)),
            ],
            Some(TypeExpr::Void),
            tree_iterator_construct_body(),
        ),
        method_with_body(
            "getPrefix",
            Vec::new(),
            Some(TypeExpr::Str),
            tree_iterator_get_prefix_body(),
        ),
        method_with_body(
            "setPrefixPart",
            vec![param("part", TypeExpr::Int), param("prefix", TypeExpr::Str)],
            Some(TypeExpr::Void),
            tree_iterator_set_prefix_part_body(),
        ),
        method_with_body(
            "getEntry",
            Vec::new(),
            Some(TypeExpr::Str),
            tree_iterator_get_entry_body(),
        ),
        method_with_body(
            "setPostfix",
            vec![param("postfix", TypeExpr::Str)],
            Some(TypeExpr::Void),
            tree_iterator_set_postfix_body(),
        ),
        method_with_body(
            "getPostfix",
            Vec::new(),
            Some(TypeExpr::Str),
            tree_iterator_get_postfix_body(),
        ),
        method_with_body(
            "current",
            Vec::new(),
            Some(mixed_type()),
            tree_iterator_current_body(),
        ),
        method_with_body("key", Vec::new(), Some(mixed_type()), tree_iterator_key_body()),
    ]
}

/// Builds the synthetic method body for the tree iterator constructor.
///
/// The default prefixes are php's; the source is wrapped in a caching iterator so each active
/// level can answer `hasNext()`, then handed to the parent with the requested traversal mode.
fn tree_iterator_construct_body() -> Vec<Stmt> {
    let default_prefix = |index: i64, value: &str| {
        property_array_assign_stmt(this_expr(), "prefix", int_expr(index), string_expr(value))
    };
    vec![
        property_assign_stmt(this_expr(), "prefix", empty_array_expr()),
        default_prefix(0, ""),
        default_prefix(1, "| "),
        default_prefix(2, "  "),
        default_prefix(3, "|-"),
        default_prefix(4, "\\-"),
        default_prefix(5, ""),
        property_assign_stmt(this_expr(), "postfix", string_expr("")),
        // php unwraps an `IteratorAggregate` source before wrapping it in the caching iterator.
        if_stmt(
            instanceof_expr(var_expr("iterator"), "IteratorAggregate"),
            vec![assign_stmt(
                "iterator",
                method_call(var_expr("iterator"), "getIterator", Vec::new()),
            )],
            None,
        ),
        expr_stmt(parent_call(
            "__construct",
            vec![
                new_object_expr(
                    "RecursiveCachingIterator",
                    vec![
                        method_call(
                            this_expr(),
                            "__elephcAssumeRecursiveIterator",
                            vec![var_expr("iterator")],
                        ),
                        var_expr("cachingIteratorFlags"),
                    ],
                ),
                var_expr("mode"),
            ],
        )),
        // AFTER the parent constructor: it assigns the inherited traversal `flags` slot, so the
        // tree keeps its own bypass flags in `treeFlags`.
        property_assign_stmt(this_expr(), "treeFlags", var_expr("flags")),
    ]
}

/// Builds the synthetic method body for `getPrefix()`.
fn tree_iterator_get_prefix_body() -> Vec<Stmt> {
    let prefix = |index: i64| array_access(property_access(this_expr(), "prefix"), int_expr(index));
    let has_next = |level: Expr| {
        method_call(
            method_call(this_expr(), "getSubIterator", vec![level]),
            "hasNext",
            Vec::new(),
        )
    };
    vec![
        assign_stmt("str", prefix(0)),
        assign_stmt("depth", method_call(this_expr(), "getDepth", Vec::new())),
        assign_stmt("level", int_expr(0)),
        while_stmt(
            binary_expr(var_expr("level"), BinOp::Lt, var_expr("depth")),
            vec![
                assign_stmt(
                    "str",
                    binary_expr(
                        var_expr("str"),
                        BinOp::Concat,
                        ternary(has_next(var_expr("level")), prefix(1), prefix(2)),
                    ),
                ),
                increment_stmt("level"),
            ],
        ),
        assign_stmt(
            "str",
            binary_expr(
                var_expr("str"),
                BinOp::Concat,
                ternary(has_next(var_expr("depth")), prefix(3), prefix(4)),
            ),
        ),
        assign_stmt(
            "str",
            binary_expr(var_expr("str"), BinOp::Concat, prefix(5)),
        ),
        return_stmt(var_expr("str")),
    ]
}

/// Builds the synthetic method body for `setPrefixPart()`.
fn tree_iterator_set_prefix_part_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            binary_expr(
                binary_expr(var_expr("part"), BinOp::Lt, int_expr(0)),
                BinOp::Or,
                binary_expr(var_expr("part"), BinOp::GtEq, int_expr(PREFIX_PART_COUNT)),
            ),
            vec![throw_stmt(new_object_expr(
                "ValueError",
                vec![string_expr(
                    "RecursiveTreeIterator::setPrefixPart(): Argument #1 ($part) must be a \
                     RecursiveTreeIterator::PREFIX_* constant",
                )],
            ))],
            None,
        ),
        property_array_assign_stmt(
            this_expr(),
            "prefix",
            var_expr("part"),
            var_expr("prefix"),
        ),
    ]
}

/// Builds the synthetic method body for `getEntry()`.
///
/// php renders an array entry as the literal `Array` and stringifies everything else. The value
/// comes from the parent's `current()`: php reads the active sub-iterator's data, which is the
/// same value, and a call through `getInnerIterator()` would type as `Iterator` and pull every
/// interface implementor (including the regex iterators) into the emitted class set.
fn tree_iterator_get_entry_body() -> Vec<Stmt> {
    vec![
        assign_stmt("value", parent_call("current", Vec::new())),
        if_stmt(
            function_call("is_array", vec![var_expr("value")]),
            return_body(string_expr("Array")),
            None,
        ),
        return_stmt(cast_expr(CastType::String, var_expr("value"))),
    ]
}

/// Builds the synthetic method body for `setPostfix()`.
fn tree_iterator_set_postfix_body() -> Vec<Stmt> {
    vec![property_assign_stmt(this_expr(), "postfix", var_expr("postfix"))]
}

/// Builds the synthetic method body for `getPostfix()`.
fn tree_iterator_get_postfix_body() -> Vec<Stmt> {
    return_body(property_access(this_expr(), "postfix"))
}

/// Builds the synthetic method body for `current()`.
fn tree_iterator_current_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            bypass_enabled(BYPASS_CURRENT),
            return_body(parent_call("current", Vec::new())),
            None,
        ),
        return_stmt(binary_expr(
            binary_expr(
                method_call(this_expr(), "getPrefix", Vec::new()),
                BinOp::Concat,
                method_call(this_expr(), "getEntry", Vec::new()),
            ),
            BinOp::Concat,
            method_call(this_expr(), "getPostfix", Vec::new()),
        )),
    ]
}

/// Builds the synthetic method body for `key()`.
fn tree_iterator_key_body() -> Vec<Stmt> {
    vec![
        assign_stmt("key", parent_call("key", Vec::new())),
        if_stmt(
            bypass_enabled(BYPASS_KEY),
            return_body(var_expr("key")),
            None,
        ),
        return_stmt(binary_expr(
            binary_expr(
                method_call(this_expr(), "getPrefix", Vec::new()),
                BinOp::Concat,
                cast_expr(CastType::String, var_expr("key")),
            ),
            BinOp::Concat,
            method_call(this_expr(), "getPostfix", Vec::new()),
        )),
    ]
}

/// Builds `($this->treeFlags & bit) !== 0`, php's bypass-flag test.
fn bypass_enabled(bit: i64) -> Expr {
    binary_expr(
        binary_expr(
            property_access(this_expr(), "treeFlags"),
            BinOp::BitAnd,
            int_expr(bit),
        ),
        BinOp::StrictNotEq,
        int_expr(0),
    )
}
