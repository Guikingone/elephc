//! Purpose:
//! Eval registry entry and implementation for `spl_autoload`.
//!
//! Called from:
//! - `crate::interpreter::builtins::symbols`.
//!
//! Key details:
//! - Class resolution drives the registered callback table in PHP invocation order.

eval_builtin! {
    contract: "spl_autoload",
    area: Symbols,
    direct: Symbols,
    values: Symbols,
}

use super::super::super::*;

/// Evaluates direct `spl_autoload(...)` calls against the registered callback table.
pub(in crate::interpreter) fn eval_spl_autoload_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_spl_autoload_void("spl_autoload", args, context, scope, values)
}

/// Evaluates materialized `spl_autoload(...)` arguments against the registered callback table.
pub(in crate::interpreter) fn eval_spl_autoload_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_spl_autoload_void_result("spl_autoload", evaluated_args, context, values)
}

/// Evaluates direct SPL autoload calls after validating their arity and source order.
pub(in crate::interpreter) fn eval_builtin_spl_autoload_void(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (class, extensions) = match name {
        "spl_autoload_call" => match args {
            [class] => (class, None),
            _ => return Err(EvalStatus::RuntimeFatal),
        },
        "spl_autoload" => match args {
            [class] => (class, None),
            [class, extensions] => (class, Some(extensions)),
            _ => return Err(EvalStatus::RuntimeFatal),
        },
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    let class = eval_expr(class, context, scope, values)?;
    if let Some(extensions) = extensions {
        let _ = eval_expr(extensions, context, scope, values)?;
    }
    eval_spl_autoload_void_result(name, &[class], context, values)
}

/// Evaluates materialized SPL autoload calls.
pub(in crate::interpreter) fn eval_spl_autoload_void_result(
    name: &str,
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "spl_autoload_call" if evaluated_args.len() == 1 => {
            eval_spl_autoload_class_value(evaluated_args[0], context, values)?;
            values.null()
        }
        "spl_autoload" if (1..=2).contains(&evaluated_args.len()) => {
            eval_spl_autoload_class_value(evaluated_args[0], context, values)?;
            values.null()
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Invokes registered callbacks for one unresolved class and reports whether a class became visible.
pub(crate) fn eval_spl_autoload_class(
    class_name: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    let class_name = class_name.trim_start_matches('\\');
    #[cfg(not(test))]
    context.sync_global_eval_classes();
    if eval_autoload_target_exists(class_name, context, values)? {
        return Ok(true);
    }
    if eval_spl_autoload_class_local(class_name, context, values)? {
        return Ok(true);
    }
    #[cfg(not(test))]
    for owner in crate::context::global_eval_autoload_contexts_snapshot() {
        if std::ptr::eq(owner, context) {
            continue;
        }
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!("[elephc-eval-trace] phase=spl_autoload_owner class={class_name:?} owner={owner:p}");
        }
        let owner_loaded = unsafe { eval_spl_autoload_class_in_owner(owner, class_name) }?;
        context.sync_global_eval_classes();
        if !context.has_class(class_name) && owner_loaded {
            let loaded_class = unsafe {
                owner
                    .as_ref()
                    .and_then(|owner| owner.class(class_name).cloned())
            };
            if let Some(loaded_class) = loaded_class {
                context.define_class(loaded_class);
            }
        }
        if eval_autoload_target_exists(class_name, context, values)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Loads a class-like declaration body needed by dynamic composition even when AOT metadata exists.
///
/// Generated metadata can report an AOT trait as existing while a dynamically included class still
/// needs the trait's EvalIR methods for `use Trait` composition. This path therefore checks only
/// context-owned declarations before invoking the ordinary SPL callback chain and synchronizing
/// the dynamic declaration registry.
pub(crate) fn eval_spl_autoload_classlike_definition(
    class_name: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    let class_name = class_name.trim_start_matches('\\');
    #[cfg(not(test))]
    context.sync_global_eval_classes();
    if context.has_class(class_name)
        || context.has_interface(class_name)
        || context.has_trait(class_name)
        || context.has_enum(class_name)
    {
        return Ok(true);
    }
    let _ = eval_spl_autoload_class_local(class_name, context, values)?;
    if context.has_class(class_name)
        || context.has_interface(class_name)
        || context.has_trait(class_name)
        || context.has_enum(class_name)
    {
        return Ok(true);
    }
    #[cfg(not(test))]
    for owner in crate::context::global_eval_autoload_contexts_snapshot() {
        if std::ptr::eq(owner, context) {
            continue;
        }
        let _ = unsafe { eval_spl_autoload_class_in_owner(owner, class_name) }?;
        context.sync_global_eval_classes();
        if context.has_class(class_name)
            || context.has_interface(class_name)
            || context.has_trait(class_name)
            || context.has_enum(class_name)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Runs callbacks registered in exactly one context while preserving recursion protection.
fn eval_spl_autoload_class_local(
    class_name: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    if !context.begin_autoload_class(class_name) {
        return Ok(false);
    }
    let class = values.string(class_name)?;
    let callbacks = context.autoload_callbacks();
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        eprintln!("[elephc-eval-trace] phase=spl_autoload_local class={class_name:?} callbacks={}", callbacks.len());
    }
    let result = (|| {
        for callback in callbacks {
            let callback = eval_callable(callback, context, values).map_err(|status| {
                if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                    eprintln!("[elephc-eval-trace] phase=spl_autoload_callback stage=normalize status={status:?}");
                }
                status
            })?;
            if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                let kind = match &callback {
                    EvaluatedCallable::Named { name, .. } => format!("named:{name}"),
                    EvaluatedCallable::BoundClosure { name, .. } => format!("closure:{name}"),
                    EvaluatedCallable::InvokableObject { .. } => "invokable-object".to_string(),
                    EvaluatedCallable::ObjectMethod { method, native_class, .. } => {
                        format!("object-method:{native_class:?}::{method}")
                    }
                    EvaluatedCallable::StaticMethod { class_name, method, native_class, .. } => {
                        format!("static-method:{class_name}:{native_class:?}::{method}")
                    }
                };
                eprintln!("[elephc-eval-trace] phase=spl_autoload_callback stage=normalized kind={kind}");
            }
            let result = eval_evaluated_callable_with_values(&callback, vec![class], context, values)
                .map_err(|status| {
                    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                        eprintln!("[elephc-eval-trace] phase=spl_autoload_callback stage=invoke status={status:?}");
                    }
                    status
                })?;
            eval_release_value(context, values, result).map_err(|status| {
                if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                    eprintln!(
                        "[elephc-eval-trace] phase=spl_autoload_callback stage=release status={status:?}",
                    );
                }
                status
            })?;
            #[cfg(not(test))]
            context.sync_global_eval_classes();
            if eval_autoload_target_exists(class_name, context, values)? {
                if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
                    eprintln!(
                        "[elephc-eval-trace] phase=spl_autoload_callback stage=completed class={class_name:?}",
                    );
                }
                return Ok(true);
            }
        }
        Ok(false)
    })();
    let release = eval_release_value(context, values, class);
    context.end_autoload_class(class_name);
    release?;
    result
}

/// Returns whether an SPL autoload request made any PHP class-like symbol visible.
///
/// PHP uses the same callback registry for classes, interfaces, traits, and enums. A trait must
/// therefore count as loaded before a dynamically included class can expand its `use Trait` list.
fn eval_autoload_target_exists(
    name: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    Ok(context.has_class(name)
        || context.has_interface(name)
        || context.has_trait(name)
        || context.has_enum(name)
        || values.class_exists(name)?
        || eval_runtime_interface_exists(name, values)?
        || values.trait_exists(name)?
        || values.enum_exists(name)?)
}

/// Runs one foreign context's callbacks through runtime hooks bound to that owner.
#[cfg(not(test))]
unsafe fn eval_spl_autoload_class_in_owner(
    owner: *mut ElephcEvalContext,
    class_name: &str,
) -> Result<bool, EvalStatus> {
    let Some(owner) = owner.as_mut() else {
        return Ok(false);
    };
    let mut values = crate::runtime_hooks::ElephcRuntimeOps::with_context(owner as *const _);
    eval_spl_autoload_class_local(class_name, owner, &mut values)
}

/// Reads a PHP class-name argument and invokes registered callbacks for it.
fn eval_spl_autoload_class_value(
    class_name: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    let class_name = values.string_bytes(class_name)?;
    let class_name = String::from_utf8(class_name).map_err(|_| EvalStatus::RuntimeFatal)?;
    eval_spl_autoload_class(&class_name, context, values)
}
