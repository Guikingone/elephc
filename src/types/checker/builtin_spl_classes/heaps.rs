//! Purpose:
//! Injects SPL heap and priority-queue metadata backed by ordinary object properties.
//! The synthetic method bodies provide PHP-visible ordering, destructive iteration, and flags.
//!
//! Called from:
//! - `super::inject_builtin_spl_classes()`.
//!
//! Key details:
//! - Physical binary-heap order and comparison timing follow the pinned Zend implementation.
//! - Storage lives in per-instance arrays, allowing normal object deep-free to finalize handles.

use std::collections::HashMap;

use crate::parser::ast::{BinOp, ClassConst, ClassMethod, ClassProperty, Expr, Stmt, TypeExpr, Visibility};
use crate::types::traits::FlattenedClass;

use super::common::*;

mod debug_info;
mod mutation;
mod readers;
mod state;
mod serialization;
mod unserialization;

use debug_info::*;
use mutation::*;
use readers::*;
use state::*;
use serialization::serialize_body;
use unserialization::unserialize_body;

/// Inserts SPL heap, max/min heap, and priority queue classes into the builtin registry.
pub(super) fn insert_classes(class_map: &mut HashMap<String, FlattenedClass>) {
    class_map.insert(
        "SplHeap".to_string(),
        FlattenedClass {
            name: "SplHeap".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec!["Iterator".to_string(), "Countable".to_string()],
            is_abstract: true,
            is_final: false,
            is_readonly_class: false,
            properties: spl_heap_properties(),
            methods: spl_heap_methods(),
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );

    class_map.insert(
        "SplMaxHeap".to_string(),
        FlattenedClass {
            name: "SplMaxHeap".to_string(),
            span: crate::span::Span::dummy(),
            extends: Some("SplHeap".to_string()),
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: Vec::new(),
            methods: vec![spl_max_heap_compare_method()],
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );

    class_map.insert(
        "SplMinHeap".to_string(),
        FlattenedClass {
            name: "SplMinHeap".to_string(),
            span: crate::span::Span::dummy(),
            extends: Some("SplHeap".to_string()),
            implements: Vec::new(),
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: Vec::new(),
            methods: vec![spl_min_heap_compare_method()],
            attributes: Vec::new(),
            constants: Vec::new(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );

    class_map.insert(
        "SplPriorityQueue".to_string(),
        FlattenedClass {
            name: "SplPriorityQueue".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec!["Iterator".to_string(), "Countable".to_string()],
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: spl_priority_queue_properties(),
            methods: spl_priority_queue_methods(),
            attributes: Vec::new(),
            constants: spl_priority_queue_constants(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );
}

/// Returns the hidden per-instance storage fields used by `SplHeap`.
fn spl_heap_properties() -> Vec<ClassProperty> {
    heap_properties(false)
}

/// Returns public and protected methods for the abstract `SplHeap` base class.
fn spl_heap_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body("__construct", Vec::new(), Some(TypeExpr::Void), construct_body(false)),
        method_with_body("insert", vec![param("value", mixed_type())], Some(TypeExpr::Bool), insert_body(false)),
        method_with_body("extract", Vec::new(), Some(mixed_type()), heap_extract_body()),
        method_with_body("top", Vec::new(), Some(mixed_type()), heap_top_body()),
        method_with_body("count", Vec::new(), Some(TypeExpr::Int), heap_count_body()),
        method_with_body("isEmpty", Vec::new(), Some(TypeExpr::Bool), heap_is_empty_body()),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("current", Vec::new(), Some(mixed_type()), heap_current_body()),
        method_with_body("key", Vec::new(), Some(mixed_type()), heap_key_body()),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), heap_next_body()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), heap_valid_body()),
        method_with_body("recoverFromCorruption", Vec::new(), Some(TypeExpr::Bool), recover_body()),
        method_with_body("isCorrupted", Vec::new(), Some(TypeExpr::Bool), return_body(corrupted_expr())),
        method_with_body("__debugInfo", Vec::new(), Some(php_array_contract_type()), debug_info_body(false)),
        method_with_body("__serialize", Vec::new(), Some(php_array_contract_type()), serialize_body(false)),
        method_with_body("__unserialize", vec![param("data", php_array_contract_type())], Some(TypeExpr::Void), unserialize_body(false)),
        protected_abstract_method(
            "compare",
            vec![param("value1", mixed_type()), param("value2", mixed_type())],
            Some(TypeExpr::Int),
        ),
        private_method_with_body("__elephcHeapAssert", vec![param("write", TypeExpr::Bool)], Some(TypeExpr::Void), consistency_body()),
        private_method_with_body("__elephcHeapCompare", vec![param("left", mixed_type()), param("right", mixed_type())], Some(TypeExpr::Int), compare_body()),
        private_method_with_body("__elephcHeapFinish", Vec::new(), Some(TypeExpr::Void), finish_body()),
        private_method_with_body("__elephcHeapRemoveTop", Vec::new(), Some(TypeExpr::Void), remove_top_body(false)),
    ]
}

/// Builds the protected `SplMaxHeap::compare()` override.
fn spl_max_heap_compare_method() -> ClassMethod {
    protected_method_with_body(
        "compare",
        vec![param("value1", mixed_type()), param("value2", mixed_type())],
        Some(TypeExpr::Int),
        return_body(binary_expr(var_expr("value1"), BinOp::Spaceship, var_expr("value2"))),
    )
}

/// Builds the protected `SplMinHeap::compare()` override.
fn spl_min_heap_compare_method() -> ClassMethod {
    protected_method_with_body(
        "compare",
        vec![param("value1", mixed_type()), param("value2", mixed_type())],
        Some(TypeExpr::Int),
        return_body(binary_expr(var_expr("value2"), BinOp::Spaceship, var_expr("value1"))),
    )
}

/// Returns the hidden storage fields used by `SplPriorityQueue`.
fn spl_priority_queue_properties() -> Vec<ClassProperty> {
    heap_properties(true)
}

/// Returns all public and hidden methods for `SplPriorityQueue`.
fn spl_priority_queue_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body("__construct", Vec::new(), Some(TypeExpr::Void), construct_body(true)),
        method_with_body(
            "compare",
            vec![param("priority1", mixed_type()), param("priority2", mixed_type())],
            Some(TypeExpr::Int),
            return_body(binary_expr(var_expr("priority1"), BinOp::Spaceship, var_expr("priority2"))),
        ),
        method_with_body(
            "insert",
            vec![param("value", mixed_type()), param("priority", mixed_type())],
            Some(TypeExpr::Bool),
            insert_body(true),
        ),
        method_with_body("setExtractFlags", vec![param("flags", TypeExpr::Int)], Some(TypeExpr::Int), priority_set_extract_flags_body()),
        method_with_body("getExtractFlags", Vec::new(), Some(TypeExpr::Int), return_body(priority_flags_expr())),
        method_with_body("extract", Vec::new(), Some(mixed_type()), priority_extract_body()),
        method_with_body("top", Vec::new(), Some(mixed_type()), priority_top_body()),
        method_with_body("count", Vec::new(), Some(TypeExpr::Int), heap_count_body()),
        method_with_body("isEmpty", Vec::new(), Some(TypeExpr::Bool), heap_is_empty_body()),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("current", Vec::new(), Some(mixed_type()), priority_current_body()),
        method_with_body("key", Vec::new(), Some(mixed_type()), heap_key_body()),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), heap_next_body()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), heap_valid_body()),
        method_with_body("recoverFromCorruption", Vec::new(), Some(TypeExpr::Bool), recover_body()),
        method_with_body("isCorrupted", Vec::new(), Some(TypeExpr::Bool), return_body(corrupted_expr())),
        method_with_body("__debugInfo", Vec::new(), Some(php_array_contract_type()), debug_info_body(true)),
        method_with_body("__serialize", Vec::new(), Some(php_array_contract_type()), serialize_body(true)),
        method_with_body("__unserialize", vec![param("data", php_array_contract_type())], Some(TypeExpr::Void), unserialize_body(true)),
        private_method_with_body("__elephcHeapAssert", vec![param("write", TypeExpr::Bool)], Some(TypeExpr::Void), consistency_body()),
        private_method_with_body("__elephcHeapCompare", vec![param("left", mixed_type()), param("right", mixed_type())], Some(TypeExpr::Int), compare_body()),
        private_method_with_body("__elephcHeapFinish", Vec::new(), Some(TypeExpr::Void), finish_body()),
        private_method_with_body("__elephcHeapRemoveTop", Vec::new(), Some(TypeExpr::Void), remove_top_body(true)),
        private_method_with_body("__elephcHeapOutput", vec![param("data", mixed_type()), param("priority", mixed_type())], Some(mixed_type()), priority_output_body()),
    ]
}

/// Returns public extraction constants for `SplPriorityQueue`.
fn spl_priority_queue_constants() -> Vec<ClassConst> {
    vec![
        class_const("EXTR_DATA", 1),
        class_const("EXTR_PRIORITY", 2),
        class_const("EXTR_BOTH", 3),
    ]
}

/// Builds a protected concrete synthetic method.
fn protected_method_with_body(
    name: &str,
    params: Vec<(String, Option<TypeExpr>, Option<Expr>, bool)>,
    return_type: Option<TypeExpr>,
    body: Vec<Stmt>,
) -> ClassMethod {
    let mut method = method_with_body(name, params, return_type, body);
    method.visibility = Visibility::Protected;
    method
}

/// Builds a protected abstract synthetic method.
fn protected_abstract_method(
    name: &str,
    params: Vec<(String, Option<TypeExpr>, Option<Expr>, bool)>,
    return_type: Option<TypeExpr>,
) -> ClassMethod {
    let mut method = abstract_method(name, params, return_type);
    method.visibility = Visibility::Protected;
    method
}

/// Builds a private synthetic helper so user subclass methods cannot intercept internal state changes.
fn private_method_with_body(
    name: &str,
    params: Vec<(String, Option<TypeExpr>, Option<Expr>, bool)>,
    return_type: Option<TypeExpr>,
    body: Vec<Stmt>,
) -> ClassMethod {
    let mut method = method_with_body(name, params, return_type, body);
    method.visibility = Visibility::Private;
    method
}
