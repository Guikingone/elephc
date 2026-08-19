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
    write_location(location, assigned, true, context, scope, values)?;
    Ok(assigned)
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
