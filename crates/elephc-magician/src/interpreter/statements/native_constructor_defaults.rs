//! Purpose:
//! Resolves AOT method metadata, invokes native constructors, and materializes defaults.
//!
//! Called from:
//! - Native class construction and signature binding.
//!
//! Key details:
//! - Parent metadata lookup, constructor visibility, and compound default ownership stay centralized.

use super::*;

/// Finds generated/AOT method metadata on a class or its native parent chain.
pub(in crate::interpreter) fn eval_aot_method_dispatch_metadata_in_hierarchy(
    class_name: &str,
    method_name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<(String, EvalVisibility, bool, bool)>, EvalStatus> {
    let mut current = class_name.trim_start_matches('\\').to_string();
    let mut seen = std::collections::HashSet::new();
    loop {
        if !seen.insert(current.to_ascii_lowercase()) {
            return Ok(None);
        }
        if let Some(metadata) = eval_aot_method_dispatch_metadata(&current, method_name, values)? {
            return Ok(Some(metadata));
        }
        let Some(parent) = context.native_class_parent(&current) else {
            return Ok(None);
        };
        current = parent.to_string();
    }
}

/// Runs one generated/AOT constructor after native signature binding.
pub(in crate::interpreter) fn eval_native_constructor_with_evaluated_args(
    class_name: &str,
    object: RuntimeCellHandle,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    eval_native_constructor_with_evaluated_args_and_ref_mode(
        class_name,
        object,
        evaluated_args,
        EvalByRefBindingMode::RequireTarget,
        context,
        values,
    )
}

/// Runs one generated/AOT constructor with caller-selected by-ref binding behavior.
pub(super) fn eval_native_constructor_with_evaluated_args_and_ref_mode(
    class_name: &str,
    object: RuntimeCellHandle,
    evaluated_args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let constructor_metadata = eval_aot_method_dispatch_metadata_in_hierarchy(
        class_name,
        "__construct",
        context,
        values,
    )?;
    let constructor_class = constructor_metadata
        .as_ref()
        .map(|(declaring_class, _, _, _)| declaring_class.clone())
        .unwrap_or_else(|| class_name.trim_start_matches('\\').to_string());
    let constructor_target_is_known = constructor_metadata.is_some();
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        eprintln!(
            "[elephc-eval-trace] phase=native_constructor stage=resolved requested={class_name:?} declaring={constructor_class:?} args={}",
            evaluated_args.len(),
        );
    }
    let access_error = eval_native_constructor_access_error(&constructor_class, context, values)
        .map_err(|status| trace_native_constructor_error("access_metadata", &constructor_class, status))?;
    if let Some(message) = access_error {
        return eval_throw_error(&message, context, values);
    }
    let bridge_scope = eval_native_constructor_bridge_scope(&constructor_class, context, values)
        .map_err(|status| trace_native_constructor_error("bridge_scope", &constructor_class, status))?;
    let mut signature = context.native_constructor_signature(&constructor_class);
    #[cfg(not(test))]
    if constructor_target_is_known && signature.is_none() {
        crate::context::sync_global_eval_aot_metadata(context);
        signature = context.native_constructor_signature(&constructor_class);
    }
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        let signature_details = signature.as_ref().map(|signature| {
            (
                signature.param_count(),
                signature.param_names().len(),
                signature.required_param_count(),
                signature.bridge_supported(),
            )
        });
        eprintln!(
            "[elephc-eval-trace] phase=native_constructor stage=signature class={constructor_class:?} details={signature_details:?}",
        );
    }
    let bound_args = bind_native_callable_bound_args_with_mode(
        signature,
        evaluated_args,
        by_ref_mode,
        context,
        values,
    )
    .map_err(|status| trace_native_constructor_error("bind", &constructor_class, status))?;
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        eprintln!(
            "[elephc-eval-trace] phase=native_constructor stage=bound class={constructor_class:?} args={}",
            bound_args.len(),
        );
    }
    let result = if let Some(scope) = bridge_scope.as_deref() {
        eval_with_native_bridge_scope(scope, context, || {
            if constructor_target_is_known {
                values.construct_object_for_class(
                    &constructor_class,
                    object,
                    native_bound_arg_values(&bound_args),
                )
            } else {
                values.construct_object(object, native_bound_arg_values(&bound_args))
            }
        })
    } else {
        if constructor_target_is_known {
            values.construct_object_for_class(
                &constructor_class,
                object,
                native_bound_arg_values(&bound_args),
            )
        } else {
            values.construct_object(object, native_bound_arg_values(&bound_args))
        }
    };
    let writeback = if native_bound_args_require_writeback(&bound_args) {
        write_back_native_callable_ref_args(&bound_args, context, values)
    } else {
        Ok(())
    };
    // A constructor answers no value, so no argument cell can escape as its result.
    let writeback = match (
        writeback,
        release_owned_bound_args(&bound_args, None, context, values),
    ) {
        (Err(status), _) | (_, Err(status)) => Err(status),
        (Ok(()), Ok(())) => Ok(()),
    };
    match (result, writeback) {
        (Err(status), _) => Err(trace_native_constructor_error(
            "construct",
            &constructor_class,
            status,
        )),
        (_, Err(status)) => Err(trace_native_constructor_error(
            "writeback",
            &constructor_class,
            status,
        )),
        (Ok(()), Ok(())) => Ok(()),
    }
}

/// Emits the failed native-constructor phase under opt-in eval tracing.
fn trace_native_constructor_error(
    stage: &str,
    constructor_class: &str,
    status: EvalStatus,
) -> EvalStatus {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        eprintln!(
            "[elephc-eval-trace] phase=native_constructor_error stage={stage} class={constructor_class:?} status={status:?}",
        );
    }
    status
}

/// Returns the generated/AOT constructor scope that the runtime bridge can recognize.
pub(super) fn eval_native_constructor_bridge_scope(
    class_name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<String>, EvalStatus> {
    let Some((declaring_class, visibility)) =
        eval_reflection_aot_non_public_constructor(class_name, values)?
    else {
        return Ok(None);
    };
    if eval_native_constructor_access_allowed(&declaring_class, visibility, context) {
        Ok(Some(declaring_class))
    } else {
        Ok(None)
    }
}

/// Returns PHP's constructor access error for generated/AOT constructors, if inaccessible.
pub(super) fn eval_native_constructor_access_error(
    class_name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<String>, EvalStatus> {
    let Some((declaring_class, visibility)) =
        eval_reflection_aot_non_public_constructor(class_name, values)?
    else {
        return Ok(None);
    };
    if eval_native_constructor_access_allowed(&declaring_class, visibility, context) {
        return Ok(None);
    }
    Ok(Some(format!(
        "Call to {} {}::__construct() from {}",
        eval_visibility_label(visibility),
        declaring_class.trim_start_matches('\\'),
        eval_native_constructor_scope_label(context)
    )))
}

/// Returns whether the current eval scope may call one generated/AOT constructor.
pub(super) fn eval_native_constructor_access_allowed(
    declaring_class: &str,
    visibility: EvalVisibility,
    context: &ElephcEvalContext,
) -> bool {
    match visibility {
        EvalVisibility::Public => true,
        EvalVisibility::Private => context
            .current_class_scope()
            .is_some_and(|current| same_eval_class_name(current, declaring_class)),
        EvalVisibility::Protected => context
            .current_class_scope()
            .is_some_and(|current| eval_classes_are_related(current, declaring_class, context)),
    }
}

/// Returns PHP's scope phrase for constructor access diagnostics.
pub(super) fn eval_native_constructor_scope_label(context: &ElephcEvalContext) -> String {
    context.current_class_scope().map_or_else(
        || String::from("global scope"),
        |class_name| format!("scope {}", class_name.trim_start_matches('\\')),
    )
}

/// Returns PHP's lowercase visibility label.
pub(super) fn eval_visibility_label(visibility: EvalVisibility) -> &'static str {
    match visibility {
        EvalVisibility::Public => "public",
        EvalVisibility::Protected => "protected",
        EvalVisibility::Private => "private",
    }
}

/// Allocates a fresh runtime cell for one invocation-safe native AOT default.
pub(in crate::interpreter) fn materialize_native_callable_default(
    default: &NativeCallableDefault,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match default {
        NativeCallableDefault::Null => values.null(),
        NativeCallableDefault::Bool(value) => values.bool_value(*value),
        NativeCallableDefault::Int(value) => values.int(*value),
        NativeCallableDefault::Float(value) => values.float(*value),
        NativeCallableDefault::String(value) => values.string(value),
        NativeCallableDefault::EmptyArray => values.array_new(0),
        NativeCallableDefault::Array(elements) => {
            materialize_native_callable_array_default(elements, context, values)
        }
        NativeCallableDefault::Object { class_name, args } => {
            materialize_native_callable_object_default(class_name, args, context, values)
        }
    }
}

/// Allocates one array-valued native AOT parameter default with fresh element cells.
pub(super) fn materialize_native_callable_array_default(
    elements: &[NativeCallableArrayDefaultElement],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let has_string_key = elements.iter().any(|element| {
        matches!(
            element.key,
            Some(NativeCallableArrayDefaultKey::String(_))
        )
    });
    let mut array = if has_string_key {
        values.assoc_new(elements.len())?
    } else {
        values.array_new(elements.len())?
    };
    let mut next_auto_key = 0;
    for element in elements {
        let key = match &element.key {
            Some(NativeCallableArrayDefaultKey::Int(value)) => {
                if *value >= next_auto_key {
                    next_auto_key = value.saturating_add(1);
                }
                values.int(*value)?
            }
            Some(NativeCallableArrayDefaultKey::String(value)) => values.string(value)?,
            None => {
                let key = values.int(next_auto_key)?;
                next_auto_key = next_auto_key.saturating_add(1);
                key
            }
        };
        let value = materialize_native_callable_default(&element.value, context, values)?;
        array = values.array_set(array, key, value)?;
    }
    Ok(array)
}

/// Allocates and constructs one object-valued native AOT parameter default.
pub(super) fn materialize_native_callable_object_default(
    class_name: &str,
    args: &[NativeCallableObjectDefaultArg],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let object = values.new_object(class_name)?;
    let mut constructor_args = Vec::with_capacity(args.len());
    for arg in args {
        constructor_args.push(EvaluatedCallArg {
            name: arg.name.clone(),
            value: materialize_native_callable_default(&arg.value, context, values)?,
            ref_target: None,
            owned: true,
        });
    }
    if let Err(err) = eval_native_constructor_with_evaluated_args(
        class_name,
        object,
        constructor_args,
        context,
        values,
    ) {
        let _ = values.release(object);
        return Err(err);
    }
    Ok(object)
}
