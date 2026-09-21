//! Purpose:
//! Constructs `ReflectionFunction` owners for eval, closure, and native targets.
//!
//! Called from:
//! - `crate::interpreter::reflection::eval_reflection_owner_new_object()`.
//!
//! Key details:
//! - Closure targets and native parameter/default metadata are attached once here.

use super::*;
use crate::context::{decode_eval_callback_adapter_capture, DecodedCallableCapture};
use std::ffi::c_void;

/// Normalized callable target used while constructing a reflected function.
struct EvalReflectionFunctionCallableArg {
    target: EvalClosureObjectTarget,
    lookup_name: String,
    display_name: String,
    metadata_closure: Option<EvalClosure>,
}

/// Builds an eval-backed `ReflectionFunction` object for eval or registered native functions.
pub(super) fn eval_reflection_function_new(
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
    let args = bind_evaluated_function_args(&[String::from("function")], evaluated_args)?;
    let callable_target =
        eval_reflection_function_callable_target_arg(args[0], context, values)?;
    let closure_target = callable_target.as_ref().map(|target| target.target.clone());
    let requested_name = match callable_target.as_ref() {
        Some(target) => target.display_name.clone(),
        None => eval_reflection_function_name_arg(args[0], context, values)?,
    };
    let lookup_name = callable_target
        .as_ref()
        .map(|target| target.lookup_name.clone())
        .unwrap_or_else(|| requested_name.trim_start_matches('\\').to_ascii_lowercase());
    let metadata_closure = callable_target
        .as_ref()
        .and_then(|target| target.metadata_closure.clone())
        .or_else(|| context.closure(&lookup_name).cloned());
    if let Some(closure) = metadata_closure {
        let function = closure.function();
        let required_parameter_count = eval_reflection_required_parameter_count(
            function.parameter_defaults(),
            function.parameter_is_variadic(),
        );
        let parameters = eval_reflection_function_parameters(
            function.name(),
            function.params(),
            function.attributes().to_vec(),
            function.parameter_attributes(),
            function.parameter_types(),
            function.parameter_defaults(),
            function.parameter_is_by_ref(),
            function.parameter_is_variadic(),
        );
        let return_type_metadata = function
            .return_type()
            .and_then(eval_reflection_parameter_type_metadata);
        return eval_reflection_function_object_result(
            &requested_name,
            function.attributes(),
            &parameters,
            return_type_metadata.as_ref(),
            required_parameter_count,
            context,
            values,
        )
        .and_then(|object| {
            eval_reflection_attach_function_callable_target(
                object,
                closure_target,
                args[0],
                context,
                values,
            )
        })
        .map(Some);
    }
    if let Some(function) = context.function(&lookup_name).cloned() {
        let required_parameter_count = eval_reflection_required_parameter_count(
            function.parameter_defaults(),
            function.parameter_is_variadic(),
        );
        let parameters = eval_reflection_function_parameters(
            function.name(),
            function.params(),
            function.attributes().to_vec(),
            function.parameter_attributes(),
            function.parameter_types(),
            function.parameter_defaults(),
            function.parameter_is_by_ref(),
            function.parameter_is_variadic(),
        );
        let return_type_metadata = function
            .return_type()
            .and_then(eval_reflection_parameter_type_metadata);
        return eval_reflection_function_object_result(
            function.name(),
            function.attributes(),
            &parameters,
            return_type_metadata.as_ref(),
            required_parameter_count,
            context,
            values,
        )
        .and_then(|object| {
            eval_reflection_attach_function_callable_target(
                object,
                closure_target,
                args[0],
                context,
                values,
            )
        })
        .map(Some);
    }
    if let Some(function) = context.native_function(&lookup_name) {
        let reflected_name = requested_name.trim_start_matches('\\');
        let required_parameter_count = function.required_param_count();
        let parameters = eval_reflection_native_function_parameters(reflected_name, &function);
        let return_type_metadata = function
            .return_type()
            .and_then(eval_reflection_parameter_type_metadata);
        return eval_reflection_function_object_result(
            reflected_name,
            &[],
            &parameters,
            return_type_metadata.as_ref(),
            required_parameter_count,
            context,
            values,
        )
        .and_then(|object| {
            eval_reflection_attach_function_callable_target(
                object,
                closure_target,
                args[0],
                context,
                values,
            )
        })
        .map(Some);
    }
    // A first-class callable built from a METHOD -- `$obj(...)`, `$obj->m(...)`, `Cls::m(...)`
    // -- has no entry in any FUNCTION table, so all three lookups above miss it and the
    // no-parameter fallback below used to claim it. PHP reflects such a Closure with the
    // method's own parameters, and this object's `ReflectionParameter` list is materialized
    // HERE, at construction: leaving it empty is what made Symfony's `ArgumentMetadataFactory`
    // -- it reads `(new \ReflectionFunction($controller(...)))->getParameters()` -- resolve
    // zero arguments for an invokable-object controller, so the controller call then bound
    // nothing to a required parameter.
    if let Some((declaring_class, method_name)) =
        eval_reflection_closure_target_method(closure_target.as_ref(), context, values)
    {
        let method_metadata =
            match eval_reflection_method_metadata(&declaring_class, &method_name, context) {
                Some(method_metadata) => Some(method_metadata),
                None => eval_reflection_aot_method_metadata_with_signature_if_exists(
                    &declaring_class,
                    &method_name,
                    context,
                    values,
                )?,
            };
        if let Some(method) = method_metadata {
            return eval_reflection_function_object_result(
                &requested_name,
                &method.attributes,
                &method.parameters,
                method.return_type_metadata.as_ref(),
                method.required_parameter_count,
                context,
                values,
            )
            .and_then(|object| {
                eval_reflection_attach_function_callable_target(
                    object,
                    closure_target,
                    args[0],
                    context,
                    values,
                )
            })
            .map(Some);
        }
    }
    if closure_target.is_some() {
        return eval_reflection_function_object_result(
            &requested_name,
            &[],
            &[],
            None,
            0,
            context,
            values,
        )
        .and_then(|object| {
            eval_reflection_attach_function_callable_target(
                object,
                closure_target,
                args[0],
                context,
                values,
            )
        })
        .map(Some);
    }
    Ok(None)
}

/// Returns the retained callable target for a Closure object or descriptor value.
fn eval_reflection_function_callable_target_arg(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalReflectionFunctionCallableArg>, EvalStatus> {
    match values.type_tag(value)? {
        EVAL_TAG_OBJECT => {
            let identity = values.object_identity(value)?;
            let Some(target) = context.closure_object_target(identity).cloned() else {
                return Ok(None);
            };
            let lookup_name = eval_reflection_function_closure_target_name(&target);
            Ok(Some(EvalReflectionFunctionCallableArg {
                display_name: lookup_name.clone(),
                lookup_name,
                target,
                metadata_closure: None,
            }))
        }
        EVAL_TAG_CALLABLE => {
            let descriptor = values.raw_value_word(value)? as usize as *mut c_void;
            let trace = crate::eval_trace::enabled();
            if trace {
                eprintln!(
                    "[elephc-eval-trace] phase=reflection-callable descriptor={descriptor:p}"
                );
            }
            let Some(decoded) = (unsafe { decode_callable_descriptor(descriptor) }) else {
                let captures = unsafe { decode_eval_callback_adapter_capture(descriptor.cast()) };
                let Some((captured_context, captured)) = captures else {
                    if trace {
                        eprintln!(
                            "[elephc-eval-trace] phase=reflection-adapter-missing descriptor={descriptor:p}"
                        );
                    }
                    return Err(EvalStatus::RuntimeFatal);
                };
                if trace {
                    eprintln!(
                        "[elephc-eval-trace] phase=reflection-adapter descriptor={descriptor:p} context=0x{captured_context:x} capture_tag={} capture=0x{:x}",
                        captured.type_tag, captured.value_word
                    );
                }
                let captured = if captured.type_tag == EVAL_TAG_MIXED {
                    RuntimeCellHandle::from_raw(
                        captured.value_word as usize as *mut crate::value::RuntimeCell,
                    )
                } else {
                    values.raw_word_value(captured.type_tag, captured.value_word)?
                };
                if values.type_tag(captured)? == EVAL_TAG_OBJECT {
                    let identity = values.object_identity(captured)?;
                    let captured_context = unsafe {
                        (captured_context as usize as *mut ElephcEvalContext).as_ref()
                    }
                    .ok_or(EvalStatus::RuntimeFatal)?;
                    if trace {
                        eprintln!(
                            "[elephc-eval-trace] phase=reflection-adapter-object identity={identity} target_found={}",
                            captured_context.closure_object_target(identity).is_some()
                        );
                    }
                    if let Some(target) = captured_context.closure_object_target(identity).cloned() {
                        let lookup_name = eval_reflection_function_closure_target_name(&target);
                        return Ok(Some(EvalReflectionFunctionCallableArg {
                            display_name: lookup_name.clone(),
                            metadata_closure: captured_context.closure(&lookup_name).cloned(),
                            lookup_name,
                            target,
                        }));
                    }
                }
                return eval_reflection_function_callable_target_arg(captured, context, values);
            };
            if trace {
                eprintln!(
                    "[elephc-eval-trace] phase=reflection-descriptor descriptor={descriptor:p} display={:?}",
                    decoded.display_name
                );
            }
            let bound_this = eval_reflection_descriptor_bound_this(decoded.bound_this, values)?;
            let lookup_name = format!("{{closure:native:{:x}}}", descriptor as usize);
            if context.native_function(&lookup_name).is_none() {
                if let Err(_existing) =
                    context.define_native_function(lookup_name.clone(), decoded.function)
                {
                    if trace {
                        eprintln!(
                            "[elephc-eval-trace] phase=reflection-descriptor-register failed=true"
                        );
                    }
                    return Err(EvalStatus::RuntimeFatal);
                }
            }
            let target = match bound_this {
                Some(bound_this) => EvalClosureObjectTarget::BoundNamed {
                    name: lookup_name.clone(),
                    bound_scope: Some(eval_closure_bound_object_class_name(
                        bound_this,
                        context,
                        values,
                    )?),
                    bound_this: Some(bound_this),
                },
                None => EvalClosureObjectTarget::Named(lookup_name.clone()),
            };
            Ok(Some(EvalReflectionFunctionCallableArg {
                target,
                lookup_name,
                display_name: decoded.display_name,
                metadata_closure: None,
            }))
        }
        _ => Ok(None),
    }
}

/// Materializes a descriptor's `$this` capture only when it contains an object receiver.
fn eval_reflection_descriptor_bound_this(
    capture: Option<DecodedCallableCapture>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<RuntimeCellHandle>, EvalStatus> {
    let Some(capture) = capture else {
        return Ok(None);
    };
    let value = values.raw_word_value(capture.type_tag, capture.value_word)?;
    if values.type_tag(value)? == EVAL_TAG_OBJECT {
        return Ok(Some(value));
    }
    values.release(value)?;
    Ok(None)
}

/// Returns the function-like name exposed for a Closure-backed ReflectionFunction.
pub(super) fn eval_reflection_function_closure_target_name(target: &EvalClosureObjectTarget) -> String {
    match target {
        EvalClosureObjectTarget::ForeignContext { target, .. } => {
            eval_reflection_function_closure_target_name(target)
        }
        EvalClosureObjectTarget::Named(name)
        | EvalClosureObjectTarget::BoundNamed { name, .. } => name.clone(),
        EvalClosureObjectTarget::InvokableObject { .. } => String::from("__invoke"),
        EvalClosureObjectTarget::ObjectMethod { method, .. }
        | EvalClosureObjectTarget::StaticMethod { method, .. } => method.clone(),
    }
}

/// Attaches original callable metadata and storage to a synthetic reflected function.
pub(super) fn eval_reflection_attach_function_callable_target(
    object: RuntimeCellHandle,
    closure_target: Option<EvalClosureObjectTarget>,
    source: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let Some(closure_target) = closure_target else {
        return Ok(object);
    };
    let identity = values.object_identity(object)?;
    context.register_eval_reflection_function_closure_target(identity, closure_target);
    if values.type_tag(source)? == EVAL_TAG_CALLABLE {
        eval_reflection_with_declaring_class_scope("ReflectionFunction", context, |_| {
            values.property_set(object, "__callable", source)
        })?;
    }
    Ok(object)
}

/// Returns parameter names for a registered native function, filling missing bridge names.
pub(super) fn eval_reflection_native_function_parameter_names(function: &NativeFunction) -> Vec<String> {
    (0..function.param_count())
        .map(|index| {
            function
                .param_names()
                .get(index)
                .filter(|name| !name.is_empty())
                .cloned()
                .unwrap_or_else(|| format!("arg{}", index))
        })
        .collect()
}

/// Builds ReflectionParameter metadata for one registered native AOT function.
pub(super) fn eval_reflection_native_function_parameters(
    function_name: &str,
    function: &NativeFunction,
) -> Vec<EvalReflectionParameterMetadata> {
    let parameter_names = eval_reflection_native_function_parameter_names(function);
    let parameter_count = parameter_names.len();
    let parameter_attributes = vec![Vec::new(); parameter_count];
    let parameter_types = eval_reflection_native_function_parameter_types(function);
    let parameter_defaults = eval_reflection_native_function_parameter_defaults(function);
    let parameter_is_by_ref = (0..parameter_count)
        .map(|index| function.param_by_ref(index))
        .collect::<Vec<_>>();
    let parameter_is_variadic = (0..parameter_count)
        .map(|index| function.param_variadic(index))
        .collect::<Vec<_>>();
    eval_reflection_function_parameters(
        function_name,
        &parameter_names,
        Vec::new(),
        &parameter_attributes,
        &parameter_types,
        &parameter_defaults,
        &parameter_is_by_ref,
        &parameter_is_variadic,
    )
}

/// Converts registered native function parameter types into reflection metadata input.
pub(super) fn eval_reflection_native_function_parameter_types(
    function: &NativeFunction,
) -> Vec<Option<EvalParameterType>> {
    (0..function.param_count())
        .map(|index| function.param_type(index).cloned())
        .collect()
}

/// Converts registered native function defaults into eval constant expressions.
pub(super) fn eval_reflection_native_function_parameter_defaults(
    function: &NativeFunction,
) -> Vec<Option<EvalExpr>> {
    (0..function.param_count())
        .map(|index| {
            function
                .param_default(index)
                .map(eval_reflection_native_callable_default_expr)
        })
        .collect()
}

/// Builds one `ReflectionFunction` object from retained eval function metadata.
pub(super) fn eval_reflection_function_object_result(
    function_name: &str,
    attributes: &[EvalAttribute],
    parameters: &[EvalReflectionParameterMetadata],
    return_type_metadata: Option<&EvalReflectionParameterTypeMetadata>,
    required_parameter_count: usize,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_reflection_owner_object(
        EVAL_REFLECTION_OWNER_FUNCTION,
        function_name,
        attributes,
        &[],
        &[],
        &[],
        &[],
        None,
        parameters,
        return_type_metadata,
        None,
        None,
        None,
        eval_reflection_callable_flags(attributes),
        required_parameter_count as u64,
        0,
        None,
        None,
        context,
        values,
    )
}
