//! Purpose:
//! Type-checks assignment arrays forms.
//! Updates type environments and validates storage-specific rules for locals, arrays, and properties.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::assignments`
//!
//! Key details:
//! - Assignment checking must distinguish value writes, by-reference mutation, nullable access, and declared property contracts.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::span::Span;
use crate::types::{
    merge_array_key_types, normalized_array_key_type, static_array_key_forces_hash_storage,
    PhpType, TypeEnv,
};

use super::super::super::Checker;

/// Validates and updates the type environment for `$array[$index] = $value` assignments.
///
/// Validates string offsets, merges element types for arrays/assoc-arrays,
/// checks buffer index type and element type compatibility, and requires ArrayAccess for objects.
/// Updates `env` with the merged key/value types; returns an error for invalid targets or type mismatches.
///
/// Errors:
/// - Undefined variable
/// - Invalid string offset index
/// - Buffer element type mismatch or packed buffer assignment via index
/// - Object assignment without ArrayAccess
pub(super) fn check_array_assign(
    checker: &mut Checker,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let arr_ty = env
        .get(array)
        .cloned()
        .or_else(|| crate::globals_array::is_alias(array).then_some(PhpType::Mixed))
        .ok_or_else(|| CompileError::new(span, &format!("Undefined variable: ${}", array)))?;
    let idx_ty = checker.infer_type_with_assignment_effects(index, env)?;
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    super::locals::update_callable_assignment_metadata(checker, array, value, &val_ty, env)?;
    if arr_ty == PhpType::Str {
        if !valid_string_offset_assignment_index(index, &idx_ty) {
            return Err(CompileError::new(
                index.span,
                "String offset assignment index must be integer",
            ));
        }
        return Ok(());
    }
    if let PhpType::Array(elem_ty) = &arr_ty {
        let normalized_idx_ty = normalized_array_key_type(index, idx_ty.clone());
        // A foreach loop key is a boxed `Mixed` cell at runtime (`Op::IterCurrentKey`)
        // even when the checker types it as `Int`/`Str` from the source array, so it
        // may hold either an integer or a string and the destination must stay indexed
        // `Array(Mixed)` with the indexed-vs-hash decision deferred to the runtime
        // write helper (`Op::ArraySetMixedKey`). A non-foreach string-typed key (a
        // literal string, or a string-valued expression like `"k" . $i` or a plain
        // string variable) always means associative hash storage in PHP, so it
        // promotes to `AssocArray` and stays usable by direct string-key reads. A
        // non-foreach `Mixed`-typed key (e.g. a `mixed` parameter) is likewise a
        // runtime-tagged cell, so it stays `Array(Mixed)` to match the lowering's
        // `ArraySetMixedKey` routing.
        let index_is_foreach_key = matches!(&index.kind, ExprKind::Variable(name) if checker.is_foreach_key(name));
        let forces_hash = matches!(normalized_idx_ty, PhpType::Str)
            || (matches!(idx_ty, PhpType::Str) && !index_is_foreach_key)
            || (matches!(elem_ty.as_ref(), PhpType::Never)
                && static_array_key_forces_hash_storage(index));
        if forces_hash {
            let merged_key = if matches!(elem_ty.as_ref(), PhpType::Never) {
                normalized_idx_ty
            } else {
                merge_array_key_types(PhpType::Int, normalized_idx_ty)
            };
            let merged_value = if matches!(elem_ty.as_ref(), PhpType::Never) {
                val_ty
            } else if elem_ty.as_ref() == &val_ty {
                *elem_ty.clone()
            } else {
                checker
                    .merge_array_element_type(elem_ty, &val_ty)
                    .unwrap_or(PhpType::Mixed)
            };
            env.insert(
                array.to_string(),
                PhpType::AssocArray {
                    key: Box::new(merged_key),
                    value: Box::new(merged_value),
                },
            );
        } else if index_is_foreach_key || matches!(idx_ty, PhpType::Mixed) {
            env.insert(
                array.to_string(),
                PhpType::Array(Box::new(PhpType::Mixed)),
            );
        } else if **elem_ty != val_ty {
            let merged_ty = checker
                .merge_array_element_type(elem_ty, &val_ty)
                .unwrap_or(PhpType::Mixed);
            env.insert(array.to_string(), PhpType::Array(Box::new(merged_ty)));
        }
    } else if let PhpType::AssocArray {
        key,
        value: existing_value,
    } = &arr_ty
    {
        let merged_key = merge_array_key_types(
            *key.clone(),
            normalized_array_key_type(index, idx_ty),
        );
        let merged_value = if **existing_value == val_ty {
            *existing_value.clone()
        } else {
            PhpType::Mixed
        };
        env.insert(
            array.to_string(),
            PhpType::AssocArray {
                key: Box::new(merged_key),
                value: Box::new(merged_value),
            },
        );
    } else if let PhpType::Buffer(elem_ty) = &arr_ty {
        if !matches!(idx_ty, PhpType::Int | PhpType::Mixed) {
            return Err(CompileError::new(span, "Buffer index must be integer"));
        }
        match elem_ty.as_ref() {
            PhpType::Packed(_) => {
                return Err(CompileError::new(
                    span,
                    "Assign packed buffer elements through field access like $buf[$i]->field",
                ))
            }
            inner if !buffer_element_accepts_assignment(inner, &val_ty) => {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "Buffer element type mismatch: expected {:?}, got {:?}",
                        inner, val_ty
                    ),
                ));
            }
            _ => {}
        }
    } else if let PhpType::Object(class_name) = &arr_ty {
        if !checker.object_type_implements_interface(class_name, "ArrayAccess") {
            return Err(CompileError::new(
                span,
                "Object array assignment requires ArrayAccess",
            ));
        }
    }
    Ok(())
}

/// Returns whether an assignment index follows PHP's accepted string-offset forms.
fn valid_string_offset_assignment_index(index: &Expr, index_ty: &PhpType) -> bool {
    matches!(index_ty, PhpType::Int | PhpType::Mixed)
        || matches!(
            &index.kind,
            ExprKind::StringLiteral(value)
                if crate::types::parse_php_string_offset_literal(value).is_some()
        )
}

/// Returns whether a buffer element accepts an assignment value after runtime coercion.
fn buffer_element_accepts_assignment(expected: &PhpType, actual: &PhpType) -> bool {
    if expected == actual {
        return true;
    }
    matches!(
        (expected, actual),
        (PhpType::Bool, PhpType::False)
            | (PhpType::Float | PhpType::Int | PhpType::Bool, PhpType::Mixed)
    )
}

/// Validates a nested array assignment like `$arr[$i] = $value` where the target itself is an array access.
///
/// Type-checks the array, index, and value expressions, then validates that the array type supports
/// nested offset assignment. Allows PHP arrays, `Mixed`, and objects implementing `ArrayAccess`;
/// rejects strings and non-container scalars.
///
/// Errors:
/// - Target is not an array access expression
/// - Target is a string (string offset assignment not supported)
/// - Target type does not support nested assignment
pub(super) fn check_nested_array_assign(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let ExprKind::ArrayAccess { array, index } = &target.kind else {
        return Err(CompileError::new(span, "Invalid assignment target"));
    };

    let arr_ty = checker.infer_type_with_assignment_effects(array, env)?;
    checker.infer_type_with_assignment_effects(index, env)?;
    checker.infer_type_with_assignment_effects(value, env)?;
    widen_nested_root_storage(checker, target, env);
    match arr_ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed => Ok(()),
        PhpType::Str => Err(CompileError::new(
            span,
            "String offset assignment is not supported",
        )),
        PhpType::Object(class_name)
            if checker.object_type_implements_interface(&class_name, "ArrayAccess") =>
        {
            Ok(())
        }
        _ => Err(CompileError::new(
            span,
            "Nested array assignment requires a Mixed or ArrayAccess target",
        )),
    }
}

/// Widens the root local or `$this->property` of a nested write to generic array storage.
///
/// A chain such as `$map[$first][$second] = $value` autovivifies an inner array. The checker must
/// expose that inner container to subsequent reads and `foreach` bindings even though its exact
/// indexed-or-hash shape depends on runtime keys. Object roots retain the same refinement only for
/// untyped `$this` properties; declared property contracts remain authoritative.
fn widen_nested_root_storage(checker: &mut Checker, target: &Expr, env: &mut TypeEnv) {
    let mut current = target;
    loop {
        match &current.kind {
            ExprKind::ArrayAccess { array, .. } => current = array,
            ExprKind::Variable(name) => {
                env.insert(
                    name.clone(),
                    PhpType::Array(Box::new(PhpType::Mixed)),
                );
                return;
            }
            ExprKind::PropertyAccess { object, property }
                if matches!(&object.kind, ExprKind::This) =>
            {
                let Some(class_name) = checker.current_class.clone() else {
                    return;
                };
                super::properties::refine_object_property_type(
                    checker,
                    &class_name,
                    property,
                    &PhpType::Array(Box::new(PhpType::Mixed)),
                );
                return;
            }
            _ => return,
        }
    }
}

/// Validates and updates the type environment for `$array[] = $value` (push) assignments.
///
/// Type-checks the value, then merges it into the element type of the array.
/// For `PhpType::Array`, updates the element type in `env` to the merged type.
/// For `PhpType::AssocArray`, merges the pushed value type and adds integer keys.
/// For buffers, returns an error (buffers do not support push).
/// For objects implementing `ArrayAccess`, allows the push without element type merging.
///
/// Errors:
/// - Undefined variable
/// - Buffer push (buffers require `buffer_new<T>(len)` for allocation)
/// - Object push without `ArrayAccess`
pub(super) fn check_array_push(
    checker: &mut Checker,
    array: &str,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let arr_ty = env
        .get(array)
        .cloned()
        .or_else(|| crate::globals_array::is_alias(array).then_some(PhpType::Mixed))
        .ok_or_else(|| CompileError::new(span, &format!("Undefined variable: ${}", array)))?;
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    super::locals::update_callable_assignment_metadata(checker, array, value, &val_ty, env)?;
    if let PhpType::Array(elem_ty) = &arr_ty {
        if **elem_ty != val_ty {
            let merged_ty = checker
                .merge_array_element_type(elem_ty, &val_ty)
                .unwrap_or(PhpType::Mixed);
            env.insert(array.to_string(), PhpType::Array(Box::new(merged_ty)));
        }
    } else if let PhpType::AssocArray {
        key,
        value: existing_value,
    } = &arr_ty
    {
        let merged_key = merge_array_key_types(*key.clone(), PhpType::Int);
        let merged_value = if **existing_value == val_ty {
            *existing_value.clone()
        } else {
            PhpType::Mixed
        };
        env.insert(
            array.to_string(),
            PhpType::AssocArray {
                key: Box::new(merged_key),
                value: Box::new(merged_value),
            },
        );
    } else if matches!(arr_ty, PhpType::Buffer(_)) {
        return Err(CompileError::new(
            span,
            "buffer<T> does not support push; allocate with buffer_new<T>(len)",
        ));
    } else if let PhpType::Object(class_name) = &arr_ty {
        if !checker.object_type_implements_interface(class_name, "ArrayAccess") {
            return Err(CompileError::new(
                span,
                "Object array push requires ArrayAccess",
            ));
        }
    }
    Ok(())
}
