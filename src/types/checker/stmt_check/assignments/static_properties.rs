//! Purpose:
//! Type-checks assignment static properties forms.
//! Updates type environments and validates storage-specific rules for locals, arrays, and properties.
//!
//! Called from:
//! - `crate::types::checker::stmt_check::assignments`
//!
//! Key details:
//! - Assignment checking must distinguish value writes, by-reference mutation, nullable access, and declared property contracts.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::span::Span;
use crate::types::{normalized_array_key_type, PhpType, TypeEnv};

use super::super::super::Checker;

/// Internal data for static property assignment resolution.
/// Holds the resolved class, declaring class, declared-type status, and current property type.
struct StaticPropertyAssignmentTarget {
    class_name: String,
    declaring_class: String,
    property_has_declared_type: bool,
    prop_ty: PhpType,
    dynamic_eval_target: bool,
}

/// Type-checks a direct static property assignment `Class::$prop = value`.
///
/// Infers the value type, resolves the property target via `resolve_static_property_assignment_target`,
/// validates type compatibility against declared types, and refines the property type when no
/// declared type is present.
pub(super) fn check_static_property_assign(
    checker: &mut Checker,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    let target = resolve_static_property_assignment_target(checker, receiver, property, span)?;

    if target.property_has_declared_type {
        checker.require_compatible_arg_type(
            &target.prop_ty,
            &val_ty,
            span,
            &format!("Static property {}::${}", target.class_name, property),
        )?;
    }

    if !target.property_has_declared_type {
        refine_static_property_assignment_type(
            checker,
            property,
            &target.declaring_class,
            &val_ty,
        );
    }
    Ok(())
}

/// Type-checks an array-push assignment `Class::$prop[] = value`.
///
/// Infers the value type, resolves the property target, validates element-type compatibility
/// against declared types, merges element types when the property is untyped, and updates the
/// property type. Rejects buffer properties and non-array static properties.
pub(super) fn check_static_property_array_push(
    checker: &mut Checker,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    let target = resolve_static_property_assignment_target(checker, receiver, property, span)?;
    if target.dynamic_eval_target {
        return Ok(());
    }
    let updated_prop_ty = match target.prop_ty {
        PhpType::Array(elem_ty) => {
            if target.property_has_declared_type {
                checker.require_compatible_arg_type(
                    elem_ty.as_ref(),
                    &val_ty,
                    span,
                    &format!("Static property {}::${}[]", target.class_name, property),
                )?;
                PhpType::Array(elem_ty)
            } else if *elem_ty == val_ty {
                PhpType::Array(elem_ty)
            } else {
                let merged_ty = checker
                    .merge_array_element_type(&elem_ty, &val_ty)
                    .unwrap_or(PhpType::Mixed);
                PhpType::Array(Box::new(merged_ty))
            }
        }
        PhpType::Int | PhpType::Void if !target.property_has_declared_type => {
            PhpType::Array(Box::new(val_ty.clone()))
        }
        PhpType::Buffer(_) => {
            return Err(CompileError::new(
                span,
                "buffer<T> does not support push; allocate with buffer_new<T>(len)",
            ))
        }
        other => {
            return Err(CompileError::new(
                span,
                &format!("Array push requires an array static property, got {}", other),
            ))
        }
    };

    if !target.property_has_declared_type {
        update_static_property_type(
            checker,
            property,
            &target.declaring_class,
            updated_prop_ty,
        );
    }
    Ok(())
}

/// Type-checks an indexed array assignment `Class::$prop[index] = value`.
///
/// Infers the index and value types, resolves the property target, validates integer index,
/// validates element-type compatibility against declared types, merges element types when the
/// property is untyped, and updates the property type. Short-circuits for `ArrayAccess` objects.
pub(super) fn check_static_property_array_assign(
    checker: &mut Checker,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let idx_ty = checker.infer_type_with_assignment_effects(index, env)?;
    let val_ty = checker.infer_type_with_assignment_effects(value, env)?;
    let target = resolve_static_property_assignment_target(checker, receiver, property, span)?;
    if target.dynamic_eval_target {
        let normalized_idx_ty = normalized_array_key_type(index, idx_ty);
        if !is_dynamic_eval_static_property_array_key_type(&normalized_idx_ty) {
            return Err(CompileError::new(span, "Array index must be integer"));
        }
        return Ok(());
    }
    if let PhpType::Object(class_name) = &target.prop_ty {
        if checker.object_type_implements_interface(class_name, "ArrayAccess") {
            return Ok(());
        }
    }
    if idx_ty != PhpType::Int && idx_ty != PhpType::Mixed {
        return Err(CompileError::new(span, "Array index must be integer"));
    }

    let updated_prop_ty = match target.prop_ty {
        PhpType::Array(elem_ty) => {
            if target.property_has_declared_type {
                checker.require_compatible_arg_type(
                    elem_ty.as_ref(),
                    &val_ty,
                    span,
                    &format!("Static property {}::${}[]", target.class_name, property),
                )?;
                PhpType::Array(elem_ty)
            } else if *elem_ty == val_ty {
                PhpType::Array(elem_ty)
            } else {
                let merged_ty = checker
                    .merge_array_element_type(&elem_ty, &val_ty)
                    .unwrap_or(PhpType::Mixed);
                PhpType::Array(Box::new(merged_ty))
            }
        }
        PhpType::Int | PhpType::Void if !target.property_has_declared_type => {
            PhpType::Array(Box::new(val_ty.clone()))
        }
        PhpType::AssocArray { key, value } => {
            // A string/int/mixed-key write into an associative static array — including one
            // de-packed to a hash by a reference-alias (`self::$a[$dir] = &self::$a[$k]`). Keep it
            // associative and widen the value type toward the written value when untyped.
            let merged_value = if target.property_has_declared_type {
                checker.require_compatible_arg_type(
                    value.as_ref(),
                    &val_ty,
                    span,
                    &format!("Static property {}::${}[]", target.class_name, property),
                )?;
                *value
            } else if *value == val_ty {
                *value
            } else {
                checker
                    .merge_array_element_type(&value, &val_ty)
                    .unwrap_or(PhpType::Mixed)
            };
            PhpType::AssocArray {
                key,
                value: Box::new(merged_value),
            }
        }
        other => {
            return Err(CompileError::new(
                span,
                &format!(
                    "Array index assignment requires an array static property, got {}",
                    other
                ),
            ))
        }
    };

    if !target.property_has_declared_type {
        update_static_property_type(
            checker,
            property,
            &target.declaring_class,
            updated_prop_ty,
        );
    }
    Ok(())
}

/// Type-checks a write through a dynamic-named static property (`self::${$n} = v`,
/// `self::${$n}[$k] = v`, `self::${$n}[] = v`).
///
/// The property name is a runtime string, so no single candidate can be pinpointed: the receiver
/// class must be statically known (else a loud deferred error, as in the read path) and must
/// declare at least one static property. Operands are inferred in source order (name, index,
/// value) for their assignment effects. Value/element type checking is intentionally permissive
/// (gradual): since the runtime name selects among candidates, individual candidate types are not
/// narrowed here — codegen picks and coerces the matching one.
pub(super) fn check_dynamic_static_property_write(
    checker: &mut Checker,
    receiver: &StaticReceiver,
    property: &Expr,
    index: Option<&Expr>,
    value: &Expr,
    span: Span,
    env: &mut TypeEnv,
) -> Result<(), CompileError> {
    let name_ty = checker.infer_type_with_assignment_effects(property, env)?;
    if !matches!(name_ty, PhpType::Str | PhpType::Mixed | PhpType::Int) {
        return Err(CompileError::new(
            property.span,
            &format!(
                "Dynamic static property name must be a string, got `{}`",
                name_ty
            ),
        ));
    }
    if let Some(index) = index {
        checker.infer_type_with_assignment_effects(index, env)?;
    }
    checker.infer_type_with_assignment_effects(value, env)?;

    let class_name = resolve_dynamic_static_write_class(checker, receiver, span)?;
    let class_info = checker.classes.get(&class_name).ok_or_else(|| {
        CompileError::new(
            span,
            "Dynamic static property access requires a statically-known class",
        )
    })?;
    if class_info.static_properties.is_empty() {
        return Err(CompileError::new(
            span,
            &format!("Class {} has no static properties", class_name),
        ));
    }
    Ok(())
}

/// Resolves the receiver of a dynamic static property write to a statically-known class name.
///
/// `Named` returns the class directly; `Self_`/`Static` require a class context; `Parent` returns
/// the current class's parent. Errors when the class is not statically resolvable so the write
/// stays a loud deferred error rather than a silent miscompile.
fn resolve_dynamic_static_write_class(
    checker: &Checker,
    receiver: &StaticReceiver,
    span: Span,
) -> Result<String, CompileError> {
    match receiver {
        StaticReceiver::Named(class_name) => Ok(class_name.as_str().to_string()),
        StaticReceiver::Self_ | StaticReceiver::Static => checker
            .current_class
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                CompileError::new(
                    span,
                    "Dynamic static property access requires a statically-known class",
                )
            }),
        StaticReceiver::Parent => {
            let current_class = checker.current_class.as_ref().ok_or_else(|| {
                CompileError::new(
                    span,
                    "Dynamic static property access requires a statically-known class",
                )
            })?;
            checker
                .classes
                .get(current_class)
                .and_then(|info| info.parent.clone())
                .ok_or_else(|| {
                    CompileError::new(
                        span,
                        &format!("Class {} has no parent class", current_class),
                    )
                })
        }
    }
}

/// Resolves `receiver` to a class name and fetches static property metadata.
///
/// Returns `StaticPropertyAssignmentTarget` with class name, declaring class,
/// declared-type flag, and current property type. Checks that the property exists
/// and that the current context can access it per PHP visibility rules.
fn resolve_static_property_assignment_target(
    checker: &Checker,
    receiver: &StaticReceiver,
    property: &str,
    span: Span,
) -> Result<StaticPropertyAssignmentTarget, CompileError> {
    let class_name = match receiver {
        StaticReceiver::Named(class_name) => class_name.as_str().to_string(),
        StaticReceiver::Self_ => checker
            .current_class
            .as_ref()
            .cloned()
            .ok_or_else(|| CompileError::new(span, "Cannot use self:: outside class method scope"))?,
        StaticReceiver::Static => checker
            .current_class
            .as_ref()
            .cloned()
            .ok_or_else(|| CompileError::new(span, "Cannot use static:: outside class method scope"))?,
        StaticReceiver::Parent => {
            let current_class = checker.current_class.as_ref().ok_or_else(|| {
                CompileError::new(span, "Cannot use parent:: outside class method scope")
            })?;
            let current_info = checker.classes.get(current_class).ok_or_else(|| {
                CompileError::new(span, &format!("Undefined class: {}", current_class))
            })?;
            current_info.parent.as_ref().cloned().ok_or_else(|| {
                CompileError::new(
                    span,
                    &format!("Class {} has no parent class", current_class),
                )
            })?
        }
    };

    if checker.eval_barrier_active
        && matches!(receiver, StaticReceiver::Named(_))
        && !checker.classes.contains_key(&class_name)
    {
        return Ok(StaticPropertyAssignmentTarget {
            class_name: class_name.clone(),
            declaring_class: class_name,
            property_has_declared_type: false,
            prop_ty: PhpType::Mixed,
            dynamic_eval_target: true,
        });
    }

    let class_info = checker
        .classes
        .get(&class_name)
        .ok_or_else(|| CompileError::new(span, &format!("Undefined class: {}", class_name)))?;
    if !class_info
        .static_properties
        .iter()
        .any(|(name, _)| name == property)
    {
        return Err(CompileError::new(
            span,
            &format!("Undefined static property: {}::{}", class_name, property),
        ));
    }
    if let Some(visibility) = class_info.static_property_visibilities.get(property) {
        let declaring_class = class_info
            .static_property_declaring_classes
            .get(property)
            .map(String::as_str)
            .unwrap_or(class_name.as_str());
        if !checker.can_access_member(declaring_class, visibility) {
            return Err(CompileError::new(
                span,
                &format!(
                    "Cannot access {} static property: {}::{}",
                    Checker::visibility_label(visibility),
                    class_name,
                    property
                ),
            ));
        }
    }
    let declaring_class = class_info
        .static_property_declaring_classes
        .get(property)
        .cloned()
        .unwrap_or_else(|| class_name.clone());
    let property_has_declared_type = class_info.declared_static_properties.contains(property);
    let prop_ty = class_info
        .static_properties
        .iter()
        .find(|(name, _)| name == property)
        .map(|(_, ty)| ty.clone())
        .unwrap_or(PhpType::Int);

    Ok(StaticPropertyAssignmentTarget {
        class_name,
        declaring_class,
        property_has_declared_type,
        prop_ty,
        dynamic_eval_target: false,
    })
}

/// Returns true when a dynamic eval static-property key can be handled by runtime array writes.
fn is_dynamic_eval_static_property_array_key_type(ty: &PhpType) -> bool {
    matches!(ty, PhpType::Int | PhpType::Str | PhpType::Mixed)
}

/// Updates the type of a static property in all classes that declare it.
///
/// Iterates over all classes and mutates the type entry for `property` on the
/// declaring class `declaring_class`. Used after array-push or index assignment
/// to propagate the new element type.
fn update_static_property_type(
    checker: &mut Checker,
    property: &str,
    declaring_class: &str,
    updated_ty: PhpType,
) {
    for class_info in checker.classes.values_mut() {
        if class_info
            .static_property_declaring_classes
            .get(property)
            .map(String::as_str)
            != Some(declaring_class)
        {
            continue;
        }
        if let Some(prop) = class_info
            .static_properties
            .iter_mut()
            .find(|(name, _)| name == property)
        {
            prop.1 = updated_ty.clone();
        }
    }
}

/// Type-checks `self::$a[$dir] = &self::$a[$k]`: aliasing one element of a static-property array to
/// an element of a static-property array (SLICE 2/3).
///
/// The `receiver::property` target element becomes a reference alias of the source element's shared
/// cell. Both operands must be array-typed static properties whose element fits in a single
/// reference-cell word (an untyped `array`/`Mixed`-element array, or an already-associative array).
/// The target — and the source when it names a different property — is de-packed program-wide from
/// an indexed `array` to the promoted associative hash type via `update_static_property_type`, so
/// codegen loads/stores/frees it as a hash (the static analog of retyping a local Array→AssocArray).
///
/// Every out-of-scope shape is a loud error rather than a silent value copy: an unknown/`Mixed`
/// class or undefined/inaccessible property (from `resolve_static_property_assignment_target`), a
/// non-array static property, a string element, a typed indexed-list container, or a reference
/// source that is not itself a static-property array element.
pub(super) fn check_ref_assign_static_prop_element(
    checker: &mut Checker,
    receiver: &StaticReceiver,
    property: &str,
    source: &Expr,
    span: Span,
) -> Result<(), CompileError> {
    // Resolve + validate the TARGET static property. Unknown/`Mixed` class and undefined/
    // inaccessible property all surface here as loud errors.
    let target = resolve_static_property_assignment_target(checker, receiver, property, span)?;
    validate_reference_array_static_property(&target.prop_ty, span)?;

    // For this slice the reference SOURCE must itself be a static-property array element
    // (`&self::$SRC[$k]`); any other shape is a follow-up slice, not a silent value copy.
    let ExprKind::ArrayAccess {
        array: src_array, ..
    } = &source.kind
    else {
        return Err(reference_source_shape_error(span));
    };
    let ExprKind::StaticPropertyAccess {
        receiver: src_receiver,
        property: src_property,
    } = &src_array.kind
    else {
        return Err(reference_source_shape_error(span));
    };
    let src_target =
        resolve_static_property_assignment_target(checker, src_receiver, src_property, span)?;
    validate_reference_array_static_property(&src_target.prop_ty, span)?;

    // This slice covers only the same-array GATE (`self::$a[$d] = &self::$a[$k]`), where both
    // operands are the SAME static property. A cross-array source (`self::$a[$d] = &self::$b[$k]`)
    // needs a separate source load/store round-trip and is a follow-up slice: loud-error it rather
    // than miscompile.
    if (src_target.declaring_class.as_str(), src_property.as_str())
        != (target.declaring_class.as_str(), property)
    {
        return Err(CompileError::new(
            span,
            "Reference between two different static-property arrays is not yet supported (source must alias the same static property)",
        ));
    }

    // De-pack signal: flip the (single, shared) target/source static property from an indexed
    // `array` to the promoted associative hash so codegen routes reads/writes/cleanup as a hash
    // program-wide.
    let target_assoc = promoted_static_array_type(&target.prop_ty);
    update_static_property_type(checker, property, &target.declaring_class, target_assoc);
    Ok(())
}

/// Validates that a static property targeted by a reference-into-element alias is an array whose
/// element fits in a single reference-cell word.
///
/// Rejects (loud errors): a non-array static property, a string element (multi-word — a one-word
/// cell would drop its length), and a concretely-typed indexed scalar list whose packed
/// representation must stay list-shaped. An untyped `array`/`Mixed`-element array and an already
/// associative array are accepted.
fn validate_reference_array_static_property(
    prop_ty: &PhpType,
    span: Span,
) -> Result<(), CompileError> {
    let repr = prop_ty.codegen_repr();
    if !matches!(repr, PhpType::Array(_) | PhpType::AssocArray { .. }) {
        return Err(CompileError::new(
            span,
            "Reference assignment into a static-property array element requires an array static property",
        ));
    }
    let element = static_array_element_type(prop_ty);
    if element.codegen_repr() == PhpType::Str {
        return Err(CompileError::new(
            span,
            "Reference into a string-element static-property array is not yet supported",
        ));
    }
    if matches!(repr, PhpType::Array(_))
        && matches!(
            element.codegen_repr(),
            PhpType::Int | PhpType::Float | PhpType::Bool
        )
    {
        return Err(CompileError::new(
            span,
            "unsupported: reference into a typed indexed-list static property",
        ));
    }
    Ok(())
}

/// Returns the element type of an array/associative container (defaulting to `Mixed`).
fn static_array_element_type(container: &PhpType) -> PhpType {
    match container.codegen_repr() {
        PhpType::AssocArray { value, .. } => *value,
        PhpType::Array(element) => *element,
        _ => PhpType::Mixed,
    }
}

/// Builds the promoted associative-hash type an indexed/`array` static property is retyped to when
/// one of its elements is reference-aliased.
fn promoted_static_array_type(container: &PhpType) -> PhpType {
    PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(static_array_element_type(container)),
    }
}

/// Builds the loud error for a reference source that is not itself a static-property array element.
fn reference_source_shape_error(span: Span) -> CompileError {
    CompileError::new(
        span,
        "Reference source for a static-property array element must be another static-property array element (e.g. &self::$a[$k])",
    )
}

/// Refines the type of a static property based on an assigned value.
///
/// Delegates the type decision to `refined_untyped_property_assignment_type` (shared with
/// instance properties), which stores scalar/array writes to untyped null-defaulted slots
/// as nullable unions so the null default stays observable. Only updates when the refined
/// type differs from the current.
fn refine_static_property_assignment_type(
    checker: &mut Checker,
    property: &str,
    declaring_class: &str,
    val_ty: &PhpType,
) {
    let refined_ty = {
        let current = checker.classes.values().find_map(|class_info| {
            if class_info
                .static_property_declaring_classes
                .get(property)
                .map(String::as_str)
                != Some(declaring_class)
            {
                return None;
            }
            class_info
                .static_properties
                .iter()
                .find(|(name, _)| name == property)
                .map(|(_, ty)| ty.clone())
        });
        let Some(current) = current else {
            return;
        };
        let Some(refined_ty) = super::properties::refined_untyped_property_assignment_type(
            checker, &current, val_ty, false,
        ) else {
            return;
        };
        refined_ty
    };
    for class_info in checker.classes.values_mut() {
        if class_info
            .static_property_declaring_classes
            .get(property)
            .map(String::as_str)
            != Some(declaring_class)
        {
            continue;
        }
        if let Some(prop) = class_info
            .static_properties
            .iter_mut()
            .find(|(name, _)| name == property)
        {
            prop.1 = refined_ty.clone();
        }
    }
}
