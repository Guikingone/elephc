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

/// One writable location, with an observation only when a read was requested.
enum EvaluatedLocation {
    GlobalVariable {
        name: String,
        current: Option<RuntimeCellHandle>,
    },
    Variable {
        name: String,
        current: Option<RuntimeCellHandle>,
        current_is_borrowed: bool,
        ownership: ScopeCellOwnership,
    },
    Property {
        object: RuntimeCellHandle,
        property: String,
        current: Option<RuntimeCellHandle>,
    },
    StaticProperty {
        class_name: String,
        property: String,
        current: Option<RuntimeCellHandle>,
    },
    ArrayElement {
        parent: Box<EvaluatedLocation>,
        index: RuntimeCellHandle,
        current: Option<RuntimeCellHandle>,
    },
    ArrayAccessElement {
        object: RuntimeCellHandle,
        index: RuntimeCellHandle,
        current: Option<RuntimeCellHandle>,
    },
}

impl EvaluatedLocation {
    /// Returns the value observed while evaluating this writable location.
    const fn current(&self) -> RuntimeCellHandle {
        match self {
            Self::GlobalVariable { current, .. }
            | Self::Variable { current, .. }
            | Self::Property { current, .. }
            | Self::StaticProperty { current, .. }
            | Self::ArrayElement { current, .. }
            | Self::ArrayAccessElement { current, .. } => {
                current.expect("a read location must carry an observed value")
            }
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
    // `$t ??= $v` reads the target in PHP's QUIET fetch mode: an uninitialized typed property
    // is "absent" and gets assigned, it does not raise. Only the READ is quiet — the default
    // expression is evaluated normally, so a throw inside it still surfaces.
    context.push_quiet_property_fetch();
    let location = evaluate_location(target, context, scope, values);
    context.pop_quiet_property_fetch();
    let mut location = location?;
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
        if let EvaluatedLocation::GlobalVariable { current, .. } = &mut location {
            *current = None;
        }
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
pub(in crate::interpreter) fn eval_assign(
    target: &EvalExpr,
    value: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let trace = crate::eval_trace::enabled();
    let location = evaluate_plain_assignment_location(target, context, scope, values).map_err(|status| {
        if trace {
            eprintln!("[elephc-eval-trace] phase=assign_error stage=location target={target:?} status={status:?}");
        }
        status
    })?;
    let assigned = eval_expr(value, context, scope, values).map_err(|status| {
        if trace {
            eprintln!("[elephc-eval-trace] phase=assign_error stage=value value={value:?} status={status:?}");
        }
        status
    })?;
    let assigned = if eval_expr_result_aliases_storage(value) {
        values.copy_value(assigned)?
    } else {
        assigned
    };
    write_location(location, assigned, true, context, scope, values).map_err(|status| {
        if trace {
            eprintln!("[elephc-eval-trace] phase=assign_error stage=write target={target:?} status={status:?}");
        }
        status
    })?;
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
    let (source_target, source_value) = eval_reference_source(source, context, scope, values)
        .map_err(|status| {
            if crate::eval_trace::enabled() {
                eprintln!(
                    "[elephc-eval-trace] phase=reference_bind_error target={name:?} source={source:?} status={status:?}"
                );
            }
            status
        })?;
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
        current: None,
    };
    write_reference_location(location, source_target, source_value, context, scope, values)
}

/// Executes `return EXPR;` inside a function declared to return BY REFERENCE.
///
/// The caller may bind to what comes back, so the reference has to survive the activation being
/// drained. Whatever outlives the call owns its reference: the value cell is RETAINED into the
/// context, and the target is rewritten to one that outlives the frame -- a static local names
/// the context store, a by-reference parameter's element names the CALLER's target, and a plain
/// local, whose storage really does die with the call, keeps only the cell.
///
/// A non-reference return is not an error: `php -n` 8.5.6 emits
/// `Notice: Only variable references should be returned by reference` and binds a copy.
pub(in crate::interpreter) fn eval_by_ref_return(
    expr: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalControl, EvalStatus> {
    if !eval_expr_binds_a_reference_source(expr) {
        // Same diagnostic channel every other notice-level message here uses.
        values.warning("Only variable references should be returned by reference")?;
        let value = eval_expr(expr, context, scope, values)?;
        return Ok(EvalControl::Return(value));
    }
    let (target, value) = eval_reference_source(expr, context, scope, values)?;
    let target = eval_persistent_return_target(target, context, scope);
    let retained = values.retain(value)?;
    if let Some(unclaimed) = context.set_pending_return_reference(target, retained) {
        eval_release_value(context, values, unclaimed)?;
    }
    Ok(EvalControl::Return(value))
}

/// Returns whether one expression is a CALL, whose reference comes from the callee's `return`.
fn eval_expr_is_call_shaped(expr: &EvalExpr) -> bool {
    matches!(
        expr,
        EvalExpr::Call { .. }
            | EvalExpr::NamespacedCall { .. }
            | EvalExpr::DynamicCall { .. }
            | EvalExpr::MethodCall { .. }
            | EvalExpr::NullsafeMethodCall { .. }
            | EvalExpr::DynamicMethodCall { .. }
            | EvalExpr::NullsafeDynamicMethodCall { .. }
            | EvalExpr::StaticMethodCall { .. }
            | EvalExpr::DynamicStaticMethodCall { .. }
    )
}

/// Returns whether one returned expression names storage PHP can hand back by reference.
fn eval_expr_binds_a_reference_source(expr: &EvalExpr) -> bool {
    matches!(
        expr,
        EvalExpr::LoadVar(_)
            | EvalExpr::ArrayGet { .. }
            | EvalExpr::PropertyGet { .. }
            | EvalExpr::DynamicPropertyGet { .. }
            | EvalExpr::StaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyNameGet { .. }
    )
}

/// Rewrites a returned reference target into one that outlives the activation scope.
fn eval_persistent_return_target(
    target: EvalReferenceTarget,
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
) -> EvalReferenceTarget {
    let function = context.current_function().unwrap_or_default().to_string();
    match target {
        EvalReferenceTarget::Variable { scope: _, name } => {
            eval_persistent_name_target(&function, &name, context, scope)
        }
        EvalReferenceTarget::ArrayElement {
            scope: _,
            array_name,
            index,
        } => {
            let array_target =
                eval_persistent_name_target(&function, &array_name, context, scope);
            EvalReferenceTarget::NestedArrayElement {
                array_target: Box::new(array_target),
                index,
            }
        }
        target => target,
    }
}

/// Returns the persistent target one scope NAME denotes, or a bare cell when nothing outlives it.
fn eval_persistent_name_target(
    function: &str,
    name: &str,
    context: &ElephcEvalContext,
    scope: &ElephcEvalScope,
) -> EvalReferenceTarget {
    if let Some(target) = scope.reference_target(name) {
        // A by-reference PARAMETER already points at the caller's storage, which outlives this
        // call by construction.
        return target.clone();
    }
    if context.static_local(function, name).is_some() {
        return EvalReferenceTarget::StaticLocal {
            function: function.to_string(),
            name: name.to_string(),
        };
    }
    // A plain local's storage really does die with the call. PHP keeps the VALUE alive because
    // the reference is refcounted, and that is what the retained cell does here; there is no
    // storage left for a write to reach, which is what php shows too.
    EvalReferenceTarget::Cell {
        cell: scope
            .visible_cell(name)
            .unwrap_or_else(|| RuntimeCellHandle::from_raw(std::ptr::null_mut())),
    }
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
    let value = eval_expr_value_for_consuming_store(value, context, scope, values)?;
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
        EvalExpr::ArrayGet { array, index } if is_globals_array(array) => {
            Ok(EvaluatedLocation::GlobalVariable {
                name: eval_global_name(index, context, scope, values)?,
                current: None,
            })
        }
        EvalExpr::LoadVar(name) => {
            let entry = scope_entry(context, scope, name).filter(|entry| entry.flags().is_visible());
            Ok(EvaluatedLocation::Variable {
                name: name.clone(),
                current: None,
                current_is_borrowed: false,
                ownership: entry.map_or(ScopeCellOwnership::Owned, |entry| entry.flags().ownership),
            })
        }
        EvalExpr::PropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property: property.clone(),
                current: None,
            })
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property,
                current: None,
            })
        }
        EvalExpr::StaticPropertyGet {
            class_name,
            property,
        } => Ok(EvaluatedLocation::StaticProperty {
            class_name: class_name.clone(),
            property: property.clone(),
            current: None,
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
                current: None,
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
                current: None,
            })
        }
        EvalExpr::ArrayGet { .. } => {
            evaluate_location(target, context, scope, values)
        }
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// How the components of an lvalue chain are fetched.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LocationFetch {
    /// The location's own value is wanted, so an uninitialized typed property raises.
    Read,
    /// The location is the CONTAINER of an array write, so an uninitialized typed property
    /// auto-initializes to an array instead of raising.
    ///
    /// PHP fetches every link of an lvalue chain in write mode: `$o->p['k'] = 1` initializes
    /// `$o->p` rather than complaining that it was never set, and the same holds nested
    /// (`$o->p['k']['j'] = 1`). Only the link the assignment finally lands on is a plain write;
    /// everything to its left is a container fetch.
    ArrayParent,
}

/// Evaluates the receiver and index components of one supported writable expression once.
fn evaluate_location(
    target: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvaluatedLocation, EvalStatus> {
    evaluate_location_with_fetch(target, LocationFetch::Read, context, scope, values)
}

/// Evaluates one writable expression's components under an explicit fetch mode.
fn evaluate_location_with_fetch(
    target: &EvalExpr,
    fetch: LocationFetch,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvaluatedLocation, EvalStatus> {
    match target {
        EvalExpr::ArrayGet { array, index } if is_globals_array(array) => {
            let name = eval_global_name(index, context, scope, values)?;
            let quiet = fetch == LocationFetch::ArrayParent || context.quiet_property_fetch();
            let current = read_global_value(&name, quiet, context, scope, values)?;
            Ok(EvaluatedLocation::GlobalVariable { name, current: Some(current) })
        }
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
                current: Some(current),
                current_is_borrowed,
                ownership,
            })
        }
        EvalExpr::PropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let current = evaluate_property_component(object, property, fetch, context, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property: property.clone(),
                current: Some(current),
            })
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            let current = evaluate_property_component(object, &property, fetch, context, values)?;
            Ok(EvaluatedLocation::Property {
                object,
                property,
                current: Some(current),
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
                current: Some(current),
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
                current: Some(current),
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
                current: Some(current),
            })
        }
        EvalExpr::ArrayGet { array, index } => {
            // Everything to the left of an index is a CONTAINER, so it auto-initializes.
            let parent =
                evaluate_location_with_fetch(array, LocationFetch::ArrayParent, context, scope, values)?;
            let container = parent.current();
            let index = eval_expr(index, context, scope, values)?;
            let current = eval_array_get_result(container, index, context, values)?;
            if values.type_tag(container)? == EVAL_TAG_OBJECT
                && eval_array_access_object_matches(container, context, values)?
            {
                return Ok(EvaluatedLocation::ArrayAccessElement {
                    object: container,
                    index,
                    current: Some(current),
                });
            }
            Ok(EvaluatedLocation::ArrayElement {
                parent: Box::new(parent),
                index,
                current: Some(current),
            })
        }
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Reads one instance-property component of an lvalue chain under the given fetch mode.
fn evaluate_property_component(
    object: RuntimeCellHandle,
    property: &str,
    fetch: LocationFetch,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match fetch {
        LocationFetch::Read => eval_property_get_result(object, property, context, values),
        LocationFetch::ArrayParent => {
            eval_property_array_target_get_result(object, property, context, values)
        }
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
        EvaluatedLocation::GlobalVariable { name, current } => {
            // A container update may transfer the observed copy itself to the slot.
            if let Some(current) = current.filter(|current| *current != value) {
                eval_release_value(context, values, current)?;
            }
            let stored = if preserve_value_for_result { values.retain(value)? } else { value };
            store_global_value(name, stored, context, scope, values)
        }
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
    if eval_expr_is_call_shaped(source) {
        // A by-reference RETURN leaves its reference on the context, already retained. Taking it
        // transfers that ownership to this bind; a callee that was NOT declared by reference
        // leaves nothing, and php binds a copy there rather than failing.
        let value = eval_expr(source, context, scope, values)?;
        if let Some((target, referenced)) = context.take_pending_return_reference() {
            eval_release_value(context, values, referenced)?;
            return Ok((target, value));
        }
        return Ok((EvalReferenceTarget::Cell { cell: value }, value));
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
    // `&$o->p` on an uninitialized typed property has PHP's own rule, and it is neither the
    // ordinary read's nor the quiet fetch's: non-nullable raises a DIFFERENT sentence, nullable
    // initializes to null and binds. The check runs before the generic by-reference
    // materializer, which would otherwise perform an ordinary read and raise the wrong message.
    match source {
        EvalExpr::PropertyGet { object, property } => {
            let receiver = eval_expr(object, context, scope, values)?;
            eval_property_reference_bind_precheck(receiver, property, context, values)?;
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let receiver = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            eval_property_reference_bind_precheck(receiver, &property, context, values)?;
        }
        _ => {}
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
    if crate::eval_trace::enabled() {
        eprintln!(
            "[elephc-eval-trace] phase=array_reference_bind identity={updated_identity:#x} key={key:?}",
        );
    }
    context.bind_array_element_alias(updated_identity, key, source_target);
    write_location(*parent, updated, false, context, scope, values)
}
