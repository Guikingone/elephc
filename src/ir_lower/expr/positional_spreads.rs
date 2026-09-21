//! Purpose:
//! Positional spread lowering and arity guards.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers one trailing indexed spread in a fixed-arity positional call.
pub(super) fn lower_positional_spread_args_with_signature(
    ctx: &mut LoweringContext<'_, '_>,
    sig: &FunctionSig,
    args: &[Expr],
) -> Option<Vec<crate::ir::ValueId>> {
    if sig.variadic.is_some() && !crate::func_args::sig_collects_surplus_args(sig) {
        return None;
    }
    let spread_idx = single_trailing_indexed_spread_arg(ctx, args)?;
    let regular_param_count = crate::types::call_args::regular_param_count(sig);
    if spread_idx > regular_param_count {
        return None;
    }
    let first_spread_param_idx = spread_idx;
    let required_len = required_positional_spread_len(sig, first_spread_param_idx, regular_param_count);
    let ExprKind::Spread(inner) = &args[spread_idx].kind else {
        return None;
    };
    if static_indexed_spread_len(inner).is_some_and(|len| len >= required_len) {
        return None;
    }

    let mut operands = Vec::with_capacity(regular_param_count);
    for (index, arg) in args[..spread_idx].iter().enumerate() {
        operands.push(lower_arg_with_signature(ctx, sig, index, arg));
    }

    let source_ty = indexed_spread_source_type(ctx, inner)?;
    // A `callable` source is a DESCRIPTOR, not an array. Boxing it hands the element reads to the
    // gradual path, which rebuilds slot 0 (the bound receiver) and slot 1 (the bare method name)
    // from the descriptor. The two steps an array source needs are skipped deliberately:
    // `narrow_gradual_indexed_spread_source` would unbox it as runtime tag 4 and read a
    // descriptor as an array, and the min-length guard would call `ArrayLen` on it — a
    // callable array is exactly two elements by construction, so there is nothing to check.
    let callable_source = matches!(source_ty, PhpType::Callable);
    let spread = lower_expr(ctx, inner);
    // Box a `callable` source so the narrowing below sees a gradual value, then let that
    // narrowing MATERIALIZE it: `__rt_mixed_spread_array` turns a callable descriptor into a real
    // two-element array. Handing the boxed descriptor downstream instead left the hidden variadic
    // tail slicing a descriptor, and the callee's own `count()` then received null.
    let spread = if callable_source {
        ctx.box_value_as_mixed(spread, PhpType::Mixed, Some(args[spread_idx].span))
    } else {
        spread
    };
    let spread = narrow_gradual_indexed_spread_source(ctx, spread, args[spread_idx].span);
    let spread_type = ctx.builder.value_php_type(spread.value);
    let temp_name = ctx.declare_hidden_temp(spread_type.clone());
    store_value_into_temp(ctx, &temp_name, spread_type, spread, args[spread_idx].span);
    let spread_expr = Expr::new(ExprKind::Variable(temp_name), inner.span);
    let spread_value = lower_expr(ctx, &spread_expr);
    emit_positional_spread_min_len_guard(
        ctx,
        spread_value.value,
        required_len,
        args[spread_idx].span,
    );

    for param_idx in first_spread_param_idx..regular_param_count {
        let element_idx = param_idx - first_spread_param_idx;
        let default = sig.defaults.get(param_idx).and_then(|default| default.as_ref());
        let expr = if let Some(default) = default {
            if element_idx < required_len {
                spread_element_expr_for_ir(
                    &spread_expr,
                    element_idx,
                    None,
                    false,
                    args[spread_idx].span,
                )
            } else {
                spread_element_or_default_expr_for_ir(
                    &spread_expr,
                    element_idx,
                    None,
                    false,
                    default.clone(),
                    args[spread_idx].span,
                )
            }
        } else {
            spread_element_expr_for_ir(
                &spread_expr,
                element_idx,
                None,
                false,
                args[spread_idx].span,
            )
        };
        operands.push(lower_expr(ctx, &expr).value);
    }

    if crate::func_args::sig_collects_surplus_args(sig) {
        operands.push(lower_hidden_variadic_spread_tail(
            ctx,
            &spread_expr,
            regular_param_count - first_spread_param_idx,
            args[spread_idx].span,
        ));
    }

    Some(operands)
}

/// Copies the part of an indexed spread that remains after the visible fixed parameters.
///
/// Argument-introspection lowering uses a compiler-private variadic slot to preserve surplus PHP
/// arguments. Calls still have a fixed source signature, so a runtime spread must materialize the
/// visible parameters individually and pass only its remaining suffix into that hidden slot.
fn lower_hidden_variadic_spread_tail(
    ctx: &mut LoweringContext<'_, '_>,
    spread_expr: &Expr,
    offset: usize,
    span: crate::span::Span,
) -> crate::ir::ValueId {
    let source = lower_expr(ctx, spread_expr);
    let offset = emit_i64_at_span(ctx, offset as i64, span);
    let target = crate::ir::RuntimeFnId::ArraySlice;
    ctx.emit_value(
        Op::RuntimeCall,
        vec![source.value, offset.value],
        Some(Immediate::RuntimeCall(
            crate::ir::RuntimeCallTarget::Function(target),
        )),
        PhpType::Array(Box::new(PhpType::Mixed)),
        target.effects(),
        Some(span),
    )
    .value
}

/// Returns the element count for a statically-known indexed spread source.
pub(super) fn static_indexed_spread_len(expr: &Expr) -> Option<usize> {
    match &expr.kind {
        ExprKind::ArrayLiteral(items) => Some(items.len()),
        _ => None,
    }
}

/// Returns the index of a single trailing positional spread that EIR can materialize.
pub(super) fn single_trailing_indexed_spread_arg(
    ctx: &LoweringContext<'_, '_>,
    args: &[Expr],
) -> Option<usize> {
    let spread_indices = args
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| matches!(arg.kind, ExprKind::Spread(_)).then_some(idx))
        .collect::<Vec<_>>();
    let [spread_idx] = spread_indices.as_slice() else {
        return None;
    };
    if *spread_idx + 1 != args.len() {
        return None;
    }
    let ExprKind::Spread(inner) = &args[*spread_idx].kind else {
        return None;
    };
    indexed_spread_source_type(ctx, inner)?;
    Some(*spread_idx)
}

/// Returns the indexed-array source type for spread-only EIR lowering.
pub(super) fn indexed_spread_source_type(
    ctx: &LoweringContext<'_, '_>,
    expr: &Expr,
) -> Option<PhpType> {
    let ty = match &expr.kind {
        ExprKind::Variable(name) => ctx.local_type(name),
        ExprKind::ArrayLiteral(items) => array_literal_type_for_ir(ctx, items, expr),
        // A property read is GRADUAL for this decision. The syntactic fallback below cannot see a
        // declared property type, so `...$this->controller` used to leave the spread path
        // entirely and lower as a single operand -- a hard `OperandCountMismatch` at compile
        // time. Treating it as gradual routes it through `__rt_mixed_spread_array`, which passes
        // a real array through, rebuilds a callable descriptor's two elements, and answers null
        // for anything else so the length guard raises PHP's own unpack error.
        ExprKind::PropertyAccess { .. }
        | ExprKind::NullsafePropertyAccess { .. }
        | ExprKind::DynamicPropertyAccess { .. } => PhpType::Mixed,
        _ => ctx
            .builtin_call_types
            .get(&(ctx.loop_storage_scope.clone(), expr.span))
            .cloned()
            .unwrap_or_else(|| infer_expr_type_syntactic(expr)),
    }
    .codegen_repr();
    // `Callable` joins the gradual shapes: its slot holds a descriptor, and a descriptor built
    // from PHP's `[$object, 'method']` syntax rebuilds both elements on demand (see
    // `lower_mixed_callable_slot_or_array_get`). The caller boxes it so those reads take the
    // gradual path.
    if matches!(
        ty,
        PhpType::Array(_) | PhpType::Mixed | PhpType::Union(_) | PhpType::Callable
    ) {
        Some(ty)
    } else {
        None
    }
}

/// Returns how many spread elements must exist to satisfy required parameters.
pub(super) fn required_positional_spread_len(
    sig: &FunctionSig,
    start_param_idx: usize,
    regular_param_count: usize,
) -> usize {
    (start_param_idx..regular_param_count)
        .rfind(|idx| sig.defaults.get(*idx).and_then(|default| default.as_ref()).is_none())
        .map(|idx| idx - start_param_idx + 1)
        .unwrap_or(0)
}

/// Emits a fatal guard when a positional spread is shorter than required parameters.
pub(super) fn emit_positional_spread_min_len_guard(
    ctx: &mut LoweringContext<'_, '_>,
    spread: crate::ir::ValueId,
    min_len: usize,
    span: crate::span::Span,
) {
    if min_len == 0 {
        return;
    }
    // `ArrayLen` reads an INDEXED array header. A gradual spread source can be hash storage —
    // `...$this->someAssoc` reaches here now that a property read counts as a spread source — and
    // handing that to `ArrayLen` fails EIR validation outright (`expected Heap(Array), actual
    // Heap(Hash)`) rather than producing a wrong count. The guard is an extra diagnostic, not a
    // correctness requirement, so a shape it cannot measure simply goes unguarded.
    if !matches!(
        ctx.builder.value_type(spread),
        crate::ir::IrType::Heap(crate::ir::IrHeapKind::Array)
    ) {
        return;
    }
    let len = ctx.emit_value(
        Op::ArrayLen,
        vec![spread],
        None,
        PhpType::Int,
        Op::ArrayLen.default_effects(),
        Some(span),
    );
    let min = emit_i64_at_span(ctx, min_len as i64, span);
    let has_required_args = ctx.emit_value(
        Op::ICmp,
        vec![len.value, min.value],
        Some(Immediate::CmpPredicate(CmpPredicate::Sge)),
        PhpType::Bool,
        Op::ICmp.default_effects(),
        Some(span),
    );
    let ok = ctx.builder.create_named_block("call.spread.len.ok", Vec::new());
    let fatal = ctx.builder.create_named_block("call.spread.len.fatal", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: has_required_args.value,
        then_target: ok,
        then_args: Vec::new(),
        else_target: fatal,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(fatal);
    let message = ctx.intern_string("Fatal error: too few arguments for spread call\n");
    ctx.builder.terminate(Terminator::Fatal { message });

    ctx.builder.position_at_end(ok);
}
