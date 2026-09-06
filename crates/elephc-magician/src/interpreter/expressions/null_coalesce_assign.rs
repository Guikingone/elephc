//! Purpose:
//! Evaluates PHP null-coalescing assignment expressions against writable EvalIR locations.
//!
//! Called from:
//! - `super::eval_expr()` for `EvalExpr::NullCoalesceAssign`.
//!
//! Key details:
//! - Target expressions and array indexes are evaluated once in source order.
//! - The default expression stays lazy and writes back through nested array/property locations.

use super::*;

/// One writable location whose current value has already been evaluated.
enum EvaluatedLocation {
    Variable {
        name: String,
        current: RuntimeCellHandle,
        current_is_borrowed: bool,
        ownership: ScopeCellOwnership,
    },
    Property {
        object: RuntimeCellHandle,
        property: String,
        current: RuntimeCellHandle,
    },
    StaticProperty {
        class_name: String,
        property: String,
        current: RuntimeCellHandle,
    },
    ArrayElement {
        parent: Box<EvaluatedLocation>,
        index: RuntimeCellHandle,
        current: RuntimeCellHandle,
    },
    ArrayAccessElement {
        object: RuntimeCellHandle,
        index: RuntimeCellHandle,
        current: RuntimeCellHandle,
    },
}

impl EvaluatedLocation {
    /// Returns the value observed while evaluating this writable location.
    const fn current(&self) -> RuntimeCellHandle {
        match self {
            Self::Variable { current, .. }
            | Self::Property { current, .. }
            | Self::StaticProperty { current, .. }
            | Self::ArrayElement { current, .. }
            | Self::ArrayAccessElement { current, .. } => *current,
        }
    }

    /// Returns whether the observed value is owned by an existing scope entry.
    const fn current_is_borrowed(&self) -> bool {
        matches!(
            self,
            Self::Variable {
                current_is_borrowed: true,
                ..
            }
        )
    }
}

/// Evaluates `target ??= default`, preserving PHP's lazy RHS and single target evaluation.
pub(super) fn eval_null_coalesce_assign(
    target: &EvalExpr,
    default: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let location = evaluate_location(target, context, scope, values)?;
    let current = location.current();
    if !values.is_null(current)? {
        return if location.current_is_borrowed() {
            values.retain(current)
        } else {
            Ok(current)
        };
    }
    if !location.current_is_borrowed() {
        eval_release_value(context, values, current)?;
    }
    let assigned = eval_expr(default, context, scope, values)?;
    write_location(location, assigned, true, context, scope, values)?;
    Ok(assigned)
}

/// Evaluates a compound assignment expression and writes its computed result back to the target.
pub(super) fn eval_compound_assign(
    target: &EvalExpr,
    op: EvalBinOp,
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let location = evaluate_location(target, context, scope, values)?;
    let current = location.current();
    let value = eval_expr(value, context, scope, values)?;
    let assigned = eval_binary_result(op, current, value, context, values)?;
    write_location(location, assigned, true, context, scope, values)?;
    Ok(assigned)
}

/// Evaluates a postfix increment or decrement and returns the value observed before mutation.
pub(super) fn eval_postfix_inc_dec(
    target: &EvalExpr,
    increment: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let location = evaluate_location(target, context, scope, values)?;
    let current = location.current();
    let result = values.retain(current)?;
    let updated = eval_inc_dec_value(current, increment, values)?;
    write_location(location, updated, false, context, scope, values)?;
    Ok(result)
}

/// Evaluates a plain assignment expression and returns the stored value.
pub(super) fn eval_assign(
    target: &EvalExpr,
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let location = evaluate_plain_assignment_location(target, context, scope, values)?;
    let assigned = eval_expr(value, context, scope, values)?;
    let assigned = if matches!(value, EvalExpr::LoadVar(_)) {
        values.copy_value(assigned)?
    } else {
        assigned
    };
    write_location(location, assigned, true, context, scope, values)?;
    Ok(assigned)
}

/// Binds a variable to a non-variable lvalue's persistent PHP reference target.
///
/// `$x = &$y;` aliases two scope names and stays on `set_reference_alias()`; every other PHP
/// reference source — a property, a static property, an array element, their dynamic-name forms
/// — names storage outside the scope, so the variable is rebound as a borrowed view of the
/// source's current cell plus the reference target that later writes travel back through.
pub(in crate::interpreter) fn eval_var_reference_bind(
    name: &str,
    source: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let (source_target, source_value) = eval_reference_source(source, context, scope, values)?;
    let replaced = scope.rebind_reference(
        name.to_string(),
        source_value,
        ScopeCellOwnership::Borrowed,
    );
    scope.set_reference_target(name.to_string(), source_target);
    if let Some(replaced) = replaced {
        eval_release_value(context, values, replaced)?;
    }
    Ok(())
}

/// Binds an array element lvalue to a persistent PHP reference source expression.
pub(in crate::interpreter) fn eval_array_reference_bind(
    target: &EvalExpr,
    source: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let (source_target, source_value) = eval_reference_source(source, context, scope, values)?;
    let location = evaluate_plain_assignment_location(target, context, scope, values)?;
    write_reference_location(location, source_target, source_value, context, scope, values)
}

/// Binds a NEWLY APPENDED element of one writable array lvalue to a persistent reference source.
///
/// The target is a general lvalue, not a bare name: PHP appends through a nested element as
/// readily as through a variable, and `$loops[$k][] = &$path;` is that shape.
pub(in crate::interpreter) fn eval_array_append_reference_bind(
    target: &EvalExpr,
    source: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let (source_target, source_value) = eval_reference_source(source, context, scope, values)?;
    let parent = evaluate_location(target, context, scope, values)?;
    let array = if values.is_null(parent.current())? {
        values.array_new(1)?
    } else {
        parent.current()
    };
    let index = eval_array_append_key(array, values)?;
    let location = EvaluatedLocation::ArrayElement {
        parent: Box::new(parent),
        index,
        current: values.null()?,
    };
    write_reference_location(location, source_target, source_value, context, scope, values)
}

/// Binds one reference and returns the bound VALUE, for `TARGET = &SOURCE` in expression position.
///
/// PHP's value here is the bound value as a COPY rather than a second alias: after
/// `$a = ($b = &$one); $one = 9;` php reports `$b` as 9 and `$a` as the 1 it copied. Returning
/// the source's current cell gives exactly that, because the outer assignment copies it.
pub(in crate::interpreter) fn eval_reference_bind_result(
    target: &EvalExpr,
    source: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    // Two plain NAMES alias SYMMETRICALLY -- writing either updates both -- and only the scope's
    // named-alias table models that; a one-way reference target does not. This is the same split
    // the statement world makes between `ReferenceAssign` and `VarReferenceBind`.
    if let (EvalExpr::LoadVar(name), EvalExpr::LoadVar(source_name)) = (target, source) {
        for replaced in set_reference_alias(context, scope, name, source_name, values)? {
            eval_release_value(context, values, replaced)?;
        }
        return visible_scope_cell(context, scope, name).map_or_else(|| values.null(), Ok);
    }
    let (source_target, source_value) = eval_reference_source(source, context, scope, values)?;
    // A NAME aliases through the scope's own alias table, which is what makes two names update
    // each other symmetrically; `write_reference_location` knows only element targets and would
    // refuse this one. The two are separate mechanisms in the statement world too
    // (`ReferenceAssign` and `VarReferenceBind` against `ArrayReferenceBind`).
    if let EvalExpr::LoadVar(name) = target {
        let replaced = scope.rebind_reference(
            name.to_string(),
            source_value,
            ScopeCellOwnership::Borrowed,
        );
        scope.set_reference_target(name.to_string(), source_target);
        if let Some(replaced) = replaced {
            eval_release_value(context, values, replaced)?;
        }
        return Ok(source_value);
    }
    let location = evaluate_plain_assignment_location(target, context, scope, values)?;
    write_reference_location(location, source_target, source_value, context, scope, values)?;
    Ok(source_value)
}

/// Appends a value through any writable array lvalue while preserving PHP evaluation order.
pub(in crate::interpreter) fn eval_array_append(
    target: &EvalExpr,
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    // The appended cell is the one the array now holds, exactly as `EvalExpr::Assign` returns
    // the cell it stored, so a statement append has nothing of its own to release.
    eval_array_append_result(target, value, context, scope, values).map(|_| ())
}

/// Appends a value and returns the ASSIGNED value, which is what `$a[] = $v` evaluates to.
///
/// PHP's append is an assignment expression: `return $this->rules[] = $r;` returns `$r`, and
/// `$a[] = $b[] = 'x'` appends the same value to both. The returned handle ALIASES the array's
/// element rather than carrying a reference of its own, which is the rule
/// `eval_expr_result_aliases_storage` records for every assignment shape.
pub(in crate::interpreter) fn eval_array_append_result(
    target: &EvalExpr,
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let location = evaluate_location(target, context, scope, values)?;
    let current = location.current();
    if values.type_tag(current)? == EVAL_TAG_OBJECT {
        if !eval_array_access_object_matches(current, context, values)? {
            return Err(EvalStatus::RuntimeFatal);
        }
        let offset = values.null()?;
        let value = eval_expr(value, context, scope, values)?;
        let result = eval_method_call_result(current, "offsetSet", vec![offset, value], context, values)?;
        // `offsetSet()` returns void in PHP; the expression's value is the ASSIGNED one.
        eval_release_value(context, values, result)?;
        return Ok(value);
    }
    let array = if values.is_null(current)? {
        values.array_new(1)?
    } else {
        current
    };
    let index = eval_array_append_key(array, values)?;
    let value = eval_expr(value, context, scope, values)?;
    let updated = values.array_set(array, index, value)?;
    write_location(location, updated, false, context, scope, values)?;
    Ok(value)
}

/// Evaluates a plain-assignment target without reading an existing property value first.
fn evaluate_plain_assignment_location(
    target: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvaluatedLocation, EvalStatus> {
    match target {
        EvalExpr::PropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property: property.clone(),
                current: object,
            })
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property,
                current: object,
            })
        }
        EvalExpr::StaticPropertyGet {
            class_name,
            property,
        } => Ok(EvaluatedLocation::StaticProperty {
            class_name: class_name.clone(),
            property: property.clone(),
            current: values.null()?,
        }),
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            Ok(EvaluatedLocation::StaticProperty {
                class_name,
                property: property.clone(),
                current: values.null()?,
            })
        }
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            Ok(EvaluatedLocation::StaticProperty {
                class_name,
                property,
                current: values.null()?,
            })
        }
        EvalExpr::LoadVar(_) | EvalExpr::ArrayGet { .. } => {
            evaluate_location(target, context, scope, values)
        }
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Evaluates the receiver and index components of one supported writable expression once.
fn evaluate_location(
    target: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvaluatedLocation, EvalStatus> {
    match target {
        EvalExpr::LoadVar(name) => {
            let entry = scope_entry(context, scope, name).filter(|entry| entry.flags().is_visible());
            let current_is_borrowed = entry.is_some();
            let ownership = entry
                .as_ref()
                .map(|entry| entry.flags().ownership)
                .unwrap_or(ScopeCellOwnership::Owned);
            let current = entry.map_or_else(|| values.null(), |entry| Ok(entry.cell()))?;
            Ok(EvaluatedLocation::Variable {
                name: name.clone(),
                current,
                current_is_borrowed,
                ownership,
            })
        }
        EvalExpr::PropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let current = eval_property_get_result(object, property, context, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property: property.clone(),
                current,
            })
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            let current = eval_property_get_result(object, &property, context, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property,
                current,
            })
        }
        EvalExpr::StaticPropertyGet {
            class_name,
            property,
        } => {
            let current = eval_static_property_get_result(class_name, property, context, values)?;
            Ok(EvaluatedLocation::StaticProperty {
                class_name: class_name.clone(),
                property: property.clone(),
                current,
            })
        }
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let current =
                eval_static_property_get_result(&class_name, property, context, values)?;
            Ok(EvaluatedLocation::StaticProperty {
                class_name,
                property: property.clone(),
                current,
            })
        }
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            let current =
                eval_static_property_get_result(&class_name, &property, context, values)?;
            Ok(EvaluatedLocation::StaticProperty {
                class_name,
                property,
                current,
            })
        }
        EvalExpr::ArrayGet { array, index } => {
            let parent = evaluate_location(array, context, scope, values)?;
            let container = parent.current();
            let index = eval_expr(index, context, scope, values)?;
            let current = eval_array_get_result(container, index, context, values)?;
            if values.type_tag(container)? == EVAL_TAG_OBJECT
                && eval_array_access_object_matches(container, context, values)?
            {
                return Ok(EvaluatedLocation::ArrayAccessElement {
                    object: container,
                    index,
                    current,
                });
            }
            Ok(EvaluatedLocation::ArrayElement {
                parent: Box::new(parent),
                index,
                current,
            })
        }
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Writes an evaluated value back through a previously materialized location.
/// Stores one ALREADY EVALUATED value into any writable PHP lvalue.
///
/// Ownership follows `eval_assign`'s convention, which is what every other writer here uses: the
/// caller does not release afterwards. The variable and element arms consume the reference, the
/// property arms retain, and mixing the two by releasing here would over-release the first two.
pub(in crate::interpreter) fn eval_store_value_in_lvalue(
    target: &EvalExpr,
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let location = evaluate_plain_assignment_location(target, context, scope, values)?;
    write_location(location, value, false, context, scope, values)
}

fn write_location(
    location: EvaluatedLocation,
    value: RuntimeCellHandle,
    preserve_value_for_result: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    match location {
        EvaluatedLocation::Variable {
            name, ownership, ..
        } => {
            if let Some(target) = scope.reference_target(&name).cloned() {
                write_back_method_ref_target(&target, value, context, values)?;
            }
            let stored = if preserve_value_for_result {
                values.retain(value)?
            } else {
                value
            };
            for replaced in set_scope_cell(context, scope, name, stored, ownership)? {
                eval_release_value(context, values, replaced)?;
            }
            Ok(())
        }
        EvaluatedLocation::Property {
            object, property, ..
        } => eval_property_set_result(object, &property, value, context, values),
        EvaluatedLocation::StaticProperty {
            class_name,
            property,
            ..
        } => eval_static_property_set_result(&class_name, &property, value, context, values),
        EvaluatedLocation::ArrayElement { parent, index, .. } => {
            let container = if values.is_null(parent.current())? {
                values.array_new(1)?
            } else {
                parent.current()
            };
            let container = eval_array_set_target_for_index(container, index, values)?;
            let container_identity = values.raw_value_word(container)?;
            if let Some(target) = eval_array_reference_key(index, values)?
                .and_then(|key| context.array_element_alias(container_identity, &key).cloned())
            {
                return write_back_method_ref_target(&target, value, context, values);
            }
            let updated = values.array_set(container, index, value)?;
            write_location(*parent, updated, false, context, scope, values)
        }
        EvaluatedLocation::ArrayAccessElement { object, index, .. } => {
            let result =
                eval_method_call_result(object, "offsetSet", vec![index, value], context, values)?;
            eval_release_value(context, values, result)
        }
    }
}

/// Resolves an lvalue expression's current value and persistent PHP reference target.
fn eval_reference_source(
    source: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(EvalReferenceTarget, RuntimeCellHandle), EvalStatus> {
    if let EvalExpr::ArrayAppendSlot { target } = source {
        // PHP CREATES the element and binds to it: after `$c = &$a[];` the array has one more
        // entry, holding null, and writing `$c` writes that entry. Appending the null first and
        // then resolving the source as `target[index]` reuses every rule the ordinary element
        // path already applies, instead of inventing a second way to name an element.
        let index = eval_array_append_slot_index(target, context, scope, values)?;
        let element = EvalExpr::ArrayGet {
            array: target.clone(),
            index: Box::new(EvalExpr::Const(EvalConst::Int(index))),
        };
        return eval_reference_source(&element, context, scope, values);
    }
    if let EvalExpr::LoadVar(source) = source {
        if let Some(target) = scope.reference_target(source).cloned() {
            let value = visible_scope_cell(context, scope, source).map_or_else(
                || values.null(),
                Ok,
            )?;
            return Ok((target, value));
        }
    }
    let (value, target) = eval_call_arg_value(source, context, scope, values)?;
    target
        .map(|target| (target, value))
        .ok_or(EvalStatus::RuntimeFatal)
}

/// Appends a null element to one writable array lvalue and returns the integer key it took.
///
/// This is the element `$target[]` names as a reference SOURCE. PHP creates it before the bind,
/// which is observable: `count()` grows by one and the new entry reads back as null even if the
/// bound name is never written.
fn eval_array_append_slot_index(
    target: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<i64, EvalStatus> {
    let location = evaluate_location(target, context, scope, values)?;
    let current = location.current();
    let array = if values.is_null(current)? {
        values.array_new(1)?
    } else {
        current
    };
    let index = eval_array_append_key(array, values)?;
    let key = eval_int_value(index, values)?;
    let empty = values.null()?;
    let updated = values.array_set(array, index, empty)?;
    write_location(location, updated, false, context, scope, values)?;
    Ok(key)
}

/// Writes one by-reference assignment through an already evaluated array-element location.
fn write_reference_location(
    location: EvaluatedLocation,
    source_target: EvalReferenceTarget,
    source_value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let EvaluatedLocation::ArrayElement { parent, index, .. } = location else {
        return Err(EvalStatus::UnsupportedConstruct);
    };
    let container = if values.is_null(parent.current())? {
        values.array_new(1)?
    } else {
        parent.current()
    };
    let container = eval_array_set_target_for_index(container, index, values)?;
    let updated = values.array_set(container, index, source_value)?;
    let key = eval_array_reference_key(index, values)?.ok_or(EvalStatus::RuntimeFatal)?;
    let updated_identity = values.raw_value_word(updated)?;
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        eprintln!(
            "[elephc-eval-trace] phase=array_reference_bind identity={updated_identity:#x} key={key:?}",
        );
    }
    context.bind_array_element_alias(updated_identity, key, source_target);
    write_location(*parent, updated, false, context, scope, values)
}
