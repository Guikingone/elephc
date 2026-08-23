//! Purpose:
//! Static method calls, enum coercion, and callable descriptors.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers a static method call.
pub(super) fn lower_static_method_call(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    if matches!(receiver, StaticReceiver::Named(name) if name.trim_start_matches('\\') == "ReflectionReference")
        && php_symbol_key(method) == "fromarrayelement"
    {
        let sig = static_method_implementation_signature(ctx, receiver, method).cloned();
        let operands = lower_args_with_signature(ctx, sig.as_ref(), args);
        release_owned_call_arg_temporaries(
            ctx,
            &operands,
            None,
            &ReturnArgAlias::None,
            expr.span,
        );
        return lower_null(ctx, expr);
    }
    // `Closure::bind($closure, $newThis [, $scope])` — static form of bindTo.
    if let StaticReceiver::Named(name) = receiver {
        if name.trim_start_matches('\\') == "Closure"
            && php_symbol_key(method) == "bind"
            && !args.is_empty()
        {
            let saved_class = ctx.current_class.clone();
            if let Some(scope) = closure_bind_scope_class(ctx, args) {
                ctx.current_class = Some(scope);
            }
            let closure = lower_expr(ctx, &args[0]);
            ctx.current_class = saved_class;
            ctx.take_pending_static_callable_result();
            let new_this = match args.get(1) {
                Some(arg) => lower_expr(ctx, arg),
                None => lower_null(ctx, expr),
            };
            if let Some(scope_arg) = args.get(2) {
                let scope_value = lower_expr(ctx, scope_arg);
                if ctx.value_is_owning_temporary(scope_value) {
                    crate::ir_lower::ownership::release_if_owned(
                        ctx,
                        scope_value,
                        Some(scope_arg.span),
                    );
                }
            }
            let inline_scope_only = matches!(args[0].kind, ExprKind::Closure { .. })
                && args
                    .get(1)
                    .is_some_and(|new_this| matches!(new_this.kind, ExprKind::Null));
            if inline_scope_only {
                return closure;
            }
            return emit_closure_bind(ctx, closure.value, new_this.value, expr);
        }
    }

    let magic_args;
    let (dispatch_method, call_args) = if let Some(args) =
        magic_static_call_dispatch_args(ctx, receiver, method, args, expr.span)
    {
        magic_args = args;
        ("__callStatic", magic_args.as_slice())
    } else {
        (method, args)
    };
    if matches!(receiver, StaticReceiver::Named(_)) && plain_positional_call_args(args) {
        if let Some(class_name) = static_receiver_class_name(ctx, receiver) {
            if !ctx.classes.contains_key(class_name.as_str()) {
                ctx.declare_eval_context_local();
                let operands = lower_args_with_signature(ctx, None, args);
                let name = format!("{}::{}", class_name, dispatch_method);
                let data = ctx.intern_string(&name);
                return ctx.emit_value(
                    Op::EvalStaticMethodCall,
                    operands,
                    Some(Immediate::Data(data)),
                    PhpType::Mixed,
                    Op::EvalStaticMethodCall.default_effects(),
                    Some(expr.span),
                );
            }
        }
    }
    let sig = static_method_implementation_signature(ctx, receiver, dispatch_method)
        .or_else(|| lexical_instance_static_call_signature(ctx, receiver, dispatch_method))
        .cloned();
    let prepared = sig.as_ref().and_then(|signature| {
        ref_place_args::prepare_ref_place_args(ctx, signature, call_args)
    });
    let call_args = prepared
        .as_ref()
        .map(|(call_args, _)| call_args.as_slice())
        .unwrap_or(call_args);
    let operands = lower_args_with_signature(ctx, sig.as_ref(), call_args);
    let operands =
        coerce_int_backed_enum_string_argument(ctx, receiver, dispatch_method, operands, expr);
    let name = format!("{}::{}", receiver_name(receiver), dispatch_method);
    let data = ctx.intern_string(&name);
    let result_type = sig
        .as_ref()
        .map(|signature| normalize_value_php_type(signature.return_type.codegen_repr()))
        .unwrap_or_else(|| {
            if ctx.has_eval_barrier() && matches!(receiver, StaticReceiver::Named(_)) {
                PhpType::Mixed
            } else {
                fallback_expr_type(expr)
            }
        });
    let late_static_receiver_type = static_late_binding_receiver_type_for_ir(ctx, receiver);
    let result_type = match (
        static_method_late_static_return_for_ir(ctx, receiver, dispatch_method),
        late_static_receiver_type.as_deref(),
    ) {
        (Some(return_type), Some(receiver_type)) => {
            late_static_return_type_for_ir(ctx, &return_type, receiver_type)
        }
        _ => result_type,
    };
    let call = ctx.emit_value(
        Op::StaticMethodCall,
        operands.clone(),
        Some(Immediate::Data(data)),
        result_type,
        Op::StaticMethodCall.default_effects(),
        Some(expr.span),
    );
    let return_alias = static_method_return_arg_alias(ctx, receiver, dispatch_method);
    release_owned_call_arg_temporaries_with_signature(
        ctx,
        &operands,
        Some(call.value),
        &return_alias,
        sig.as_ref(),
        expr.span,
    );
    if let Some((_, plans)) = prepared {
        ref_place_args::write_back_ref_place_args(ctx, plans);
    }
    call
}

/// Resolves a statically knowable `Closure::bind()` visibility scope for closure lowering.
///
/// The runtime descriptor only needs the rebound receiver, but `self`, `parent`, private members,
/// and static properties inside the closure are lexical constructs that must be resolved while
/// its EIR body is built. Dynamic scope expressions remain gradual and keep the enclosing scope.
fn closure_bind_scope_class(
    ctx: &LoweringContext<'_, '_>,
    args: &[Expr],
) -> Option<String> {
    let scope = args.get(2)?;
    match &scope.kind {
        ExprKind::StringLiteral(name) => Some(name.trim_start_matches('\\').to_string()),
        ExprKind::ClassConstant { receiver } => match receiver {
            StaticReceiver::Named(name) => {
                Some(name.as_str().trim_start_matches('\\').to_string())
            }
            StaticReceiver::Self_ | StaticReceiver::Static => ctx.current_class.clone(),
            StaticReceiver::Parent => ctx
                .current_class
                .as_ref()
                .and_then(|class| ctx.classes.get(class))
                .and_then(|class| class.parent.clone()),
        },
        _ => instance_callable_object_class(ctx, scope),
    }
}

/// Returns preserved late-static return syntax for EIR static dispatch.
pub(super) fn static_method_late_static_return_for_ir(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
) -> Option<TypeExpr> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let method_key = php_symbol_key(method);
    let class_info = ctx.classes.get(&class_name)?;
    if static_method_implementation_signature(ctx, receiver, method).is_some() {
        return class_info
            .late_static_static_method_returns
            .get(&method_key)
            .cloned();
    }
    lexical_instance_static_call_signature(ctx, receiver, method)?;
    class_info.late_static_method_returns.get(&method_key).cloned()
}

/// Resolves the receiver type used to bind `static` for an EIR static-style call.
pub(super) fn static_late_binding_receiver_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<String> {
    match receiver {
        StaticReceiver::Named(name) => Some(name.as_str().trim_start_matches('\\').to_string()),
        StaticReceiver::Self_ | StaticReceiver::Static | StaticReceiver::Parent => {
            ctx.current_class.clone()
        }
    }
}

/// PHP coerces a numeric string to the integer backing value for an int-backed enum's
/// `from()`/`tryFrom()`. When the sole argument lowered to a string, insert an explicit
/// `EnumBackingStringToInt` coercion (issue #349) so the enum call receives a plain integer
/// operand: the backing scan then runs on an int rather than a heap string, and a
/// non-numeric string throws `TypeError` inside the coercion at runtime. Non-matching
/// receivers/methods/argument types pass the operands through unchanged.
pub(super) fn coerce_int_backed_enum_string_argument(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
    mut operands: Vec<crate::ir::ValueId>,
    expr: &Expr,
) -> Vec<crate::ir::ValueId> {
    let key = php_symbol_key(method);
    if (key != "from" && key != "tryfrom") || operands.len() != 1 {
        return operands;
    }
    let StaticReceiver::Named(name) = receiver else {
        return operands;
    };
    let enum_name = name.trim_start_matches('\\');
    let is_int_backed = ctx
        .enums
        .get(enum_name)
        .and_then(|info| info.backing_type.as_ref())
        .is_some_and(|backing| matches!(backing, PhpType::Int));
    if !is_int_backed {
        return operands;
    }
    let method_display = if key == "tryfrom" { "tryFrom" } else { "from" };
    // A `string` argument coerces via a strict numeric probe; a `Mixed` argument dispatches
    // on its runtime tag (int/bool/float/null coerce, string coerces, others `TypeError`).
    // The string op carries the full message; the Mixed op carries the message prefix and
    // appends the runtime type word in codegen.
    let (op, message) = match ctx.builder.value_php_type(operands[0]).codegen_repr() {
        PhpType::Str => (
            Op::EnumBackingStringToInt,
            format!(
                "{}::{}(): Argument #1 ($value) must be of type int, string given",
                enum_name, method_display
            ),
        ),
        PhpType::Mixed | PhpType::Union(_) => (
            Op::EnumBackingMixedToInt,
            format!(
                "{}::{}(): Argument #1 ($value) must be of type int, ",
                enum_name, method_display
            ),
        ),
        _ => return operands,
    };
    let message_data = ctx.intern_string(&message);
    let coerced = ctx.emit_value(
        op,
        vec![operands[0]],
        Some(Immediate::Data(message_data)),
        PhpType::Int,
        op.default_effects(),
        Some(expr.span),
    );
    operands[0] = coerced.value;
    operands
}

/// Builds synthetic `__callStatic` arguments when a class lacks the requested static method.
pub(super) fn magic_static_call_dispatch_args(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
    args: &[Expr],
    span: Span,
) -> Option<Vec<Expr>> {
    if static_method_implementation_signature(ctx, receiver, method).is_some()
        || lexical_instance_static_call_signature(ctx, receiver, method).is_some()
    {
        return None;
    }
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let class_info = ctx.classes.get(class_name.as_str())?;
    if class_info.methods.contains_key(&php_symbol_key(method)) {
        return None;
    }
    static_method_implementation_signature(ctx, receiver, "__callStatic")?;
    Some(vec![
        Expr::new(ExprKind::StringLiteral(method.to_string()), span),
        Expr::new(ExprKind::ArrayLiteral(args.to_vec()), span),
    ])
}

/// Lowers a static-method callable-array call through a descriptor invoker.
pub(super) fn lower_static_method_descriptor_call(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    let sig = static_method_implementation_signature(ctx, receiver, method).cloned();
    let wrapper_sig = sig
        .as_ref()
        .map(crate::codegen::callable_dispatch::static_method_runtime_wrapper_sig);
    let target = CallableTarget::StaticMethod {
        receiver: receiver.clone(),
        method: method.to_string(),
    };
    let descriptor = lower_first_class_callable(ctx, &target, expr);
    let mut operands = Vec::with_capacity(args.len() + 1);
    operands.push(descriptor.value);
    operands.extend(lower_args_with_signature(ctx, wrapper_sig.as_ref(), args));
    let result_type = sig
        .as_ref()
        .map(|signature| normalize_value_php_type(signature.return_type.codegen_repr()))
        .unwrap_or_else(|| fallback_expr_type(expr));
    ctx.emit_value(
        Op::ExprCall,
        operands,
        callable_profile_immediate(),
        result_type,
        Op::ExprCall.default_effects(),
        Some(expr.span),
    )
}

/// Lowers a static-method descriptor call when operands have already been evaluated.
pub(super) fn lower_static_method_descriptor_value_call(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
    args: Vec<crate::ir::ValueId>,
    expr: &Expr,
) -> Option<LoweredValue> {
    let sig = static_method_implementation_signature(ctx, receiver, method).cloned();
    let target = CallableTarget::StaticMethod {
        receiver: receiver.clone(),
        method: method.to_string(),
    };
    let descriptor = lower_first_class_callable(ctx, &target, expr);
    let mut operands = Vec::with_capacity(args.len() + 1);
    operands.push(descriptor.value);
    operands.extend(args);
    let result_type = sig
        .as_ref()
        .map(|signature| normalize_value_php_type(signature.return_type.codegen_repr()))
        .unwrap_or_else(|| fallback_expr_type(expr));
    Some(ctx.emit_value(
        Op::ExprCall,
        operands,
        callable_profile_immediate(),
        result_type,
        Op::ExprCall.default_effects(),
        Some(expr.span),
    ))
}

/// Returns the conservative return-to-argument alias summary for static dispatch.
pub(super) fn static_method_return_arg_alias(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
) -> ReturnArgAlias {
    let Some(class_name) = static_receiver_class_name(ctx, receiver) else {
        return ReturnArgAlias::Unknown;
    };
    let method_key = php_symbol_key(method);
    let Some(class_info) = ctx.classes.get(&class_name) else {
        return ReturnArgAlias::Unknown;
    };
    if !matches!(receiver, StaticReceiver::Static)
        || class_info.is_final
        || class_info.final_static_methods.contains(&method_key)
    {
        return class_static_method_return_arg_alias(ctx, &class_name, &method_key)
            .unwrap_or(ReturnArgAlias::Unknown);
    }

    let mut summary: Option<ReturnArgAlias> = None;
    for candidate in ctx.classes.keys() {
        if !is_same_or_descendant_class(ctx, candidate, &class_name) {
            continue;
        }
        let Some(alias) = class_static_method_return_arg_alias(ctx, candidate, &method_key) else {
            continue;
        };
        summary = Some(match summary {
            Some(current) => current.merge(&alias),
            None => alias,
        });
    }
    summary.unwrap_or(ReturnArgAlias::Unknown)
}

/// Resolves one class's static implementation and its source alias summary.
pub(super) fn class_static_method_return_arg_alias(
    ctx: &LoweringContext<'_, '_>,
    class_name: &str,
    method_key: &str,
) -> Option<ReturnArgAlias> {
    let class_info = ctx.classes.get(class_name)?;
    class_info.static_methods.get(method_key)?;
    let impl_class = class_info
        .static_method_impl_classes
        .get(method_key)
        .map(String::as_str)
        .unwrap_or(class_name);
    Some(
        ctx.return_alias_summaries
            .method(impl_class, method_key)
            .cloned()
            .unwrap_or(ReturnArgAlias::Unknown),
    )
}

/// Returns the implementation signature used by the static method symbol that will run.
pub(super) fn static_method_implementation_signature<'a>(
    ctx: &'a LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
) -> Option<&'a FunctionSig> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let key = php_symbol_key(method);
    let receiver_info = ctx.classes.get(class_name.as_str())?;
    let impl_class = receiver_info
        .static_method_impl_classes
        .get(&key)
        .map(String::as_str)
        .unwrap_or(class_name.as_str());
    ctx.classes
        .get(impl_class)
        .and_then(|class_info| class_info.static_methods.get(&key))
}

/// Returns the declared result type for a static method call before its arguments are lowered.
pub(in crate::ir_lower) fn static_method_call_expr_type_for_ir(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
) -> Option<PhpType> {
    let nominal = static_method_implementation_signature(ctx, receiver, method)
        .or_else(|| lexical_instance_static_call_signature(ctx, receiver, method))
        .map(|signature| normalize_value_php_type(signature.return_type.codegen_repr()))?;
    match (
        static_method_late_static_return_for_ir(ctx, receiver, method),
        static_late_binding_receiver_type_for_ir(ctx, receiver),
    ) {
        (Some(return_type), Some(receiver_type)) => Some(late_static_return_type_for_ir(
            ctx,
            &return_type,
            &receiver_type,
        )),
        _ => Some(nominal),
    }
}

/// Returns the instance-method signature used by `self::method()` or `parent::method()`.
pub(super) fn lexical_instance_static_call_signature<'a>(
    ctx: &'a LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    method: &str,
) -> Option<&'a FunctionSig> {
    if !matches!(receiver, StaticReceiver::Self_ | StaticReceiver::Parent)
        && !named_receiver_is_current_or_ancestor(ctx, receiver)
    {
        return None;
    }
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let key = php_symbol_key(method);
    class_method_signature(ctx, &class_name, &key)
}

/// Returns whether a named static-syntax receiver denotes the current instance class or an ancestor.
fn named_receiver_is_current_or_ancestor(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> bool {
    let StaticReceiver::Named(name) = receiver else {
        return false;
    };
    let target = name.as_str().trim_start_matches('\\');
    let Some(mut current) = ctx.current_class.as_deref() else {
        return false;
    };
    loop {
        if php_symbol_key(current) == php_symbol_key(target) {
            return true;
        }
        let Some(parent) = ctx
            .classes
            .get(current)
            .and_then(|class_info| class_info.parent.as_deref())
        else {
            return false;
        };
        current = parent;
    }
}

/// Resolves a static receiver to a concrete class name when lexical metadata is available.
pub(super) fn static_receiver_class_name(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<String> {
    match receiver {
        StaticReceiver::Named(name) => Some(name.as_str().trim_start_matches('\\').to_string()),
        StaticReceiver::Self_ | StaticReceiver::Static => ctx.current_class.clone(),
        StaticReceiver::Parent => {
            let current = ctx.current_class.as_deref()?;
            ctx.classes.get(current).and_then(|class_info| class_info.parent.clone())
        }
    }
}
