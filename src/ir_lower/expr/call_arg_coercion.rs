//! Purpose:
//! General call argument lowering and parameter storage coercion.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.
//! - By-value boxed arguments are snapshotted before later argument side effects.

use super::*;

/// Lowers positional/named/spread call arguments in source order.
pub(super) fn lower_args(ctx: &mut LoweringContext<'_, '_>, args: &[Expr]) -> Vec<crate::ir::ValueId> {
    args.iter().map(|arg| lower_expr(ctx, arg).value).collect()
}

/// Lowers one argument while applying by-reference storage normalization from a signature.
pub(super) fn lower_arg_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    arg: &Expr,
) -> crate::ir::ValueId {
    // Specialized builtin lowerers consume already-bound operands and may have
    // additional source-position guards. Keep their contract separate from the
    // ordinary user-call preparation/binding stages below.
    if let Some(value) = lower_by_ref_array_element_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    if let Some(value) = lower_by_ref_array_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    let lowered = lower_expr(ctx, arg);
    let lowered = coerce_scalar_arg_to_param_storage(ctx, sig, index, lowered, arg);
    snapshot_call_argument(ctx, lowered, sig.ref_params.get(index).copied().unwrap_or(false), arg.span)
}

/// Evaluates one source operand without invoking its declared scalar conversion.
pub(super) fn lower_arg_source_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    arg: &Expr,
) -> crate::ir::ValueId {
    if let Some(value) = lower_by_ref_array_element_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    if let Some(value) = lower_by_ref_array_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    let lowered = lower_expr(ctx, arg);
    snapshot_call_argument(ctx, lowered, sig.ref_params.get(index).copied().unwrap_or(false), arg.span)
}

/// Snapshots a planned value argument without performing scalar binding early.
/// Named/spread planning consumes the same primitive after source-order evaluation.
pub(super) fn snapshot_call_argument(
    ctx: &mut LoweringContext<'_, '_>,
    lowered: LoweredValue,
    by_ref: bool,
    span: Span,
) -> crate::ir::ValueId {
    // Immutable data-section strings already survive every later source effect;
    // persisting them would invent a heap owner at legacy borrowed call surfaces.
    if ctx.builder.value_php_type(lowered.value).codegen_repr() == PhpType::Str
        && ctx.builder.value_ownership(lowered.value) == Ownership::Persistent
    {
        return lowered.value;
    }
    if !by_ref
        && Ownership::php_type_needs_lifetime_tracking(&ctx.builder.value_php_type(lowered.value))
        && !matches!(ctx.builder.value_php_type(lowered.value).codegen_repr(), PhpType::Buffer(_))
    {
        // Entry-time parameter copying is too late: another argument may mutate the
        // source cell before the callee starts. Reference arguments remain lvalues.
        let owning_source = ctx.value_is_owning_temporary(lowered);
        let snapshot = crate::ir_lower::ownership::copy_for_php_value_binding(
            ctx, lowered, Some(span),
        );
        if owning_source {
            crate::ir_lower::ownership::release_if_owned(ctx, lowered, Some(span));
        }
        super::arg_evaluation::hold_argument_snapshot(ctx, snapshot, span)
    } else {
        lowered.value
    }
}

/// Coerces a positional argument to storage owned explicitly by EIR when required.
///
/// Integer-to-float conversion selects the callee's floating-point ABI class. Mixed-to-string
/// conversion is also explicit here because it allocates caller-owned storage whose lifetime
/// depends on the call's return/argument alias contract; leaving that conversion hidden in ABI
/// materialization would give EIR no value to transfer or release after the call.
pub(super) fn coerce_scalar_arg_to_param_storage(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    value: LoweredValue,
    arg: &Expr,
) -> LoweredValue {
    let Some((_, param_ty)) = sig.params.get(index) else {
        return value;
    };
    // A by-reference parameter must receive the caller's storage, not a converted temporary,
    // so declared-parameter scalar binding never applies to one. The checker keeps those on
    // the strict path for the same reason.
    let bindable = sig.declared_params.get(index).copied().unwrap_or(false)
        && !sig.ref_params.get(index).copied().unwrap_or(false);
    let param_ty = param_ty.codegen_repr();
    if value.ir_type == IrType::I64 && param_ty == PhpType::Float {
        return coerce_to_float(ctx, value, arg);
    }
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    if matches!(param_ty, PhpType::Array(_) | PhpType::AssocArray { .. })
        && matches!(source_ty, PhpType::Mixed | PhpType::Union(_))
    {
        return coerce_container_to_mixed_payload(
            ctx,
            value,
            &source_ty,
            &param_ty,
            arg.span,
        );
    }
    if param_ty == PhpType::Str && matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) {
        let parameter = ctx.intern_string(&sig.params[index].0);
        let guard = if ctx.argument_strict_types {
            Op::StrictStringArgumentGuard
        } else { Op::StringArgumentGuard };
        ctx.emit_void(guard, vec![value.value], Some(Immediate::Data(parameter)), guard.default_effects(), Some(arg.span));
        return coerce_to_string(ctx, value, arg);
    }
    if bindable {
        if let Some(cast) = crate::types::param_binding::scalar_param_cast(&param_ty, &source_ty) {
            return apply_scalar_param_cast(ctx, cast, value, Some(arg.span));
        }
    }
    value
}

/// Applies a declared-parameter scalar binding to an already-lowered argument value.
///
/// The conversion is the one elephc emits for the equivalent explicit cast, which is why the
/// binding is expressed as a `CastType`: `(string)` and `(bool)` are total over the scalar
/// sources `crate::types::param_binding` admits, so no runtime failure path is needed here.
fn apply_scalar_param_cast(
    ctx: &mut LoweringContext<'_, '_>,
    cast: CastType,
    value: LoweredValue,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    match cast {
        CastType::String => coerce_to_string_at_span(ctx, value, span),
        CastType::Bool => lower_truthy_bool(ctx, value, span),
        // `param_binding::scalar_param_cast` only ever reports the two total scalar casts.
        CastType::Int | CastType::Float | CastType::Array => value,
    }
}

/// Normalizes reordered call operands to their declared scalar parameter storage.
///
/// Named and spread arguments are evaluated in source order and then reordered, so their
/// int-to-float and Mixed-to-string conversions happen here in parameter order. By-reference
/// parameters and the variadic tail remain untouched. String conversions become owned EIR
/// values so normal alias-aware call cleanup can transfer or release them safely.
pub(super) fn coerce_operands_to_params(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    mut operands: Vec<crate::ir::ValueId>,
) -> Vec<crate::ir::ValueId> {
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    let limit = operands.len().min(regular_param_count);
    for index in 0..limit {
        if sig.ref_params.get(index).copied().unwrap_or(false) {
            continue;
        }
        let value = operands[index];
        // A declared PHP array does not establish an indexed physical layout.
        // Preserve the actual Array/Hash payload through an owned raw view. The
        // storage-widening helper assumes indexed layout and cannot bind a PHP
        // array decoded as Hash (such as a Parallel transfer).
        if sig.params.get(index).is_some_and(|(_, ty)| matches!(ty.codegen_repr(),
            PhpType::Array(_) | PhpType::AssocArray { .. }))
            && matches!(ctx.builder.value_php_type(value).codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        {
            let ty = sig.params[index].1.clone();
            let unboxed = ctx.emit_value(
                Op::MixedUnbox, vec![value], None, ty,
                Op::MixedUnbox.default_effects(), None,
            );
            let source = lowered_value_from_id(ctx, value);
            if ctx.value_is_owning_temporary(source) {
                crate::ir_lower::ownership::release_if_owned(ctx, source, None);
            }
            operands[index] = super::arg_evaluation::hold_argument_snapshot(
                ctx, unboxed, Span::dummy(),
            );
            continue;
        }
        let binding_expr = Expr::new(ExprKind::Variable(format!("#arg{index}")), Span::dummy());
        let lowered = lowered_value_from_id(ctx, value);
        operands[index] = coerce_scalar_arg_to_param_storage(ctx, sig, index, lowered, &binding_expr).value;
        if operands[index] != value {
            let converted = LoweredValue {
                value: operands[index],
                ir_type: ctx.builder.value_type(operands[index]),
            };
            if ctx.value_is_owning_temporary(converted) {
                operands[index] = super::arg_evaluation::hold_argument_snapshot(
                    ctx, converted, crate::span::Span::dummy(),
                );
            }
        }
    }
    operands
}

/// Widens local indexed-array storage before passing it to an `array<mixed>` ref parameter.
pub(super) fn lower_by_ref_array_arg_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    arg: &Expr,
) -> Option<crate::ir::ValueId> {
    if !sig.ref_params.get(index).copied().unwrap_or(false) {
        return None;
    }
    let (_, param_ty) = sig.params.get(index)?;
    let ExprKind::Variable(name) = &arg.kind else {
        return None;
    };
    if !by_ref_array_arg_needs_mixed_storage(ctx, name, param_ty) {
        return None;
    }
    let array_ty = PhpType::Array(Box::new(PhpType::Mixed));
    let local = ctx.load_local(name, Some(arg.span));
    let converted = ctx.emit_value(
        Op::ArrayToMixed,
        vec![local.value],
        None,
        array_ty.clone(),
        Op::ArrayToMixed.default_effects(),
        Some(arg.span),
    );
    ctx.store_call_normalized_local(name, converted, array_ty, Some(arg.span));
    Some(ctx.load_local(name, Some(arg.span)).value)
}

/// Lowers `$array[$index]` as a direct by-reference argument cell address.
pub(super) fn lower_by_ref_array_element_arg_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    index: usize,
    arg: &Expr,
) -> Option<crate::ir::ValueId> {
    if !sig.ref_params.get(index).copied().unwrap_or(false) {
        return None;
    }
    let ExprKind::ArrayAccess { array, index: element_index } = &arg.kind else {
        return None;
    };
    let ExprKind::Variable(array_name) = &array.kind else {
        return None;
    };
    let PhpType::Array(elem_ty) = ctx.local_type(array_name).codegen_repr() else {
        return None;
    };
    let (_, param_ty) = sig.params.get(index)?;
    let element_ty = match normalize_value_php_type(*elem_ty) {
        PhpType::Void => normalize_value_php_type(param_ty.codegen_repr()),
        other => other,
    };
    let array_value = ctx.load_local(array_name, Some(array.span));
    let element_index = lower_expr(ctx, element_index);
    let element_index = coerce_to_int_at_span(ctx, element_index, Some(arg.span));
    let value = ctx
        .builder
        .emit_with_effects(
            Op::ArrayElemAddr,
            vec![array_value.value, element_index.value],
            None,
            IrType::I64,
            element_ty,
            Ownership::NonHeap,
            Op::ArrayElemAddr.default_effects(),
            Some(arg.span),
        )
        .expect("array_elem_addr produces a value");
    Some(value)
}

/// Returns true when a local array must be converted before a by-reference call.
pub(super) fn by_ref_array_arg_needs_mixed_storage(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    param_ty: &PhpType,
) -> bool {
    let PhpType::Array(param_elem) = param_ty.codegen_repr() else {
        return false;
    };
    if param_elem.codegen_repr() != PhpType::Mixed {
        return false;
    }
    let PhpType::Array(local_elem) = ctx.local_type(name).codegen_repr() else {
        return false;
    };
    local_elem.codegen_repr() != PhpType::Mixed
}

/// Lowers positional call arguments with omitted optional defaults and variadic tail packing.
pub(super) fn lower_args_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    lower_args_with_signature_options(ctx, sig, args, false)
}

/// Lowers arguments for an EIR callable whose variadic capture uses a tagged Mixed ABI slot.
pub(super) fn lower_args_with_eir_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    lower_args_with_eir_abi(ctx, None, sig, args)
}

/// Keeps PHP argument planning separate from a user function's effective boxed EIR ABI.
pub(super) fn lower_args_with_eir_user_function_signature(
    ctx: &mut LoweringContext<'_, '_>,
    owner: &str,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    lower_args_with_eir_abi(ctx, Some(owner), sig, args)
}

fn lower_args_with_eir_abi(
    ctx: &mut LoweringContext<'_, '_>,
    owner: Option<&str>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    let Some(sig) = sig else {
        return lower_args(ctx, args);
    };
    let eir_sig = if let Some(owner) = owner {
        crate::ir_lower::function::eir_signature_with_php_param_contracts(owner, sig, ctx.callable_param_sigs)
    } else {
        crate::ir_lower::function::eir_signature_with_associative_variadic_storage(sig)
    };
    let mut operands = lower_args_with_signature(ctx, Some(sig), args);
    // Only a resolved user function supplies the authoritative EIR parameter ABI here.
    // Native constructors/method intrinsics may expose a permissive PHP signature while
    // their dedicated lowering still owns scalar/string materialization.
    let boxed_regular_count = if owner.is_some() {
        crate::types::call_args::regular_param_count(sig).min(operands.len())
    } else {
        0
    };
    for index in 0..boxed_regular_count {
        if eir_sig.ref_params.get(index).copied().unwrap_or(false)
            || eir_sig.params[index].1.codegen_repr() != PhpType::Mixed
            || ctx.builder.value_php_type(operands[index]).codegen_repr() == PhpType::Mixed
        {
            continue;
        }
        // Expose allocating ABI conversions to EIR so both success and throw argument-cleanup
        // edges can release them. Source expressions have already been evaluated exactly once.
        let source = lowered_value_from_id(ctx, operands[index]);
        let span = args.first().map(|arg| arg.span);
        operands[index] = ctx.box_value_as_mixed(source, PhpType::Mixed, span).value;
    }
    let Some(variadic_name) = eir_sig.variadic.as_deref() else {
        return operands;
    };
    let Some(variadic_index) = eir_sig
        .params
        .iter()
        .position(|(name, _)| name == variadic_name)
    else {
        return operands;
    };
    if eir_sig
        .ref_params
        .get(variadic_index)
        .copied()
        .unwrap_or(false)
    {
        return operands;
    }
    let Some(value) = operands.get(variadic_index).copied() else {
        return operands;
    };
    let target_ty = eir_sig.params[variadic_index].1.clone();
    let source_ty = ctx.builder.value_php_type(value).codegen_repr();
    let conversion = match (&target_ty, &source_ty) {
        (PhpType::AssocArray { value: target_value, .. }, PhpType::Array(_))
            if target_value.codegen_repr() == PhpType::Mixed => Some(Op::ArrayToHash),
        (PhpType::AssocArray { value: target_value, .. }, PhpType::AssocArray { value, .. })
            if target_value.codegen_repr() == PhpType::Mixed
                && value.codegen_repr() != PhpType::Mixed =>
        {
            Some(Op::HashToMixed)
        }
        _ => None,
    };
    if let Some(op) = conversion {
        let span = args.first().map(|arg| arg.span);
        let hash = ctx.emit_value(
            op,
            vec![value],
            None,
            target_ty,
            op.default_effects(),
            span,
        );
        operands[variadic_index] = hash.value;
    }
    operands
}

/// Lowers arguments while preserving omission of trailing default-only parameter slots.
pub(super) fn lower_args_with_signature_trimming_trailing_defaults(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    lower_args_with_signature_options(ctx, sig, args, true)
}

/// Applies shared argument planning with optional elision of trailing defaults.
fn lower_args_with_signature_options(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
    trim_trailing_defaults: bool,
) -> Vec<crate::ir::ValueId> {
    if sig.is_some() && !args.is_empty() {
        return super::arg_evaluation::protect_argument_evaluation(ctx, args[0].span, |ctx| {
            lower_args_with_signature_options_inner(ctx, sig, args, trim_trailing_defaults)
        });
    }
    lower_args_with_signature_options_inner(ctx, sig, args, trim_trailing_defaults)
}

fn lower_args_with_signature_options_inner(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
    trim_trailing_defaults: bool,
) -> Vec<crate::ir::ValueId> {
    let Some(sig) = sig else {
        return lower_args(ctx, args);
    };
    let literal_bound = rewrite_literal_param_bindings(sig, args);
    let args = literal_bound.as_deref().unwrap_or(args);
    if crate::types::call_args::has_named_args(args) {
        let operands = if trim_trailing_defaults {
            lower_named_args_with_signature_options(ctx, sig, args, true)
        } else {
            lower_named_args_with_signature(ctx, sig, args)
        };
        return coerce_operands_to_params(ctx, sig, operands);
    }
    if let Some(operands) = lower_positional_spread_args_with_signature(ctx, sig, args) {
        return coerce_operands_to_params(ctx, sig, operands);
    }
    let static_spread_args = if has_static_call_spread_args(args) {
        Some(expand_static_call_spread_args(args))
    } else {
        None
    };
    let args = static_spread_args.as_deref().unwrap_or(args);
    if let Some(operands) = lower_assoc_spread_only_args(ctx, sig, args) {
        return coerce_operands_to_params(ctx, sig, operands);
    }
    if args.iter().any(is_spread_arg) {
        return lower_args(ctx, args);
    }
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    let fixed_arg_count = if sig.variadic.is_some() {
        args.len().min(regular_param_count)
    } else {
        args.len()
    };
    if sig.variadic.is_none() && fixed_arg_count >= regular_param_count {
        let operands = args
            .iter()
            .enumerate()
            .map(|(index, arg)| lower_arg_source_with_signature(ctx, sig, index, arg))
            .collect();
        return coerce_operands_to_params(ctx, sig, operands);
    }
    let mut operands: Vec<crate::ir::ValueId> = args[..fixed_arg_count]
        .iter()
        .enumerate()
        .map(|(index, arg)| lower_arg_source_with_signature(ctx, sig, index, arg))
        .collect();
    if !trim_trailing_defaults {
        for idx in fixed_arg_count..regular_param_count {
            let Some(Some(default)) = sig.defaults.get(idx) else {
                break;
            };
            operands.push(lower_expr(ctx, default).value);
        }
    }
    if sig.variadic.is_some() {
        let tail = if args.len() > regular_param_count {
            &args[regular_param_count..]
        } else {
            &[]
        };
        let source_values = prepare_variadic_tail_values(ctx, sig, tail);
        operands = coerce_operands_to_params(ctx, sig, operands);
        operands.push(lower_variadic_tail_array_from_values(ctx, sig, tail, &source_values).value);
        return operands;
    }
    coerce_operands_to_params(ctx, sig, operands)
}
