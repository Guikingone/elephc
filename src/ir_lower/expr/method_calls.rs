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
        ctx.throw_access_sites
            .get(&(ctx.loop_storage_scope.clone(), expr.span))
            .and_then(|info| {
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
    if op == Op::MethodCall {
        if let Some(value) =
            lower_reflection_function_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
            return value;
        }
        if let Some(value) =
            lower_reflection_method_invoke_call(ctx, Some(object_expr), method, args, expr)
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
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
        if let Some(value) = lower_reflection_property_value_call(
            ctx,
            Some(object_expr),
            object,
            method,
            args,
            expr,
        )
        {
            release_owning_receiver_temporary(ctx, object, expr.span);
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
    if op == Op::MethodCall {
        if let Some(receiver) = object_static_method_receiver(ctx, object.value, method) {
            let call = lower_static_method_call(ctx, &receiver, method, args, expr);
            release_owning_receiver_temporary(ctx, object, expr.span);
            return call;
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
    let sig = method_call_argument_signature(ctx, object_expr, object.value, dispatch_method);
    if let Some(call) = lower_runtime_spread_method_call(
        ctx,
        object,
        dispatch_method,
        args,
        sig.as_ref(),
        result_type.clone(),
        expr,
    ) {
        return call;
    }
    let mut operands = vec![object.value];
    promote_eval_bridge_method_argument_locals(ctx, object.value, sig.as_ref(), args);
    promote_pdo_binding_ref_argument(ctx, object.value, dispatch_method, args);
    let prepared = sig
        .as_ref()
        .and_then(|signature| ref_place_args::prepare_ref_place_args(ctx, signature, args));
    let call_args = prepared
        .as_ref()
        .map(|(call_args, _)| call_args.as_slice())
        .unwrap_or(args);
    let arg_values = lower_args_with_signature(ctx, sig.as_ref(), call_args);
    operands.extend(arg_values.iter().copied());
    let data = ctx.intern_string(dispatch_method);
    let call = ctx.emit_value(
        op,
        operands,
        Some(Immediate::Data(data)),
        result_type,
        op.default_effects(),
        Some(expr.span),
    );
    let return_alias = method_return_arg_alias(ctx, object.value, dispatch_method);
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &arg_values,
        Some(call.value),
        &return_alias,
        sig.as_ref(),
        expr.span,
    );
    if let Some((_, plans)) = prepared {
        ref_place_args::write_back_ref_place_args(ctx, plans);
    }
    release_owning_receiver_temporary(ctx, object, expr.span);
    call
}

/// Promotes local arguments that an eval-backed mixed receiver may later mutate by reference.
pub(super) fn promote_eval_bridge_method_argument_locals(
    ctx: &mut LoweringContext<'_, '_>,
    object: ValueId,
    signature: Option<&FunctionSig>,
    args: &[Expr],
) {
    if !dynamic_method_receiver_needs_mixed_fallback(&ctx.builder.value_php_type(object)) {
        return;
    }
    // A no-argument call still needs the per-request eval context: its receiver can be an
    // eval-owned object or an AOT class outside the statically emitted candidate set. Without
    // it, codegen has no generic fallback and turns a live object into a misleading null-method
    // fatal. Web handlers retain one context for the request, so this remains generic and does
    // not depend on any framework or package layout.
    if ctx.web {
        ctx.declare_eval_context_local();
    }
    if !plain_positional_call_args(args) {
        return;
    }
    let bridge_args = match signature {
        Some(signature) => signature
            .ref_params
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, is_ref)| is_ref.then(|| args.get(index)).flatten())
            .collect::<Vec<_>>(),
        None => args.iter().collect(),
    };
    if bridge_args.is_empty() {
        return;
    }
    ctx.declare_eval_context_local();
    for argument in bridge_args {
        if let ExprKind::Variable(name) = &argument.kind {
            ctx.set_local_type(name, PhpType::Mixed);
        }
    }
}

/// Routes a method call with a spread that cannot use a fixed ABI through the
/// callable descriptor ABI.
///
/// A known signature with one trailing indexed spread can materialize its visible
/// parameters directly. Every other spread, including a concrete method that is
/// absent from an interface's static signature, must preserve its runtime keys so
/// the shared invoker can apply PHP positional/named argument rules before
/// selecting the method wrapper.
fn lower_runtime_spread_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    method: &str,
    args: &[Expr],
    signature: Option<&FunctionSig>,
    result_type: PhpType,
    expr: &Expr,
) -> Option<LoweredValue> {
    if !args.iter().any(is_spread_arg) {
        return None;
    }
    if signature.is_some() && has_statically_indexed_trailing_method_spread(ctx, args) {
        return None;
    }
    let data = ctx.intern_string(&format!("object::{}", method));
    let descriptor = ctx.emit_value(
        Op::FirstClassCallableNew,
        vec![object.value],
        Some(Immediate::ProfiledData {
            data,
            strict_php: crate::strict_php::is_enabled(),
        }),
        PhpType::Callable,
        Op::FirstClassCallableNew.default_effects(),
        Some(expr.span),
    );
    let arg_container = lower_untyped_descriptor_invoker_arg_container(ctx, args, expr.span)?;
    let call = emit_callable_descriptor_invoke(
        ctx,
        descriptor,
        arg_container,
        result_type,
        expr.span,
    );
    Some(call)
}

/// Returns whether a fixed method signature can materialize one trailing
/// spread directly without discarding runtime named-argument keys.
fn has_statically_indexed_trailing_method_spread(
    ctx: &LoweringContext<'_, '_>,
    args: &[Expr],
) -> bool {
    let Some(spread_idx) = single_trailing_indexed_spread_arg(ctx, args) else {
        return false;
    };
    let ExprKind::Spread(inner) = &args[spread_idx].kind else {
        return false;
    };
    matches!(
        indexed_spread_source_type(ctx, inner),
        Some(PhpType::Array(_))
    )
}

/// Resolves a static-only method invoked through an object receiver.
///
/// PHP permits `$object->staticMethod()`. The object expression is still evaluated by the caller,
/// but dispatch ignores its value and uses the receiver's class exactly like `Class::method()`.
/// An ordinary instance method with the same name always wins.
pub(super) fn object_static_method_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    method: &str,
) -> Option<StaticReceiver> {
    let object_ty = ctx.builder.value_php_type(object);
    let (class_name, _) = singular_object_class(&object_ty)?;
    static_only_object_method_receiver(ctx, class_name, method)
}

/// Resolves static-syntax object calls whose receiver type names one concrete class.
///
/// The parser represents `$object::method()` as an internal callable-array invocation, so this
/// helper recovers the known static target before that dynamic path discards static methods.
pub(super) fn object_static_method_receiver_for_expr(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    method: &str,
) -> Option<StaticReceiver> {
    let class_name = instance_callable_object_class(ctx, object)?;
    static_only_object_method_receiver(ctx, &class_name, method)
}

/// Resolves a static-only method without shadowing a same-named instance method.
fn static_only_object_method_receiver(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    method: &str,
) -> Option<StaticReceiver> {
    let normalized = class_name.trim_start_matches('\\');
    let method_key = php_symbol_key(method);
    let class_info = ctx.classes.get(normalized)?;
    if class_info.methods.contains_key(&method_key) {
        return None;
    }
    let receiver = StaticReceiver::Named(Name::from(normalized.to_string()));
    static_method_implementation_signature(ctx, &receiver, method)?;
    Some(receiver)
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
                callable_profile_immediate(),
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
