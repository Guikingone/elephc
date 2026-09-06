//! Purpose:
//! Evaluates EvalIR expressions, match expressions, and function-like calls.
//!
//! Called from:
//! - `crate::interpreter::statements` for expression statements and expression-bearing statements.
//! - Eval builtin modules when they need to evaluate unevaluated argument expressions.
//!
//! Key details:
//! - PHP call argument evaluation order is preserved before binding or ABI-like materialization.
//! - Language constructs such as `eval`, `isset`, `empty`, and `unset` receive unevaluated expressions.

use super::*;

mod calls;
mod null_coalesce_assign;

pub(in crate::interpreter) use calls::*;
mod evaluation;

pub(in crate::interpreter) use evaluation::{
    eval_array_access_object_matches, eval_array_get_result, eval_binary_result,
    eval_closure_object_expr, eval_dynamic_class_name, eval_dynamic_member_name, eval_match_expr,
    eval_new_object_result,
};
pub(in crate::interpreter) use evaluation::eval_object_array_cast_value;
use evaluation::*;
pub(in crate::interpreter) use null_coalesce_assign::{
    eval_array_append, eval_array_append_reference_bind, eval_array_append_result,
    eval_array_reference_bind, eval_by_ref_return, eval_reference_bind_result,
    eval_store_value_in_lvalue,
    eval_var_reference_bind,
};
use null_coalesce_assign::{
    eval_assign, eval_compound_assign, eval_null_coalesce_assign, eval_postfix_inc_dec,
};

/// Evaluates one expression to an opaque runtime-cell handle.
/// Returns whether this expression's value is an alias of storage that still owns it.
///
/// `eval_expr` normally hands the caller a reference it may keep, but four shapes give back a
/// cell some storage holds: a variable read returns the scope's own cell, and the three
/// assignment forms return the very cell they just wrote. A caller that discards such a value
/// must NOT release it — the variable, property or element is still pointing at it.
///
/// `?:` and `??` are transparent, so they inherit the answer from whichever side can be taken.
/// A postfix `++`/`--` is deliberately absent: it retains before returning the old value.
pub(in crate::interpreter) fn eval_expr_result_aliases_storage(expr: &EvalExpr) -> bool {
    match expr {
        EvalExpr::LoadVar(_)
        | EvalExpr::Assign { .. }
        | EvalExpr::ReferenceBind { .. }
        | EvalExpr::ArrayAppendAssign { .. }
        | EvalExpr::CompoundAssign { .. }
        | EvalExpr::NullCoalesceAssign { .. } => true,
        EvalExpr::Ternary {
            then_branch,
            else_branch,
            ..
        } => {
            then_branch
                .as_deref()
                .is_some_and(eval_expr_result_aliases_storage)
                || eval_expr_result_aliases_storage(else_branch)
        }
        EvalExpr::NullCoalesce { value, default } => {
            eval_expr_result_aliases_storage(value) || eval_expr_result_aliases_storage(default)
        }
        _ => false,
    }
}

/// Evaluates one expression, naming it when it fails with nothing else to say.
///
/// A quarter of php-src's language corpus used to end in a bare `Fatal error: eval() runtime
/// failed` with no phase, no name and no line -- 225 of 2556 cases, which cannot even be GROUPED,
/// let alone fixed. The descriptions are first-writer-wins and the innermost frame writes first,
/// so a path that already names itself keeps its message and only the genuinely anonymous ones
/// pick up the expression kind here.
pub(in crate::interpreter) fn eval_expr(
    expr: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let result = eval_expr_dispatch(expr, context, scope, values);
    if matches!(result, Err(EvalStatus::RuntimeFatal)) {
        note_eval_runtime_failure(
            format!("unsupported {} expression", eval_expr_kind(expr)),
            context,
        );
    }
    result
}

/// Names one expression variant for a diagnostic.
///
/// Reads the variant name off `Debug` rather than repeating sixty match arms that would drift from
/// the enum the first time somebody added a variant. Only ever reached on the failure path, which
/// is terminal, so rendering the node once costs nothing that matters.
fn eval_expr_kind(expr: &EvalExpr) -> String {
    let rendered = format!("{expr:?}");
    rendered
        .split(|ch: char| ch == '(' || ch == '{' || ch == ' ')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Evaluates one expression by variant.
fn eval_expr_dispatch(
    expr: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match expr {
        EvalExpr::Array(elements) => {
            if elements
                .iter()
                .any(|element| {
                    matches!(
                        element,
                        EvalArrayElement::KeyValue { .. }
                            | EvalArrayElement::KeyReference { .. }
                    )
                })
            {
                eval_assoc_array(elements, context, scope, values)
            } else {
                eval_indexed_array(elements, context, scope, values)
            }
        }
        // `echo $a[];` is the fatal `Cannot use [] for reading` in php, so this node only ever
        // reaches the reference-source path and never produces a value.
        EvalExpr::ArrayAppendSlot { .. } => Err(EvalStatus::UnsupportedConstruct),
        EvalExpr::ArrayGet { array, index } => {
            let array = eval_expr(array, context, scope, values)?;
            let index = eval_expr(index, context, scope, values)?;
            eval_array_get_result(array, index, context, values)
        }
        EvalExpr::ArrayDestructureAssign { targets, value } => {
            eval_array_destructure_assign(targets, value, context, scope, values)
        }
        EvalExpr::Call { name, args } => eval_call(name, args, context, scope, values),
        EvalExpr::Cast { target, expr } => eval_cast_expr(target, expr, context, scope, values),
        EvalExpr::Const(value) => eval_const(value, values),
        EvalExpr::ConstFetch(name) => eval_const_fetch(name, context, values),
        EvalExpr::Closure {
            function,
            captures,
            is_static,
        } => eval_closure_expr(function, captures, *is_static, context, scope, values),
        EvalExpr::FunctionCallable {
            name,
            fallback_name,
        } => eval_function_callable_expr(name, fallback_name.as_deref(), context, values),
        EvalExpr::InvokableCallable { object } => {
            eval_invokable_callable_expr(object, context, scope, values)
        }
        EvalExpr::MethodCallable { object, method } => {
            eval_method_callable_expr(object, method, context, scope, values)
        }
        EvalExpr::StaticMethodCallable { class_name, method } => {
            eval_static_method_callable_expr(class_name, method, context, scope, values)
        }
        EvalExpr::DynamicStaticMethodCallable { class_name, method } => {
            eval_dynamic_static_method_callable_expr(class_name, method, context, scope, values)
        }
        EvalExpr::DynamicCall { callee, args } => {
            eval_dynamic_call(callee, args, context, scope, values)
        }
        EvalExpr::DynamicMethodCall {
            object,
            method,
            args,
        } => {
            let receiver_is_temporary = eval_method_receiver_is_temporary(object);
            let object = eval_expr(object, context, scope, values)?;
            let result = (|| {
                let method = eval_dynamic_member_name(method, context, scope, values)?;
                let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
                eval_method_call_result_with_evaluated_args(
                    object,
                    &method,
                    evaluated_args,
                    context,
                    values,
                )
            })();
            eval_method_call_with_temporary_receiver_cleanup(
                object,
                receiver_is_temporary,
                result,
                context,
                values,
            )
        }
        EvalExpr::DynamicNewObject { class_name, args } => {
            let class_name = eval_expr(class_name, context, scope, values).map_err(|status| {
                trace_dynamic_new_error("class_expression", None, status, context)
            })?;
            let class_name = eval_dynamic_class_name(class_name, context, values).map_err(|status| {
                trace_dynamic_new_error("class_name", None, status, context)
            })?;
            let args = eval_method_call_arg_values(args, context, scope, values).map_err(|status| {
                trace_dynamic_new_error("arguments", Some(&class_name), status, context)
            })?;
            eval_new_object_result(&class_name, args, context, scope, values).map_err(|status| {
                trace_dynamic_new_error("construction", Some(&class_name), status, context)
            })
        }
        EvalExpr::DynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            eval_property_get_result(object, &property, context, values)
        }
        EvalExpr::DynamicStaticMethodCall {
            class_name,
            method,
            args,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let method = eval_dynamic_member_name(method, context, scope, values)?;
            let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
            eval_static_method_call_result_from_scope(
                &class_name,
                &method,
                evaluated_args,
                scope,
                context,
                values,
            )
        }
        EvalExpr::DynamicStaticPropertyGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            eval_static_property_get_result(&class_name, property, context, values)
        }
        EvalExpr::DynamicStaticPropertyNameGet {
            class_name,
            property,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            eval_static_property_get_result(&class_name, &property, context, values)
        }
        EvalExpr::DynamicClassConstantFetch {
            class_name,
            constant,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            eval_class_constant_fetch_result(&class_name, constant, context, values)
        }
        EvalExpr::DynamicClassConstantNameFetch {
            class_name,
            constant,
        } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            let class_name = eval_dynamic_class_name(class_name, context, values)?;
            let constant = eval_dynamic_member_name(constant, context, scope, values)?;
            eval_class_constant_fetch_result(&class_name, &constant, context, values)
        }
        EvalExpr::DynamicClassNameFetch { class_name } => {
            let class_name = eval_expr(class_name, context, scope, values)?;
            eval_dynamic_class_name_fetch_result(class_name, context, values)
        }
        EvalExpr::Include {
            path,
            required,
            once,
        } => eval_include_expr(path, *required, *once, context, scope, values),
        EvalExpr::InstanceOf { value, target } => {
            eval_instanceof_expr(value, target, context, scope, values)
        }
        EvalExpr::LoadVar(name) => match visible_scope_cell(context, scope, name) {
            Some(cell) => Ok(cell),
            None => {
                // php: `Warning: Undefined variable $x`, and the value is still null -- an unset
                // variable is a WARNING, never an error. 18 cases in the php-src sweep printed
                // the right value and no diagnostic at all.
                //
                // `@` already suppresses through `errors_suppressed()`. `??` and `isset()` are
                // the other two readers php keeps quiet, and they suppress around their own
                // operand rather than being special-cased here, because the rule is theirs.
                if !context.errors_suppressed() {
                    values.warning(&format!("Undefined variable ${name}"))?;
                }
                values.null()
            }
        },
        EvalExpr::Magic(magic) => eval_magic_const(magic, context, values),
        EvalExpr::Match {
            subject,
            arms,
            default,
        } => eval_match_expr(subject, arms, default.as_deref(), context, scope, values),
        EvalExpr::Clone(object) => {
            let object = eval_expr(object, context, scope, values)?;
            eval_object_clone_result(object, context, values)
        }
        EvalExpr::NamespacedCall {
            name,
            fallback_name,
            args,
        } => eval_namespaced_call(name, fallback_name, args, context, scope, values),
        EvalExpr::NamespacedConstFetch {
            name,
            fallback_name,
        } => eval_namespaced_const_fetch(name, fallback_name, context, values),
        EvalExpr::NewObject { class_name, args } => {
            let args = eval_method_call_arg_values(args, context, scope, values)
                .map_err(|status| trace_new_object_error("arguments", class_name, status, context))?;
            let class_name = eval_new_object_class_name(class_name, context).map_err(|status| {
                trace_new_object_error("class_name", class_name, status, context)
            })?;
            eval_new_object_result(&class_name, args, context, scope, values)
        }
        EvalExpr::NewAnonymousClass { class, args } => {
            ensure_eval_anonymous_class_decl(class, context, scope, values)?;
            let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
            let class = context
                .class(class.name())
                .cloned()
                .ok_or(EvalStatus::RuntimeFatal)?;
            eval_dynamic_class_new_object(&class, evaluated_args, context, scope, values)
        }
        EvalExpr::StaticMethodCall {
            class_name,
            method,
            args,
        } => {
            let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
            eval_static_method_call_result_from_scope(
                class_name,
                method,
                evaluated_args,
                scope,
                context,
                values,
            )
        }
        EvalExpr::StaticPropertyGet {
            class_name,
            property,
        } => eval_static_property_get_result(class_name, property, context, values),
        EvalExpr::ClassConstantFetch {
            class_name,
            constant,
        } => eval_class_constant_fetch_result(class_name, constant, context, values),
        EvalExpr::ClassNameFetch { class_name } => {
            eval_class_name_fetch_result(class_name, context, values)
        }
        EvalExpr::MethodCall {
            object,
            method,
            args,
        } => {
            let receiver_is_temporary = eval_method_receiver_is_temporary(object);
            let object = eval_expr(object, context, scope, values)?;
            let result = (|| {
                let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
                eval_method_call_result_with_evaluated_args(
                    object,
                    method,
                    evaluated_args,
                    context,
                    values,
                )
            })();
            eval_method_call_with_temporary_receiver_cleanup(
                object,
                receiver_is_temporary,
                result,
                context,
                values,
            )
        }
        EvalExpr::NullsafeMethodCall {
            object,
            method,
            args,
        } => {
            let receiver_is_temporary = eval_method_receiver_is_temporary(object);
            let object = eval_expr(object, context, scope, values)?;
            let result = (|| {
                if values.is_null(object)? {
                    return values.null();
                }
                let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
                eval_method_call_result_with_evaluated_args(
                    object,
                    method,
                    evaluated_args,
                    context,
                    values,
                )
            })();
            eval_method_call_with_temporary_receiver_cleanup(
                object,
                receiver_is_temporary,
                result,
                context,
                values,
            )
        }
        EvalExpr::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            let receiver_is_temporary = eval_method_receiver_is_temporary(object);
            let object = eval_expr(object, context, scope, values)?;
            let result = (|| {
                if values.is_null(object)? {
                    return values.null();
                }
                let method = eval_dynamic_member_name(method, context, scope, values)?;
                let evaluated_args = eval_method_call_arg_values(args, context, scope, values)?;
                eval_method_call_result_with_evaluated_args(
                    object,
                    &method,
                    evaluated_args,
                    context,
                    values,
                )
            })();
            eval_method_call_with_temporary_receiver_cleanup(
                object,
                receiver_is_temporary,
                result,
                context,
                values,
            )
        }
        EvalExpr::NullCoalesce { value, default } => {
            // `$x ?? $default` asks whether `$x` is there and must not complain that it is not:
            // php raises no `Undefined variable` for the left operand of `??`. Suppressing around
            // it is exactly that rule, and reuses the depth counter `@` already maintains.
            //
            // Suppression alone is not enough, because an uninitialized typed property THROWS
            // rather than warning, and `@` does not stop a throw: `$o->t ?? 'D'` died with
            // `Typed property C::$t must not be accessed before initialization` where php
            // answers `'D'`. The quiet-fetch mode is the second half of the same rule.
            context.push_error_suppression();
            context.push_quiet_property_fetch();
            let value = eval_expr(value, context, scope, values);
            context.pop_quiet_property_fetch();
            context.pop_error_suppression();
            let value = value?;
            if values.is_null(value)? {
                eval_expr(default, context, scope, values)
            } else {
                Ok(value)
            }
        }
        EvalExpr::NullCoalesceAssign { target, default } => {
            eval_null_coalesce_assign(target, default, context, scope, values)
        }
        EvalExpr::CompoundAssign { target, op, value } => {
            eval_compound_assign(target, *op, value, context, scope, values)
        }
        EvalExpr::PostfixIncDec { target, increment } => {
            eval_postfix_inc_dec(target, *increment, context, scope, values)
        }
        EvalExpr::Assign { target, value } => {
            eval_assign(target, value, context, scope, values)
        }
        EvalExpr::ReferenceBindAssign { target, source } => {
            eval_var_reference_bind(target, source, context, scope, values)?;
            eval_expr(&EvalExpr::LoadVar(target.clone()), context, scope, values)
        }
        EvalExpr::ArrayAppendAssign { target, value } => {
            eval_array_append_result(target, value, context, scope, values)
        }
        EvalExpr::ReferenceBind { target, source } => {
            eval_reference_bind_result(target, source, context, scope, values)
        }
        EvalExpr::NullsafePropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            if values.is_null(object)? {
                return values.null();
            }
            eval_property_get_result(object, property, context, values)
        }
        EvalExpr::NullsafeDynamicPropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            if values.is_null(object)? {
                return values.null();
            }
            let property = eval_dynamic_member_name(property, context, scope, values)?;
            eval_property_get_result(object, &property, context, values)
        }
        EvalExpr::PropertyGet { object, property } => {
            let object = eval_expr(object, context, scope, values)?;
            eval_property_get_result(object, property, context, values)
        }
        EvalExpr::Print(inner) => {
            let value = eval_expr(inner, context, scope, values)?;
            let value = eval_string_context_value(value, context, values)?;
            values.echo(value)?;
            values.int(1)
        }
        EvalExpr::Ternary {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = eval_expr(condition, context, scope, values)?;
            if values.truthy(condition)? {
                if let Some(then_branch) = then_branch {
                    eval_expr(then_branch, context, scope, values)
                } else {
                    Ok(condition)
                }
            } else {
                eval_expr(else_branch, context, scope, values)
            }
        }
        EvalExpr::Throw(inner) => {
            let thrown = eval_expr(inner, context, scope, values)?;
            if values.type_tag(thrown)? != EVAL_TAG_OBJECT {
                return Err(EvalStatus::RuntimeFatal);
            }
            context.set_pending_throw(thrown);
            Err(EvalStatus::UncaughtThrowable)
        }
        EvalExpr::Unary {
            op: EvalUnaryOp::ErrorSuppress,
            expr,
        } => {
            context.push_error_suppression();
            let result = eval_expr(expr, context, scope, values);
            context.pop_error_suppression();
            result
        }
        EvalExpr::Unary { op, expr } => {
            let value = eval_expr(expr, context, scope, values)?;
            match op {
                EvalUnaryOp::Plus => {
                    let zero = values.int(0)?;
                    values.add(zero, value)
                }
                EvalUnaryOp::Negate => {
                    let zero = values.int(0)?;
                    values.sub(zero, value)
                }
                EvalUnaryOp::LogicalNot => {
                    let truthy = values.truthy(value)?;
                    values.bool_value(!truthy)
                }
                EvalUnaryOp::BitNot => values.bit_not(value),
                EvalUnaryOp::ErrorSuppress => unreachable!("handled before unary value evaluation"),
            }
        }
        EvalExpr::Binary { op, left, right } => {
            if *op == EvalBinOp::LogicalAnd {
                let left = eval_expr(left, context, scope, values)?;
                if !values.truthy(left)? {
                    return values.bool_value(false);
                }
                let right = eval_expr(right, context, scope, values)?;
                let truthy = values.truthy(right)?;
                return values.bool_value(truthy);
            }
            if *op == EvalBinOp::LogicalOr {
                let left = eval_expr(left, context, scope, values)?;
                if values.truthy(left)? {
                    return values.bool_value(true);
                }
                let right = eval_expr(right, context, scope, values)?;
                let truthy = values.truthy(right)?;
                return values.bool_value(truthy);
            }
            let left = eval_expr(left, context, scope, values)?;
            let right = eval_expr(right, context, scope, values)?;
            eval_binary_result(*op, left, right, context, values)
        }
    }
}

/// Returns whether evaluating a method receiver produces a disposable value owner.
fn eval_method_receiver_is_temporary(expr: &EvalExpr) -> bool {
    matches!(
        expr,
        EvalExpr::NewObject { .. }
            | EvalExpr::DynamicNewObject { .. }
            | EvalExpr::NewAnonymousClass { .. }
            | EvalExpr::Clone(_)
            | EvalExpr::MethodCall { .. }
            | EvalExpr::DynamicMethodCall { .. }
            | EvalExpr::NullsafeMethodCall { .. }
            | EvalExpr::NullsafeDynamicMethodCall { .. }
            | EvalExpr::StaticMethodCall { .. }
            | EvalExpr::DynamicStaticMethodCall { .. }
    )
}

/// Releases a temporary receiver once its method call either returns or fails.
fn eval_method_call_with_temporary_receiver_cleanup(
    receiver: RuntimeCellHandle,
    receiver_is_temporary: bool,
    result: Result<RuntimeCellHandle, EvalStatus>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !receiver_is_temporary {
        return result;
    }
    let result = result.and_then(|result| {
        if result == receiver {
            values.retain(result)
        } else {
            Ok(result)
        }
    });
    let cleanup = eval_release_value(context, values, receiver);
    match (result, cleanup) {
        (Err(status), _) => Err(status),
        (Ok(_), Err(status)) => Err(status),
        (Ok(result), Ok(())) => Ok(result),
    }
}

/// Emits the failing dynamic-construction stage when opt-in eval tracing is enabled.
fn trace_dynamic_new_error(
    stage: &str,
    class_name: Option<&str>,
    status: EvalStatus,
    context: &ElephcEvalContext,
) -> EvalStatus {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=dynamic_new_error stage={stage} class={class_name:?} status={status:?} file={:?} line={}",
            call_site.0,
            call_site.2,
        );
    }
    status
}
