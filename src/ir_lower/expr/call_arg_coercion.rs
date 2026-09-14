//! Purpose:
//! General call argument lowering and parameter storage coercion.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

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
    if let Some(value) = lower_by_ref_array_element_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    if let Some(value) = lower_by_ref_array_arg_with_signature(ctx, sig, index, arg) {
        return value;
    }
    let lowered = lower_expr(ctx, arg);
    coerce_scalar_arg_to_param_storage(ctx, sig, index, lowered, arg).value
}

/// Coerces a positional argument to the parameter storage owned explicitly by EIR.
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
    let declared_param_ty = param_ty;
    let param_ty = declared_param_ty.codegen_repr();
    if value.ir_type == IrType::I64 && param_ty == PhpType::Float {
        return coerce_to_float(ctx, value, arg);
    }
    let source_php_ty = ctx.builder.value_php_type(value.value).clone();
    let source_ty = source_php_ty.codegen_repr();
    if bindable && param_ty == PhpType::Mixed && source_ty != PhpType::Mixed {
        return ctx.box_value_as_mixed(value, PhpType::Mixed, Some(arg.span));
    }
    if param_ty == PhpType::Str
        && matches!(source_ty, PhpType::Mixed | PhpType::TaggedScalar | PhpType::Union(_))
    {
        return coerce_to_string(ctx, value, arg);
    }
    if (bindable || matches!(source_ty, PhpType::Object(_)))
        && !param_accepts_object_without_string_coercion(ctx, declared_param_ty, &source_ty)
    {
        if let Some(cast) =
            crate::types::param_binding::scalar_param_cast(declared_param_ty, &source_php_ty)
        {
            return apply_scalar_param_cast(
                ctx,
                cast,
                value,
                declared_param_ty,
                Some(arg.span),
            );
        }
    }
    if sig.ref_params.get(index).copied().unwrap_or(false) {
        return value;
    }
    let value = if bindable {
        guard_nominal_object_param(ctx, value, declared_param_ty, Some(arg.span))
    } else {
        value
    };
    let value = if bindable {
        guard_nullable_int_param(ctx, value, declared_param_ty, Some(arg.span))
    } else {
        value
    };
    let value = if bindable {
        guard_gradual_union_param(ctx, value, declared_param_ty, Some(arg.span))
    } else {
        value
    };
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    let value = crate::ir_lower::expr::coerce_container_to_mixed_payload(
        ctx,
        value,
        &source_ty,
        &param_ty,
        Some(arg.span),
    );
    crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
        ctx,
        value,
        &param_ty,
        Some(arg.span),
    )
}

/// Checks the active tag of a boxed gradual value against a declared scalar/array union.
fn guard_gradual_union_param(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    expected: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let source_type = ctx.builder.value_php_type(value.value).clone();
    if !crate::types::param_binding::gradual_union_requires_runtime_param_guard(
        expected,
        &source_type,
    ) {
        return value;
    }
    let type_name = ctx.intern_string(&expected.to_string());
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        Some(Immediate::TypeName(type_name)),
        expected.clone(),
        effects_lookup::runtime_effects(),
        span,
    )
}

/// Checks a compact nullable integer before passing it to a declared `int` parameter.
fn guard_nullable_int_param(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    expected: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let source_type = ctx.builder.value_php_type(value.value).codegen_repr();
    if !crate::types::param_binding::nullable_int_requires_runtime_param_guard(
        expected,
        &source_type,
    ) {
        return value;
    }
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        None,
        PhpType::Int,
        effects_lookup::runtime_effects(),
        span,
    )
}

/// Inserts the runtime class guard required by a declared object parameter.
///
/// The checker only admits this path for by-value source declarations. Nullable target storage
/// is boxed again after the object check so the callee still receives its declared union ABI.
fn guard_nominal_object_param(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    expected: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let source_type = ctx.builder.value_php_type(value.value).codegen_repr();
    let gradual_source =
        crate::types::param_binding::gradual_object_requires_runtime_nominal_guard(
            expected,
            &source_type,
        );
    let needs_nominal_guard = gradual_source
        || crate::types::param_binding::object_requires_runtime_nominal_guard(
            expected,
            &source_type,
        );
    if !needs_nominal_guard
        || param_accepts_object_without_string_coercion(ctx, expected, &source_type)
    {
        return value;
    }
    let Some(target_name) =
        crate::types::param_binding::nominal_object_boundary_target(expected)
    else {
        return value;
    };
    let target_data = ctx.intern_class_name(&target_name);
    let guarded_type = if gradual_source {
        expected.clone()
    } else {
        PhpType::Object(target_name.clone())
    };
    let guarded = ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        Some(Immediate::NominalObject {
            target: target_data,
            boundary: crate::ir::NominalObjectBoundary::Parameter,
        }),
        guarded_type,
        effects_lookup::runtime_effects(),
        span,
    );
    if !gradual_source && expected.codegen_repr() == PhpType::Mixed {
        ctx.box_value_as_mixed(guarded, expected.clone(), span)
    } else {
        guarded
    }
}

/// Returns whether an object argument already satisfies a declared parameter without selecting
/// a weakly coercive `string` arm.
///
/// PHP resolves a union by identity before coercion: an object implementing `IteratorAggregate`
/// remains an object for `string|iterable`, even when it also implements `Stringable`. This
/// mirrors the checker's object/iterable/callable acceptance so lowering only consumes the
/// checker's weak-string proof when no non-string member already accepts the object.
pub(in crate::ir_lower) fn param_accepts_object_without_string_coercion(
    ctx: &LoweringContext<'_, '_>,
    expected: &PhpType,
    actual: &PhpType,
) -> bool {
    let PhpType::Object(actual_name) = actual else {
        return false;
    };
    match expected {
        PhpType::Mixed => true,
        PhpType::Union(members) => members
            .iter()
            .any(|member| param_accepts_object_without_string_coercion(ctx, member, actual)),
        PhpType::Object(expected_name) => {
            expected_name.is_empty()
                || class_extends_class(ctx, actual_name, expected_name)
                || object_name_satisfies_interface_for_ir(ctx, actual_name, expected_name)
        }
        PhpType::Iterable => {
            object_name_satisfies_interface_for_ir(ctx, actual_name, "Traversable")
                || object_name_satisfies_interface_for_ir(ctx, actual_name, "Iterator")
                || object_name_satisfies_interface_for_ir(ctx, actual_name, "IteratorAggregate")
        }
        PhpType::Callable => actual.is_closure_object(),
        _ => false,
    }
}

/// Applies a declared-parameter scalar binding to an already-lowered argument value.
///
/// The conversion is the one elephc emits for the equivalent explicit cast, which is why the
/// binding is expressed as a `CastType`: scalar inputs use the same total `(string)` / `(bool)`
/// conversions, while objects reach `(string)` only after checker proof of `Stringable`.
/// When PHP selects a scalar member of a union, the converted value is boxed back into the
/// union's `Mixed` ABI storage before the callee receives it.
pub(crate) fn apply_scalar_param_cast(
    ctx: &mut LoweringContext<'_, '_>,
    cast: CastType,
    value: LoweredValue,
    declared_param_ty: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    if cast == CastType::String
        && value_is_nullable(ctx, value.value)
        && php_type_accepts_null(declared_param_ty)
    {
        return apply_nullable_string_param_cast(ctx, value, declared_param_ty, span);
    }
    let converted = match cast {
        CastType::String => coerce_to_string_at_span(ctx, value, span),
        CastType::Bool => lower_truthy_bool(ctx, value, span),
        // `param_binding::scalar_param_cast` only ever reports the two total scalar casts.
        CastType::Int | CastType::Float | CastType::Array | CastType::Object => value,
    };
    if declared_param_ty.codegen_repr() == PhpType::Mixed
        && ctx
            .builder
            .value_php_type(converted.value)
            .codegen_repr()
            != PhpType::Mixed
    {
        ctx.box_value_as_mixed(converted, declared_param_ty.clone(), span)
    } else {
        converted
    }
}

/// Preserves PHP null while coercing every non-null scalar member to a string union arm.
///
/// The source has already been evaluated once. Each control-flow edge consumes that same value:
/// the null edge stores a boxed null and releases an owning source cell, while the non-null edge
/// uses the ordinary string coercer and its existing ownership cleanup.
fn apply_nullable_string_param_cast(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    declared_param_ty: &PhpType,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let source_span = span.unwrap_or_else(crate::span::Span::dummy);
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![value.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        span,
    );
    let temp_name = ctx.declare_owned_hidden_temp(declared_param_ty.clone());
    let split_initialized = ctx.initialized_slots_snapshot();
    let null_block = ctx
        .builder
        .create_named_block("param_cast.null", Vec::new());
    let scalar_block = ctx
        .builder
        .create_named_block("param_cast.scalar", Vec::new());
    let merge_block = ctx
        .builder
        .create_named_block("param_cast.merge", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: null_block,
        then_args: Vec::new(),
        else_target: scalar_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(null_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let null_value = ctx.emit_value(
        Op::ConstNull,
        Vec::new(),
        None,
        PhpType::Void,
        Op::ConstNull.default_effects(),
        span,
    );
    store_value_into_temp(
        ctx,
        &temp_name,
        declared_param_ty.clone(),
        null_value,
        source_span,
    );
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    let null_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge_block);

    ctx.builder.position_at_end(scalar_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let converted = coerce_to_string_at_span(ctx, value, span);
    store_value_into_temp(
        ctx,
        &temp_name,
        declared_param_ty.clone(),
        converted,
        source_span,
    );
    let scalar_initialized = ctx.initialized_slots_snapshot();
    branch_to(ctx, merge_block);

    ctx.builder.position_at_end(merge_block);
    ctx.restore_initialized_slots(merge_initialized_slots_for_expr(
        &split_initialized,
        null_initialized,
        true,
        scalar_initialized,
        true,
    ));
    take_owned_temp(ctx, &temp_name, source_span)
}

/// Returns whether a declared parameter union admits PHP null without coercion.
fn php_type_accepts_null(ty: &PhpType) -> bool {
    match ty {
        PhpType::Mixed | PhpType::Void => true,
        PhpType::Union(members) => members.iter().any(php_type_accepts_null),
        _ => false,
    }
}

/// Normalizes reordered call operands to their declared parameter storage.
///
/// Named and spread arguments are evaluated in source order and then reordered, so their
/// int-to-float, boxed-scalar, and checked heap narrowing happen here in parameter order.
/// By-reference parameters and the variadic tail remain untouched. Allocating conversions become
/// owned EIR values so normal alias-aware call cleanup can transfer or release them safely.
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
        let Some((_, declared_param_ty)) = sig.params.get(index) else {
            continue;
        };
        let value = operands[index];
        let value = if sig.declared_params.get(index).copied().unwrap_or(false) {
            guard_nominal_object_param(ctx, LoweredValue {
                value,
                ir_type: ctx.builder.value_type(value),
            }, declared_param_ty, None)
        } else {
            LoweredValue {
                value,
                ir_type: ctx.builder.value_type(value),
            }
        };
        let value = if sig.declared_params.get(index).copied().unwrap_or(false) {
            guard_nullable_int_param(ctx, value, declared_param_ty, None)
        } else {
            value
        };
        operands[index] = value.value;
        let value = value.value;
        let operand_php_ty = ctx.builder.value_php_type(value).clone();
        let operand_ty = operand_php_ty.codegen_repr();
        let param_ty = declared_param_ty.codegen_repr();
        if sig.declared_params.get(index).copied().unwrap_or(false)
            && param_ty == PhpType::Mixed
            && operand_ty != PhpType::Mixed
        {
            let lowered = LoweredValue {
                value,
                ir_type: ctx.builder.value_type(value),
            };
            operands[index] = ctx
                .box_value_as_mixed(lowered, PhpType::Mixed, None)
                .value;
        } else if param_ty == PhpType::Float
            && matches!(operand_ty, PhpType::Int | PhpType::Bool)
        {
            let lowered = LoweredValue {
                value,
                ir_type: IrType::I64,
            };
            operands[index] = coerce_to_float_at_span(ctx, lowered, None).value;
        } else if param_ty == PhpType::Str
            && matches!(
                operand_ty,
                PhpType::Mixed | PhpType::TaggedScalar | PhpType::Union(_)
            )
        {
            let lowered = LoweredValue {
                value,
                ir_type: ctx.builder.value_type(value),
            };
            operands[index] = coerce_to_string_at_span(ctx, lowered, None).value;
        } else if (sig.declared_params.get(index).copied().unwrap_or(false)
            || matches!(operand_ty, PhpType::Object(_)))
            && !param_accepts_object_without_string_coercion(
                ctx,
                declared_param_ty,
                &operand_ty,
            )
        {
            // Same declared-parameter scalar binding the positional path applies, run here in
            // parameter order because named and spread arguments are lowered in source order
            // and only reordered afterwards.
            if let Some(cast) =
                crate::types::param_binding::scalar_param_cast(
                    declared_param_ty,
                    &operand_php_ty,
                )
            {
                let lowered = LoweredValue {
                    value,
                    ir_type: ctx.builder.value_type(value),
                };
                operands[index] =
                    apply_scalar_param_cast(ctx, cast, lowered, declared_param_ty, None).value;
            }
        }
        let lowered = LoweredValue {
            value: operands[index],
            ir_type: ctx.builder.value_type(operands[index]),
        };
        let source_ty = ctx.builder.value_php_type(lowered.value).codegen_repr();
        let lowered = crate::ir_lower::expr::coerce_container_to_mixed_payload(
            ctx,
            lowered,
            &source_ty,
            &param_ty,
            None,
        );
        operands[index] = crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
            ctx,
            lowered,
            &param_ty,
            None,
        )
        .value;
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
    if !sig.declared_params.get(index).copied().unwrap_or(false)
        || !by_ref_array_arg_needs_mixed_storage(ctx, name, param_ty)
    {
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
    let PhpType::Array(_) = ctx.local_type(array_name).codegen_repr() else {
        return None;
    };
    sig.params.get(index)?;
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
            PhpType::Pointer(None),
            Ownership::NonHeap,
            Op::ArrayElemAddr.default_effects(),
            Some(arg.span),
        )
        .expect("array_elem_addr produces a value");
    Some(value)
}

/// Returns true when a local array's concrete slots differ from a generic array parameter.
///
/// Callers must additionally establish that the parameter is source-declared. Internal
/// operations have typed mutation strategies and must retain the concrete representation their
/// backend selected rather than inheriting this declared-parameter normalization.
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
            .map(|(index, arg)| lower_arg_with_signature(ctx, sig, index, arg))
            .collect();
        return coerce_operands_to_params(ctx, sig, operands);
    }
    let mut operands: Vec<crate::ir::ValueId> = args[..fixed_arg_count]
        .iter()
        .enumerate()
        .map(|(index, arg)| lower_arg_with_signature(ctx, sig, index, arg))
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
        operands.push(lower_variadic_tail_array(ctx, sig, tail).value);
    }
    coerce_operands_to_params(ctx, sig, operands)
}
