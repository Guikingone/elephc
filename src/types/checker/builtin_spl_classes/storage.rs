//! Purpose:
//! Injects storage-oriented SPL iterator metadata: EmptyIterator, ArrayIterator, RecursiveArrayIterator, and ArrayObject.
//! Owns synthetic PHP-like bodies for array key/value snapshots and ArrayAccess behavior.
//!
//! Called from:
//! - `super::inject_builtin_spl_classes()`.
//!
//! Key details:
//! - ArrayIterator and ArrayObject store parallel key/value arrays to preserve PHP keys.
//! - RecursiveArrayIterator exposes internal narrowing hooks for recursive traversal codegen.

use std::collections::HashMap;

use crate::parser::ast::{BinOp, ClassMethod, ClassProperty, Expr, Stmt, TypeExpr};
use crate::types::traits::FlattenedClass;

use super::common::*;

/// Inserts classes into the supplied builtin metadata registry.
pub(super) fn insert_classes(class_map: &mut HashMap<String, FlattenedClass>) {
    class_map.insert(
        "EmptyIterator".to_string(),
        FlattenedClass {
            name: "EmptyIterator".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec!["Iterator".to_string()],
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: Vec::new(),
            methods: spl_empty_iterator_methods(),
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );

    class_map.insert(
        "ArrayIterator".to_string(),
        FlattenedClass {
            name: "ArrayIterator".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec![
                "Iterator".to_string(),
                "ArrayAccess".to_string(),
                "SeekableIterator".to_string(),
                "Countable".to_string(),
            ],
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: array_iterator_properties(),
            methods: spl_array_iterator_methods(),
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );

    class_map.insert(
        "ArrayObject".to_string(),
        FlattenedClass {
            name: "ArrayObject".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec![
                "IteratorAggregate".to_string(),
                "ArrayAccess".to_string(),
                "Countable".to_string(),
            ],
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: array_object_properties(),
            methods: spl_array_object_methods(),
            attributes: Vec::new(),
            constants: vec![
                class_const("STD_PROP_LIST", 1),
                class_const("ARRAY_AS_PROPS", 2),
            ],
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );
}

/// Builds the method list for SPL empty iterator.
fn spl_empty_iterator_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body("current", Vec::new(), Some(mixed_type()), null_return_body()),
        method_with_body("key", Vec::new(), Some(mixed_type()), null_return_body()),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), return_body(bool_expr(false))),
    ]
}

/// Builds the property list for array iterator.
fn array_iterator_properties() -> Vec<ClassProperty> {
    vec![
        storage_property("storage", mixed_type()),
        protected_storage_property("__elephc_position", TypeExpr::Int),
        protected_storage_property("__elephc_flags", TypeExpr::Int),
    ]
}

/// Builds the property list for array object.
fn array_object_properties() -> Vec<ClassProperty> {
    vec![
        // A generic PHP array (packed or string-keyed), so it is typed `mixed`: an `array<T>` slot
        // is a PACKED list and would drop string keys.
        storage_property("storage", mixed_type()),
        storage_property("__elephc_flags", TypeExpr::Int),
        // The class `getIterator()` instantiates. elephc always builds an `ArrayIterator`, so the
        // value is stored and reported by `getIteratorClass()` but does not change `getIterator()`.
        storage_property("__elephc_iteratorClass", TypeExpr::Str),
    ]
}

/// Builds the method list for SPL array iterator.
fn spl_array_iterator_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body(
            "__construct",
            vec![
                // PHP declares this parameter as `array|object`. It is typed `mixed` here because
                // elephc's checker rejects a `mixed` argument for an `array|object` parameter
                // (PHP defers that check to runtime), and several php-src tests pass a `mixed`
                // value. The synthetic constructor body normalizes an object to its public
                // properties, so arrays and objects both work (see `array_object_construct_body`).
                param_default("array", mixed_type(), empty_array_expr()),
                param_default("flags", TypeExpr::Int, int_expr(0)),
            ],
            Some(TypeExpr::Void),
            array_iterator_construct_body(),
        ),
        method_with_body("current", Vec::new(), Some(mixed_type()), array_current_body()),
        method_with_body("key", Vec::new(), Some(mixed_type()), array_key_body()),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), array_next_body()),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), array_rewind_body()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), array_valid_body()),
        method_with_body(
            "asort",
            vec![param_default("flags", TypeExpr::Int, int_expr(0))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Asort),
        ),
        method_with_body(
            "ksort",
            vec![param_default("flags", TypeExpr::Int, int_expr(0))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Ksort),
        ),
        method_with_body(
            "uasort",
            vec![param("callback", named_type("callable"))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Uasort),
        ),
        method_with_body(
            "uksort",
            vec![param("callback", named_type("callable"))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Uksort),
        ),
        method_with_body(
            "seek",
            vec![param("offset", TypeExpr::Int)],
            Some(TypeExpr::Void),
            vec![property_assign_stmt(this_expr(), "__elephc_position", var_expr("offset"))],
        ),
        method_with_body("count", Vec::new(), Some(TypeExpr::Int), array_count_body()),
        method_with_body("getFlags", Vec::new(), Some(TypeExpr::Int), return_body(flags_expr())),
        method_with_body(
            "setFlags",
            vec![param("flags", TypeExpr::Int)],
            Some(TypeExpr::Void),
            vec![property_assign_stmt(this_expr(), "__elephc_flags", var_expr("flags"))],
        ),
        method_with_body(
            "offsetExists",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Bool),
            array_offset_exists_body(),
        ),
        method_with_body(
            "offsetGet",
            vec![param("offset", mixed_type())],
            Some(mixed_type()),
            array_offset_get_body(),
        ),
        method_with_body(
            "offsetSet",
            vec![param("offset", mixed_type()), param("value", mixed_type())],
            Some(TypeExpr::Void),
            array_offset_set_body(),
        ),
        method_with_body(
            "offsetUnset",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Void),
            array_offset_unset_body(),
        ),
        method_with_body(
            "append",
            vec![param("value", mixed_type())],
            Some(TypeExpr::Void),
            array_append_body(),
        ),
        method_with_body("getArrayCopy", Vec::new(), Some(array_type()), array_copy_body()),
    ]
}

/// Builds the method list for SPL array object.
fn spl_array_object_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body(
            "__construct",
            vec![
                // PHP declares this parameter as `array|object`. It is typed `mixed` here because
                // elephc's checker rejects a `mixed` argument for an `array|object` parameter
                // (PHP defers that check to runtime), and several php-src tests pass a `mixed`
                // value. The synthetic constructor body normalizes an object to its public
                // properties, so arrays and objects both work (see `array_object_construct_body`).
                param_default("array", mixed_type(), empty_array_expr()),
                param_default("flags", TypeExpr::Int, int_expr(0)),
                // PHP's third argument names the iterator class `getIterator()` returns; elephc
                // always returns an `ArrayIterator`, so it is accepted and ignored.
                param_default("iteratorClass", TypeExpr::Str, string_expr("ArrayIterator")),
            ],
            Some(TypeExpr::Void),
            array_object_ctor_body(),
        ),
        method_with_body("getIterator", Vec::new(), Some(named_type("Iterator")), array_object_get_iterator_body()),
        method_with_body(
            "asort",
            vec![param_default("flags", TypeExpr::Int, int_expr(0))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Asort),
        ),
        method_with_body(
            "ksort",
            vec![param_default("flags", TypeExpr::Int, int_expr(0))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Ksort),
        ),
        method_with_body(
            "uasort",
            vec![param("callback", named_type("callable"))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Uasort),
        ),
        method_with_body(
            "uksort",
            vec![param("callback", named_type("callable"))],
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Uksort),
        ),
        method_with_body("count", Vec::new(), Some(TypeExpr::Int), array_count_body()),
        method_with_body("getFlags", Vec::new(), Some(TypeExpr::Int), return_body(flags_expr())),
        method_with_body(
            "setFlags",
            vec![param("flags", TypeExpr::Int)],
            Some(TypeExpr::Void),
            vec![property_assign_stmt(this_expr(), "__elephc_flags", var_expr("flags"))],
        ),
        method_with_body(
            "getIteratorClass",
            Vec::new(),
            Some(TypeExpr::Str),
            return_body(property_access(this_expr(), "__elephc_iteratorClass")),
        ),
        method_with_body(
            "setIteratorClass",
            vec![param("iteratorClass", TypeExpr::Str)],
            Some(TypeExpr::Void),
            vec![property_assign_stmt(this_expr(), "__elephc_iteratorClass", var_expr("iteratorClass"))],
        ),
        method_with_body(
            "exchangeArray",
            vec![param("array", mixed_type())],
            Some(mixed_type()),
            array_exchange_body(),
        ),
        method_with_body(
            "offsetExists",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Bool),
            array_offset_exists_body(),
        ),
        method_with_body(
            "offsetGet",
            vec![param("offset", mixed_type())],
            Some(mixed_type()),
            array_offset_get_body(),
        ),
        method_with_body(
            "offsetSet",
            vec![param("offset", mixed_type()), param("value", mixed_type())],
            Some(TypeExpr::Void),
            array_offset_set_body(),
        ),
        method_with_body(
            "offsetUnset",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Void),
            array_offset_unset_body(),
        ),
        method_with_body(
            "append",
            vec![param("value", mixed_type())],
            Some(TypeExpr::Void),
            array_append_body(),
        ),
        method_with_body("getArrayCopy", Vec::new(), Some(array_type()), array_copy_body()),
    ]
}

/// Builds the AST expression for the backing storage array.
fn storage_expr() -> Expr {
    property_access(this_expr(), "storage")
}

/// Builds the AST expression for position.
fn position_expr() -> Expr {
    property_access(this_expr(), "__elephc_position")
}

/// Builds the AST expression for flags.
fn flags_expr() -> Expr {
    property_access(this_expr(), "__elephc_flags")
}

/// Builds `array_keys($this->storage)[$index]`, the key at a positional index.
fn key_at(index: Expr) -> Expr {
    array_access(function_call("array_keys", vec![storage_expr()]), index)
}

/// Builds `array_values($this->storage)[$index]`, the value at a positional index.
fn value_at(index: Expr) -> Expr {
    array_access(function_call("array_values", vec![storage_expr()]), index)
}

/// Normalizes an `array|object` backing argument to an array in place.
///
/// PHP's `ArrayObject`/`ArrayIterator` constructors accept `array|object`; an object contributes
/// its PUBLIC properties as the backing entries. PHP also emits the object-backing deprecation
/// named by `deprecation` first.
pub(super) fn storage_normalize_stmt(deprecation: &str) -> Stmt {
    crate::synthetic_class::s_if(
        function_call("is_object", vec![var_expr("array")]),
        vec![
            expr_stmt(function_call(
                "trigger_error",
                vec![
                    string_expr(deprecation),
                    crate::synthetic_class::e_const("E_USER_DEPRECATED"),
                ],
            )),
            crate::synthetic_class::s_assign(
                "array",
                function_call("get_object_vars", vec![var_expr("array")]),
            ),
        ],
        vec![],
        None,
    )
}

/// Builds PHP's object-backing deprecation message for one container constructor or method.
pub(super) fn object_backing_deprecation(class: &str, method: &str) -> String {
    format!(
        "{class}::{method}(): Using an object as a backing array for {class} is deprecated, \
         as it allows violating class constraints and invariants"
    )
}

/// Builds the synthetic method body for array iterator construct.
fn array_iterator_construct_body() -> Vec<Stmt> {
    vec![
        storage_normalize_stmt(&object_backing_deprecation("ArrayIterator", "__construct")),
        property_assign_stmt(this_expr(), "storage", var_expr("array")),
        property_assign_stmt(this_expr(), "__elephc_position", int_expr(0)),
        property_assign_stmt(this_expr(), "__elephc_flags", var_expr("flags")),
    ]
}

/// Builds the synthetic method body for array object construct.
fn array_object_ctor_body() -> Vec<Stmt> {
    vec![
        storage_normalize_stmt(&object_backing_deprecation("ArrayObject", "__construct")),
        property_assign_stmt(this_expr(), "storage", var_expr("array")),
        property_assign_stmt(this_expr(), "__elephc_flags", var_expr("flags")),
        property_assign_stmt(this_expr(), "__elephc_iteratorClass", var_expr("iteratorClass")),
    ]
}

/// Builds the synthetic method body for array current.
fn array_current_body() -> Vec<Stmt> {
    return_body(value_at(position_expr()))
}

/// Builds the synthetic method body for array key.
fn array_key_body() -> Vec<Stmt> {
    return_body(key_at(position_expr()))
}

/// Builds the synthetic method body for array next.
fn array_next_body() -> Vec<Stmt> {
    vec![property_assign_stmt(
        this_expr(),
        "__elephc_position",
        binary_expr(position_expr(), BinOp::Add, int_expr(1)),
    )]
}

/// Builds the synthetic method body for array rewind.
fn array_rewind_body() -> Vec<Stmt> {
    vec![property_assign_stmt(this_expr(), "__elephc_position", int_expr(0))]
}

/// Builds the synthetic method body for array valid.
fn array_valid_body() -> Vec<Stmt> {
    return_body(binary_expr(position_expr(), BinOp::Lt, count_expr(storage_expr())))
}

/// Builds the synthetic method body for array count.
fn array_count_body() -> Vec<Stmt> {
    return_body(count_expr(storage_expr()))
}

/// Builds the synthetic method body for array append.
fn array_append_body() -> Vec<Stmt> {
    vec![
        assign_stmt("__storage", storage_expr()),
        array_push_stmt("__storage", var_expr("value")),
        property_assign_stmt(this_expr(), "storage", var_expr("__storage")),
    ]
}

/// Builds the synthetic method body for array offset exists.
fn array_offset_exists_body() -> Vec<Stmt> {
    return_body(function_call(
        "array_key_exists",
        vec![var_expr("offset"), storage_expr()],
    ))
}

/// Builds the synthetic method body for array offset get.
fn array_offset_get_body() -> Vec<Stmt> {
    return_body(array_access(storage_expr(), var_expr("offset")))
}

/// Builds the synthetic method body for array offset set.
///
/// The keyed element write goes through a local copy: a runtime-typed (`mixed`) property of a
/// typed object does not lower `$obj->prop[$key] = $value` directly. The append case calls
/// `append()` so the method stays referenced by the vtable lowering.
fn array_offset_set_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            binary_expr(var_expr("offset"), BinOp::StrictEq, null_expr()),
            vec![
                expr_stmt(method_call(this_expr(), "append", vec![var_expr("value")])),
                return_void_stmt(),
            ],
            None,
        ),
        assign_stmt("__storage", storage_expr()),
        array_assign_stmt("__storage", var_expr("offset"), var_expr("value")),
        property_assign_stmt(this_expr(), "storage", var_expr("__storage")),
    ]
}

/// Builds the synthetic method body for array offset unset.
///
/// `unset()` only lowers for a variable or an `ArrayAccess` receiver, and a runtime-typed local
/// cannot take the removal path either, so the storage is rebuilt without the key from its
/// `array_keys`/`array_values` projection. That also preserves PHP's key hole for an indexed
/// array (the surviving keys are not renumbered).
fn array_offset_unset_body() -> Vec<Stmt> {
    let key_at_i = || array_access(var_expr("__keys"), var_expr("__i"));
    vec![
        assign_stmt("__keys", function_call("array_keys", vec![storage_expr()])),
        assign_stmt("__values", function_call("array_values", vec![storage_expr()])),
        assign_stmt("__new", empty_array_expr()),
        assign_stmt("__i", int_expr(0)),
        assign_stmt("__limit", count_expr(storage_expr())),
        while_stmt(
            binary_expr(var_expr("__i"), BinOp::Lt, var_expr("__limit")),
            vec![
                if_stmt(
                    not_expr(binary_expr(key_at_i(), BinOp::StrictEq, var_expr("offset"))),
                    vec![array_assign_stmt(
                        "__new",
                        key_at_i(),
                        array_access(var_expr("__values"), var_expr("__i")),
                    )],
                    None,
                ),
                increment_stmt("__i"),
            ],
        ),
        property_assign_stmt(this_expr(), "storage", var_expr("__new")),
    ]
}

/// Builds the synthetic method body for array copy.
fn array_copy_body() -> Vec<Stmt> {
    return_body(storage_expr())
}

/// Builds the synthetic method body for `exchangeArray()`.
///
/// Returns the previous contents as an array copy, then replaces the backing store with the
/// supplied `array|object` (an object contributes its public properties).
fn array_exchange_body() -> Vec<Stmt> {
    vec![
        assign_stmt("old", storage_expr()),
        storage_normalize_stmt(&object_backing_deprecation("ArrayObject", "exchangeArray")),
        property_assign_stmt(this_expr(), "storage", var_expr("array")),
        return_stmt(var_expr("old")),
    ]
}

/// Builds the synthetic method body for array object get iterator.
fn array_object_get_iterator_body() -> Vec<Stmt> {
    vec![return_stmt(new_object_expr(
        "ArrayIterator",
        vec![storage_expr()],
    ))]
}

/// Builds an in-place, stable insertion sort of the storage from its key/value projection.
///
/// PHP's `asort`/`ksort` keep key association and leave a key hole rather than renumbering, which
/// the insertion over parallel key/value lists preserves. `uasort`/`uksort` compare through the
/// `$callback`; `asort`/`ksort` compare directly with `>`.
fn array_sort_body(sort: ArraySort) -> Vec<Stmt> {
    let compares_value = matches!(sort, ArraySort::Asort | ArraySort::Uasort);
    let uses_callback = matches!(sort, ArraySort::Uasort | ArraySort::Uksort);
    let projected = if compares_value {
        array_access(var_expr("__values"), var_expr("__j"))
    } else {
        array_access(var_expr("__keys"), var_expr("__j"))
    };
    let current = if compares_value { "__cv" } else { "__ck" };
    let comparison = if uses_callback {
        binary_expr(
            crate::synthetic_class::e_closure_call("callback", vec![projected, var_expr(current)]),
            BinOp::Gt,
            int_expr(0),
        )
    } else {
        binary_expr(projected, BinOp::Gt, var_expr(current))
    };
    let next_index = || binary_expr(var_expr("__j"), BinOp::Add, int_expr(1));
    vec![
        assign_stmt("__keys", function_call("array_keys", vec![storage_expr()])),
        assign_stmt("__values", function_call("array_values", vec![storage_expr()])),
        assign_stmt("__n", count_expr(var_expr("__keys"))),
        assign_stmt("__i", int_expr(1)),
        while_stmt(
            binary_expr(var_expr("__i"), BinOp::Lt, var_expr("__n")),
            vec![
                assign_stmt("__ck", array_access(var_expr("__keys"), var_expr("__i"))),
                assign_stmt("__cv", array_access(var_expr("__values"), var_expr("__i"))),
                assign_stmt("__j", binary_expr(var_expr("__i"), BinOp::Sub, int_expr(1))),
                while_stmt(
                    binary_expr(
                        binary_expr(var_expr("__j"), BinOp::GtEq, int_expr(0)),
                        BinOp::And,
                        comparison,
                    ),
                    vec![
                        array_assign_stmt(
                            "__keys",
                            next_index(),
                            array_access(var_expr("__keys"), var_expr("__j")),
                        ),
                        array_assign_stmt(
                            "__values",
                            next_index(),
                            array_access(var_expr("__values"), var_expr("__j")),
                        ),
                        assign_stmt("__j", binary_expr(var_expr("__j"), BinOp::Sub, int_expr(1))),
                    ],
                ),
                array_assign_stmt("__keys", next_index(), var_expr("__ck")),
                array_assign_stmt("__values", next_index(), var_expr("__cv")),
                assign_stmt("__i", binary_expr(var_expr("__i"), BinOp::Add, int_expr(1))),
            ],
        ),
        assign_stmt("__new", empty_array_expr()),
        assign_stmt("__i", int_expr(0)),
        while_stmt(
            binary_expr(var_expr("__i"), BinOp::Lt, var_expr("__n")),
            vec![
                array_assign_stmt(
                    "__new",
                    array_access(var_expr("__keys"), var_expr("__i")),
                    array_access(var_expr("__values"), var_expr("__i")),
                ),
                increment_stmt("__i"),
            ],
        ),
        property_assign_stmt(this_expr(), "storage", var_expr("__new")),
        return_stmt(bool_expr(true)),
    ]
}

/// Selects which side of the key/value projection an [`array_sort_body`] sorts.
#[derive(Clone, Copy)]
enum ArraySort {
    /// `asort`: values ascending with `>`.
    Asort,
    /// `ksort`: keys ascending with `>`.
    Ksort,
    /// `uasort`: values through `$callback`.
    Uasort,
    /// `uksort`: keys through `$callback`.
    Uksort,
}
