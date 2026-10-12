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

use crate::parser::ast::{
    BinOp, CastType, ClassMethod, ClassProperty, Expr, ExprKind, Stmt, TypeExpr,
};
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
            // php accepts dynamic property creation on every container with its 8.5 deprecation.
            attributes: vec![deprecated_dynamic_properties_attribute()],
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
            // php accepts dynamic property creation on every container with its 8.5 deprecation.
            attributes: vec![deprecated_dynamic_properties_attribute()],
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
        method_with_body(
            "current",
            Vec::new(),
            Some(mixed_type()),
            empty_iterator_bad_call_body("Accessing the value of an EmptyIterator"),
        ),
        method_with_body(
            "key",
            Vec::new(),
            Some(mixed_type()),
            empty_iterator_bad_call_body("Accessing the key of an EmptyIterator"),
        ),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), return_body(bool_expr(false))),
    ]
}

/// Builds an `EmptyIterator` reader that raises php's `BadMethodCallException`.
///
/// `EmptyIterator` never has a current element, and php refuses both readers with its own wording
/// rather than answering null.
fn empty_iterator_bad_call_body(message: &str) -> Vec<Stmt> {
    vec![throw_stmt(new_object_expr(
        "BadMethodCallException",
        vec![string_expr(message)],
    ))]
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
        method_with_body(
            "__elephcStorageArray",
            Vec::new(),
            Some(mixed_type()),
            storage_array_body(),
        ),
        method_with_body(
            "__elephcStorageArrayOf",
            vec![param("store", mixed_type())],
            Some(mixed_type()),
            storage_array_of_body(),
        ),
        method_with_body(
            "__elephcStorageRawStore",
            Vec::new(),
            Some(mixed_type()),
            storage_raw_body(),
        ),
        method_with_body(
            "__elephcWriteStoreProperty",
            vec![
                param("store", mixed_type()),
                param("name", mixed_type()),
                param("value", mixed_type()),
            ],
            Some(TypeExpr::Void),
            storage_write_property_body(),
        ),
        method_with_body(
            "__elephcUnsetStoreProperty",
            vec![param("store", mixed_type()), param("name", TypeExpr::Str)],
            Some(TypeExpr::Void),
            storage_unset_property_body(),
        ),
        method_with_body(
            "__elephcWriteStorageOffset",
            vec![param("offset", mixed_type()), param("value", mixed_type())],
            Some(TypeExpr::Void),
            storage_offset_write_body(),
        ),
        method_with_body(
            "__elephcRemoveStorageOffset",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Void),
            storage_offset_remove_body(),
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
        method_with_body("natsort", Vec::new(), Some(TypeExpr::Bool), array_sort_body(ArraySort::Natsort)),
        method_with_body(
            "natcasesort",
            Vec::new(),
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Natcasesort),
        ),
        method_with_body(
            "__elephcStrnatcmp",
            vec![
                param("a", TypeExpr::Str),
                param("b", TypeExpr::Str),
                param("fold", TypeExpr::Bool),
            ],
            Some(TypeExpr::Int),
            strnatcmp_body(),
        ),
        method_with_body(
            "__elephcIsNatSpace",
            vec![param("ch", TypeExpr::Str)],
            Some(TypeExpr::Bool),
            is_nat_space_body(),
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
        method_with_body(
            "__elephcStorageArray",
            Vec::new(),
            Some(mixed_type()),
            storage_array_body(),
        ),
        method_with_body(
            "__elephcStorageArrayOf",
            vec![param("store", mixed_type())],
            Some(mixed_type()),
            storage_array_of_body(),
        ),
        method_with_body(
            "__elephcStorageRawStore",
            Vec::new(),
            Some(mixed_type()),
            storage_raw_body(),
        ),
        method_with_body(
            "__elephcWriteStoreProperty",
            vec![
                param("store", mixed_type()),
                param("name", mixed_type()),
                param("value", mixed_type()),
            ],
            Some(TypeExpr::Void),
            storage_write_property_body(),
        ),
        method_with_body(
            "__elephcUnsetStoreProperty",
            vec![param("store", mixed_type()), param("name", TypeExpr::Str)],
            Some(TypeExpr::Void),
            storage_unset_property_body(),
        ),
        method_with_body(
            "__elephcWriteStorageOffset",
            vec![param("offset", mixed_type()), param("value", mixed_type())],
            Some(TypeExpr::Void),
            storage_offset_write_body(),
        ),
        method_with_body(
            "__elephcRemoveStorageOffset",
            vec![param("offset", mixed_type())],
            Some(TypeExpr::Void),
            storage_offset_remove_body(),
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
        method_with_body("natsort", Vec::new(), Some(TypeExpr::Bool), array_sort_body(ArraySort::Natsort)),
        method_with_body(
            "natcasesort",
            Vec::new(),
            Some(TypeExpr::Bool),
            array_sort_body(ArraySort::Natcasesort),
        ),
        method_with_body(
            "__elephcStrnatcmp",
            vec![
                param("a", TypeExpr::Str),
                param("b", TypeExpr::Str),
                param("fold", TypeExpr::Bool),
            ],
            Some(TypeExpr::Int),
            strnatcmp_body(),
        ),
        method_with_body(
            "__elephcIsNatSpace",
            vec![param("ch", TypeExpr::Str)],
            Some(TypeExpr::Bool),
            is_nat_space_body(),
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

/// Builds `$object->{$name}`, a dynamic property access on a runtime-named property.
fn dynamic_property_access(object: Expr, name: Expr) -> Expr {
    expr(ExprKind::DynamicPropertyAccess {
        object: Box::new(object),
        property: Box::new(name),
    })
}

/// Builds the AST expression for the backing store projected as an array.
///
/// `__elephcStorageArray()` answers an array store unchanged and projects an object store through
/// `get_object_vars()`, so every reader works for both backing shapes.
fn storage_array_expr() -> Expr {
    method_call(this_expr(), "__elephcStorageArray", Vec::new())
}

/// Builds `__elephcStorageArray()`, the array view of the backing store.
fn storage_array_body() -> Vec<Stmt> {
    vec![return_stmt(method_call(
        this_expr(),
        "__elephcStorageArrayOf",
        vec![storage_expr()],
    ))]
}

/// Builds `__elephcStorageArrayOf($store)`, the array view of one backing store value.
///
/// The store arrives as a PARAMETER because the lowering context types parameters from the
/// checker's environment, while a property expression falls through the syntactic inference
/// (which cannot see declared property types) to its Int fallback.
///
/// php's object backing follows an `ArrayObject`/`ArrayIterator` store to ITS OWN backing
/// (recursively, so nested wrappers land on the innermost array) and projects any other object
/// through `get_object_vars()`. A SELF-backed store (`parent::__construct($this, ...)`) stops the
/// follow at the identity — php reads the property table there, and php's `storage` is a
/// synthesized var_dump entry rather than a real property, so the projection drops the key the
/// elephc layout declares.
fn storage_array_of_body() -> Vec<Stmt> {
    vec![
        while_stmt(
            binary_expr(
                function_call("is_object", vec![var_expr("store")]),
                BinOp::And,
                binary_expr(
                    binary_expr(var_expr("store"), BinOp::StrictNotEq, this_expr()),
                    BinOp::And,
                    binary_expr(
                        instanceof_expr(var_expr("store"), "ArrayObject"),
                        BinOp::Or,
                        instanceof_expr(var_expr("store"), "ArrayIterator"),
                    ),
                ),
            ),
            vec![
                assign_stmt(
                    "raw",
                    method_call(var_expr("store"), "__elephcStorageRawStore", Vec::new()),
                ),
                if_stmt(
                    binary_expr(var_expr("raw"), BinOp::StrictNotEq, var_expr("store")),
                    vec![assign_stmt("store", var_expr("raw"))],
                    None,
                ),
            ],
        ),
        if_stmt(
            function_call("is_object", vec![var_expr("store")]),
            vec![
                assign_stmt(
                    "props",
                    function_call("get_object_vars", vec![var_expr("store")]),
                ),
                if_stmt(
                    binary_expr(var_expr("store"), BinOp::StrictEq, this_expr()),
                    vec![
                        // php synthesizes its internal state into var_dump output instead of
                        // declaring it as properties, so a self-backed projection drops the
                        // elephc-internal keys php's property table never carried.
                        expr_stmt(function_call(
                            "unset",
                            vec![array_access(
                                var_expr("props"),
                                string_expr("storage"),
                            )],
                        )),
                        expr_stmt(function_call(
                            "unset",
                            vec![array_access(
                                var_expr("props"),
                                string_expr("__elephc_flags"),
                            )],
                        )),
                        expr_stmt(function_call(
                            "unset",
                            vec![array_access(
                                var_expr("props"),
                                string_expr("__elephc_iteratorClass"),
                            )],
                        )),
                        expr_stmt(function_call(
                            "unset",
                            vec![array_access(
                                var_expr("props"),
                                string_expr("__elephc_position"),
                            )],
                        )),
                    ],
                    None,
                ),
                return_stmt(var_expr("props")),
            ],
            None,
        ),
        return_stmt(var_expr("store")),
    ]
}

/// Builds `__elephcStorageRawStore()`, the RAW backing store without any projection.
///
/// `__elephcStorageArrayOf()` walks an `ArrayObject`-shaped store through this accessor to reach
/// the wrapped value one level down, so the two helpers must stay recursive over the same shape.
fn storage_raw_body() -> Vec<Stmt> {
    vec![return_stmt(storage_expr())]
}

/// Builds `__elephcWriteStoreProperty($store, $name, $value)`, the keyed write for an object store.
///
/// The receiver is a PARAMETER so the dynamic property write lowers through the mixed ladder.
/// php delegates the write to the wrapped object's property table, so a declared slot is written
/// and an undeclared one is created (with php's dynamic-property deprecation).
fn storage_write_property_body() -> Vec<Stmt> {
    vec![expr_stmt(expr(ExprKind::Assignment {
        target: Box::new(dynamic_property_access(
            var_expr("store"),
            var_expr("name"),
        )),
        value: Box::new(var_expr("value")),
        result_target: None,
        prelude: Vec::new(),
        conditional_value_temp: None,
    }))]
}

/// Builds `__elephcUnsetStoreProperty($store, string $name)`, the keyed removal for an object store.
///
/// `unset($store->$name)` lowers through `Op::DynamicPropUnset`, whose runtime ladder needs both
/// operands typed by the lowering context, so the store and the string name arrive as parameters.
fn storage_unset_property_body() -> Vec<Stmt> {
    vec![expr_stmt(function_call(
        "unset",
        vec![dynamic_property_access(var_expr("store"), var_expr("name"))],
    ))]
}

/// Builds `__elephcWriteStorageOffset($offset, $value)`, the keyed write for either backing shape.
///
/// An object store receives the write as a property named by the offset; an array store is
/// copied, updated and written back because a `mixed` property cannot be written through directly.
fn storage_offset_write_body() -> Vec<Stmt> {
    vec![if_stmt(
        function_call("is_object", vec![storage_expr()]),
        vec![expr_stmt(method_call(
            this_expr(),
            "__elephcWriteStoreProperty",
            vec![storage_expr(), var_expr("offset"), var_expr("value")],
        ))],
        Some(vec![
            assign_stmt("__storage", storage_expr()),
            array_assign_stmt("__storage", var_expr("offset"), var_expr("value")),
            property_assign_stmt(this_expr(), "storage", var_expr("__storage")),
        ]),
    )]
}

/// Builds `__elephcRemoveStorageOffset($offset)`, the keyed removal for either backing shape.
///
/// An object store drops the property through `unset()` on a string-typed name; an array store is
/// rebuilt without the key from its `array_keys`/`array_values` projection, which preserves PHP's
/// key hole for an indexed array (the surviving keys are not renumbered).
fn storage_offset_remove_body() -> Vec<Stmt> {
    let key_at_i = || array_access(var_expr("__keys"), var_expr("__i"));
    vec![if_stmt(
        function_call("is_object", vec![storage_expr()]),
        vec![expr_stmt(method_call(
            this_expr(),
            "__elephcUnsetStoreProperty",
            vec![
                storage_expr(),
                cast_expr(CastType::String, var_expr("offset")),
            ],
        ))],
        Some(vec![
            assign_stmt("__keys", function_call("array_keys", vec![storage_array_expr()])),
            assign_stmt("__values", function_call("array_values", vec![storage_array_expr()])),
            assign_stmt("__new", empty_array_expr()),
            assign_stmt("__i", int_expr(0)),
            assign_stmt("__limit", count_expr(storage_array_expr())),
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
        ]),
    )]
}

/// Builds the AST expression for position.
fn position_expr() -> Expr {
    property_access(this_expr(), "__elephc_position")
}

/// Builds the AST expression for flags.
fn flags_expr() -> Expr {
    property_access(this_expr(), "__elephc_flags")
}

/// Builds `array_keys($this->__elephcStorageArray())[$index]`, the key at a positional index.
fn key_at(index: Expr) -> Expr {
    array_access(
        function_call("array_keys", vec![storage_array_expr()]),
        index,
    )
}

/// Builds `array_values($this->__elephcStorageArray())[$index]`, the value at a positional index.
fn value_at(index: Expr) -> Expr {
    array_access(
        function_call("array_values", vec![storage_array_expr()]),
        index,
    )
}

/// Normalizes an `array|object` backing argument to an array in place.
///
/// PHP's `ArrayObject`/`ArrayIterator` constructors accept `array|object`; an object contributes
/// its PUBLIC properties as the backing entries. PHP also emits the object-backing deprecation
/// named by `deprecation` first.
pub(super) fn storage_normalize_stmt(deprecation: &str) -> Stmt {
    // php KEEPS an object as the backing store (its properties are the entries, read and written
    // through) and reports the deprecation named by `deprecation`. `__elephcStorageArrayOf()`
    // projects an object store for the readers, and the keyed writers route through the
    // `__elephc*StoreProperty` helpers.
    crate::synthetic_class::s_if(
        function_call("is_object", vec![var_expr("array")]),
        vec![expr_stmt(function_call(
            "trigger_error",
            vec![
                string_expr(deprecation),
                crate::synthetic_class::e_const("E_USER_DEPRECATED"),
            ],
        ))],
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
    return_body(binary_expr(
        position_expr(),
        BinOp::Lt,
        count_expr(storage_array_expr()),
    ))
}

/// Builds the synthetic method body for array count.
fn array_count_body() -> Vec<Stmt> {
    return_body(count_expr(storage_array_expr()))
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
        vec![var_expr("offset"), storage_array_expr()],
    ))
}

/// Builds the synthetic method body for array offset get.
fn array_offset_get_body() -> Vec<Stmt> {
    return_body(array_access(storage_array_expr(), var_expr("offset")))
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
        expr_stmt(method_call(
            this_expr(),
            "__elephcWriteStorageOffset",
            vec![var_expr("offset"), var_expr("value")],
        )),
    ]
}

/// Builds the synthetic method body for array offset unset.
///
/// `unset()` only lowers for a variable or an `ArrayAccess` receiver, and a runtime-typed local
/// cannot take the removal path either, so the storage is rebuilt without the key from its
/// `array_keys`/`array_values` projection. That also preserves PHP's key hole for an indexed
/// array (the surviving keys are not renumbered).
fn array_offset_unset_body() -> Vec<Stmt> {
    vec![expr_stmt(method_call(
        this_expr(),
        "__elephcRemoveStorageOffset",
        vec![var_expr("offset")],
    ))]
}

/// Builds the synthetic method body for array copy.
fn array_copy_body() -> Vec<Stmt> {
    return_body(storage_array_expr())
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
        // php answers the old CONTENTS as an array: an object store contributes its properties
        // (or, for an ArrayObject-shaped store, the array it follows), an array store answers
        // itself. `__elephcStorageArrayOf()` is exactly that projection.
        return_stmt(method_call(
            this_expr(),
            "__elephcStorageArrayOf",
            vec![var_expr("old")],
        )),
    ]
}

/// Builds the synthetic method body for array object get iterator.
fn array_object_get_iterator_body() -> Vec<Stmt> {
    vec![return_stmt(new_object_expr(
        "ArrayIterator",
        vec![storage_array_expr()],
    ))]
}

/// Builds an in-place, stable insertion sort of the storage from its key/value projection.
///
/// PHP's `asort`/`ksort` keep key association and leave a key hole rather than renumbering, which
/// the insertion over parallel key/value lists preserves. `uasort`/`uksort` compare through the
/// `$callback`; `asort`/`ksort` compare directly with `>`.
fn array_sort_body(sort: ArraySort) -> Vec<Stmt> {
    let compares_value = matches!(
        sort,
        ArraySort::Asort | ArraySort::Uasort | ArraySort::Natsort | ArraySort::Natcasesort
    );
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
    } else if matches!(sort, ArraySort::Natsort | ArraySort::Natcasesort) {
        binary_expr(
            method_call(
                this_expr(),
                "__elephcStrnatcmp",
                vec![
                    projected,
                    var_expr(current),
                    bool_expr(matches!(sort, ArraySort::Natcasesort)),
                ],
            ),
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

/// Builds the synthetic method body for the natural-order comparison php's `natsort` and
/// `natcasesort` sort through (php-src's `strnatcmp_ex`, including its leading-zero preamble,
/// whitespace runs, digit-run compares, and the case fold), with `$fold` selecting
/// `natsort`'s case-sensitive compare or `natcasesort`'s case-insensitive one.
///
/// Characters past a string's end answer the C NUL php-src's pointer walk reads: `""` compares
/// below every printable character and stops both the whitespace and digit runs.
fn strnatcmp_body() -> Vec<Stmt> {
    // A single character is a decimal digit exactly when it sorts inside "0".."9"; the
    // end-of-string marker `""` fails both comparisons the way the C NUL is not a digit.
    let digit_at = |name: &'static str, index: Expr| {
        let ch = array_access(var_expr(name), index);
        binary_expr(
            binary_expr(ch.clone(), BinOp::GtEq, string_expr("0")),
            BinOp::And,
            binary_expr(ch, BinOp::LtEq, string_expr("9")),
        )
    };
    vec![
        assign_stmt("alen", function_call("strlen", vec![var_expr("a")])),
        assign_stmt("blen", function_call("strlen", vec![var_expr("b")])),
        if_stmt(
            binary_expr(
                binary_expr(var_expr("alen"), BinOp::StrictEq, int_expr(0)),
                BinOp::Or,
                binary_expr(var_expr("blen"), BinOp::StrictEq, int_expr(0)),
            ),
            vec![return_stmt(crate::synthetic_class::e_ternary(
                binary_expr(var_expr("alen"), BinOp::StrictEq, var_expr("blen")),
                int_expr(0),
                crate::synthetic_class::e_ternary(
                    binary_expr(var_expr("alen"), BinOp::Gt, var_expr("blen")),
                    int_expr(1),
                    int_expr(-1),
                ),
            ))],
            None,
        ),
        assign_stmt("ai", int_expr(0)),
        assign_stmt("bi", int_expr(0)),
        // php-src skips leading zeros ONLY while another digit follows.
        while_stmt(
            binary_expr(
                binary_expr(
                    array_access(var_expr("a"), var_expr("ai")),
                    BinOp::StrictEq,
                    string_expr("0"),
                ),
                BinOp::And,
                digit_at("a", binary_expr(var_expr("ai"), BinOp::Add, int_expr(1))),
            ),
            vec![increment_stmt("ai")],
        ),
        while_stmt(
            binary_expr(
                binary_expr(
                    array_access(var_expr("b"), var_expr("bi")),
                    BinOp::StrictEq,
                    string_expr("0"),
                ),
                BinOp::And,
                digit_at("b", binary_expr(var_expr("bi"), BinOp::Add, int_expr(1))),
            ),
            vec![increment_stmt("bi")],
        ),
        while_stmt(bool_expr(true), vec![
            while_stmt(
                binary_expr(
                    binary_expr(var_expr("ai"), BinOp::Lt, var_expr("alen")),
                    BinOp::And,
                    method_call(
                        this_expr(),
                        "__elephcIsNatSpace",
                        vec![array_access(var_expr("a"), var_expr("ai"))],
                    ),
                ),
                vec![increment_stmt("ai")],
            ),
            while_stmt(
                binary_expr(
                    binary_expr(var_expr("bi"), BinOp::Lt, var_expr("blen")),
                    BinOp::And,
                    method_call(
                        this_expr(),
                        "__elephcIsNatSpace",
                        vec![array_access(var_expr("b"), var_expr("bi"))],
                    ),
                ),
                vec![increment_stmt("bi")],
            ),
            assign_stmt(
                "ca",
                crate::synthetic_class::e_ternary(
                    binary_expr(var_expr("ai"), BinOp::Lt, var_expr("alen")),
                    array_access(var_expr("a"), var_expr("ai")),
                    string_expr(""),
                ),
            ),
            assign_stmt(
                "cb",
                crate::synthetic_class::e_ternary(
                    binary_expr(var_expr("bi"), BinOp::Lt, var_expr("blen")),
                    array_access(var_expr("b"), var_expr("bi")),
                    string_expr(""),
                ),
            ),
            // A run of digits compares through php's right-aligned (integer) or left-aligned
            // (fractional) walk; the C for-increment runs after each non-returning iteration,
            // so both indexes advance past the compared digits.
            if_stmt(
                binary_expr(
                    digit_at("a", var_expr("ai")),
                    BinOp::And,
                    digit_at("b", var_expr("bi")),
                ),
                vec![
                    assign_stmt(
                        "fractional",
                        binary_expr(
                            binary_expr(var_expr("ca"), BinOp::StrictEq, string_expr("0")),
                            BinOp::Or,
                            binary_expr(var_expr("cb"), BinOp::StrictEq, string_expr("0")),
                        ),
                    ),
                    assign_stmt("result", int_expr(0)),
                    while_stmt(bool_expr(true), vec![
                        assign_stmt(
                            "ad",
                            binary_expr(
                                binary_expr(var_expr("ai"), BinOp::Lt, var_expr("alen")),
                                BinOp::And,
                                digit_at("a", var_expr("ai")),
                            ),
                        ),
                        assign_stmt(
                            "bd",
                            binary_expr(
                                binary_expr(var_expr("bi"), BinOp::Lt, var_expr("blen")),
                                BinOp::And,
                                digit_at("b", var_expr("bi")),
                            ),
                        ),
                        if_stmt(
                            binary_expr(var_expr("ad"), BinOp::StrictNotEq, var_expr("bd")),
                            vec![
                                assign_stmt(
                                    "result",
                                    crate::synthetic_class::e_ternary(
                                        var_expr("bd"),
                                        int_expr(-1),
                                        int_expr(1),
                                    ),
                                ),
                                crate::synthetic_class::s_break(1),
                            ],
                            None,
                        ),
                        if_stmt(
                            binary_expr(var_expr("ad"), BinOp::StrictEq, bool_expr(false)),
                            vec![crate::synthetic_class::s_break(1)],
                            None,
                        ),
                        if_stmt(
                            var_expr("fractional"),
                            vec![
                                if_stmt(
                                    binary_expr(
                                        array_access(var_expr("a"), var_expr("ai")),
                                        BinOp::Lt,
                                        array_access(var_expr("b"), var_expr("bi")),
                                    ),
                                    vec![assign_stmt("result", int_expr(-1)), crate::synthetic_class::s_break(1)],
                                    None,
                                ),
                                if_stmt(
                                    binary_expr(
                                        array_access(var_expr("a"), var_expr("ai")),
                                        BinOp::Gt,
                                        array_access(var_expr("b"), var_expr("bi")),
                                    ),
                                    vec![assign_stmt("result", int_expr(1)), crate::synthetic_class::s_break(1)],
                                    None,
                                ),
                            ],
                            Some(vec![
                                if_stmt(
                                    binary_expr(
                                        array_access(var_expr("a"), var_expr("ai")),
                                        BinOp::Lt,
                                        array_access(var_expr("b"), var_expr("bi")),
                                    ),
                                    vec![if_stmt(
                                        binary_expr(var_expr("result"), BinOp::StrictEq, int_expr(0)),
                                        vec![assign_stmt("result", int_expr(-1))],
                                        None,
                                    )],
                                    None,
                                ),
                                if_stmt(
                                    binary_expr(
                                        array_access(var_expr("a"), var_expr("ai")),
                                        BinOp::Gt,
                                        array_access(var_expr("b"), var_expr("bi")),
                                    ),
                                    vec![if_stmt(
                                        binary_expr(var_expr("result"), BinOp::StrictEq, int_expr(0)),
                                        vec![assign_stmt("result", int_expr(1))],
                                        None,
                                    )],
                                    None,
                                ),
                            ]),
                        ),
                        increment_stmt("ai"),
                        increment_stmt("bi"),
                    ]),
                    if_stmt(
                        binary_expr(var_expr("result"), BinOp::StrictNotEq, int_expr(0)),
                        vec![return_stmt(var_expr("result"))],
                        None,
                    ),
                    if_stmt(
                        binary_expr(
                            binary_expr(var_expr("ai"), BinOp::GtEq, var_expr("alen")),
                            BinOp::And,
                            binary_expr(var_expr("bi"), BinOp::GtEq, var_expr("blen")),
                        ),
                        vec![return_stmt(int_expr(0))],
                        None,
                    ),
                    if_stmt(
                        binary_expr(var_expr("ai"), BinOp::GtEq, var_expr("alen")),
                        vec![return_stmt(int_expr(-1))],
                        None,
                    ),
                    if_stmt(
                        binary_expr(var_expr("bi"), BinOp::GtEq, var_expr("blen")),
                        vec![return_stmt(int_expr(1))],
                        None,
                    ),
                    assign_stmt("ca", array_access(var_expr("a"), var_expr("ai"))),
                    assign_stmt("cb", array_access(var_expr("b"), var_expr("bi"))),
                ],
                None,
            ),
            if_stmt(
                var_expr("fold"),
                vec![
                    assign_stmt("ca", function_call("strtoupper", vec![var_expr("ca")])),
                    assign_stmt("cb", function_call("strtoupper", vec![var_expr("cb")])),
                ],
                None,
            ),
            if_stmt(
                binary_expr(var_expr("ca"), BinOp::Lt, var_expr("cb")),
                vec![return_stmt(int_expr(-1))],
                None,
            ),
            if_stmt(
                binary_expr(var_expr("ca"), BinOp::Gt, var_expr("cb")),
                vec![return_stmt(int_expr(1))],
                None,
            ),
            increment_stmt("ai"),
            increment_stmt("bi"),
            if_stmt(
                binary_expr(
                    binary_expr(var_expr("ai"), BinOp::GtEq, var_expr("alen")),
                    BinOp::And,
                    binary_expr(var_expr("bi"), BinOp::GtEq, var_expr("blen")),
                ),
                vec![return_stmt(int_expr(0))],
                None,
            ),
            if_stmt(
                binary_expr(var_expr("ai"), BinOp::GtEq, var_expr("alen")),
                vec![return_stmt(int_expr(-1))],
                None,
            ),
            if_stmt(
                binary_expr(var_expr("bi"), BinOp::GtEq, var_expr("blen")),
                vec![return_stmt(int_expr(1))],
                None,
            ),
        ]),
    ]
}

/// Builds the synthetic method body for the whitespace test php-src's `strnatcmp_ex` skips runs
/// of (`isspace` in the C locale: space, tab, newline, vertical tab, form feed, carriage return).
fn is_nat_space_body() -> Vec<Stmt> {
    let ch = var_expr("ch");
    vec![return_stmt(binary_expr(
        binary_expr(
            binary_expr(ch.clone(), BinOp::StrictEq, string_expr(" ")),
            BinOp::Or,
            binary_expr(ch.clone(), BinOp::StrictEq, string_expr("\t")),
        ),
        BinOp::Or,
        binary_expr(
            binary_expr(
                binary_expr(ch.clone(), BinOp::StrictEq, string_expr("\n")),
                BinOp::Or,
                binary_expr(ch.clone(), BinOp::StrictEq, string_expr("\r")),
            ),
            BinOp::Or,
            binary_expr(
                binary_expr(ch.clone(), BinOp::StrictEq, string_expr("\u{0b}")),
                BinOp::Or,
                binary_expr(ch, BinOp::StrictEq, string_expr("\u{0c}")),
            ),
        ),
    ))]
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
    /// `natsort`: values ascending in php's natural order (case-sensitive).
    Natsort,
    /// `natcasesort`: values ascending in php's natural order, case-insensitively.
    Natcasesort,
}
