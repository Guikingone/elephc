//! Purpose:
//! Binds native callable arguments and prepares by-reference writeback metadata.
//!
//! Called from:
//! - Native instance, static, constructor, and call_user_func dispatch.
//!
//! Key details:
//! - Named, variadic, typed, and degraded by-value reference modes share one binder.

use super::*;

/// Binds native AOT callable args using the selected by-reference degradation mode.
pub(super) fn bind_native_callable_bound_args_with_mode(
    signature: Option<NativeCallableSignature>,
    args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<BoundMethodArg>, EvalStatus> {
    let Some(signature) = signature else {
        return positional_evaluated_bound_args(None, args, by_ref_mode, context, values);
    };
    if !signature.bridge_supported() {
        return Err(EvalStatus::RuntimeFatal);
    }
    if signature.param_names().len() == signature.param_count() {
        bind_native_signature_args(&signature, args, by_ref_mode, context, values)
    } else {
        positional_evaluated_bound_args(Some(&signature), args, by_ref_mode, context, values)
    }
}

/// Binds positional-only native AOT args and validates registered by-reference slots.
pub(super) fn positional_evaluated_bound_args(
    signature: Option<&NativeCallableSignature>,
    args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<BoundMethodArg>, EvalStatus> {
    if args.iter().any(|arg| arg.name.is_some()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let mut bound_args = args
        .into_iter()
        .enumerate()
        .map(|(index, arg)| {
            let owned = arg.owned;
            let ref_target = match signature {
                Some(signature) => native_parameter_ref_target(
                    signature,
                    Some(index),
                    arg.ref_target,
                    by_ref_mode,
                    values,
                )?,
                None => None,
            };
            Ok(BoundMethodArg {
                value: arg.value,
                ref_target,
                variadic_ref_targets: Vec::new(),
                owned,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(signature) = signature {
        apply_native_callable_bound_arg_types(signature, &mut bound_args, context, values)?;
        copy_native_call_user_func_by_value_ref_args(
            signature,
            &mut bound_args,
            by_ref_mode,
            values,
        )?;
    }
    Ok(bound_args)
}

/// Returns only runtime cell values from bound native AOT call arguments.
pub(in crate::interpreter) fn native_bound_arg_values(
    args: &[BoundMethodArg],
) -> Vec<RuntimeCellHandle> {
    args.iter().map(|arg| arg.value).collect()
}

/// Releases the bound-argument cells the caller still owed once the callee has been handed them.
///
/// A spread element comes out of `array_get` owned (`mixed_array_get.rs`: every successful return
/// is an owned `Mixed*`) while the generated bridge only borrows what it is handed, so the debt
/// outlives the call and is paid here. A callee that hands one of those cells back as its result
/// transfers the reference to the caller instead, so a returned handle is never released.
pub(in crate::interpreter) fn release_owned_bound_args(
    bound_args: &[BoundMethodArg],
    returned: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for bound_arg in bound_args {
        if !bound_arg.owned || Some(bound_arg.value) == returned {
            continue;
        }
        eval_release_value(context, values, bound_arg.value)?;
    }
    Ok(())
}

/// Returns whether native argument write-back has any caller storage to update.
///
/// Ordinary by-value calls carry no reference targets, so skipping their no-op write-back avoids
/// crossing the runtime-value boundary after a native result has been produced.
pub(in crate::interpreter) fn native_bound_args_require_writeback(
    args: &[BoundMethodArg],
) -> bool {
    args.iter()
        .any(|arg| arg.ref_target.is_some() || !arg.variadic_ref_targets.is_empty())
}

/// Writes native AOT by-reference argument cells back to their eval caller targets.
pub(in crate::interpreter) fn write_back_native_callable_ref_args(
    bound_args: &[BoundMethodArg],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for bound_arg in bound_args {
        if let Some(target) = bound_arg.ref_target.as_ref() {
            write_back_method_ref_target(target, bound_arg.value, context, values)?;
        }
        for (key, target) in &bound_arg.variadic_ref_targets {
            let value = values.array_get(bound_arg.value, *key)?;
            write_back_method_ref_target(target, value, context, values)?;
        }
    }
    Ok(())
}

/// Binds native AOT callable args and fills omitted defaults from metadata.
pub(super) fn bind_native_signature_args(
    signature: &NativeCallableSignature,
    args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<BoundMethodArg>, EvalStatus> {
    let mut bound_args = vec![None; signature.param_count()];
    let variadic_index = native_callable_variadic_index(signature);
    let mut next_positional = 0;
    let mut next_variadic_index = 0_i64;

    if let Some(index) = variadic_index {
        let array = values.array_new(args.len())?;
        bound_args[index] = Some(BoundMethodArg {
            value: array,
            ref_target: None,
            variadic_ref_targets: Vec::new(),
            owned: false,
        });
    }

    for arg in args {
        if let Some(name) = arg.name {
            bind_native_named_signature_arg(
                signature,
                variadic_index,
                &mut bound_args,
                &name,
                arg.value,
                arg.ref_target,
                arg.owned,
                by_ref_mode,
                values,
            )?;
        } else if let Some((overflow, owned)) = bind_native_positional_signature_arg(
            signature,
            &mut bound_args,
            variadic_index,
            &mut next_positional,
            &mut next_variadic_index,
            arg.value,
            arg.ref_target,
            arg.owned,
            by_ref_mode,
            values,
        )? {
            // The callable has no slot left for this one. It never joins `bound_args`, so the
            // release that runs after the call will not see it and this is its only owner.
            if owned {
                eval_release_value(context, values, overflow)?;
            }
        }
    }

    for (position, value) in bound_args.iter_mut().enumerate() {
        if Some(position) == variadic_index {
            continue;
        }
        if value.is_some() {
            continue;
        }
        if position < signature.required_param_count() {
            return Err(EvalStatus::RuntimeFatal);
        }
        let Some(default) = signature.param_default(position) else {
            return Err(EvalStatus::RuntimeFatal);
        };
        *value = Some(BoundMethodArg {
            value: materialize_native_callable_default(default, context, values)?,
            ref_target: None,
            variadic_ref_targets: Vec::new(),
            owned: true,
        });
    }

    let mut bound_args = bound_args
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or(EvalStatus::RuntimeFatal)?;
    apply_native_callable_bound_arg_types(signature, &mut bound_args, context, values)?;
    copy_native_call_user_func_by_value_ref_args(
        signature,
        &mut bound_args,
        by_ref_mode,
        values,
    )?;
    Ok(bound_args)
}

/// Applies registered native AOT parameter types after argument binding and default filling.
pub(super) fn apply_native_callable_bound_arg_types(
    signature: &NativeCallableSignature,
    bound_args: &mut [BoundMethodArg],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for (position, bound_arg) in bound_args.iter_mut().enumerate() {
        let Some(param_type) = signature.param_type(position) else {
            continue;
        };
        if signature.param_variadic(position) {
            apply_native_callable_variadic_arg_type(param_type, bound_arg, context, values)?;
        } else {
            let coerced =
                eval_method_parameter_value(param_type, bound_arg.value, context, values)?;
            settle_coerced_bound_arg(bound_arg, coerced, values)?;
        }
    }
    Ok(())
}

/// Applies one registered native variadic parameter type to each collected argument.
pub(super) fn apply_native_callable_variadic_arg_type(
    param_type: &EvalParameterType,
    bound_arg: &mut BoundMethodArg,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let len = values.array_len(bound_arg.value)?;
    for position in 0..len {
        let key = values.array_iter_key(bound_arg.value, position)?;
        let value = values.array_get(bound_arg.value, key)?;
        let value = eval_method_parameter_value(param_type, value, context, values)?;
        bound_arg.value = values.array_set(bound_arg.value, key, value)?;
    }
    Ok(())
}

/// Copies by-value degraded by-ref native method args before the generated bridge mutates them.
pub(super) fn copy_native_call_user_func_by_value_ref_args(
    signature: &NativeCallableSignature,
    bound_args: &mut [BoundMethodArg],
    by_ref_mode: EvalByRefBindingMode<'_>,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if !matches!(by_ref_mode, EvalByRefBindingMode::WarnByValue { .. }) {
        return Ok(());
    }
    let variadic_index = native_callable_variadic_index(signature);
    for (position, bound_arg) in bound_args.iter_mut().enumerate() {
        let param_index = if variadic_index.is_some_and(|index| position >= index) {
            variadic_index.ok_or(EvalStatus::RuntimeFatal)?
        } else {
            position
        };
        if !signature.param_by_ref(param_index) || bound_arg.ref_target.is_some() {
            continue;
        }
        let copy = copy_native_call_user_func_by_value_ref_arg(bound_arg.value, values)?;
        settle_coerced_bound_arg(bound_arg, copy, values)?;
    }
    Ok(())
}

/// Allocates a temporary runtime cell for one by-value degraded by-ref native method arg.
pub(super) fn copy_native_call_user_func_by_value_ref_arg(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let tag = values.type_tag(value)?;
    match tag {
        EVAL_TAG_INT | EVAL_TAG_FLOAT | EVAL_TAG_BOOL | EVAL_TAG_RESOURCE => {
            let word = values.raw_value_word(value)?;
            values.raw_word_value(tag, word)
        }
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(value)?;
            values.string_bytes_value(&bytes)
        }
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => values.array_clone_shallow(value),
        EVAL_TAG_OBJECT => {
            let word = values.raw_value_word(value)?;
            let retained = values.retain_raw_heap_word(word)?;
            values.raw_heap_word_value(retained)
        }
        EVAL_TAG_NULL => values.null(),
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Returns the native callable variadic slot, if metadata registered one.
pub(super) fn native_callable_variadic_index(signature: &NativeCallableSignature) -> Option<usize> {
    (0..signature.param_count()).find(|index| signature.param_variadic(*index))
}

/// Binds one positional native AOT argument to a fixed slot or variadic array.
///
/// Returns the argument BACK, with its ownership flag, when the callable has no slot left for it.
/// PHP accepts extra positional arguments to a userland function -- they are simply not bound to
/// a parameter, reachable only through `func_get_args()` -- and refusing them here turned an
/// ordinary call into a runtime fatal. Symfony's route loading is exactly that call:
/// `ObjectLoader::load()` invokes `$loaderObject->$method($this, $this->env)` against
/// `MicroKernelTrait::loadRoutes(LoaderInterface $loader)`, which declares ONE parameter.
///
/// The extra value does not reach the compiled body's `func_get_args()`, which is fixed at
/// compile time; dropping it is what the callee can actually observe.
pub(super) fn bind_native_positional_signature_arg(
    signature: &NativeCallableSignature,
    bound_args: &mut [Option<BoundMethodArg>],
    variadic_index: Option<usize>,
    next_positional: &mut usize,
    next_variadic_index: &mut i64,
    value: RuntimeCellHandle,
    ref_target: Option<EvalReferenceTarget>,
    owned: bool,
    by_ref_mode: EvalByRefBindingMode<'_>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<(RuntimeCellHandle, bool)>, EvalStatus> {
    if variadic_index.is_some_and(|index| *next_positional >= index) {
        let key = values.int(*next_variadic_index)?;
        *next_variadic_index = next_variadic_index
            .checked_add(1)
            .ok_or(EvalStatus::RuntimeFatal)?;
        let ref_target =
            native_parameter_ref_target(signature, variadic_index, ref_target, by_ref_mode, values)?;
        bind_native_variadic_arg(
            bound_args,
            variadic_index,
            key,
            value,
            ref_target,
            owned,
            values,
        )?;
        return Ok(None);
    }
    let param_index = *next_positional;
    if param_index >= bound_args.len() {
        return Ok(Some((value, owned)));
    }
    if bound_args[param_index].is_some() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let ref_target =
        native_parameter_ref_target(signature, Some(param_index), ref_target, by_ref_mode, values)?;
    bound_args[param_index] = Some(BoundMethodArg {
        value,
        ref_target,
        variadic_ref_targets: Vec::new(),
        owned,
    });
    *next_positional += 1;
    Ok(None)
}

/// Binds one named native AOT argument to a fixed non-variadic slot.
pub(super) fn bind_native_named_signature_arg(
    signature: &NativeCallableSignature,
    variadic_index: Option<usize>,
    bound_args: &mut [Option<BoundMethodArg>],
    name: &str,
    value: RuntimeCellHandle,
    ref_target: Option<EvalReferenceTarget>,
    owned: bool,
    by_ref_mode: EvalByRefBindingMode<'_>,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if let Some(param_index) = native_regular_param_index(signature, variadic_index, name) {
        if bound_args[param_index].is_some() {
            return Err(EvalStatus::RuntimeFatal);
        }
        let ref_target = native_parameter_ref_target(
            signature,
            Some(param_index),
            ref_target,
            by_ref_mode,
            values,
        )?;
        bound_args[param_index] = Some(BoundMethodArg {
            value,
            ref_target,
            variadic_ref_targets: Vec::new(),
            owned,
        });
        return Ok(());
    }
    Err(EvalStatus::RuntimeFatal)
}

/// Returns the caller writeback target required by a native by-reference parameter.
pub(super) fn native_parameter_ref_target(
    signature: &NativeCallableSignature,
    param_index: Option<usize>,
    ref_target: Option<EvalReferenceTarget>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalReferenceTarget>, EvalStatus> {
    let Some(param_index) = param_index else {
        return Ok(None);
    };
    if !signature.param_by_ref(param_index) {
        return Ok(None);
    }
    if let Some(ref_target) = ref_target {
        return Ok(Some(ref_target));
    }
    match by_ref_mode {
        EvalByRefBindingMode::RequireTarget => Err(EvalStatus::RuntimeFatal),
        EvalByRefBindingMode::WarnByValue { callable_name } => {
            let param_name = native_callable_param_warning_name(signature, param_index);
            values.warning(&format!(
                "{callable_name}(): Argument #{} (${param_name}) must be passed by reference, value given",
                param_index + 1
            ))?;
            Ok(None)
        }
    }
}

/// Returns the PHP parameter name used in native method by-reference warnings.
pub(super) fn native_callable_param_warning_name(
    signature: &NativeCallableSignature,
    param_index: usize,
) -> String {
    signature
        .param_names()
        .get(param_index)
        .filter(|name| !name.is_empty())
        .cloned()
        .unwrap_or_else(|| format!("arg{}", param_index + 1))
}

/// Returns the matching non-variadic native parameter index for one named arg.
pub(super) fn native_regular_param_index(
    signature: &NativeCallableSignature,
    variadic_index: Option<usize>,
    name: &str,
) -> Option<usize> {
    signature
        .param_names()
        .iter()
        .enumerate()
        .position(|(index, param)| Some(index) != variadic_index && param == name)
}

/// Appends one value into the native AOT variadic argument array.
pub(super) fn bind_native_variadic_arg(
    bound_args: &mut [Option<BoundMethodArg>],
    variadic_index: Option<usize>,
    key: RuntimeCellHandle,
    value: RuntimeCellHandle,
    ref_target: Option<EvalReferenceTarget>,
    owned: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let index = variadic_index.ok_or(EvalStatus::RuntimeFatal)?;
    let bound = bound_args[index].as_mut().ok_or(EvalStatus::RuntimeFatal)?;
    let array = values.array_set(bound.value, key, value)?;
    bound.value = array;
    if let Some(ref_target) = ref_target {
        bound.variadic_ref_targets.push((key, ref_target));
    }
    // `__elephc_eval_value_array_set` increfs the value before the setter consumes it, so the
    // variadic array now holds its own reference and any release the caller owed is settled here.
    if owned {
        values.release(value)?;
    }
    Ok(())
}
