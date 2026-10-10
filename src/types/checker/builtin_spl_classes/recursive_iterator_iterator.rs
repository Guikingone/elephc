//! Purpose:
//! Injects RecursiveIteratorIterator metadata and traversal-state synthetic method bodies.
//! Owns stack/depth/mode handling for recursive SPL traversal.
//!
//! Called from:
//! - `super::inject_builtin_spl_classes()`.
//!
//! Key details:
//! - State arrays track active iterators, per-frame traversal state, and frame depths.
//! - Traversal mode controls self-first, child-first, and leaves-only behavior.

use std::collections::HashMap;

use crate::parser::ast::{ClassConst, ClassMethod, ClassProperty, TypeExpr};
use crate::types::traits::FlattenedClass;

use super::common::*;
use super::recursive_iterator_iterator_traversal::*;

/// Inserts class into the supplied builtin metadata registry.
pub(super) fn insert_class(class_map: &mut HashMap<String, FlattenedClass>) {
    class_map.insert(
        "RecursiveIteratorIterator".to_string(),
        FlattenedClass {
            name: "RecursiveIteratorIterator".to_string(),
            span: crate::span::Span::dummy(),
            extends: None,
            implements: vec!["OuterIterator".to_string()],
            is_abstract: false,
            is_final: false,
            is_readonly_class: false,
            properties: recursive_iterator_iterator_properties(),
            methods: spl_recursive_iterator_iterator_methods(),
            attributes: Vec::new(),
            constants: recursive_iterator_iterator_constants(),
            used_traits: Vec::new(),
            trait_aliases: Vec::new(),
        },
    );
}

/// Builds the property list for recursive iterator iterator.
fn recursive_iterator_iterator_properties() -> Vec<ClassProperty> {
    vec![
        storage_property("root", named_type("RecursiveIterator")),
        storage_property("mode", TypeExpr::Int),
        storage_property("flags", TypeExpr::Int),
        storage_property("iterators", array_type()),
        storage_property("states", array_type()),
        storage_property("depths", array_type()),
        storage_property("depth", TypeExpr::Int),
        storage_property("slot", TypeExpr::Int),
        storage_property("currentValid", TypeExpr::Bool),
        // php guards `beginIteration()`/`endIteration()` with an `in_iteration` flag so a nested
        // rewind cannot re-fire the begin hook and an exhausted `valid()` fires the end hook once.
        storage_property("inIteration", TypeExpr::Bool),
    ]
}

/// Builds the method list for SPL recursive iterator iterator.
fn spl_recursive_iterator_iterator_methods() -> Vec<ClassMethod> {
    vec![
        method_with_body(
            "__construct",
            vec![
                // PHP declares this as `Traversable`; the constructor validates at runtime that a
                // `RecursiveIterator` was supplied, so the stored root goes through the assume hook.
                param("iterator", named_type("Traversable")),
                param_default("mode", TypeExpr::Int, int_expr(0)),
                param_default("flags", TypeExpr::Int, int_expr(0)),
            ],
            Some(TypeExpr::Void),
            recursive_iterator_iterator_construct_body(),
        ),
        method_with_body("rewind", Vec::new(), Some(TypeExpr::Void), recursive_iterator_iterator_rewind_body()),
        method_with_body("valid", Vec::new(), Some(TypeExpr::Bool), recursive_iterator_iterator_valid_body()),
        method_with_body("current", Vec::new(), Some(mixed_type()), recursive_iterator_iterator_current_body()),
        method_with_body("key", Vec::new(), Some(mixed_type()), recursive_iterator_iterator_key_body()),
        method_with_body("next", Vec::new(), Some(TypeExpr::Void), recursive_iterator_iterator_next_body()),
        method_with_body(
            "getDepth",
            Vec::new(),
            Some(TypeExpr::Int),
            recursive_iterator_iterator_get_depth_body(),
        ),
        method_with_body(
            "getInnerIterator",
            Vec::new(),
            Some(TypeExpr::Nullable(Box::new(named_type("Iterator")))),
            recursive_iterator_iterator_get_inner_iterator_body(),
        ),
        method_with_body(
            "getSubIterator",
            vec![param_default("level", TypeExpr::Int, int_expr(-1))],
            Some(TypeExpr::Nullable(Box::new(named_type("RecursiveIterator")))),
            recursive_iterator_iterator_get_sub_iterator_body(),
        ),
        method_with_body(
            "__elephcAdvance",
            Vec::new(),
            Some(TypeExpr::Void),
            recursive_iterator_iterator_advance_body(),
        ),
        method_with_body(
            "__elephcSlotForDepth",
            vec![param("level", TypeExpr::Int)],
            Some(TypeExpr::Int),
            recursive_iterator_iterator_slot_for_depth_body(),
        ),
        method_with_body(
            "__elephcAssumeRecursiveIterator",
            vec![param("iterator", mixed_type())],
            Some(named_type("RecursiveIterator")),
            return_body(var_expr("iterator")),
        ),
        // php's traversal hooks; the traversal calls them, subclasses override them.
        method_with_body("beginIteration", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("endIteration", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("beginChildren", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("endChildren", Vec::new(), Some(TypeExpr::Void), Vec::new()),
        method_with_body("nextElement", Vec::new(), Some(TypeExpr::Void), Vec::new()),
    ]
}

/// Provides the Recursive iterator iterator constants helper used by the recursive iterator iterator module.
fn recursive_iterator_iterator_constants() -> Vec<ClassConst> {
    vec![
        class_const("LEAVES_ONLY", 0),
        class_const("SELF_FIRST", 1),
        class_const("CHILD_FIRST", 2),
        class_const("CATCH_GET_CHILD", 16),
    ]
}
