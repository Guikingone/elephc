//! Purpose:
//! Selects concrete native enum signatures before materializing interface-call arguments.
//!
//! Called from:
//! - Regular and guarded instance-method expression lowering.
//!
//! Key details:
//! - The receiver is evaluated once; only the selected branch evaluates source arguments.
//! - Ordinary call planning owns defaults, named/spread arguments and reference returns.
//! - Interface declarations remain unchanged for reflection and non-enum fallback dispatch.
//! - Rejected binding throws only in its selected branch after source argument evaluation.

use super::*;
use super::nullable_method_calls::lower_method_call_with_receiver_direct;

/// Uses a final enum's signature, rather than its interface's defaults, for its runtime branch.
pub(super) fn lower_enum_interface_call(
    ctx: &mut LoweringContext<'_, '_>, object: LoweredValue, method: &str,
    args: &[Expr], op: Op, expr: &Expr,
) -> Option<LoweredValue> {
    if !matches!(op, Op::MethodCall | Op::NullsafeMethodCall) { return None; }
    let object_type = ctx.builder.value_php_type(object.value);
    let (interface, _) = singular_object_class(&object_type)?;
    if !ctx.interfaces.contains_key(interface) { return None; }
    let mut candidates: Vec<_> = ctx.classes.iter()
        .filter(|(name, info)| ctx.enums.contains_key(*name)
            && info.interfaces.iter().any(|name| name == interface))
        .map(|(name, info)| (info.class_id, name.clone())).collect();
    if candidates.is_empty() { return None; }
    candidates.sort();
    let has_class_fallback = ctx.classes.iter().any(|(name, info)|
        !ctx.enums.contains_key(name) && info.interfaces.iter().any(|name| name == interface));
    let result_type = method_call_result_type(ctx, object.value, method, op, expr);
    let reference_assignment = ctx.reference_call_context.as_ref()
        .is_some_and(|context| context.depth == ctx.expression_depth);
    let dynamic_unpack = args.iter().any(is_spread_arg)
        && crate::types::call_args::expand_planned_positional_spreads(args).is_none();
    let result = if reference_assignment { None } else {
        let name = ctx.declare_owned_hidden_temp(result_type.clone());
        register_owned_call_operand(ctx, ctx.local_slots[&name], expr.span);
        Some(name)
    };
    let (object, owner) = root_owned_call_operand(ctx, object, expr.span);
    let merge = ctx.builder.create_named_block("enum.interface.merge", Vec::new());
    for (_, class) in candidates {
        let selected = ctx.builder.create_named_block("enum.interface.selected", Vec::new());
        let next = ctx.builder.create_named_block("enum.interface.next", Vec::new());
        let data = ctx.intern_class_name(&class);
        let matches = ctx.emit_value(Op::InstanceOf, vec![object.value],
            Some(Immediate::Data(data)), PhpType::Bool, Op::InstanceOf.default_effects(), Some(expr.span));
        ctx.builder.terminate(Terminator::CondBr {
            cond: matches.value, then_target: selected, then_args: Vec::new(),
            else_target: next, else_args: Vec::new(),
        });
        ctx.builder.position_at_end(selected);
        let signature = ctx.classes.get(&class)
            .and_then(|info| info.methods.get(&php_symbol_key(method))).cloned();
        if let Some(signature) = signature.as_ref() {
            let plan = crate::types::call_args::plan_call_args_with_regular_param_count_and_assoc_spreads(
                &signature, args, expr.span, crate::types::call_args::regular_param_count(&signature),
                false, signature.variadic.is_some()
                    && !crate::func_args::sig_collects_surplus_args(&signature),
                &assoc_spread_sources(ctx, args),
            );
            let error = match plan {
                Ok(plan) => plan.statically_missing_positionals(&signature).map(|(provided, required)| {
                    let bound = if required < crate::types::call_args::regular_param_count(signature) {
                        "at least"
                    } else { "exactly" };
                    ("ArgumentCountError", format!(
                        "Too few arguments to function {class}::{method}(), {provided} passed and {bound} {required} expected"
                    ))
                }),
                Err(error) => Some(enum_argument_error(&class, method, &signature, error)),
            };
            if let Some((exception_class, message)) = error {
                terminate_enum_argument_error(ctx, args, exception_class, message, expr.span);
                ctx.builder.position_at_end(next);
                continue;
            }
        }
        let concrete = ctx.emit_value(Op::Borrow, vec![object.value], None,
            PhpType::Object(class), Op::Borrow.default_effects(), Some(expr.span));
        let call = if dynamic_unpack && !reference_assignment {
            // Runtime unpacks can carry either packed values or named/sparse hash keys. The
            // ordinary descriptor binder owns those rules and catchable missing-argument errors.
            let callable = lower_instance_callable_from_receiver(ctx, concrete, method, expr.span);
            let callable = root_descriptor_callback(
                ctx, callable, descriptor_invoker_result_type(signature.as_ref()), expr.span,
            );
            let arguments = lower_descriptor_invoker_arg_container_for_call_user_func(
                ctx, args, signature.as_ref(), expr.span,
            );
            emit_callable_descriptor_invoke(ctx, callable, arguments, expr.span)
        } else {
            lower_method_call_with_receiver_direct(ctx, concrete, method, args, op, expr)
        };
        if let Some(result) = &result {
            store_value_into_temp(ctx, result, result_type.clone(), call, expr.span);
        }
        branch_to(ctx, merge);
        ctx.builder.position_at_end(next);
    }
    if has_class_fallback {
        let fallback = ctx.emit_value(Op::Borrow, vec![object.value], None,
            object_type, Op::Borrow.default_effects(), Some(expr.span));
        let call = lower_method_call_with_receiver_direct(ctx, fallback, method, args, op, expr);
        if let Some(result) = &result {
            store_value_into_temp(ctx, result, result_type, call, expr.span);
        }
        branch_to(ctx, merge);
    } else {
        // No native class implements this interface. Do not emit an impossible
        // declaration-shaped call that would reintroduce its required/default slots.
        let exception = Expr::new(ExprKind::NewObject {
            class_name: Name::unqualified("Error"),
            args: vec![Expr::new(ExprKind::StringLiteral(
                format!("Unknown native implementation of {interface}::{method}")
            ), expr.span)],
        }, expr.span);
        let exception = lower_expr(ctx, &exception);
        ctx.builder.terminate(Terminator::Throw { value: exception.value });
    }
    ctx.builder.position_at_end(merge);
    if let Some(owner) = owner { retire_owned_call_operand(ctx, owner, expr.span); }
    Some(if let Some(result) = result {
        unregister_owned_call_operand(ctx, ctx.local_slots[&result], expr.span);
        take_owned_temp(ctx, &result, expr.span)
    } else { lower_null(ctx, expr) })
}

/// Evaluates rejected branch arguments once and throws through ordinary operand-owner cleanup.
fn terminate_enum_argument_error(
    ctx: &mut LoweringContext<'_, '_>, args: &[Expr],
    exception_class: &str, message: String, span: Span,
) {
    for arg in args {
        let value = lower_call_source_arg(ctx, arg);
        let value = LoweredValue { value, ir_type: ctx.builder.value_type(value) };
        let _ = root_owned_call_operand(ctx, value, arg.span);
    }
    let exception = Expr::new(ExprKind::NewObject {
        class_name: Name::unqualified(exception_class),
        args: vec![Expr::new(ExprKind::StringLiteral(message), span)],
    }, span);
    let exception = lower_expr(ctx, &exception);
    ctx.builder.terminate(Terminator::Throw { value: exception.value });
}

/// Maps shared binding failures to PHP's selected implementation exception and diagnostic.
fn enum_argument_error(
    class: &str, method: &str, signature: &FunctionSig,
    error: crate::types::call_args::CallArgPlanError,
) -> (&'static str, String) {
    use crate::types::call_args::CallArgPlanError;
    match error {
        CallArgPlanError::MissingRequired { param_idx, .. } => (
            "ArgumentCountError", format!("{class}::{method}(): Argument #{} (${}) not passed",
                param_idx + 1, signature.params[param_idx].0),
        ),
        CallArgPlanError::UnknownNamed { name, .. } =>
            ("Error", format!("Unknown named parameter ${name}")),
        CallArgPlanError::Duplicate { name, .. } =>
            ("Error", format!("Named parameter ${name} overwrites previous argument")),
        _ => ("Error", format!("Invalid arguments to enum method {method}()")),
    }
}
