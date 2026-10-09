//! Purpose:
//! Core instance-method dispatch and Closure rebinding methods.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers an object method call.
pub(super) fn lower_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
    args: &[Expr],
    op: Op,
    expr: &Expr,
) -> LoweredValue {
    // A statically-decided private/protected method access from an inaccessible
    // scope raises a catchable `Error` in PHP rather than a compile-time error,
    // but the receiver expression must still be evaluated first.
    let throw_access_message = if op == Op::MethodCall {
        ctx.throw_access_sites.get(&expr.span).and_then(|info| {
            if let ThrowAccessKind::PrivateMethod {
                visibility,
                class_name,
                method: m,
            } = &info.kind
            {
                Some(format!(
                    "Call to {} method {}::{}() from global scope",
                    visibility, class_name, m
                ))
            } else {
                None
            }
        })
    } else {
        None
    };
    let object_expr = object;
    let object = lower_expr(ctx, object_expr);
    if let Some(message) = throw_access_message {
        release_owning_receiver_temporary(ctx, object, expr.span);
        return crate::ir_lower::stmt::lower_throw_access_error_expr(ctx, &message, expr.span);
    }
    if op == Op::MethodCall && value_is_definitely_null(ctx, object.value) {
        let null_value = lower_null(ctx, expr);
        terminate_method_call_on_null(ctx, method);
        return null_value;
    }
    if op == Op::MethodCall && is_parallel_task_group_spawn(ctx, object.value, method) {
        let spawned = lower_parallel_spawn(ctx, object, args, expr);
        release_owning_receiver_temporary(ctx, object, expr.span);
        return spawned;
    }
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_function_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            return value;
        }
        if let Some(value) =
            lower_reflection_method_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            return value;
        }
    }
    if op == Op::MethodCall
        && (value_is_nullable(ctx, object.value)
            || value_may_carry_container_miss(ctx, object.value))
    {
        return lower_nullable_regular_method_call(ctx, object, method, args, expr);
    }
    if op == Op::MethodCall && is_reflection_class_new_instance_call(ctx, object.value, method) {
        return lower_reflection_class_new_instance(ctx, Some(object_expr), object, args, expr);
    }
    if op == Op::MethodCall && is_reflection_class_new_instance_args_call(ctx, object.value, method)
    {
        return lower_reflection_class_new_instance_args(
            ctx,
            Some(object_expr),
            object,
            args,
            expr,
        );
    }
    if op == Op::MethodCall
        && is_reflection_class_new_instance_without_constructor_call(ctx, object.value, method)
    {
        return lower_reflection_class_new_instance_without_constructor(ctx, object, args, expr);
    }
    if op == Op::MethodCall {
        if let Some(value) = lower_reflection_class_static_property_value_call(
            ctx,
            Some(object_expr),
            method,
            args,
            expr,
        ) {
            return value;
        }
    }
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_class_member_list_call(ctx, Some(object_expr), method, args, expr)
        {
            return value;
        }
    }
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_property_value_call(ctx, Some(object_expr), method, args, expr)
        {
            return value;
        }
    }
    if matches!(
        ctx.builder.value_php_type(object.value).codegen_repr(),
        PhpType::Callable
    ) {
        if let Some(result) = lower_closure_bind_method(ctx, &object, method, args, expr) {
            return result;
        }
    }
    let magic_args;
    let (dispatch_method, args) = if let Some(args) =
        magic_call_dispatch_args(ctx, object.value, method, args, object_expr.span)
    {
        magic_args = args;
        ("__call", magic_args.as_slice())
    } else {
        (method, args)
    };
    let result_type = method_call_result_type(ctx, object.value, dispatch_method, op, expr);
    let mut operands = vec![object.value];
    let sig = method_call_argument_signature(ctx, object_expr, object.value, dispatch_method);
    promote_pdo_binding_ref_argument(ctx, object.value, dispatch_method, args);
    let user_owner = singular_object_class(&ctx.builder.value_php_type(object.value))
        .and_then(|(class, _)| {
            let class = class.trim_start_matches('\\');
            // Fiber switching consumes transferred receivers/arguments in its
            // private runtime ABI, including escape. It is not a user-method
            // call and must not also get this caller-side cleanup handler.
            if php_symbol_key(class) == "fiber" || crate::types::builtin_classes::intrinsic_class_names()
                .any(|intrinsic| php_symbol_key(intrinsic) == php_symbol_key(class)) {
                return None;
            }
            let info = ctx.classes.get(class)?;
            let key = php_symbol_key(dispatch_method);
            let implementation = info.method_impl_classes.get(&key).map(String::as_str).unwrap_or(class);
            Some(format!("{implementation}::{key}"))
        });
    let arg_values = if let Some(owner) = &user_owner {
        lower_args_with_eir_user_function_signature(ctx, owner, sig.as_ref(), args)
    } else {
        lower_args_with_eir_signature(ctx, sig.as_ref(), args)
    };
    operands.extend(arg_values.iter().copied());
    let cleanup = (user_owner.is_some() &&
        (super::function_calls::call_has_owning_temporary_arg(ctx, &arg_values)
            || ctx.value_is_owning_temporary(object))).then(|| {
        let handler = ctx.builder.create_named_block("method.call_cleanup", Vec::new());
        let after = ctx.builder.create_named_block("method.call_after", Vec::new());
        let result = (result_type != PhpType::Void)
            .then(|| ctx.declare_owned_hidden_temp(result_type.clone()));
        ctx.emit_void(Op::TryPushHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
            Op::TryPushHandler.default_effects(), Some(expr.span));
        (handler, after, result)
    });
    let data = ctx.intern_string(dispatch_method);
    let call = ctx.emit_value(
        op,
        operands,
        Some(Immediate::Data(data)),
        result_type.clone(),
        op.default_effects(),
        Some(expr.span),
    );
    if let Some((_, _, Some(result))) = &cleanup {
        store_value_into_temp(ctx, result, result_type, call, expr.span);
    }
    let return_alias = match ctx.builder.value_php_type(object.value).codegen_repr() {
        PhpType::Object(class_name)
            if php_symbol_key(class_name.trim_start_matches('\\'))
                == "elephc\\async\\__scheduler"
                && php_symbol_key(dispatch_method) == "runroot" =>
        {
            ReturnArgAlias::None
        }
        PhpType::Object(class_name)
            if php_symbol_key(class_name.trim_start_matches('\\'))
                == "elephc\\async\\taskgroup"
                && php_symbol_key(dispatch_method) == "spawn" =>
        {
            ReturnArgAlias::None
        }
        _ => method_return_arg_alias(ctx, object.value, dispatch_method),
    };
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &arg_values,
        Some(call.value),
        &return_alias,
        sig.as_ref(),
        expr.span,
    );
    if fiber_switch_transfers_receiver(ctx, object, dispatch_method) {
        ctx.builder.set_value_ownership(object.value, Ownership::Owned);
    } else {
        release_owning_receiver_temporary(ctx, object, expr.span);
    }
    if let Some((handler, after, result)) = cleanup {
        pop_method_cleanup_handler(ctx, handler, expr.span);
        branch_to(ctx, after);
        ctx.builder.position_at_end(handler);
        ctx.clear_static_callable_locals();
        pop_method_cleanup_handler(ctx, handler, expr.span);
        release_owned_call_arg_temporaries_with_signature(ctx, &arg_values, None,
            &ReturnArgAlias::None, sig.as_ref(), expr.span);
        release_owning_receiver_temporary(ctx, object, expr.span);
        let current = ctx.emit_owned_value(Op::CatchBind, Vec::new(), None,
            PhpType::Object("Throwable".to_string()), Op::CatchBind.default_effects(), Some(expr.span));
        ctx.builder.terminate(Terminator::Throw { value: current.value });
        ctx.builder.position_at_end(after);
        ctx.clear_static_callable_locals();
        return result.map(|result| take_owned_temp(ctx, &result, expr.span)).unwrap_or(call);
    }
    call
}

fn pop_method_cleanup_handler(ctx: &mut LoweringContext<'_, '_>, handler: BlockId, span: Span) {
    ctx.emit_void(Op::TryPopHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
        Op::TryPopHandler.default_effects(), Some(span));
}

fn is_parallel_task_group_spawn(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> bool {
    // A user program can legally declare a same-named class when it has not opted into the
    // Parallel prelude. The dedicated worker ABI exists only with that prelude's bridge extern,
    // so the class name alone must never select special lowering.
    if php_symbol_key(method) != "spawn"
        || !ctx
            .extern_functions
            .contains_key("elephc_parallel_job_input_php_prepare")
    {
        return false;
    }
    matches!(
        ctx.builder.value_php_type(object).codegen_repr(),
        PhpType::Object(ref class_name)
            if php_symbol_key(class_name.trim_start_matches('\\'))
                == "elephc\\parallel\\taskgroup"
    )
}

/// Returns whether a Fiber switching method transfers an owning receiver temporary to runtime.
pub(super) fn fiber_switch_transfers_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
) -> bool {
    matches!(
        ctx.builder.value_php_type(object.value).codegen_repr(),
        PhpType::Object(ref class_name) if php_symbol_key(class_name.trim_start_matches('\\')) == "fiber"
    ) && matches!(php_symbol_key(method).as_str(), "start" | "resume" | "throw")
        && ctx.value_is_owning_temporary(object)
        // A named local owns its own frame slot and must keep that ownership until its
        // explicit unset/epilogue. The Fiber ABI only consumes expression temporaries whose
        // ordinary post-call release would otherwise be skipped by a switch/longjmp.
        && !matches!(ctx.builder.value_defining_op(object.value), Some(Op::LoadLocal))
}

/// Lowers the `Closure` rebinding methods on a closure (`Callable`) receiver:
/// `$closure->bindTo($newThis [, $scope])` and `$closure->call($newThis, ...$args)`.
/// Returns `None` for any other method so normal dispatch (and its diagnostics)
/// still apply. The `$scope` argument is accepted and ignored — visibility is
/// resolved at compile time in elephc's closed-world model.
pub(super) fn lower_closure_bind_method(
    ctx: &mut LoweringContext<'_, '_>,
    closure: &LoweredValue,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    match php_symbol_key(method).as_str() {
        "bindto" => {
            let new_this = match args.first() {
                Some(arg) => lower_expr(ctx, arg),
                None => lower_null(ctx, expr),
            };
            Some(emit_closure_bind(ctx, closure.value, new_this.value, expr))
        }
        "call" => {
            // `$closure->call($newThis, ...$args)`: bind `$this` then invoke the
            // bound closure with the remaining arguments in one step.
            let new_this = match args.first() {
                Some(arg) => lower_expr(ctx, arg),
                None => lower_null(ctx, expr),
            };
            let bound = emit_closure_bind(ctx, closure.value, new_this.value, expr);
            let call_args = &args[args.len().min(1)..];
            let arg_container =
                lower_untyped_descriptor_invoker_arg_container(ctx, call_args, expr.span)?;
            Some(ctx.emit_value(
                Op::CallableDescriptorInvoke,
                vec![bound.value, arg_container.value],
                callable_profile_immediate(ctx),
                PhpType::Mixed,
                Op::CallableDescriptorInvoke.default_effects(),
                Some(expr.span),
            ))
        }
        _ => None,
    }
}

/// Emits the `closure_bind` runtime call that rebinds a closure's captured
/// `$this`, yielding a new closure (`Callable`) descriptor.
pub(super) fn emit_closure_bind(
    ctx: &mut LoweringContext<'_, '_>,
    closure: crate::ir::ValueId,
    new_this: crate::ir::ValueId,
    expr: &Expr,
) -> LoweredValue {
    ctx.emit_value(
        Op::ClosureBind,
        vec![closure, new_this],
        None,
        PhpType::Callable,
        Op::ClosureBind.default_effects(),
        Some(expr.span),
    )
}

/// Builds synthetic `__call` arguments when a class lacks the requested method.
pub(super) fn magic_call_dispatch_args(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
    args: &[Expr],
    span: Span,
) -> Option<Vec<Expr>> {
    if method_signature(ctx, object, method).is_some() {
        return None;
    }
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, _)) = singular_object_class(&object_ty) else {
        return None;
    };
    let normalized = class_name.trim_start_matches('\\');
    class_method_signature(ctx, normalized, &php_symbol_key("__call"))?;
    Some(vec![
        Expr::new(ExprKind::StringLiteral(method.to_string()), span),
        Expr::new(ExprKind::ArrayLiteral(args.to_vec()), span),
    ])
}

/// Returns the signature to use for method-call argument normalization.
pub(super) fn method_call_argument_signature(
    ctx: &LoweringContext<'_, '_>,
    object_expr: &Expr,
    object: crate::ir::ValueId,
    method: &str,
) -> Option<FunctionSig> {
    if method_is_fiber_start(ctx, object, method) {
        return crate::ir_lower::fibers::start_sig_for_expr(ctx, object_expr);
    }
    method_signature(ctx, object, method)
}

/// Returns true when a method call targets PHP's built-in `Fiber::start()`.
pub(super) fn method_is_fiber_start(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> bool {
    if php_symbol_key(method) != "start" {
        return false;
    }
    let object_ty = ctx.builder.value_php_type(object);
    let Some((class_name, _)) = singular_object_class(&object_ty) else {
        return false;
    };
    php_symbol_key(class_name.trim_start_matches('\\')) == "fiber"
}

/// Lowers `?Object->method()` calls so null receivers fatal before argument evaluation.
pub(super) fn lower_nullable_regular_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    let result_type = method_call_result_type(ctx, object.value, method, Op::MethodCall, expr);
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let fatal_block = ctx
        .builder
        .create_named_block("method.null.fatal", Vec::new());
    let call_block = ctx
        .builder
        .create_named_block("method.non_null.call", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("method.nullable.merge", Vec::new());
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![object.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: fatal_block,
        then_args: Vec::new(),
        else_target: call_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(fatal_block);
    terminate_method_call_on_null(ctx, method);

    ctx.builder.position_at_end(call_block);
    let call = lower_method_call_with_receiver(ctx, object, method, args, Op::MethodCall, expr);
    store_value_into_temp(ctx, &temp_name, result_type.clone(), call, expr.span);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}
