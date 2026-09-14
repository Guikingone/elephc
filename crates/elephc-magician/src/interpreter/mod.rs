//! Purpose:
//! Interprets EvalIR against a materialized caller scope.
//! The interpreter is generic over runtime value operations so it can execute
//! by manipulating opaque elephc runtime-cell handles.
//!
//! Called from:
//! - Future `crate::__elephc_eval_execute()` implementation.
//! - `cargo test -p elephc-magician` for scope/value-flow validation.
//!
//! Key details:
//! - This module does not own PHP values. Constants and operations are delegated
//!   to `RuntimeValueOps`, which will be backed by elephc runtime hooks.

mod array_literals;
pub mod builtin_metadata;
mod builtin_interfaces;
mod builtins;
mod constant_eval;
mod constants;
mod control;
mod dynamic_functions;
mod expressions;
mod generators;
mod globals;
mod include_exec;
mod libc_shims;
mod reflection;
mod return_type_compat;
mod return_values;
mod runtime_ops;
mod scope_cells;
mod statements;
#[cfg(not(test))]
mod output_handlers;
#[cfg(not(test))]
mod pcntl_escape;
mod throwables;

use crate::context::{
    decode_callable_descriptor, eval_builtin_is_backtrace_visible, ElephcEvalContext,
    ElephcEvalExecutionScope, EvalArrayCursor, EvalArrayReferenceKey, EvalCallFrame,
    EvalCallFrameKind, EvalClosure, EvalClosureCaptureBinding, EvalClosureObjectTarget,
    EvalReferenceTarget, NativeCallableDefault, NativeCallableSignature, NativeFunction,
};
use crate::errors::{report_fatal_diagnostic, EvalParseDiagnostic, EvalStatus};
use crate::eval_ir::{
    EvalArrayElement, EvalAttribute, EvalAttributeArg, EvalBinOp, EvalCallArg, EvalCatch,
    EvalCastType, EvalClass, EvalClassConstant, EvalClassMethod, EvalClassProperty, EvalConst,
    EvalDestructureSlot, EvalDestructureTarget,
    EvalEnum, EvalEnumBackingType, EvalEnumCase, EvalExpr, EvalFunction, EvalInstanceOfTarget,
    EvalInterface, EvalInterfaceMethod, EvalInterfaceProperty, EvalMagicConst, EvalMatchArm,
    EvalParameterType, EvalParameterTypeVariant, EvalProgram, EvalStmt, EvalSwitchCase, EvalTrait,
    EvalTraitAdaptation, EvalUnaryOp, EvalVisibility,
};
#[cfg(test)]
use crate::parser::parse_fragment;
use crate::scope::{ElephcEvalScope, ScopeCellOwnership, ScopeEntry};
use crate::value::RuntimeCellHandle;
use array_literals::*;
use builtin_interfaces::*;
use builtins::*;
use constant_eval::*;
use constants::*;
pub use control::EvalOutcome;
use control::{
    BoundMethodArg, BoundNativeFunctionArgs, BoundNativeFunctionRefSlot, EvalArraySpliceDirectArgs,
    EvalByRefBindingMode, EvalControl, EvalPredefinedConstant, EvalSprintfSpec,
    EvaluatedCallArg, EvaluatedCallable,
};
use dynamic_functions::*;
use expressions::*;
use generators::*;
use globals::*;
use include_exec::*;
use libc_shims::*;
use reflection::*;
use return_type_compat::*;
use return_values::*;
pub use runtime_ops::RuntimeValueOps;
pub(crate) use builtins::eval_spl_autoload_class as eval_spl_autoload_class_bridge;
pub(crate) use builtins::eval_spl_autoload_classlike_definition;
use runtime_ops::*;
#[cfg(not(test))]
pub(crate) use pcntl_escape::value_contains_foreign_pcntl_callable;
use scope_cells::*;
#[cfg(not(test))]
pub(crate) use statements::eval_dynamic_destructor_for_object_cell;
#[cfg(not(test))]
pub(crate) use generators::eval_generator_protocol_result;
#[cfg(not(test))]
pub(crate) use builtins::eval_unserialize_declared_object_from_hash;
#[cfg(not(test))]
pub(crate) use builtins::eval_serialize_object_fragment;
#[cfg(not(test))]
pub(crate) use output_handlers::eval_ob_handler_callback;
use statements::*;
use throwables::*;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::net::ToSocketAddrs;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

/// Registers one AOT-provided SPL callback through the interpreter's shared callback semantics.
pub(crate) fn register_runtime_spl_autoload_callback(
    callback: RuntimeCellHandle,
    prepend: bool,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    register_spl_autoload_callback_unchecked(callback, prepend, context, values)
}

/// Executes an EvalIR program and returns the eval result cell.
pub fn execute_program(
    program: &EvalProgram,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut context = ElephcEvalContext::new();
    execute_program_with_context(&mut context, program, scope, values)
}

/// Executes an EvalIR program with a persistent eval context for dynamic declarations.
pub fn execute_program_with_context(
    context: &mut ElephcEvalContext,
    program: &EvalProgram,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match execute_program_outcome_with_context(context, program, scope, values)? {
        EvalOutcome::Value(result) => Ok(result),
        EvalOutcome::Throwable(error) => {
            context.set_pending_throw(error);
            Err(EvalStatus::UncaughtThrowable)
        }
    }
}

/// Executes an EvalIR program and preserves escaping Throwable cells.
pub fn execute_program_outcome_with_context(
    context: &mut ElephcEvalContext,
    program: &EvalProgram,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match execute_statements(program.statements(), context, scope, values) {
        Ok(EvalControl::None | EvalControl::ReturnVoid) => values.null().map(EvalOutcome::Value),
        Ok(EvalControl::Return(result)) => Ok(EvalOutcome::Value(result)),
        Ok(EvalControl::Throw(result)) => Ok(EvalOutcome::Throwable(result)),
        Ok(EvalControl::Break(_) | EvalControl::Continue(_) | EvalControl::Goto(_)) => {
            Err(EvalStatus::UnsupportedConstruct)
        }
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Executes an already materialized runtime include and preserves escaping Throwable cells.
pub fn execute_include_outcome_with_context(
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    path: RuntimeCellHandle,
    required: bool,
    once: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match include_exec::eval_include_value(path, required, once, context, scope, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Executes a zero-argument function declared in the shared eval context.
pub fn execute_context_function_zero_args(
    context: &mut ElephcEvalContext,
    name: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    execute_context_function(context, name, Vec::new(), values)
}

/// Executes a function declared in the shared eval context with prepared argument cells.
pub fn execute_context_function(
    context: &mut ElephcEvalContext,
    name: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match execute_context_function_outcome(context, name, args, values)? {
        EvalOutcome::Value(result) => Ok(result),
        EvalOutcome::Throwable(error) => {
            context.set_pending_throw(error);
            Err(EvalStatus::UncaughtThrowable)
        }
    }
}

/// Executes a function declared in the shared eval context and preserves thrown cells.
pub fn execute_context_function_outcome(
    context: &mut ElephcEvalContext,
    name: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    let Some(function) = context.function(name).cloned() else {
        // A name this context does not declare can still be one the bridge answers itself: the
        // two backtrace functions, the OPcache family and the procedural date aliases are
        // runtime handlers rather than PHP-visible builtins. `eval_context_function_exists()`
        // already reports exactly that set, so answering the CALL from the same table is what
        // stops the two from disagreeing. They did: an unqualified `debug_backtrace()` written
        // inside a namespace arrives here as PHP's global fallback, and refusing it turned the
        // call into `Call to undefined function <namespace>\\debug_backtrace()`.
        return match eval_builtin_with_values(name, &args, context, values)? {
            Some(result) => Ok(EvalOutcome::Value(result)),
            None => Err(EvalStatus::UnsupportedConstruct),
        };
    };
    match eval_dynamic_function_with_values(&function, args, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Reports whether the bridge can answer a call to this function name by itself.
///
/// This is the predicate `function_exists()` answers with AND the set
/// `execute_context_function_outcome()` dispatches through, deliberately the same one: a name this
/// reports callable is a name that call resolves, so the existence answer and the call answer
/// cannot drift apart the way they had.
pub fn context_function_is_callable(context: &ElephcEvalContext, name: &str) -> bool {
    eval_function_probe_exists(context, name)
}

/// Executes a named eval-context callable with arguments from a PHP array container.
pub fn execute_context_function_call_array(
    context: &mut ElephcEvalContext,
    name: &str,
    arg_array: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match execute_context_function_call_array_outcome(context, name, arg_array, values)? {
        EvalOutcome::Value(result) => Ok(result),
        EvalOutcome::Throwable(error) => {
            context.set_pending_throw(error);
            Err(EvalStatus::UncaughtThrowable)
        }
    }
}

/// Executes a named eval-context callable from an argument array and preserves thrown cells.
pub fn execute_context_function_call_array_outcome(
    context: &mut ElephcEvalContext,
    name: &str,
    arg_array: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    if !values.is_array_like(arg_array)? {
        return Err(EvalStatus::RuntimeFatal);
    }
    let evaluated_args = eval_array_call_arg_values(arg_array, context, values)?;
    match eval_callable_with_call_array_args(name, evaluated_args, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Executes a callback value with a prepared argument array in the shared eval context.
pub fn execute_context_callable_call_array_outcome(
    context: &mut ElephcEvalContext,
    callback: RuntimeCellHandle,
    arg_array: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match eval_call_user_func_array_with_values(callback, arg_array, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Probes whether a callback value is callable in the shared eval context.
pub fn execute_context_is_callable(
    context: &ElephcEvalContext,
    callback: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    eval_is_callable_value(callback, None, context, values)
}

/// Constructs a class declared in the shared eval context with prepared positional arguments.
///
/// THE TERMINAL DECISION for `new` through the bridge. Its `try_` half answers `None` when no
/// route could construct the name — not AOT metadata, not this context, not an autoloader — and
/// that `None` used to become `EvalStatus::UnsupportedConstruct`, printed as
/// `Fatal error: eval() fragment uses an unsupported construct`. That wording is wrong for both
/// of the two things it covered, and they are not the same thing:
///
/// A name PHP itself provides as a builtin class is a GAP IN THIS BUILD. PHP would have
/// constructed the object, so the message has to say so in elephc's own voice rather than blame
/// the program. The catalog is `reflection::class_lookup::eval_reflection_class_like_is_internal`,
/// the predicate `ReflectionClass::isInternal()` already answers from, so the two cannot drift.
///
/// Any other name is genuinely undefined, and PHP's answer is a CATCHABLE `Error` reading
/// `Class "X" not found`. Measured with `php -n` 8.5.6: a runtime include followed by
/// `try { new NoSuchClassAnywhere(); } catch (\Throwable $e) { … }` prints
/// `loaded;Error: Class "NoSuchClassAnywhere" not found;done` and exits 0. Reporting a fatal
/// there took the `catch` away, and it matters more now that the checker DEFERS unknown class
/// names for programs that include PHP at run time — deferring a compile-time diagnostic is only
/// honest if the runtime raises the right thing, in a form user code can receive.
///
/// The throwable is handed back as an `EvalOutcome::Throwable`, the same channel the arms inside
/// `execute_context_try_new_object_outcome` already use, so the AOT side unwinds into the
/// caller's `catch` instead of aborting.
pub fn execute_context_new_object_outcome(
    context: &mut ElephcEvalContext,
    name: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    if let Some(outcome) = execute_context_try_new_object_outcome(context, name, args, values)? {
        return Ok(outcome);
    }
    if eval_reflection_class_like_is_internal(name) {
        note_eval_runtime_failure(
            format!(
                "builtin class \"{}\" is not available in this build",
                name.trim_start_matches('\\')
            ),
            context,
        );
        return Err(EvalStatus::RuntimeFatal);
    }
    match eval_throw_class_not_found_error::<()>(name, context, values) {
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
        Ok(()) => Err(EvalStatus::RuntimeFatal),
    }
}

/// Constructs a runtime Reflection owner from prepared positional arguments.
pub fn execute_reflection_new_object_outcome(
    context: &mut ElephcEvalContext,
    name: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    let evaluated_args = args
        .into_iter()
        .map(|value| EvaluatedCallArg {
            name: None,
            value,
            ref_target: None,
            owned: false,
        })
        .collect();
    match eval_reflection_owner_new_object(name, evaluated_args, context, values) {
        Ok(Some(result)) => Ok(EvalOutcome::Value(result)),
        Ok(None) => Err(EvalStatus::UnsupportedConstruct),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Attempts to construct a runtime-owned built-in or eval-declared class.
/// Returns `None` when neither runtime surface recognizes the class name.
pub fn execute_context_try_new_object_outcome(
    context: &mut ElephcEvalContext,
    name: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalOutcome>, EvalStatus> {
    let evaluated_args = args
        .into_iter()
        .map(|value| EvaluatedCallArg {
            name: None,
            value,
            ref_target: None,
            owned: false,
        })
        .collect::<Vec<_>>();
    match eval_reflection_owner_new_object(name, evaluated_args.clone(), context, values) {
        Ok(Some(result)) => return Ok(Some(EvalOutcome::Value(result))),
        Ok(None) => {}
        Err(EvalStatus::UncaughtThrowable) => {
            return context
                .take_pending_throw()
                .map(EvalOutcome::Throwable)
                .map(Some)
                .ok_or(EvalStatus::UncaughtThrowable);
        }
        Err(status) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=try_new_object_error stage=reflection class={name:?} status={status:?}",
                );
            }
            return Err(status);
        }
    }
    let class_name = name.trim_start_matches('\\');
    if values.class_exists(class_name)? {
        let mut scope = ElephcEvalScope::new();
        return match eval_new_object_result(
            class_name,
            evaluated_args,
            context,
            &mut scope,
            values,
        ) {
            Ok(result) => Ok(Some(EvalOutcome::Value(result))),
            Err(EvalStatus::UncaughtThrowable) => context
                .take_pending_throw()
                .map(EvalOutcome::Throwable)
                .map(Some)
                .ok_or(EvalStatus::UncaughtThrowable),
            Err(status) => {
                if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                    eprintln!(
                        "[elephc-eval-trace] phase=try_new_object_error stage=aot_class class={class_name:?} status={status:?}",
                    );
                }
                Err(status)
            }
        };
    }
    if !context.has_class(class_name) {
        if let Err(status) = eval_spl_autoload_classlike_definition(class_name, context, values) {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=try_new_object_error stage=autoload class={class_name:?} status={status:?}",
                );
            }
            return Err(status);
        }
    }
    let Some(class) = context.class(class_name).cloned() else {
        return Ok(None);
    };
    let mut scope = ElephcEvalScope::new();
    match eval_dynamic_class_new_object(&class, evaluated_args, context, &mut scope, values) {
        Ok(result) => Ok(Some(EvalOutcome::Value(result))),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .map(Some)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => {
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                eprintln!(
                    "[elephc-eval-trace] phase=try_new_object_error stage=dynamic_class class={class_name:?} status={status:?}",
                );
            }
            Err(status)
        }
    }
}

/// Calls a method on a value that may be an eval-created object.
pub fn execute_context_method_call_outcome(
    context: &mut ElephcEvalContext,
    object: RuntimeCellHandle,
    method: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    let evaluated_args = eval_bridge_positional_args(args, values)?;
    match eval_method_call_result_with_evaluated_args(object, method, evaluated_args, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Reads an instance property through the active eval context.
pub fn execute_context_property_get_outcome(
    context: &mut ElephcEvalContext,
    object: RuntimeCellHandle,
    property: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match eval_property_get_result(object, property, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Converts one boxed value through eval's PHP string-context semantics.
///
/// Dynamic objects dispatch their eval-declared `__toString()` implementation, while an
/// exception raised by that method is returned as an uncaught throwable outcome for the C ABI.
pub fn execute_context_string_outcome(
    context: &mut ElephcEvalContext,
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match eval_string_context_value(value, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Calls a static method on a class-like symbol known to the shared eval context.
pub fn execute_context_static_method_call_outcome(
    context: &mut ElephcEvalContext,
    class_name: &str,
    method: &str,
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    let evaluated_args = eval_bridge_positional_args(args, values)?;
    match eval_static_method_call_result(class_name, method, evaluated_args, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Decodes native bridge reference markers into positional eval call arguments.
fn eval_bridge_positional_args(
    args: Vec<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<Vec<EvaluatedCallArg>, EvalStatus> {
    args.into_iter()
        .map(|value| {
            let (value, ref_target) = eval_invoker_ref_arg_value_and_target(value, None, values)?;
            Ok(EvaluatedCallArg {
                name: None,
                value,
                ref_target,
                owned: false,
            })
        })
        .collect()
}

/// Resolves object class-name builtins against eval dynamic-object metadata first.
pub fn execute_context_object_class_name(
    context: &mut ElephcEvalContext,
    lookup: &str,
    object_or_class: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match lookup {
        "get_class" => eval_get_class_result(object_or_class, context, values),
        "get_parent_class" => eval_get_parent_class_result(object_or_class, context, values),
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Resolves class/interface/trait relation metadata through eval dynamic metadata.
pub fn execute_context_class_relation(
    context: &mut ElephcEvalContext,
    name: &str,
    target: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_class_relation_result(name, &[target], context, values)
}

/// Fetches a class-like constant through eval dynamic metadata and runtime fallback hooks.
pub fn execute_context_class_constant_fetch(
    context: &mut ElephcEvalContext,
    class_name: &str,
    constant_name: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match eval_class_constant_fetch_result(class_name, constant_name, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Reads a static property through eval dynamic metadata and runtime fallback hooks.
pub fn execute_context_static_property_get(
    context: &mut ElephcEvalContext,
    class_name: &str,
    property_name: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalOutcome, EvalStatus> {
    match eval_static_property_get_result(class_name, property_name, context, values) {
        Ok(result) => Ok(EvalOutcome::Value(result)),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Writes a static property through eval dynamic metadata and runtime fallback hooks.
pub fn execute_context_static_property_set(
    context: &mut ElephcEvalContext,
    class_name: &str,
    property_name: &str,
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalOutcome>, EvalStatus> {
    match eval_static_property_set_result(class_name, property_name, value, context, values) {
        Ok(()) => Ok(None),
        Err(EvalStatus::UncaughtThrowable) => context
            .take_pending_throw()
            .map(EvalOutcome::Throwable)
            .map(Some)
            .ok_or(EvalStatus::UncaughtThrowable),
        Err(status) => Err(status),
    }
}

/// Tests an object relation against eval dynamic-object metadata before AOT metadata.
pub fn execute_context_object_is_a(
    context: &mut ElephcEvalContext,
    object: RuntimeCellHandle,
    target_class: &str,
    exclude_self: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    if values.type_tag(object)? != EVAL_TAG_OBJECT {
        return Ok(false);
    }
    let target_class = target_class.trim_start_matches('\\');
    let resolved_target_class = context
        .resolve_class_like_name(target_class)
        .unwrap_or_else(|| target_class.to_string());
    dynamic_object_is_a(
        object,
        &resolved_target_class,
        exclude_self,
        context,
        values,
    )?
    .map_or_else(
        || values.object_is_a(object, &resolved_target_class, exclude_self),
        Ok,
    )
}

/// Tests an object relation when the target is a runtime string or object cell.
pub fn execute_context_object_is_a_dynamic(
    context: &mut ElephcEvalContext,
    object: RuntimeCellHandle,
    target: RuntimeCellHandle,
    exclude_self: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    let target_class = match values.type_tag(target)? {
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(target)?;
            let target = String::from_utf8(bytes).map_err(|_| EvalStatus::RuntimeFatal)?;
            target.trim_start_matches('\\').to_string()
        }
        EVAL_TAG_OBJECT => {
            let identity = values.object_identity(target)?;
            if let Some((_, class)) = context.dynamic_object_declaring_class(identity) {
                class.name().to_string()
            } else {
                let class_name = values.object_class_name(target)?;
                let bytes = values.string_bytes(class_name);
                values.release(class_name)?;
                let class_name = String::from_utf8(bytes?).map_err(|_| EvalStatus::RuntimeFatal)?;
                class_name.trim_start_matches('\\').to_string()
            }
        }
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    execute_context_object_is_a(context, object, &target_class, exclude_self, values)
}

/// Tests whether a method or property exists through eval dynamic metadata.
pub fn execute_context_member_exists(
    context: &mut ElephcEvalContext,
    name: &str,
    target: RuntimeCellHandle,
    member: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    let result = eval_member_exists_result(name, &[target, member], context, values)?;
    let exists = values.truthy(result)?;
    values.release(result)?;
    Ok(exists)
}

/// Returns the current interpreter availability status for the ABI stub.
pub fn current_stub_status() -> EvalStatus {
    EvalStatus::UnsupportedConstruct
}

#[cfg(test)]
mod tests;
