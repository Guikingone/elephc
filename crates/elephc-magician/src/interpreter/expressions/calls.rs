//! Purpose:
//! Evaluates function-like EvalIR calls and first-class callable expressions.
//!
//! Called from:
//! - `crate::interpreter::expressions::eval_expr()` for call-shaped expressions.
//!
//! Key details:
//! - Source-sensitive constructs and by-reference builtins keep unevaluated or
//!   ref-target arguments before ordinary registry direct-call dispatch.
//! - Dynamic callables preserve PHP source-order argument evaluation before
//!   normalized callable invocation.

use super::*;
use crate::context::decode_eval_callback_adapter_capture;

mod first_class;
mod first_class_support;

pub(in crate::interpreter) use first_class::*;
use first_class_support::*;

/// Returns cloned positional argument expressions, rejecting named arguments.
pub(in crate::interpreter) fn positional_call_arg_exprs(
    args: &[EvalCallArg],
) -> Result<Vec<EvalExpr>, EvalStatus> {
    if args
        .iter()
        .any(|arg| arg.name().is_some() || arg.is_spread())
    {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(args.iter().map(|arg| arg.value().clone()).collect())
}

/// Evaluates method-call arguments, preserving named metadata for eval method binding.
pub(in crate::interpreter) fn eval_method_call_arg_values(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<EvaluatedCallArg>, EvalStatus> {
    eval_call_arg_values(args, context, scope, values)
}

/// Evaluates supported function-like calls from a runtime eval fragment.
pub(in crate::interpreter) fn eval_call(
    name: &str,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if eval_expr_language_construct_name(name) {
        let args = positional_call_arg_exprs(args)?;
        return eval_positional_expr_call(name, &args, context, scope, values);
    }
    if name == "trigger_error" {
        let args = positional_call_arg_exprs(args)?;
        return eval_builtin_trigger_error(&args, context, scope, values);
    }
    if name == "flock" {
        return eval_builtin_flock(args, context, scope, values);
    }
    if name == "preg_match" {
        return eval_builtin_preg_match_call(args, context, scope, values);
    }
    if name == "preg_match_all" {
        return eval_builtin_preg_match_all_call(args, context, scope, values);
    }
    if name == "preg_replace" {
        return eval_builtin_preg_replace_call(args, context, scope, values);
    }
    if name == "preg_replace_callback" {
        return eval_builtin_preg_replace_callback_call(args, context, scope, values);
    }
    if name == "openssl_encrypt" {
        return eval_builtin_openssl_encrypt_call(args, context, scope, values);
    }
    if name == "is_callable" {
        return eval_builtin_is_callable_call(args, context, scope, values);
    }
    if matches!(name, "fsockopen" | "pfsockopen") {
        return eval_builtin_fsockopen_call(args, context, scope, values);
    }
    // `debug_backtrace` and `debug_print_backtrace` describe interpreter frames, which only
    // the interpreter has, so they are dispatched as plain runtime handlers rather than
    // through the PHP-visible builtin registry. See `builtins::core::debug_backtrace`.
    if matches!(name, "debug_backtrace" | "debug_print_backtrace") {
        return eval_debug_backtrace_call(name, args, context, scope, values);
    }
    // The two by-reference curl builtins, intercepted here for the same reason every other
    // by-reference builtin above is: `eval_positional_expr_call` hands its hook only
    // `&[EvalExpr]`, which has already lost named-argument metadata, and the by-value
    // dispatchers have no reference targets at all. See
    // `crate::interpreter::builtins::curl::curl_multi_exec`'s header.
    #[cfg(feature = "curl")]
    if name == "curl_multi_exec" {
        return eval_builtin_curl_multi_exec_call(args, context, scope, values);
    }
    #[cfg(feature = "curl")]
    if name == "curl_multi_info_read" {
        return eval_builtin_curl_multi_info_read_call(args, context, scope, values);
    }
    if name.starts_with("pcntl_") && eval_php_visible_builtin_exists(name) {
        return eval_builtin_pcntl_call(name, args, context, scope, values);
    }
    // The xml surface forwards to the host's compiled prelude; the source-level path keeps
    // `xml_parse_into_struct()`'s by-reference outputs and named arguments intact.
    if eval_xml_builtin_name(name) {
        return eval_builtin_xml_call(name, args, context, scope, values);
    }
    // `opcache_get_configuration` is prelude-provided on the native side (not a
    // catalog builtin), so eval dispatches it as a plain runtime handler rather than
    // through the PHP-visible builtin registry, keeping the two builtin sets in sync.
    if name == "opcache_get_configuration" {
        return eval_opcache_get_configuration_call(args, context, scope, values);
    }
    // `opcache_reset` is likewise prelude-provided on native; eval returns the CLI
    // default cache-enabled boolean (false) as a plain runtime handler.
    if name == "opcache_reset" {
        return eval_opcache_reset_call(args, context, scope, values);
    }
    // `opcache_get_status` is likewise prelude-provided on native; eval reports the CLI
    // default (cache disabled) and so returns `false` as a plain runtime handler.
    if name == "opcache_get_status" {
        return eval_opcache_get_status_call(args, context, scope, values);
    }
    // The five OPcache file/script functions are likewise prelude-provided on native; eval
    // reports the CLI default (cache disabled) and returns `false` from each, except the
    // `void` `opcache_jit_blacklist`, which evaluates to `NULL`. See
    // `network_env::opcache_file_functions`.
    if name == "opcache_is_script_cached" {
        return eval_opcache_is_script_cached_call(args, context, scope, values);
    }
    if name == "opcache_invalidate" {
        return eval_opcache_invalidate_call(args, context, scope, values);
    }
    if name == "opcache_compile_file" {
        return eval_opcache_compile_file_call(args, context, scope, values);
    }
    if name == "opcache_is_script_cached_in_file_cache" {
        return eval_opcache_is_script_cached_in_file_cache_call(args, context, scope, values);
    }
    if name == "opcache_jit_blacklist" {
        return eval_opcache_jit_blacklist_call(args, context, scope, values);
    }
    if let Some(result) = eval_date_procedural_alias_call(name, args, context, scope, values)? {
        return Ok(result);
    }
    if name == "stream_select" {
        return eval_builtin_stream_select_call(args, context, scope, values);
    }
    if name == "stream_socket_accept" {
        return eval_builtin_stream_socket_accept_call(args, context, scope, values);
    }
    if name == "stream_socket_recvfrom" {
        return eval_builtin_stream_socket_recvfrom_call(args, context, scope, values);
    }
    if name == "parse_str" {
        return eval_builtin_parse_str_call(args, context, scope, values);
    }
    if matches!(
        name,
        "array_pop"
            | "array_push"
            | "array_shift"
            | "array_splice"
            | "array_unshift"
            | "array_walk"
            | "arsort"
            | "asort"
            | "end"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "next"
            | "prev"
            | "reset"
            | "rsort"
            | "shuffle"
            | "sort"
            | "settype"
            | "uasort"
            | "uksort"
            | "usort"
    ) {
        return eval_builtin_array_mutating_declared_call(name, args, context, scope, values);
    }
    if eval_php_visible_builtin_exists(name) {
        if eval_call_args_are_plain_positional(args) {
            let args = positional_call_arg_exprs(args)?;
            return eval_positional_expr_call(name, &args, context, scope, values);
        }
        return eval_builtin_call(name, args, context, scope, values);
    }

    // Answered before the lookup: these are not functions anywhere, in php or here. The compiler
    // desugars them at compile time, so there is nothing to find in any table.
    if eval_is_func_args_intrinsic(name) {
        return eval_func_args_intrinsic(name, args, context, scope, values);
    }
    if let Some(function) = context.function(name).cloned() {
        return eval_dynamic_function(&function, args, context, scope, values);
    }
    if let Some(function) = context.native_function(name) {
        return eval_native_function(function, args, context, scope, values);
    }
    // Last: a function declared by an include that ran inside ANOTHER eval context. Composer's
    // `files` autoload requires its shims from inside a bound closure, and every compiled frame
    // that includes gets its own context, so `trigger_deprecation()` lives in one this call has
    // never seen. See `ElephcEvalContext::adopt_global_function`.
    if context.adopt_global_function(name) {
        if let Some(function) = context.function(name).cloned() {
            return eval_dynamic_function(&function, args, context, scope, values);
        }
    }
    note_eval_runtime_failure(format!("call to undefined function {name}()"), context);
    Err(EvalStatus::UnsupportedConstruct)
}

/// Evaluates an unqualified namespaced function call with PHP's global fallback.
pub(in crate::interpreter) fn eval_namespaced_call(
    name: &str,
    fallback_name: &str,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if let Some(function) = context.function(name).cloned() {
        return eval_dynamic_function(&function, args, context, scope, values);
    }
    if let Some(function) = context.native_function(name) {
        return eval_native_function(function, args, context, scope, values);
    }
    eval_call(fallback_name, args, context, scope, values)
}

/// Evaluates a variable or expression callable and dispatches it with source-order arguments.
pub(in crate::interpreter) fn eval_dynamic_call(
    callee: &EvalExpr,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let callback = eval_expr(callee, context, scope, values)?;
    let callback_tag = values.type_tag(callback)?;
    if crate::eval_trace::enabled() {
        eprintln!("[elephc-eval-trace] phase=dynamic_call callback_tag={callback_tag}");
    }
    if callback_tag == EVAL_TAG_CALLABLE {
        let descriptor = values.raw_value_word(callback)? as usize as *mut std::ffi::c_void;
        if crate::eval_trace::enabled() {
            let word0 = (descriptor as usize >= 4096)
                .then(|| unsafe { *(descriptor as *const u64) });
            eprintln!(
                "[elephc-eval-trace] phase=dynamic_call descriptor={descriptor:p} word0={word0:?}",
            );
        }
        if let Some(decoded) = unsafe { decode_callable_descriptor(descriptor) } {
            if crate::eval_trace::enabled() {
                eprintln!("[elephc-eval-trace] phase=dynamic_call descriptor=direct");
            }
            return eval_native_function(decoded.function, args, context, scope, values);
        }
        if let Some((captured_context, captured)) =
            unsafe { decode_eval_callback_adapter_capture(descriptor.cast()) }
        {
            let captured = if captured.type_tag == EVAL_TAG_MIXED {
                RuntimeCellHandle::from_raw(
                    captured.value_word as usize as *mut crate::value::RuntimeCell,
                )
            } else {
                values.raw_word_value(captured.type_tag, captured.value_word)?
            };
            let captured_context = unsafe {
                (captured_context as usize as *mut ElephcEvalContext).as_mut()
            }
            .ok_or(EvalStatus::RuntimeFatal)?;
            let callback = eval_callable(captured, captured_context, values)?;
            let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
            return eval_evaluated_callable_with_call_array_args(
                &callback,
                evaluated_args,
                captured_context,
                values,
            );
        }
        return Err(EvalStatus::UnsupportedConstruct);
    }
    if callback_tag == EVAL_TAG_OBJECT {
        let is_closure_object = values
            .object_identity(callback)
            .ok()
            .and_then(|identity| context.closure_object_target(identity))
            .is_some();
        let is_detached_pcntl_handler = context
            .pcntl_foreign_callable_owner(callback)
            .is_some()
            || crate::context::pcntl_runtime::is_handler_callable(callback);
        if !is_closure_object && !is_detached_pcntl_handler {
            eval_invokable_object_precheck(callback, context, values)?;
            let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
            return eval_invokable_object_call_result(callback, evaluated_args, context, values);
        }
    }
    let callback = eval_callable(callback, context, values)?;
    let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
    eval_evaluated_callable_with_call_array_args(&callback, evaluated_args, context, values)
}

/// Returns true for language constructs that need unevaluated argument expressions.
pub(in crate::interpreter) fn eval_expr_language_construct_name(name: &str) -> bool {
    matches!(name, "empty" | "eval" | "isset" | "unset")
}

/// Returns true when every source argument is plain positional.
pub(in crate::interpreter) fn eval_call_args_are_plain_positional(args: &[EvalCallArg]) -> bool {
    args.iter()
        .all(|arg| arg.name().is_none() && !arg.is_spread())
}

/// Evaluates registry-backed direct builtins and language constructs after positional-only validation.
pub(in crate::interpreter) fn eval_positional_expr_call(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if name == "eval" {
        return eval_nested_eval(args, context, scope, values);
    }

    if let Some(result) = eval_declared_builtin_direct_call(name, args, context, scope, values)? {
        return Ok(result);
    }

    note_eval_runtime_failure(
        format!("call to unsupported builtin {name}()"),
        context,
    );
    Err(EvalStatus::UnsupportedConstruct)
}
