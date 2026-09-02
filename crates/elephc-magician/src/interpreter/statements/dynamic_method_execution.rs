//! Purpose:
//! Executes eval-declared instance and static methods from prepared argument values.
//!
//! Called from:
//! - Method dispatch and callable invocation after target resolution.
//!
//! Key details:
//! - Called-class scope, reference modes, positional extraction, and writeback stay paired.

use super::*;

/// Installs a class declaration's physical source as the active magic-constant site.
fn enter_dynamic_class_method_source(
    class_name: &str,
    method: &EvalClassMethod,
    context: &mut ElephcEvalContext,
) -> Option<(String, String, i64, Option<String>)> {
    let file = context.class_source_file(class_name)?.to_string();
    let previous = context.call_site();
    let dir = std::path::Path::new(&file)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    let line = method
        .source_location()
        .map_or(previous.2, |location| location.start_line() as i64);
    context.set_call_site(file.clone(), dir, line);
    context.set_file_magic_override(Some(file));
    Some(previous)
}

/// Restores the caller's source metadata after a dynamic class method returns.
fn leave_dynamic_class_method_source(
    previous: Option<(String, String, i64, Option<String>)>,
    context: &mut ElephcEvalContext,
) {
    if let Some((file, dir, line, override_file)) = previous {
        context.set_call_site(file, dir, line);
        context.set_file_magic_override(override_file);
    }
}

/// Executes one eval-declared class method with `$this` bound in method scope.
pub(in crate::interpreter) fn eval_dynamic_method_with_values(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    object: RuntimeCellHandle,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_dynamic_method_with_values_and_ref_flags(
        class_name,
        called_class_name,
        method,
        object,
        method.parameter_is_by_ref(),
        evaluated_args,
        context,
        values,
    )
}

/// Executes one eval-declared class method with caller-selected by-ref binding flags.
pub(in crate::interpreter) fn eval_dynamic_method_with_values_and_ref_flags(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    object: RuntimeCellHandle,
    parameter_is_by_ref: &[bool],
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_dynamic_method_with_values_and_ref_mode(
        class_name,
        called_class_name,
        method,
        object,
        parameter_is_by_ref,
        evaluated_args,
        EvalByRefBindingMode::RequireTarget,
        context,
        values,
    )
}

/// Executes one eval-declared class method with caller-selected by-ref mode.
pub(in crate::interpreter) fn eval_dynamic_method_with_values_and_ref_mode(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    object: RuntimeCellHandle,
    parameter_is_by_ref: &[bool],
    evaluated_args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let qualified_method_name =
        format!("{}::{}", class_name.trim_start_matches('\\'), method.name());
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=dynamic_method_start method={qualified_method_name:?} called_class={called_class_name:?} object_identity={:?} file={:?} line={}",
            values.object_identity(object).ok(),
            call_site.0,
            call_site.2,
        );
    }
    let static_names = static_var_names(method.body());
    context.push_function(qualified_method_name.clone());
    context.push_class_scope(class_name.to_string());
    context.push_called_class_scope(called_class_name.to_string());
    context.push_method_magic_scope(class_name, method);
    let evaluated_args = match bind_evaluated_method_args_with_ref_mode(
        method.params(),
        method.parameter_types(),
        method.parameter_defaults(),
        parameter_is_by_ref,
        method.parameter_is_variadic(),
        evaluated_args,
        by_ref_mode,
        context,
        values,
    ) {
        Ok(args) => args,
        Err(status) => {
            context.pop_magic_scope();
            context.pop_called_class_scope();
            context.pop_class_scope();
            context.pop_function();
            return Err(status);
        }
    };
    let mut method_scope = ElephcEvalScope::new();
    method_scope.set("this", object, ScopeCellOwnership::Borrowed);
    let scope_parameter_is_by_ref =
        method_scope_parameter_ref_flags(parameter_is_by_ref, &evaluated_args, by_ref_mode);
    bind_method_scope_args(
        &mut method_scope,
        method.params(),
        &scope_parameter_is_by_ref,
        &evaluated_args,
    );
    let previous_source = enter_dynamic_class_method_source(class_name, method, context);
    let result = execute_statements(method.body(), context, &mut method_scope, values);
    let persist_result = persist_static_locals(
        context,
        &qualified_method_name,
        &static_names,
        &method_scope,
        values,
    );
    let writeback_result = write_back_method_ref_args(
        method.params(),
        &evaluated_args,
        &method_scope,
        context,
        values,
    );
    let return_result = match (persist_result, writeback_result, result) {
        (Err(status), _, _) | (_, Err(status), _) | (_, _, Err(status)) => Err(status),
        (Ok(()), Ok(()), Ok(control)) => eval_declared_return_control_value(
            method.return_type(),
            Some(class_name),
            Some(called_class_name),
            control,
            context,
            values,
        ),
    };
    leave_dynamic_class_method_source(previous_source, context);
    context.pop_magic_scope();
    context.pop_called_class_scope();
    context.pop_class_scope();
    context.pop_function();
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=dynamic_method_end method={qualified_method_name:?} success={} file={:?} line={}",
            return_result.is_ok(),
            call_site.0,
            call_site.2,
        );
    }
    return_result
}

/// Executes one eval-declared static class method without binding `$this`.
pub(in crate::interpreter) fn eval_dynamic_static_method_with_values(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_dynamic_static_method_with_values_and_ref_flags(
        class_name,
        called_class_name,
        method,
        method.parameter_is_by_ref(),
        evaluated_args,
        context,
        values,
    )
}

/// Executes one eval-declared static method with caller-selected by-ref binding flags.
pub(in crate::interpreter) fn eval_dynamic_static_method_with_values_and_ref_flags(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    parameter_is_by_ref: &[bool],
    evaluated_args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_dynamic_static_method_with_values_and_ref_mode(
        class_name,
        called_class_name,
        method,
        parameter_is_by_ref,
        evaluated_args,
        EvalByRefBindingMode::RequireTarget,
        context,
        values,
    )
}

/// Executes one eval-declared static method with caller-selected by-ref mode.
pub(in crate::interpreter) fn eval_dynamic_static_method_with_values_and_ref_mode(
    class_name: &str,
    called_class_name: &str,
    method: &EvalClassMethod,
    parameter_is_by_ref: &[bool],
    evaluated_args: Vec<EvaluatedCallArg>,
    by_ref_mode: EvalByRefBindingMode<'_>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let qualified_method_name =
        format!("{}::{}", class_name.trim_start_matches('\\'), method.name());
    let static_names = static_var_names(method.body());
    context.push_function(qualified_method_name.clone());
    context.push_class_scope(class_name.to_string());
    context.push_called_class_scope(called_class_name.to_string());
    context.push_method_magic_scope(class_name, method);
    let evaluated_args = match bind_evaluated_method_args_with_ref_mode(
        method.params(),
        method.parameter_types(),
        method.parameter_defaults(),
        parameter_is_by_ref,
        method.parameter_is_variadic(),
        evaluated_args,
        by_ref_mode,
        context,
        values,
    ) {
        Ok(args) => args,
        Err(status) => {
            context.pop_magic_scope();
            context.pop_called_class_scope();
            context.pop_class_scope();
            context.pop_function();
            return Err(status);
        }
    };
    let mut method_scope = ElephcEvalScope::new();
    let scope_parameter_is_by_ref =
        method_scope_parameter_ref_flags(parameter_is_by_ref, &evaluated_args, by_ref_mode);
    bind_method_scope_args(
        &mut method_scope,
        method.params(),
        &scope_parameter_is_by_ref,
        &evaluated_args,
    );
    let previous_source = enter_dynamic_class_method_source(class_name, method, context);
    let result = execute_statements(method.body(), context, &mut method_scope, values);
    let persist_result = persist_static_locals(
        context,
        &qualified_method_name,
        &static_names,
        &method_scope,
        values,
    );
    let writeback_result = write_back_method_ref_args(
        method.params(),
        &evaluated_args,
        &method_scope,
        context,
        values,
    );
    let return_result = match (persist_result, writeback_result, result) {
        (Err(status), _, _) | (_, Err(status), _) | (_, _, Err(status)) => Err(status),
        (Ok(()), Ok(()), Ok(control)) => eval_declared_return_control_value(
            method.return_type(),
            Some(class_name),
            Some(called_class_name),
            control,
            context,
            values,
        ),
    };
    leave_dynamic_class_method_source(previous_source, context);
    context.pop_magic_scope();
    context.pop_called_class_scope();
    context.pop_class_scope();
    context.pop_function();
    return_result
}

/// Wraps positional method arguments into the shared dynamic-call binding shape.
pub(in crate::interpreter) fn positional_args(
    args: Vec<RuntimeCellHandle>,
) -> Vec<EvaluatedCallArg> {
    args.into_iter()
        .map(|value| EvaluatedCallArg {
            name: None,
            value,
            ref_target: None,
        })
        .collect()
}

/// Extracts positional runtime values and rejects named args before runtime method dispatch.
pub(in crate::interpreter) fn positional_evaluated_arg_values(
    args: Vec<EvaluatedCallArg>,
) -> Result<Vec<RuntimeCellHandle>, EvalStatus> {
    if args.iter().any(|arg| arg.name.is_some()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(args.into_iter().map(|arg| arg.value).collect())
}
