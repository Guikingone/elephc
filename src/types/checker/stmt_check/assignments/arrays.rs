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
/// Validates that the target is not a string, merges element types for arrays/assoc-arrays,
/// checks buffer index type and element type compatibility, and requires ArrayAccess for objects.
/// Updates `env` with the merged key/value types; returns an error for invalid targets or type mismatches.
///
/// Errors:
/// - Undefined variable
/// - String offset assignment
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
        .ok_or_else(|| CompileError::new(span, &format!("Undefined variable: ${}", array)))?;
    let idx_ty = checker.infer_type_with_assignment_effects(index, env)?;
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    super::locals::update_callable_assignment_metadata(checker, array, value, &val_ty, env)?;
    if arr_ty == PhpType::Str {
        // PHP string offset assignment (`$s[$i] = $c`): write the replacement's first byte at
        // byte offset `$i`. Supported for a plain string local with an int-coercible offset and
        // a weakly string-coercible replacement; the runtime helper copies the source into fresh
        // storage (never mutating a shared string in place), so aliases stay copy-on-write safe.
        // The offset may be a boxed `Mixed`/scalar (a widened integer loop counter such as
        // `$s[++$j]`): the lowering coerces it to int via the same `__rt_mixed_cast_int` path PHP
        // uses. The replacement follows the SAME admitted set, because PHP applies its ordinary
        // weak string conversion to it before taking the first byte (`$s[0] = 65` writes `6`,
        // `$s[0] = true` writes `1`); `lower_string_offset_set` emits that conversion, and the
        // `Op::StrOffsetSet` codegen raises PHP's `Error: Cannot assign an empty string to a
        // string offset` when the converted replacement is empty (`null`/`false`/`""`). A
        // reference-bound local would need a write-through path and stays loud.
        let offset_is_int_coercible = matches!(
            idx_ty.codegen_repr(),
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Mixed
        );
        let value_is_string_coercible = matches!(
            val_ty.codegen_repr(),
            PhpType::Str | PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Mixed
        );
        if offset_is_int_coercible
            && value_is_string_coercible
            && !checker.active_ref_params.contains(array)
        {
            // The local stays a string; leave `env` unchanged.
            return Ok(());
        }
        return Err(CompileError::new(
            span,
            "String offset assignment is not supported",
        ));
    }
    // A reference-bound local whose alias inner is a scalar (int/float/bool) cannot be indexed
    // as an array: PHP fatals "Cannot use a scalar value as an array". The checker types such a
    // local to the referenced element type, so a scalar type here means the alias cell holds a
    // scalar at runtime. Reject loudly instead of silently miscompiling through the runtime
    // Mixed-box writer (which would mutate a non-array payload).
    if checker.active_ref_params.contains(array) && type_is_scalar_for_array_index(&arr_ty) {
        return Err(CompileError::new(
            span,
            "Cannot use a scalar value as an array",
        ));
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
/// nested offset assignment. Allows `Mixed`, empty-array misses that can autovivify, and objects
/// implementing `ArrayAccess`; rejects strings and unsupported concrete container unions.
///
/// Errors:
/// - Target is not an array access expression
/// - Target is a string (string offset assignment not supported)
/// - Target type does not support nested assignment (not gradual/autovivifiable or `ArrayAccess`)
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
    // A NESTED write whose base is a reference-bound LOCAL (`$x = &$arr[0]` then `$x[1][0] = 9`)
    // routes through the explicit per-level write-back lowering (`lower_nested_ref_bound_local_assign`),
    // which materializes each intermediate as the correct container and writes the mutated inner back
    // through the kind-6 ref cell. The alias inner is runtime-tagged (Mixed box / hash / array), so
    // accept any container-shaped target here and let the lowering handle the per-level dispatch.
    // A scalar intermediate (the union-typed `Heap(Hash) got I64` scenario) is loud-errored below.
    let root_is_ref_bound = nested_array_access_root_variable(target)
        .map(|name| checker.active_ref_params.contains(name))
        .unwrap_or(false);
    let root_is_concrete_local =
        nested_local_access_chain_is_concrete(checker, target, env);
    let static_property_chain_is_supported =
        nested_static_property_chain_is_supported(checker, target, env);
    match arr_ty {
        PhpType::Mixed => {
            record_empty_root_nested_write(target, env);
            Ok(())
        }
        PhpType::Never if nested_array_access_root_started_empty(target, env) => {
            record_empty_root_nested_write(target, env);
            Ok(())
        }
        PhpType::Str => Err(CompileError::new(
            span,
            "String offset assignment is not supported",
        )),
        PhpType::Object(class_name)
            if checker.object_type_implements_interface(&class_name, "ArrayAccess") =>
        {
            Ok(())
        }
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Union(_) if root_is_ref_bound => {
            Ok(())
        }
        // A homogeneous matrix (`array<array<T>>`) can preserve its concrete element type:
        // EIR lowering COW-splits every intermediate into an owned temp, mutates the leaf,
        // then writes each child back into its parent. Restrict this path to indexed roots
        // and statically-integer keys; gradual/string keys still need the boxed-Mixed
        // autovivification path and must remain loud until that representation is selected.
        PhpType::Array(_) | PhpType::AssocArray { .. }
            if root_is_concrete_local || static_property_chain_is_supported =>
        {
            Ok(())
        }
        // KEPT LOUD (campaign H1, PART C): a `array|false`/`?array` nested target
        // (`$arr[$k]["j"] = v` where `$arr[$k]` is `array|false`) was probed for the same
        // false/null auto-vivify acceptance as the single-level `$this->v["k"] = v` case in
        // `properties.rs`, but the nested-write lowering (`lower_nested_array_assign`'s general
        // fallback) reads the leaf through `__rt_mixed_array_get` and mutates whatever cell that
        // read returns in place. For a MISS/scalar leaf (exactly the false/null-vivify case this
        // feature is about) that read allocates a fresh, disconnected `Mixed(null)` cell instead
        // of a live pointer into the parent structure, so the write silently no-ops — proven with
        // the PRE-EXISTING `PhpType::Mixed` arm too (`$x[0]["k"]=1` on a `false`-valued Mixed
        // leaf from `json_decode` already silently no-ops on this branch, independent of this
        // campaign). Accepting the nested Union case here would let a checker-legal program
        // silently drop exactly the write PART C exists to support — the cardinal sin. Filed as
        // a follow-up bug affecting the existing `Mixed` arm as well as the prospective `Union`
        // relaxation; fixing it needs the nested-write lowering to vivify into the PARENT slot
        // via a proper set, not the read-then-mutate-same-cell trick.
        _ => Err(CompileError::new(
            span,
            "Nested array assignment requires a Mixed or ArrayAccess target",
        )),
    }
}

/// Returns whether a local-rooted nested chain has a concrete container at every level.
///
/// Indexed arrays require integer-shaped keys, while associative arrays accept proven integer
/// or string keys. This mirrors the explicit EIR child-to-parent write-back path, allowing both
/// homogeneous matrices and maps whose values are concrete indexed arrays.
fn nested_local_access_chain_is_concrete(
    checker: &mut Checker,
    target: &Expr,
    env: &TypeEnv,
) -> bool {
    let Some(root_name) = nested_array_access_root_variable(target) else {
        return false;
    };
    let Some(mut container_ty) = env.get(root_name).cloned() else {
        return false;
    };
    let mut keys = Vec::new();
    let mut node = target;
    loop {
        match &node.kind {
            ExprKind::ArrayAccess { array, index } => {
                keys.push(index.as_ref());
                node = array;
            }
            ExprKind::Variable(_) => break,
            _ => return false,
        }
    }
    keys.reverse();
    for key in keys {
        let Ok(key_ty) = checker.infer_type(key, env) else {
            return false;
        };
        let key_ty = normalized_array_key_type(key, key_ty).codegen_repr();
        container_ty = match container_ty.codegen_repr() {
            PhpType::Array(element_ty) if key_ty == PhpType::Int => *element_ty,
            PhpType::AssocArray { value, .. }
                if matches!(key_ty, PhpType::Int | PhpType::Str | PhpType::Mixed) =>
            {
                *value
            }
            _ => return false,
        };
    }
    true
}

/// Returns whether a two-level static-property write matches the dedicated EIR lowering.
///
/// `C::$cache[$outer][$inner] = value` is supported after a preceding static-element `=&` alias
/// has promoted the property schema to associative reference-cell storage. The outer key must be
/// a valid PHP hash key and the inner tuple/list key integer-shaped. Ordinary static arrays remain
/// loud because the specialized lowering requires the shared cell installed by that alias.
fn nested_static_property_chain_is_supported(
    checker: &mut Checker,
    target: &Expr,
    env: &TypeEnv,
) -> bool {
    let ExprKind::ArrayAccess {
        array: parent,
        index: inner_key,
    } = &target.kind
    else {
        return false;
    };
    let ExprKind::ArrayAccess {
        array: root,
        index: outer_key,
    } = &parent.kind
    else {
        return false;
    };
    if !matches!(root.kind, ExprKind::StaticPropertyAccess { .. }) {
        return false;
    }
    let Ok(root_ty) = checker.infer_type(root, env) else {
        return false;
    };
    if !matches!(root_ty.codegen_repr(), PhpType::AssocArray { .. }) {
        return false;
    }
    let Ok(outer_ty) = checker.infer_type(outer_key, env) else {
        return false;
    };
    let outer_ty = normalized_array_key_type(outer_key, outer_ty).codegen_repr();
    if !matches!(outer_ty, PhpType::Int | PhpType::Str | PhpType::Mixed) {
        return false;
    }
    let Ok(inner_ty) = checker.infer_type(inner_key, env) else {
        return false;
    };
    normalized_array_key_type(inner_key, inner_ty).codegen_repr() == PhpType::Int
}

/// Returns whether a nested access chain starts from a local currently typed as an empty array.
fn nested_array_access_root_started_empty(target: &Expr, env: &TypeEnv) -> bool {
    nested_array_access_root(target)
        .and_then(|(name, _)| env.get(name))
        .is_some_and(
            |ty| matches!(ty, PhpType::Array(elem) if matches!(elem.as_ref(), PhpType::Never)),
        )
}

/// Records the storage shape produced when a nested write autovivifies an empty local.
///
/// Integer keys that preserve packed storage widen the root to `array<mixed>`. String,
/// null, non-zero literal integer, and gradual keys use a Mixed-key hash, matching the
/// EIR path that promotes ambiguous runtime keys before fetching the parent for write.
fn record_empty_root_nested_write(target: &Expr, env: &mut TypeEnv) {
    let Some((name, root_index)) = nested_array_access_root(target) else {
        return;
    };
    if !matches!(
        env.get(name),
        Some(PhpType::Array(elem)) if matches!(elem.as_ref(), PhpType::Never)
    ) {
        return;
    }
    let key_ty = normalized_array_key_type(root_index, PhpType::Mixed);
    let updated = if matches!(key_ty, PhpType::Int)
        && !static_array_key_forces_hash_storage(root_index)
    {
        PhpType::Array(Box::new(PhpType::Mixed))
    } else {
        PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(PhpType::Mixed),
        }
    };
    env.insert(name.to_string(), updated);
}

/// Returns the root local and its first key for a nested `ArrayAccess` chain.
fn nested_array_access_root(target: &Expr) -> Option<(&str, &Expr)> {
    let mut node = target;
    loop {
        match &node.kind {
            ExprKind::ArrayAccess { array, index } => {
                if let ExprKind::Variable(name) = &array.kind {
                    return Some((name, index));
                }
                node = array;
            }
            _ => return None,
        }
    }
}

/// Walks an `ArrayAccess` chain (`$x[1][0]`, `$x["a"]["b"]`) to its root `ExprKind::Variable`,
/// returning the variable name. Returns `None` if the chain bottoms out in a non-variable
/// expression (property/static-property/dynamic-property base).
fn nested_array_access_root_variable(target: &Expr) -> Option<&str> {
    nested_array_access_root(target).map(|(name, _)| name)
}

/// Returns true when `ty` is a scalar PHP value that cannot be indexed as an array (int, float,
/// bool, null). Used to loud-error "Cannot use a scalar value as an array" for reference-bound
/// locals whose alias inner is scalar, instead of silently miscompiling.
fn type_is_scalar_for_array_index(ty: &PhpType) -> bool {
    matches!(
        ty.codegen_repr(),
        PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Void | PhpType::Never
    )
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
