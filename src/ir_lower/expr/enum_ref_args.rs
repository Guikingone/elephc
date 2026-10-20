//! Purpose:
//! Captures selected enum object-reference places and validates their current referents.
//!
//! Called from:
//! - `enum_static_args` before source evaluation and during parameter-order binding.
//!
//! Key details:
//! - An alias freezes the place once, not the value it held before later arguments ran.
//! - Ordinary reference assignment owns promotion, interior-cell leases and reference returns.
//! - Guards run after all source effects and never replace or coerce the caller's reference.

use super::*;

/// Returns an explicitly declared object contract, including a boxed physical signature.
fn object_contract(ctx: &LoweringContext<'_, '_>, signature: &FunctionSig, index: usize) -> Option<PhpType> {
    if !signature.declared_params.get(index).copied().unwrap_or(false) { return None; }
    let ty = match &signature.params.get(index)?.1 {
        ty @ PhpType::Object(_) => ty.clone(),
        PhpType::Mixed => ctx.type_expr_to_php_type_for_value(signature.param_type_exprs.get(index)?.as_ref()?),
        _ => return None,
    };
    matches!(ty, PhpType::Object(_)).then_some(ty)
}

/// Retains the selected reference place without applying an early object type guard.
pub(super) fn stage_argument(
    ctx: &mut LoweringContext<'_, '_>, signature: &FunctionSig, index: usize,
    source: &Expr, span: Span,
) -> Option<(ValueId, String)> {
    let ty = object_contract(ctx, signature, index)?;
    if let ExprKind::Variable(name) = &source.kind {
        // Branch restoration can recover an Object hint after another branch already
        // widened the cell to Mixed. The alias must inherit the real payload layout,
        // otherwise its guard boxes a Mixed cell as though it were a raw object.
        let payload = ctx.ref_cell_owner_slot(name)
            .or_else(|| ctx.local_slots.get(name).copied())
            .map(|slot| ctx.builder.local_php_type(slot));
        if payload.is_some_and(|ty| ty.codegen_repr() == PhpType::Mixed) {
            ctx.set_local_type(name, PhpType::Mixed);
        }
    }
    let alias = ctx.declare_synthetic_php_local(ty);
    crate::ir_lower::stmt::lower_ref_assign(ctx, &alias, source, span);
    if let Some(slot) = ctx.ref_cell_owner_slot(&alias) {
        register_owned_call_operand(ctx, slot, span);
        let payload = ctx.builder.local_php_type(slot);
        ctx.call_argument_evaluation_scopes.last_mut().expect("selected reference argument scope")
            .owners.push(crate::ir_lower::context::CallArgumentEvaluationOwner {
                value: None, borrow: None, temp_name: alias.clone(), slot, span,
                ref_cell_payload: Some(payload), scoped_receiver_alias: true,
            });
    }
    let value = ctx.load_local(&alias, Some(span));
    Some((value.value, alias))
}

/// Checks the live captured referent after every caller expression has completed.
pub(super) fn guard_argument(
    ctx: &mut LoweringContext<'_, '_>, signature: &FunctionSig, index: usize,
    alias: &str, class: &str, method: &str, span: Span,
) {
    let mut logical = signature.clone();
    logical.params[index].1 = object_contract(ctx, signature, index)
        .expect("staged object reference has a declared contract");
    let value = ctx.load_local(alias, Some(span));
    let value = if matches!(value.ir_type, IrType::Heap(IrHeapKind::Mixed | IrHeapKind::Union)) {
        ctx.emit_owned_value(Op::MixedClone, vec![value.value], None, PhpType::Mixed,
            Op::MixedClone.default_effects(), Some(span))
    } else { ctx.box_value_as_mixed(value, PhpType::Mixed, Some(span)) };
    let slot = enum_method_args::call_intermediate(ctx, value, span);
    let value = Expr::new(ExprKind::Variable(slot), span);
    enum_argument_guards::guard_parameter(ctx, &logical, index, &value, class, method, span);
}
