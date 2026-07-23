//! Purpose:
//! Type-checks assignment locals forms.
//! Updates type environments and validates storage-specific rules for locals, arrays, and properties.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::assignments`
//!
//! Key details:
//! - Assignment checking must distinguish value writes, by-reference mutation, nullable access, and declared property contracts.

use crate::errors::CompileError;
use crate::errors::CompileWarning;
use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{
    is_compound_assignment_self_read, CallableTarget, Expr, ExprKind, StaticReceiver, TypeExpr,
};
use crate::span::Span;
use crate::types::{PhpType, TypeEnv};

use super::super::super::builtins::array_arg_is_gradually_acceptable;
use super::super::super::Checker;

/// Extracts the default expression from a null-coalescing assignment to a specific variable.
///
/// Returns `Some(&default)` if `value` is a `NullCoalesce` expression where the current
/// value is a variable matching `name`. Otherwise returns `None`.
///
/// This is used during null-coalescing assignment type-checking to determine whether
/// the assignment targets an existing variable and what its default expression is.
fn null_coalesce_assignment_default<'a>(name: &str, value: &'a Expr) -> Option<&'a Expr> {
    if let ExprKind::NullCoalesce {
        value: current,
        default,
    } = &value.kind
    {
        if matches!(&current.kind, ExprKind::Variable(current_name) if current_name == name) {
            return Some(default);
        }
    }
    None
}

/// Determines the resulting type of a null-coalescing assignment operation.
///
/// Combines the existing type of `existing` with the inferred type of `default_ty`,
/// returning the merged type or an error if the types are incompatible.
/// Handles special cases: `Void` existing types, `Mixed`, and union types.
///
/// Returns `Ok(PhpType)` with the resolved type, or `Err` if the null-coalescing
/// assignment would violate type constraints.
fn null_coalesce_assignment_type(
    checker: &Checker,
    name: &str,
    existing: &PhpType,
    default_ty: &PhpType,
    default: &Expr,
    span: Span,
) -> Result<PhpType, CompileError> {
    if *existing == PhpType::Void {
        return Ok(default_ty.clone());
    }
    if *existing == PhpType::Mixed {
        return Ok(PhpType::Mixed);
    }
    if matches!(existing, PhpType::Union(_)) {
        if *default_ty == PhpType::Void || checker.type_accepts(existing, default_ty) {
            return Ok(existing.clone());
        }
        return Err(CompileError::new(
            span,
            &format!(
                "Type error: null coalescing assignment for ${} must keep {}, got {}",
                name, existing, default_ty
            ),
        ));
    }
    if existing == default_ty || matches!(default.kind, ExprKind::Null) {
        return Ok(existing.clone());
    }
    Err(CompileError::new(
        span,
        &format!(
            "Type error: null coalescing assignment for ${} must keep {}, got {}",
            name, existing, default_ty
        ),
    ))
}

/// Type-checks a simple variable assignment (`$name = value`).
///
/// Handles null-coalescing assignment by extracting the default expression and combining
/// types appropriately. Preserves callable metadata when assigning closures or callable
/// expressions. Updates the type environment with the merged assignment type.
///
/// On success, updates `env` with the resolved type for `name`. On error, returns a
/// type mismatch diagnostic.
pub(super) fn check_assign(
    checker: &mut Checker,
    name: &str,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    // A direct reassignment (`$k = ...`) makes `$k` an ordinary local: it is no
    // longer the boxed `Mixed` `foreach` iteration key. Drop the foreach-key
    // marker so a subsequent `$dst[$k] = $v` is routed by `$k`'s real type (e.g.
    // a string key promotes the destination to `AssocArray`) instead of being
    // forced onto the `Array(Mixed)` / `Op::ArraySetMixedKey` path, which would
    // disagree with the lowering (it routes on the index's runtime IR type) and
    // produce a spurious `AssocArray -> Array(Mixed)` backend error.
    checker.foreach_key_locals.remove(name);

    // PHP allows compound assignment on an undefined variable (`$x += 1`),
    // treating the undefined variable as null/0 with a warning. The parser
    // lowers `$x += 1` to `Assign { name: "x", value: BinaryOp { left:
    // Variable("x"), op: Add, right: 1 } }`. When `$x` is not in `env`, the
    // inference of the BinaryOp would fail with "Undefined variable: $x".
    // Inject the variable as Void (null) so inference treats it as 0/null,
    // matching PHP semantics. `??=` is exempt: null-coalesce is designed to
    // handle undefined variables without warning.
    if !env.contains_key(name) && is_compound_assignment_self_read(value, name, span) {
        let is_null_coalesce = matches!(&value.kind, ExprKind::NullCoalesce { .. });
        env.insert(name.to_string(), PhpType::Void);
        if !is_null_coalesce {
            checker.warnings.push(CompileWarning {
                span,
                message: format!("Undefined variable: ${} (treated as null)", name),
            });
        }
    }

    let null_coalesce_default = null_coalesce_assignment_default(name, value);
    let saved_self_ref_ty = if closure_captures_name_by_ref(value, name) {
        if let Some(previous_ty) = env.insert(name.to_string(), PhpType::Callable) {
            Some(Some(previous_ty))
        } else {
            Some(None)
        }
    } else {
        None
    };
    let callable_source = if let Some(default) = null_coalesce_default {
        if matches!(env.get(name), Some(existing) if *existing == PhpType::Void) {
            default
        } else {
            value
        }
    } else {
        value
    };
    let reflection_class_target = reflection_class_assignment_target(checker, callable_source, env);
    let ty_result: Result<PhpType, CompileError> = (|| {
        if let Some(default) = null_coalesce_default {
            if let Some(existing) = env.get(name).cloned() {
                let default_ty = if existing == PhpType::Void {
                    checker.infer_type_with_assignment_effects(default, env)?
                } else {
                    let mut default_env = env.clone();
                    checker.infer_type_with_assignment_effects(default, &mut default_env)?
                };
                null_coalesce_assignment_type(checker, name, &existing, &default_ty, default, span)
            } else {
                checker.infer_type_with_assignment_effects(value, env)
            }
        } else {
            checker.infer_type_with_assignment_effects(value, env)
        }
    })()
    .and_then(|ty| specialize_callable_array_assignment_type(checker, callable_source, ty, env));
    let metadata_result = match &ty_result {
        Ok(ty) => update_callable_assignment_metadata(checker, name, callable_source, ty, env),
        Err(_) => Ok(()),
    };
    if let Some(previous) = saved_self_ref_ty {
        match previous {
            Some(previous_ty) => {
                env.insert(name.to_string(), previous_ty);
            }
            None => {
                env.remove(name);
            }
        }
    }
    let ty = match ty_result {
        Ok(ty) => ty,
        Err(err) => {
            // Error recovery: a plain `$name = <rhs>` assignment unconditionally binds `$name`
            // in PHP, even when `<rhs>` fails to type-check (the right-hand-side error is a
            // separate, already-reported problem). Bind the variable so later uses are not
            // reported as spurious "Undefined variable" cascades behind the real error, which is
            // still propagated here. Prefer the callee's declared return type over the infectious
            // `Mixed` so the recovered binding does not spuriously widen unrelated typed code that
            // later observes the variable. Only fill in a missing binding; never clobber an
            // existing type narrowed by an earlier assignment. Restricted to function/method/
            // closure bodies: a top-level synthetic binding would leak into the shared `global_env`
            // that every method body clones and corrupt unrelated typed code there.
            if checker.in_callable_body && !env.contains_key(name) {
                let fallback = checker
                    .assignment_recovery_call_return_type(value)
                    .unwrap_or(PhpType::Mixed);
                env.insert(name.to_string(), fallback);
            }
            return Err(err);
        }
    };
    metadata_result?;
    update_reflection_class_assignment_metadata(checker, name, reflection_class_target);
    merge_local_assignment_type(checker, name, &ty, span, env)
}

/// Type-checks a reference alias assignment (`$target =& <source>`).
///
/// The source may be a plain variable (`$b`), an object property (`$obj->prop`), or a
/// call to a by-reference-returning callee (`f()`, `$o->m()`). For a property source the
/// property is recorded for reference-property promotion so codegen routes every access
/// through its ref-cell (write-through). The target is rebound to the source value type
/// and marked as by-reference storage.
pub(super) fn check_ref_assign(
    checker: &mut Checker,
    target: &str,
    source: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    match &source.kind {
        ExprKind::Variable(source_name) => {
            check_ref_assign_variable(checker, target, source_name, span, env)
        }
        ExprKind::PropertyAccess { object, property } => {
            let object_ty = checker.infer_type(object, env)?;
            let target_ty = checker.infer_type(source, env)?;
            if let Some(class) =
                crate::types::checker::single_object_class_name(&object_ty)
            {
                checker
                    .reference_property_promotions
                    .insert((class, property.clone()));
            }
            env.insert(target.to_string(), target_ty);
            checker.active_ref_params.insert(target.to_string());
            clear_callable_metadata(checker, target);
            Ok(())
        }
        ExprKind::DynamicPropertyAccess { object, property: _ } => {
            check_ref_assign_dynamic_property(checker, target, object, span, env)
        }
        ExprKind::StaticPropertyAccess { .. } => {
            // `$e = &self::$n`: the local aliases the static property's global storage.
            // The static property is an always-addressable global slot, so no promotion
            // bookkeeping is needed; the local is typed to the property type and marked
            // by-reference so codegen routes its loads/stores through the shared slot.
            let target_ty = checker.infer_type(source, env)?;
            env.insert(target.to_string(), target_ty);
            checker.active_ref_params.insert(target.to_string());
            clear_callable_metadata(checker, target);
            Ok(())
        }
        ExprKind::FunctionCall { .. }
        | ExprKind::MethodCall { .. }
        | ExprKind::StaticMethodCall { .. }
        | ExprKind::ClosureCall { .. }
        | ExprKind::ExprCall { .. } => {
            let target_ty = checker.infer_type(source, env)?;
            env.insert(target.to_string(), target_ty);
            checker.active_ref_params.insert(target.to_string());
            clear_callable_metadata(checker, target);
            Ok(())
        }
        ExprKind::ArrayAccess { array, index } => {
            let array_ty = checker.infer_type(array, env)?;
            let index_ty = checker.infer_type(index, env)?;
            if !matches!(array_ty, PhpType::Array(_)) {
                return Err(CompileError::new(
                    span,
                    "Reference assignment to an array element requires an indexed array",
                ));
            }
            if !matches!(index_ty, PhpType::Int) {
                return Err(CompileError::new(
                    span,
                    "Reference assignment to an array element requires an integer index",
                ));
            }
            let target_ty = checker.infer_type(source, env)?;
            env.insert(target.to_string(), target_ty);
            checker.active_ref_params.insert(target.to_string());
            clear_callable_metadata(checker, target);
            Ok(())
        }
        _ => Err(CompileError::new(
            span,
            "Reference assignment source must be a variable, array/property element, or a by-reference call",
        )),
    }
}

/// Type-checks a reference alias whose source is a DYNAMIC-named property (`$x = &$this->$name`).
///
/// The runtime property name is unknown at compile time, so any array-typed declared property of
/// the receiver class could be the target. Every such candidate is promoted to a reference
/// property program-wide (reusing the same `reference_property_promotions` mechanism the static
/// `PropertyAccess` arm uses), so each candidate holds a ref-cell and its reads/writes stay
/// ref-correct regardless of which name is selected at runtime. The target local is typed to the
/// candidates' common element type, or `Mixed` when they are heterogeneous, and marked as
/// by-reference storage. A non-object (`Mixed`/`Union`/unknown) receiver is a loud, deferred error
/// rather than a silent miscompile.
fn check_ref_assign_dynamic_property(
    checker: &mut Checker,
    target: &str,
    object: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let object_ty = checker.infer_type(object, env)?;
    let Some(class) = crate::types::checker::single_object_class_name(&object_ty) else {
        return Err(CompileError::new(
            span,
            "Reference to a dynamic property is only supported on a statically-typed object receiver",
        ));
    };
    // Collect the class's array-typed declared properties — the candidates the static arm would
    // have accepted. A single machine-word ref-cell cannot represent a multi-word (string) slot,
    // so only array-typed properties are reference-eligible for the runtime-name path.
    let candidates: Vec<(String, PhpType)> = checker
        .classes
        .get(&class)
        .map(|info| {
            info.properties
                .iter()
                .filter(|(_, ty)| {
                    matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. })
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    if candidates.is_empty() {
        return Err(CompileError::new(
            span,
            "Reference to a dynamic property requires the object to declare at least one array-typed property",
        ));
    }
    // Promote every candidate program-wide via the same mechanism the static `PropertyAccess`
    // arm uses, so each candidate slot holds a ref-cell for the object's lifetime.
    for (property, _) in &candidates {
        checker
            .reference_property_promotions
            .insert((class.clone(), property.clone()));
    }
    // Type the target to the candidates' common element type, widening to `Mixed` when the
    // array-typed candidates are heterogeneous.
    let first_ty = candidates[0].1.clone();
    let target_ty = if candidates.iter().all(|(_, ty)| *ty == first_ty) {
        first_ty
    } else {
        PhpType::Mixed
    };
    // The target of a reference-alias is a plain variable, mirroring the static arm's storage
    // restriction (enforced by the `RefAssign` parser producing a `target: String`).
    env.insert(target.to_string(), target_ty);
    checker.active_ref_params.insert(target.to_string());
    clear_callable_metadata(checker, target);
    Ok(())
}

/// Type-checks a reference assignment whose left-hand side is a property access
/// (`$obj->prop = &$src`) or an array element (`$arr[$k] = &$src`).
///
/// The target lvalue becomes an alias to `source`'s storage. For a property target,
/// the property is promoted to a reference property program-wide so every access lowers
/// through its ref-cell; a property source is promoted too (the cell is shared). For a
/// plain-variable source, the variable is rebound to the property type and marked as
/// by-reference storage (it will alias the property's cell). Array-element targets are
/// not yet supported (PHP references inside arrays need runtime support that does not
/// exist yet) and produce a diagnostic.
pub(super) fn check_ref_assign_to_target(
    checker: &mut Checker,
    target: &Expr,
    source: &Expr,
    append: bool,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    if append {
        return check_ref_assign_append_target(checker, target, source, span, env);
    }
    match &target.kind {
        ExprKind::PropertyAccess { object, property } => {
            let object_ty = checker.infer_type(object, env)?;
            if let Some(class) =
                crate::types::checker::single_object_class_name(&object_ty)
            {
                checker
                    .reference_property_promotions
                    .insert((class.clone(), property.clone()));
                // The target slot is overwritten to alias another object's ref-cell, so its own
                // cell must not be freed by the destructor (double-free guard).
                checker
                    .reference_property_rebind_targets
                    .insert((class, property.clone()));
            }
            // A property source shares its ref-cell, so promote it as well.
            if let ExprKind::PropertyAccess {
                object: src_object,
                property: src_property,
            } = &source.kind
            {
                let src_object_ty = checker.infer_type(src_object, env)?;
                if let Some(class) =
                    crate::types::checker::single_object_class_name(&src_object_ty)
                {
                    checker
                        .reference_property_promotions
                        .insert((class, src_property.clone()));
                }
            }
            let target_ty = checker.infer_type(target, env)?;
            // A plain-variable source will alias the property's cell (reverse-bind), so
            // its type follows the property and it becomes by-reference storage.
            if let ExprKind::Variable(source_name) = &source.kind {
                if !env.contains_key(source_name) {
                    return Err(CompileError::new(
                        span,
                        &format!("Undefined variable: ${}", source_name),
                    ));
                }
                env.insert(source_name.clone(), target_ty);
                checker.active_ref_params.insert(source_name.clone());
                clear_callable_metadata(checker, source_name);
            } else {
                checker.infer_type(source, env)?;
            }
            Ok(())
        }
        ExprKind::ArrayAccess { array, .. } => match &array.kind {
            // `self::$a[$dir] = &self::$a[$k]`: aliasing an element of a static-property array.
            // The dedicated helper resolves + validates both operands and de-packs the container.
            ExprKind::StaticPropertyAccess { receiver, property } => {
                super::static_properties::check_ref_assign_static_prop_element(
                    checker, receiver, property, source, span,
                )
            }
            // `$a[$k] = &$var`: aliasing an explicit-key element of a plain LOCAL array to a
            // plain-variable source (the reverse-bind, mirroring the SLICE-1 `$x = &$a[$k]` shape).
            ExprKind::Variable(array_name) => check_ref_assign_local_array_element(
                checker, array_name, source, false, span, env,
            ),
            // An instance-property-base element (`$obj->arr[$k] = &$x`) target is a follow-up slice.
            _ => Err(CompileError::new(
                span,
                "Reference assignment into an array element is not supported",
            )),
        },
        _ => Err(CompileError::new(
            span,
            "Reference assignment target must be a variable, array element, or object property",
        )),
    }
}

/// Type-checks an append reference-assignment target (`$a[] = &$var`, `$a[$k][] = &$var`, or the
/// out-of-scope `self::$a[] = &$var` / `$obj->p[] = &$var`).
///
/// For the append form the `target` is the CONTAINER itself. Only a plain LOCAL array variable
/// (`$a[] = &$var`, flat) or a nested LOCAL array element (`$a[$k][] = &$var`, whose base is a
/// plain variable) are supported; appending a reference into a static or instance property array
/// is a loud, deferred error (a follow-up slice) rather than a silent value copy.
fn check_ref_assign_append_target(
    checker: &mut Checker,
    target: &Expr,
    source: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    match &target.kind {
        // `$a[] = &$var`: append a reference into a flat local array.
        ExprKind::Variable(array_name) => {
            check_ref_assign_local_array_element(checker, array_name, source, false, span, env)
        }
        // `$a[$k][] = &$var`: append a reference into a nested local array element.
        ExprKind::ArrayAccess { array, .. } => match &array.kind {
            ExprKind::Variable(array_name) => {
                check_ref_assign_local_array_element(checker, array_name, source, true, span, env)
            }
            _ => Err(CompileError::new(
                span,
                "Appending a reference into a nested array element is only supported on a plain local array variable base",
            )),
        },
        // `self::$a[] = &$var` / `$obj->p[] = &$var`: appending a reference into a static or
        // instance property array is not supported (a follow-up slice).
        _ => Err(CompileError::new(
            span,
            "Appending a reference into a static or instance property array is not supported",
        )),
    }
}

/// Type-checks a reference alias INTO a LOCAL array element whose source is a plain variable:
/// `$a[$k] = &$var` (explicit-key, `nested == false`), `$a[] = &$var` (flat append,
/// `nested == false`), or `$a[$k][] = &$var` (nested append, `nested == true`).
///
/// The container local must be array-typed; the source must be a plain variable whose value fits in
/// one reference-cell word (a multi-word string source is a loud, deferred error, mirroring the
/// SLICE-1 string-element guard). The container is de-packed to the promoted associative hash type
/// (nested containers to a two-level hash) so codegen loads/stores/frees it as a hash, and the
/// source variable is reverse-bound: it is retyped to the element type, marked as active
/// by-reference storage, and stripped of any callable metadata so its reads/writes route through the
/// shared cell.
fn check_ref_assign_local_array_element(
    checker: &mut Checker,
    array_name: &str,
    source: &Expr,
    nested: bool,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    // Container eligibility: the base local must be a defined array variable.
    let Some(container_ty) = env.get(array_name).cloned() else {
        return Err(CompileError::new(
            span,
            &format!(
                "Reference assignment into an array element requires an array variable (${array_name} is undefined)"
            ),
        ));
    };
    if !matches!(
        container_ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. }
    ) {
        return Err(CompileError::new(
            span,
            "Reference assignment into an array element requires an array variable",
        ));
    }
    // The reference source MUST be a plain variable for this slice; any other shape (element,
    // property, call) is a follow-up slice and is loud-errored rather than silently value-copied.
    let ExprKind::Variable(src_name) = &source.kind else {
        return Err(CompileError::new(
            span,
            "Reference source for a local array-element reference must be a plain variable (e.g. &$x)",
        ));
    };
    if !env.contains_key(src_name) {
        return Err(CompileError::new(
            span,
            &format!("Undefined variable: ${src_name}"),
        ));
    }
    // A superglobal (`&$_GET`) or `global`-imported source lives in GLOBAL storage, not a
    // frame slot; the kind-6 cell machinery only adopts frame locals today, so binding one
    // silently reads stale/empty data through the entry. Reject loudly (a follow-up slice)
    // instead of miscompiling.
    if crate::superglobals::is_superglobal(src_name) || checker.active_globals.contains(src_name)
    {
        return Err(CompileError::new(
            span,
            "Reference to a superglobal or global-imported source in a local array element is not yet supported",
        ));
    }
    // A by-reference PARAMETER (or by-ref `use` capture) slot holds the caller-provided raw
    // reference ADDRESS, not a kind-6 reference cell; `ensure_local_ref_cell` would wrap that
    // address as the cell's inner VALUE, so every read through the entry yields pointer
    // garbage. Reject loudly (a follow-up slice) instead of miscompiling. Locals that became
    // ref-bound via an earlier `=&` bind are NOT in this set — re-binding them shares the
    // one persistent cell and stays supported.
    if checker.declared_byref_param_locals.contains(src_name) {
        return Err(CompileError::new(
            span,
            "Reference to a by-reference parameter or by-ref capture in a local array element is not yet supported",
        ));
    }
    let element_ty = env.get(src_name).cloned().unwrap_or(PhpType::Mixed);
    // A kind-6 reference cell holds a single inner value word; a multi-word string source would drop
    // its length, so reject it loudly instead of miscompiling.
    if element_ty.codegen_repr() == PhpType::Str {
        return Err(CompileError::new(
            span,
            "Reference to a string-valued source in a local array element is not yet supported",
        ));
    }
    // De-pack retype (checker soundness only; codegen re-derives the de-pack via
    // `set_local_type_exact`). Taking a reference into an indexed array promotes it to a hash so
    // downstream reads, cleanup, and `array_is_list()` agree with the runtime representation. A
    // nested append types the outer container as a two-level hash so both levels de-pack.
    let current_repr = env.get(array_name).map(|ty| ty.codegen_repr());
    if nested {
        if matches!(
            current_repr,
            Some(PhpType::Array(_)) | Some(PhpType::AssocArray { .. })
        ) {
            env.insert(
                array_name.to_string(),
                PhpType::AssocArray {
                    key: Box::new(PhpType::Mixed),
                    value: Box::new(PhpType::AssocArray {
                        key: Box::new(PhpType::Mixed),
                        value: Box::new(PhpType::Mixed),
                    }),
                },
            );
        }
    } else if matches!(current_repr, Some(PhpType::Array(_))) {
        env.insert(
            array_name.to_string(),
            PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            },
        );
    }
    // Reverse-bind: the source variable aliases the element's reference cell. The LOCAL alias
    // keeps the original element type (mirroring the non-append SLICE-1 alias typing at line 284)
    // so the same-type store fast path (`int->int`, `array->array`) routes through the alias
    // without boxing. A whole-value reassign that changes the inner type (int->array, int->string)
    // is read back through the ELEMENT value type (Mixed, widened above): `__rt_deref_if_reference`
    // returns the cell's inner value-tag stamped by `__rt_ref_cell_store`, and the Mixed read
    // path dispatches on it. The alias type only governs how the store represents the value word
    // in the slot; `__rt_ref_cell_store` carries the NEW value's runtime tag independently.
    env.insert(src_name.clone(), element_ty);
    checker.active_ref_params.insert(src_name.clone());
    clear_callable_metadata(checker, src_name);
    Ok(())
}

/// Type-checks `$target =& $source` where the source is a plain variable.
fn check_ref_assign_variable(
    checker: &mut Checker,
    target: &str,
    source: &str,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let source_ty = env.get(source).cloned().ok_or_else(|| {
        CompileError::new(span, &format!("Undefined variable: ${}", source))
    })?;
    if target == source {
        return Ok(());
    }
    env.insert(target.to_string(), source_ty.clone());
    checker.active_ref_params.insert(target.to_string());
    if source_ty == PhpType::Callable || is_callable_array_type(&source_ty) {
        copy_callable_metadata(checker, target, source);
    } else {
        clear_callable_metadata(checker, target);
    }
    copy_reflection_class_metadata(checker, target, source);
    Ok(())
}


/// Returns `true` if `value` is a closure that captures `name` both by value and by reference.
///
/// Used to detect self-referential assignments (`$x = fn() use($x) { ... }`) where the
/// variable being assigned is captured by the closure on both sides of the assignment.
/// When detected, the variable's type is temporarily promoted to `Callable` during inference.
fn closure_captures_name_by_ref(value: &Expr, name: &str) -> bool {
    matches!(
        &value.kind,
        ExprKind::Closure {
            captures,
            capture_refs,
            ..
        } if captures.iter().any(|capture| capture == name)
            && capture_refs.iter().any(|capture| capture == name)
    )
}

impl Checker {
    /// Type-checks a local variable assignment and returns the resulting type.
    ///
    /// Validates that the variable exists in `env` after assignment, returning its type.
    /// Used by the checker when processing assignment expressions to propagate the
    /// assigned type back to the caller.
    pub(crate) fn check_local_assignment_expression(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
        env: &mut TypeEnv,
    ) -> Result<PhpType, CompileError> {
        check_assign(self, name, value, span, env)?;
        env.get(name).cloned().ok_or_else(|| {
            CompileError::new(span, &format!("Undefined variable: ${}", name))
        })
    }
}

/// Updates callability metadata when assigning a callable expression to a variable.
///
/// When `ty` is `Callable`, extracts and stores the callable signature, closure return
/// type, capture list, and first-class callable target on the checker. When `ty` is not
/// callable, clears any previously stored metadata for `name`.
///
/// This ensures that subsequent uses of the variable can resolve its callable signature
/// and closure metadata. Handles closures, variables, array access, and first-class callables.
pub(super) fn update_callable_assignment_metadata(
    checker: &mut Checker,
    name: &str,
    callable_source: &Expr,
    ty: &PhpType,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    update_callable_array_assignment_metadata(checker, name, callable_source, env)?;

    if *ty == PhpType::Callable {
        if let Some(sig) = checker.resolve_expr_callable_sig(callable_source, env)? {
            checker
                .closure_return_types
                .insert(name.to_string(), sig.return_type.clone());
            checker.callable_sigs.insert(name.to_string(), sig);
            if let ExprKind::Closure {
                captures,
                capture_refs,
                ..
            } = &callable_source.kind
            {
                let capture_types = captures
                    .iter()
                    .map(|capture| {
                        (
                            capture.clone(),
                            env.get(capture).cloned().unwrap_or(PhpType::Mixed),
                            capture_refs.iter().any(|ref_capture| ref_capture == capture),
                        )
                    })
                    .collect();
                checker
                    .callable_captures
                    .insert(name.to_string(), capture_types);
            } else if let ExprKind::Variable(src_name) = &callable_source.kind {
                if let Some(captures) = checker.callable_captures.get(src_name).cloned() {
                    checker.callable_captures.insert(name.to_string(), captures);
                } else {
                    checker.callable_captures.remove(name);
                }
            } else if let ExprKind::ArrayAccess { array, .. } = &callable_source.kind {
                if let ExprKind::Variable(src_name) = &array.kind {
                    if let Some(captures) = checker.callable_captures.get(src_name).cloned() {
                        checker.callable_captures.insert(name.to_string(), captures);
                    } else {
                        checker.callable_captures.remove(name);
                    }
                } else {
                    checker.callable_captures.remove(name);
                }
            } else {
                checker.callable_captures.remove(name);
            }
            if let ExprKind::FirstClassCallable(target) = &callable_source.kind {
                checker
                    .first_class_callable_targets
                    .insert(name.to_string(), target.clone());
            } else if let ExprKind::Variable(src_name) = &callable_source.kind {
                if let Some(target) = checker.first_class_callable_targets.get(src_name).cloned() {
                    checker
                        .first_class_callable_targets
                        .insert(name.to_string(), target);
                } else {
                    checker.first_class_callable_targets.remove(name);
                }
            } else if let ExprKind::ArrayAccess { array, .. } = &callable_source.kind {
                if let ExprKind::Variable(src_name) = &array.kind {
                    if let Some(target) =
                        checker.first_class_callable_targets.get(src_name).cloned()
                    {
                        checker
                            .first_class_callable_targets
                            .insert(name.to_string(), target);
                    } else {
                        checker.first_class_callable_targets.remove(name);
                    }
                } else {
                    checker.first_class_callable_targets.remove(name);
                }
            } else {
                checker.first_class_callable_targets.remove(name);
            }
        } else {
            checker.closure_return_types.remove(name);
            checker.callable_sigs.remove(name);
            checker.callable_captures.remove(name);
            checker.first_class_callable_targets.remove(name);
        }
    } else if is_callable_array_type(ty) {
        if let Some(sig) = checker.resolve_expr_callable_array_sig(callable_source, env)? {
            checker
                .closure_return_types
                .insert(name.to_string(), sig.return_type.clone());
            checker.callable_sigs.insert(name.to_string(), sig);
            if let ExprKind::Variable(src_name) = &callable_source.kind {
                if let Some(captures) = checker.callable_captures.get(src_name).cloned() {
                    checker.callable_captures.insert(name.to_string(), captures);
                } else {
                    checker.callable_captures.remove(name);
                }
                if let Some(target) = checker.callable_array_targets.get(src_name).cloned() {
                    checker
                        .callable_array_targets
                        .insert(name.to_string(), target);
                } else {
                    checker.callable_array_targets.remove(name);
                }
                if let Some(target) = checker.first_class_callable_targets.get(src_name).cloned() {
                    checker
                        .first_class_callable_targets
                        .insert(name.to_string(), target);
                } else {
                    checker.first_class_callable_targets.remove(name);
                }
            } else {
                checker.callable_captures.remove(name);
                checker.callable_array_targets.remove(name);
                checker.first_class_callable_targets.remove(name);
            }
        } else {
            clear_callable_metadata(checker, name);
        }
    } else {
        checker.closure_return_types.remove(name);
        checker.callable_sigs.remove(name);
        checker.callable_captures.remove(name);
        checker.first_class_callable_targets.remove(name);
    }
    Ok(())
}

/// Updates static `ReflectionClass` metadata for one assigned local.
fn update_reflection_class_assignment_metadata(
    checker: &mut Checker,
    name: &str,
    reflected_class: Option<String>,
) {
    if let Some(reflected_class) = reflected_class {
        checker
            .reflection_class_targets
            .insert(name.to_string(), reflected_class);
    } else {
        checker.reflection_class_targets.remove(name);
    }
}

/// Mirrors static `ReflectionClass` metadata across a reference alias assignment.
fn copy_reflection_class_metadata(checker: &mut Checker, target: &str, source: &str) {
    if let Some(reflected_class) = checker.reflection_class_targets.get(source).cloned() {
        checker
            .reflection_class_targets
            .insert(target.to_string(), reflected_class);
    } else {
        checker.reflection_class_targets.remove(target);
    }
}

/// Resolves the statically-known class reflected by a `new ReflectionClass(...)` expression.
fn reflection_class_assignment_target(
    checker: &Checker,
    source: &Expr,
    env: &TypeEnv,
) -> Option<String> {
    let ExprKind::NewObject { class_name, args } = &source.kind else {
        return None;
    };
    if php_symbol_key(class_name.as_str().trim_start_matches('\\')) != "reflectionclass" {
        return None;
    }
    let class_arg = reflection_class_constructor_arg(args)?;
    match &class_arg.kind {
        ExprKind::StringLiteral(class_name) => {
            resolve_class_name(checker, class_name).map(str::to_string)
        }
        ExprKind::ClassConstant { receiver } => {
            resolve_static_receiver_class(checker, receiver, class_arg.span).ok()
        }
        _ => match env
            .get(variable_name(class_arg).unwrap_or_default())
            .cloned()
            .unwrap_or_else(|| crate::types::checker::infer_expr_type_syntactic(class_arg))
            .codegen_repr()
        {
            PhpType::Object(class_name) if !class_name.is_empty() => Some(class_name),
            _ => None,
        },
    }
}

/// Returns the ReflectionClass constructor class argument after simple named-argument matching.
fn reflection_class_constructor_arg(args: &[Expr]) -> Option<&Expr> {
    if let Some(positional) = args.iter().find(|arg| !matches!(arg.kind, ExprKind::NamedArg { .. })) {
        return Some(positional);
    }
    args.iter().find_map(|arg| {
        let ExprKind::NamedArg { name, value } = &arg.kind else {
            return None;
        };
        match php_symbol_key(name).as_str() {
            "class" | "classname" | "class_name" | "objectorclass" | "object_or_class" => {
                Some(value.as_ref())
            }
            _ => None,
        }
    })
}

/// Returns true when a local assignment stores an array whose elements are callable descriptors.
fn is_callable_array_type(ty: &PhpType) -> bool {
    match ty {
        PhpType::Array(elem_ty) => elem_ty.as_ref() == &PhpType::Callable,
        PhpType::AssocArray { value, .. } => value.as_ref() == &PhpType::Callable,
        _ => false,
    }
}

/// Specializes generic array assignment types when callable-array metadata is known.
fn specialize_callable_array_assignment_type(
    checker: &mut Checker,
    callable_source: &Expr,
    ty: PhpType,
    env: &TypeEnv,
) -> Result<PhpType, CompileError> {
    if is_callable_array_type(&ty) {
        return Ok(ty);
    }
    if !matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. }) {
        return Ok(ty);
    }
    if checker
        .resolve_expr_callable_array_sig(callable_source, env)?
        .is_none()
    {
        return Ok(ty);
    }
    Ok(match ty {
        PhpType::AssocArray { key, .. } => PhpType::AssocArray {
            key,
            value: Box::new(PhpType::Callable),
        },
        _ => PhpType::Array(Box::new(PhpType::Callable)),
    })
}

/// Provides the Update callable array assignment metadata helper used by the locals module.
fn update_callable_array_assignment_metadata(
    checker: &mut Checker,
    name: &str,
    callable_source: &Expr,
    env: &TypeEnv,
) -> Result<(), CompileError> {
    if let Some(target) = resolve_callable_array_target(checker, callable_source, env)? {
        checker
            .callable_array_targets
            .insert(name.to_string(), target);
    } else if let ExprKind::Variable(src_name) = &callable_source.kind {
        if let Some(target) = checker.callable_array_targets.get(src_name).cloned() {
            checker
                .callable_array_targets
                .insert(name.to_string(), target);
        } else {
            checker.callable_array_targets.remove(name);
        }
    } else {
        checker.callable_array_targets.remove(name);
    }
    Ok(())
}

/// Resolves callable array target using the available compile-time metadata.
fn resolve_callable_array_target(
    checker: &Checker,
    expr: &Expr,
    env: &TypeEnv,
) -> Result<Option<CallableTarget>, CompileError> {
    let Some((receiver, method)) = callable_array_parts(expr) else {
        return Ok(None);
    };
    if let Some(receiver) = static_callable_receiver(checker, receiver, expr.span)? {
        return Ok(Some(CallableTarget::StaticMethod {
            receiver,
            method: method.to_string(),
        }));
    }
    let receiver_ty = env
        .get(variable_name(receiver).unwrap_or_default())
        .cloned()
        .unwrap_or_else(|| crate::types::checker::infer_expr_type_syntactic(receiver));
    if checker.invokable_class_for_type(&receiver_ty).is_some() {
        return Ok(Some(CallableTarget::Method {
            object: Box::new(receiver.clone()),
            method: method.to_string(),
        }));
    }
    Ok(None)
}

/// Provides the Callable array parts helper used by the locals module.
fn callable_array_parts(expr: &Expr) -> Option<(&Expr, &str)> {
    let elems = match &expr.kind {
        ExprKind::ArrayLiteral(elems) => elems,
        _ => return None,
    };
    if elems.len() != 2 {
        return None;
    }
    let ExprKind::StringLiteral(method) = &elems[1].kind else {
        return None;
    };
    Some((&elems[0], method.as_str()))
}

/// Provides the Variable name helper used by the locals module.
fn variable_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name),
        _ => None,
    }
}

/// Provides the Static callable receiver helper used by the locals module.
fn static_callable_receiver(
    checker: &Checker,
    receiver: &Expr,
    span: Span,
) -> Result<Option<StaticReceiver>, CompileError> {
    let class_name = match &receiver.kind {
        ExprKind::StringLiteral(class_name) => resolve_class_name(checker, class_name)
            .map(str::to_string),
        ExprKind::ClassConstant { receiver } => {
            Some(resolve_static_receiver_class(checker, receiver, span)?)
        }
        _ => None,
    };
    Ok(class_name.map(|class_name| StaticReceiver::Named(Name::from(class_name))))
}

/// Resolves static receiver class using the available compile-time metadata.
fn resolve_static_receiver_class(
    checker: &Checker,
    receiver: &StaticReceiver,
    span: Span,
) -> Result<String, CompileError> {
    match receiver {
        StaticReceiver::Named(name) => resolve_class_name(checker, name.as_str())
            .map(str::to_string)
            .ok_or_else(|| CompileError::new(span, &format!("Undefined class: {}", name))),
        StaticReceiver::Self_ | StaticReceiver::Static => checker
            .current_class
            .clone()
            .ok_or_else(|| CompileError::new(span, "Cannot use self::class outside a class context")),
        StaticReceiver::Parent => {
            let current_class = checker.current_class.as_ref().ok_or_else(|| {
                CompileError::new(span, "Cannot use parent::class outside a class context")
            })?;
            checker
                .classes
                .get(current_class)
                .and_then(|class_info| class_info.parent.clone())
                .ok_or_else(|| {
                    CompileError::new(
                        span,
                        &format!("Class '{}' has no parent class", current_class),
                    )
                })
        }
    }
}

/// Resolves class name using the available compile-time metadata.
fn resolve_class_name<'a>(checker: &'a Checker, class_name: &str) -> Option<&'a str> {
    let class_key = php_symbol_key(class_name.trim_start_matches('\\'));
    checker
        .classes
        .keys()
        .find(|existing| php_symbol_key(existing) == class_key)
        .map(String::as_str)
}

/// Merges the assigned type into the type environment for the given variable.
///
/// For an **inferred** local, the resulting type is the flow-insensitive JOIN of the existing
/// type and the newly assigned type: a single concrete type when they merge (e.g. `int` then
/// `bool`), otherwise their least-upper-bound union (which lowers to boxed `Mixed` storage).
/// Reassignment of an inferred local is never rejected — PHP locals are gradually typed.
///
/// For a **declared-typed** local (`Type $x = ...`), the declared hint is enforced: a
/// merge-compatible value or a gradual `Mixed`/union source keeps the declared type, while a
/// concrete-disjoint value (e.g. assigning a `string` to a `?int` local) is a real type error.
///
/// If `name` does not exist, the type is inserted directly.
fn merge_local_assignment_type(
    checker: &Checker,
    name: &str,
    ty: &PhpType,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    if let Some(existing) = env.get(name).cloned() {
        if checker.declared_typed_locals.contains(name) {
            // Declared-typed local: a merge-compatible value or a gradual Mixed/union source
            // keeps the declared hint; a concrete-disjoint value is a real type error.
            if checker.merged_assignment_type(&existing, ty).is_some()
                || checker.gradual_boundary_accepts(&existing, ty)
            {
                return Ok(());
            }
            return Err(CompileError::new(
                span,
                &format!(
                    "Type error: cannot reassign ${} from {} to {}",
                    name, existing, ty
                ),
            ));
        }
        // Inferred local: gradual reassignment widens to the least-upper-bound join instead of
        // rejecting. A union join lowers to a boxed `Mixed` slot for the whole scope, so the
        // backend stores and loads stay representation-consistent.
        let resolved = checker
            .merged_assignment_type(&existing, ty)
            .unwrap_or_else(|| checker.normalize_union_type(vec![existing.clone(), ty.clone()]));
        if resolved != existing {
            env.insert(name.to_string(), resolved);
        }
    } else {
        env.insert(name.to_string(), ty.clone());
    }
    Ok(())
}

/// Type-checks a typed local variable declaration with a type hint (`Type $name = value`).
///
/// Resolves the declared type from `type_expr`, infers the value's type, validates
/// that the value is assignable to the declared type, and inserts the declared type
/// (not the inferred type) into the environment.
///
/// Unlike `check_assign`, this uses the declared type as the final type rather than
/// inferring from the value, enforcing the programmer's type hint.
pub(super) fn check_typed_assign(
    checker: &mut Checker,
    type_expr: &TypeExpr,
    name: &str,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let declared_ty = checker.resolve_declared_local_type_hint(
        type_expr,
        span,
        &format!("Typed local ${}", name),
    )?;
    let value_ty = checker.infer_type(value, env)?;
    if !checker.type_accepts(&declared_ty, &value_ty)
        && !checker.gradual_boundary_accepts(&declared_ty, &value_ty)
    {
        return Err(CompileError::new(
            span,
            &format!(
                "Type error: cannot initialize ${} as {} with {}",
                name, declared_ty, value_ty
            ),
        ));
    }
    // Record the declared hint so later reassignments enforce it (a concrete-disjoint value is a
    // real type error) instead of gradually widening like an inferred local.
    checker.declared_typed_locals.insert(name.to_string());
    env.insert(name.to_string(), declared_ty);
    let reflected_class = reflection_class_assignment_target(checker, value, env);
    update_reflection_class_assignment_metadata(checker, name, reflected_class);
    Ok(())
}

/// Type-checks a constant declaration and records it in the checker's constant table.
///
/// Infers the type of the constant value expression and inserts it into
/// `checker.constants` under `name`. Unlike variable assignments, constants are
/// stored on the checker itself and not in the local type environment.
///
/// Runs with `compile_time_const_depth` incremented: a top-level `const` value is a
/// genuinely compile-time-evaluated context (PHP itself rejects any function call there —
/// "Constant expression contains invalid operations"), so the curated late-bound
/// undefined-function carve-out must not apply inside it.
pub(super) fn check_const_decl(
    checker: &mut Checker,
    name: &str,
    value: &Expr,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    checker.compile_time_const_depth += 1;
    let ty = checker.infer_type(value, env);
    checker.compile_time_const_depth -= 1;
    let ty = ty?;
    checker.constants.entry(name.to_string()).or_insert(ty);
    Ok(())
}

/// Type-checks a list unpacking assignment (`[$a, $b, ...] = $arr`).
///
/// Infers the right-hand side and accepts homogeneous indexed arrays or associative arrays.
/// Indexed arrays propagate their element type, while associative values bind adaptively as
/// `Mixed`. Returns an error for non-array types, including unresolved nullable unions.
pub(super) fn check_list_unpack(
    checker: &mut Checker,
    vars: &[String],
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let arr_ty = checker.infer_type(value, env)?;
    let unpack_ty = match &arr_ty {
        PhpType::Array(elem_ty) => *elem_ty.clone(),
        // Associative arrays can contain integer keys used by positional destructuring. Their
        // element type stays adaptive because hash values may be heterogeneous or absent.
        PhpType::AssocArray { .. } => PhpType::Mixed,
        // Gradual right-hand side (`Mixed`, `?array`, or a union containing an array): the
        // positional element types are not statically homogeneous, so bind each target as
        // `Mixed`. PHP reads each positional offset at runtime (a missing offset yields null
        // with a warning). A proven non-array RHS stays loud below.
        t if array_arg_is_gradually_acceptable(t) => PhpType::Mixed,
        _ => {
            return Err(CompileError::new(
                span,
                "List unpacking requires an array on the right-hand side",
            ));
        }
    };
    for var in vars {
        env.insert(var.clone(), unpack_ty.clone());
        update_list_unpack_callable_metadata(checker, var, value, &unpack_ty);
        checker.reflection_class_targets.remove(var);
    }
    Ok(())
}

/// Updates callable metadata for a variable assigned by list unpacking.
///
/// List unpacking copies an array element into a local without constructing an
/// explicit `ArrayAccess` assignment node. When the homogeneous element type is
/// callable and the source array has known callable metadata, propagate that
/// signature/capture/target information to the destination variable. Non-callable
/// unpack targets clear stale callable metadata.
fn update_list_unpack_callable_metadata(
    checker: &mut Checker,
    dest: &str,
    source_array: &Expr,
    elem_ty: &PhpType,
) {
    if elem_ty != &PhpType::Callable {
        clear_callable_metadata(checker, dest);
        return;
    }
    if let ExprKind::Variable(src_name) = &source_array.kind {
        copy_callable_metadata(checker, dest, src_name);
    } else {
        clear_callable_metadata(checker, dest);
    }
}

/// Copies callable signature, capture, first-class target, and callable-array metadata.
///
/// The checker stores homogeneous callable-array metadata under the source array
/// variable name so later element reads can be treated as callable variables with
/// a known signature. This helper mirrors that metadata onto a list-unpack target.
fn copy_callable_metadata(checker: &mut Checker, dest: &str, src: &str) {
    if let Some(return_ty) = checker.closure_return_types.get(src).cloned() {
        checker
            .closure_return_types
            .insert(dest.to_string(), return_ty);
    } else {
        checker.closure_return_types.remove(dest);
    }
    if let Some(sig) = checker.callable_sigs.get(src).cloned() {
        checker.callable_sigs.insert(dest.to_string(), sig);
    } else {
        checker.callable_sigs.remove(dest);
    }
    if let Some(captures) = checker.callable_captures.get(src).cloned() {
        checker.callable_captures.insert(dest.to_string(), captures);
    } else {
        checker.callable_captures.remove(dest);
    }
    if let Some(target) = checker.callable_array_targets.get(src).cloned() {
        checker
            .callable_array_targets
            .insert(dest.to_string(), target);
    } else {
        checker.callable_array_targets.remove(dest);
    }
    if let Some(target) = checker.first_class_callable_targets.get(src).cloned() {
        checker
            .first_class_callable_targets
            .insert(dest.to_string(), target);
    } else {
        checker.first_class_callable_targets.remove(dest);
    }
}

/// Clears all callable metadata for a list-unpack destination.
fn clear_callable_metadata(checker: &mut Checker, dest: &str) {
    checker.closure_return_types.remove(dest);
    checker.callable_sigs.remove(dest);
    checker.callable_captures.remove(dest);
    checker.callable_array_targets.remove(dest);
    checker.first_class_callable_targets.remove(dest);
}

/// Type-checks a `global` declaration and registers the variables as globals.
///
/// For each variable name, marks it as a global in `checker.active_globals` and
/// populates the local type environment with the variable's type from
/// `checker.top_level_env` if available, otherwise defaults to `Int`.
pub(super) fn check_global(
    checker: &mut Checker,
    vars: &[String],
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    for var in vars {
        checker.active_globals.insert(var.clone());
        if !env.contains_key(var) {
            if let Some(global_ty) = checker.top_level_env.get(var) {
                env.insert(var.clone(), global_ty.clone());
            } else {
                env.insert(var.clone(), PhpType::Int);
            }
        }
    }
    Ok(())
}

/// Type-checks a `static` variable declaration and registers it in the checker.
///
/// Infers the type of the initializer expression, marks the variable as static in
/// `checker.active_statics`, and inserts the inferred type into the local environment.
/// Static variables retain their values across function calls.
pub(super) fn check_static_var(
    checker: &mut Checker,
    name: &str,
    init: &Expr,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let ty = checker.infer_type(init, env)?;
    checker.active_statics.insert(name.to_string());
    env.insert(name.to_string(), ty);
    Ok(())
}
