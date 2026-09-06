//! Purpose:
//! Executes generated/AOT instance and static methods through native bridge scopes.
//!
//! Called from:
//! - Method, static-method, Reflection, and call_user_func dispatch.
//!
//! Key details:
//! - Checked and unchecked entry points share binding, bridge scope, and reference-mode handling.

use super::*;

/// Calls one generated/AOT instance method after native signature binding.
pub(in crate::interpreter) fn eval_native_method_with_evaluated_args(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_method_with_evaluated_args_bridge_scope(
        object,
        class_name,
        method_name,
        evaluated_args,
        None,
        None,
        context,
        values,
    )
}

/// Calls one generated/AOT instance method after validation with an optional bridge scope.
pub(super) fn eval_native_method_with_evaluated_args_bridge_scope(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut resolved_bridge_scope = bridge_scope.map(str::to_string);
    if resolved_bridge_scope.is_none() {
        if let Some(shadow_scope) =
            eval_private_scope_shadow_bridge_scope(class_name, method_name, context, values)?
        {
            // The calling scope's own private method shadows any override on
            // the receiver's class; access is inherently allowed, so skip the
            // hierarchy resolution (it would find the override instead).
            return eval_native_method_with_evaluated_args_unchecked_bridge_scope(
                object,
                class_name,
                method_name,
                evaluated_args,
                Some(&shadow_scope),
                called_class_scope,
                context,
                values,
            );
        }
    }
    let metadata =
        eval_aot_method_dispatch_metadata_in_hierarchy(class_name, method_name, context, values)?;
    if let Some((declaring_class, visibility, _, is_abstract)) = metadata {
        if resolved_bridge_scope.is_none() {
            resolved_bridge_scope = Some(declaring_class.clone());
        }
        if !is_abstract
            && validate_eval_member_access(&declaring_class, visibility, context).is_err()
        {
            if eval_native_instance_magic_method_available(class_name, context, values)? {
                return eval_native_magic_instance_method_call(
                    object,
                    class_name,
                    method_name,
                    evaluated_args,
                    context,
                    values,
                );
            }
            return eval_throw_method_access_error(
                &declaring_class,
                method_name,
                visibility,
                context,
                values,
            );
        }
    } else if eval_native_instance_magic_method_available(class_name, context, values)? {
        return eval_native_magic_instance_method_call(
            object,
            class_name,
            method_name,
            evaluated_args,
            context,
            values,
        );
    }
    eval_native_method_with_evaluated_args_unchecked_bridge_scope(
        object,
        class_name,
        method_name,
        evaluated_args,
        resolved_bridge_scope.as_deref(),
        called_class_scope,
        context,
        values,
    )
}

/// Calls one generated/AOT instance method without enforcing member visibility.
pub(super) fn eval_native_method_with_evaluated_args_unchecked(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_method_with_evaluated_args_unchecked_bridge_scope(
        object,
        class_name,
        method_name,
        evaluated_args,
        None,
        None,
        context,
        values,
    )
}

/// Calls one generated/AOT instance method without visibility checks using an optional bridge scope.
pub(in crate::interpreter) fn eval_native_method_with_evaluated_args_unchecked_bridge_scope(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
        object,
        class_name,
        method_name,
        evaluated_args,
        bridge_scope,
        called_class_scope,
        EvalByRefBindingMode::RequireTarget,
        context,
        values,
    )
}

/// Calls one generated/AOT instance method for `call_user_func()` by-value by-ref degradation.
pub(in crate::interpreter) fn eval_native_method_with_evaluated_args_for_call_user_func_unchecked_bridge_scope(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let signature_owner = bridge_scope.unwrap_or(class_name);
    let callable_name = format!("{}::{}", signature_owner.trim_start_matches('\\'), method_name);
    eval_native_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
        object,
        class_name,
        method_name,
        evaluated_args,
        bridge_scope,
        called_class_scope,
        EvalByRefBindingMode::WarnByValue {
            callable_name: &callable_name,
        },
        context,
        values,
    )
}

/// Calls one generated/AOT instance method with a selected by-reference binding mode.
pub(super) fn eval_native_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let signature_owner = bridge_scope.unwrap_or(class_name);
    let signature = context.native_method_signature(signature_owner, method_name);
    let return_type = signature.as_ref().and_then(|signature| signature.return_type().cloned());
    let bound_args = bind_native_callable_bound_args_with_mode(
        signature,
        evaluated_args,
        by_ref_mode,
        context,
        values,
    )
    .map_err(|status| {
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!(
                "[elephc-eval-trace] phase=native_method_error stage=bind class={class_name:?} method={method_name:?} status={status:?} class_scope={:?}",
                context.current_class_scope(),
            );
        }
        status
    })?;
    let mut result = RuntimeCellHandle::from_raw(std::ptr::null_mut());
    let call_result = if let Some(scope) = bridge_scope {
        eval_native_method_call_with_scope_out(
            scope,
            called_class_scope,
            object,
            method_name,
            native_bound_arg_values(&bound_args),
            &mut result,
            context,
            values,
        )
    } else {
        values.method_call_out(
            object,
            method_name,
            native_bound_arg_values(&bound_args),
            &mut result,
        )
    };
    let writeback = if native_bound_args_require_writeback(&bound_args) {
        write_back_native_callable_ref_args(&bound_args, context, values)
    } else {
        Ok(())
    };
    let released = release_owned_bound_args(
        &bound_args,
        call_result.is_ok().then_some(result),
        context,
        values,
    );
    let writeback = match (writeback, released) {
        (Err(status), _) | (_, Err(status)) => Err(status),
        (Ok(()), Ok(())) => Ok(()),
    };
    match (call_result, writeback) {
        (Err(status), _) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=native_method_error stage=invoke class={class_name:?} method={method_name:?} status={status:?} class_scope={:?}",
                    context.current_class_scope(),
                );
            }
            Err(status)
        }
        (_, Err(status)) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=native_method_error stage=writeback class={class_name:?} method={method_name:?} status={status:?} class_scope={:?}",
                    context.current_class_scope(),
                );
            }
            Err(status)
        }
        (Ok(()), Ok(())) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                let result_ptr = result.as_ptr();
                let words = unsafe { std::slice::from_raw_parts(result_ptr.cast::<u64>(), 3) };
                let tag = values.type_tag(result);
                let object_refs = matches!(tag, Ok(EVAL_TAG_OBJECT)).then(|| {
                    let object_ptr = words[1] as *const u8;
                    (!object_ptr.is_null())
                        .then(|| unsafe { object_ptr.sub(12).cast::<u32>().read() } & 0x7fff_ffff)
                });
                eprintln!(
                    "[elephc-eval-trace] phase=native_method_return_contract owner={signature_owner:?} class={class_name:?} method={method_name:?} result={result_ptr:p} words=[{:#x}, {:#x}, {:#x}] return_type={return_type:?} tag={tag:?} object_refs={object_refs:?}",
                    words[0], words[1], words[2],
                );
            }
            eval_declared_native_return_value(
                return_type.as_ref(),
                Some(signature_owner),
                called_class_scope.or(Some(class_name)),
                result,
                context,
                values,
            )
        }
        .map_err(|status| {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=native_method_error stage=return_value class={class_name:?} method={method_name:?} status={status:?} class_scope={:?}",
                    context.current_class_scope(),
                );
            }
            status
        }),
    }
}

/// Calls one generated/AOT static method after native signature binding.
pub(in crate::interpreter) fn eval_native_static_method_with_evaluated_args(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_static_method_with_evaluated_args_bridge_scope(
        class_name,
        method_name,
        evaluated_args,
        None,
        None,
        context,
        values,
    )
}

/// Calls one generated/AOT static method after validation with an optional bridge scope.
pub(super) fn eval_native_static_method_with_evaluated_args_bridge_scope(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut resolved_bridge_scope = bridge_scope.map(str::to_string);
    let metadata =
        eval_aot_method_dispatch_metadata_in_hierarchy(class_name, method_name, context, values)?;
    if let Some((declaring_class, visibility, is_static, is_abstract)) = metadata {
        if resolved_bridge_scope.is_none() {
            resolved_bridge_scope = Some(declaring_class.clone());
        }
        if is_static
            && !is_abstract
            && validate_eval_member_access(&declaring_class, visibility, context).is_err()
        {
            if eval_native_static_magic_method_available(class_name, context, values)? {
                return eval_native_magic_static_method_call(
                    class_name,
                    method_name,
                    evaluated_args,
                    context,
                    values,
                );
            }
            return eval_throw_method_access_error(
                &declaring_class,
                method_name,
                visibility,
                context,
                values,
            );
        }
    } else if eval_native_static_magic_method_available(class_name, context, values)? {
        return eval_native_magic_static_method_call(
            class_name,
            method_name,
            evaluated_args,
            context,
            values,
        );
    }
    eval_native_static_method_with_evaluated_args_unchecked_bridge_scope(
        class_name,
        method_name,
        evaluated_args,
        resolved_bridge_scope.as_deref(),
        called_class_scope,
        context,
        values,
    )
}

/// Calls one generated/AOT static method without enforcing member visibility.
pub(super) fn eval_native_static_method_with_evaluated_args_unchecked(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_static_method_with_evaluated_args_unchecked_bridge_scope(
        class_name,
        method_name,
        evaluated_args,
        None,
        None,
        context,
        values,
    )
}

/// Calls one generated/AOT static method without visibility checks using an optional bridge scope.
pub(in crate::interpreter) fn eval_native_static_method_with_evaluated_args_unchecked_bridge_scope(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_native_static_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
        class_name,
        method_name,
        evaluated_args,
        bridge_scope,
        called_class_scope,
        EvalByRefBindingMode::RequireTarget,
        context,
        values,
    )
}

/// Calls one generated/AOT static method for `call_user_func()` by-value by-ref degradation.
pub(in crate::interpreter) fn eval_native_static_method_with_evaluated_args_for_call_user_func_unchecked_bridge_scope(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let signature_owner = bridge_scope.unwrap_or(class_name);
    let callable_name = format!("{}::{}", signature_owner.trim_start_matches('\\'), method_name);
    eval_native_static_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
        class_name,
        method_name,
        evaluated_args,
        bridge_scope,
        called_class_scope,
        EvalByRefBindingMode::WarnByValue {
            callable_name: &callable_name,
        },
        context,
        values,
    )
}

/// Calls one generated/AOT static method with a selected by-reference binding mode.
pub(super) fn eval_native_static_method_with_evaluated_args_unchecked_bridge_scope_with_ref_mode(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    bridge_scope: Option<&str>,
    called_class_scope: Option<&str>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let signature_owner = bridge_scope.unwrap_or(class_name);
    let signature = context.native_static_method_signature(signature_owner, method_name);
    let return_type = signature.as_ref().and_then(|signature| signature.return_type().cloned());
    // Three things can refuse below and every one of them meant the same anonymous fatal: no
    // signature recorded for the owner, a binding that rejects the arguments, and the generated
    // static-call helper itself.
    let traced = std::env::var_os("ELEPHC_EVAL_TRACE").is_some();
    if traced {
        eprintln!(
            "[elephc-eval-trace] phase=native_static_call class={class_name:?} method={method_name:?} owner={signature_owner:?} signature={} args={}",
            signature.is_some(),
            evaluated_args.len(),
        );
    }
    let bound_args = match bind_native_callable_bound_args_with_mode(
        signature,
        evaluated_args,
        by_ref_mode,
        context,
        values,
    ) {
        Ok(bound_args) => bound_args,
        Err(status) => {
            if traced {
                eprintln!(
                    "[elephc-eval-trace] phase=native_static_call stage=bind class={class_name:?} method={method_name:?} status={status:?}"
                );
            }
            return Err(status);
        }
    };
    let result = if let Some(scope) = bridge_scope {
        eval_native_static_method_call_with_scope(
            scope,
            called_class_scope,
            class_name,
            method_name,
            native_bound_arg_values(&bound_args),
            context,
            values,
        )
    } else {
        values.static_method_call(class_name, method_name, native_bound_arg_values(&bound_args))
    };
    if traced {
        eprintln!(
            "[elephc-eval-trace] phase=native_static_call stage=called class={class_name:?} method={method_name:?} outcome={}",
            match &result {
                Ok(_) => "ok".to_string(),
                Err(status) => format!("{status:?}"),
            },
        );
    }
    let writeback = if native_bound_args_require_writeback(&bound_args) {
        write_back_native_callable_ref_args(&bound_args, context, values)
    } else {
        Ok(())
    };
    let released =
        release_owned_bound_args(&bound_args, result.as_ref().ok().copied(), context, values);
    match (result, writeback, released) {
        (Err(status), _, _) | (_, Err(status), _) | (_, _, Err(status)) => Err(status),
        (Ok(result), Ok(()), Ok(())) => eval_declared_native_return_value(
            return_type.as_ref(),
            Some(signature_owner),
            called_class_scope.or(Some(class_name)),
            result,
            context,
            values,
        ),
    }
}

/// Returns whether a generated/AOT class has an instance `__call()` fallback.
pub(super) fn eval_native_instance_magic_method_available(
    class_name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    Ok(eval_aot_method_dispatch_metadata_in_hierarchy(class_name, "__call", context, values)?
        .is_some_and(|(_, _, is_static, is_abstract)| !is_static && !is_abstract))
}

/// Returns whether a generated/AOT class has a static `__callStatic()` fallback.
pub(super) fn eval_native_static_magic_method_available(
    class_name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    Ok(
        eval_aot_method_dispatch_metadata_in_hierarchy(
            class_name,
            "__callStatic",
            context,
            values,
        )?
        .is_some_and(|(_, _, is_static, is_abstract)| is_static && !is_abstract),
    )
}

/// Dispatches a missing or inaccessible generated/AOT instance method through `__call()`.
pub(in crate::interpreter) fn eval_native_magic_instance_method_call(
    object: RuntimeCellHandle,
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let magic_args = eval_magic_call_args(method_name, evaluated_args, values)?;
    eval_native_method_with_evaluated_args_unchecked(
        object,
        class_name,
        "__call",
        magic_args,
        context,
        values,
    )
}

/// Dispatches a missing or inaccessible generated/AOT static method through `__callStatic()`.
pub(in crate::interpreter) fn eval_native_magic_static_method_call(
    class_name: &str,
    method_name: &str,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let magic_args = eval_magic_call_args(method_name, evaluated_args, values)?;
    eval_native_static_method_with_evaluated_args_unchecked(
        class_name,
        "__callStatic",
        magic_args,
        context,
        values,
    )
}
