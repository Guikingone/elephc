//! Purpose:
//! Binds array_push to a live property or element reference before evaluating its values.
//!
//! Called from:
//! - The by-reference builtin place dispatcher.
//!
//! Key details:
//! - A scoped managed alias preserves argument-selected storage and partial writes on throws.
//! - Managed cells preserve selection after object and index snapshots are retired.

use super::*;
use super::super::{array_push_spread, begin_call_argument_evaluation,
    container_property_slot_is_addressable, prepare_scoped_addressable_ref_array_receiver};

/// Routes native array places through a managed alias instead of an early value snapshot.
pub(super) fn lower_live_array_push_place(
    ctx: &mut LoweringContext<'_, '_>, name: &Name, sig: &FunctionSig,
    args: &[Expr], expr: &Expr,
) -> Option<LoweredValue> {
    let definition = crate::builtins::registry::lookup(name.as_str())?;
    if !matches!(definition.spec.semantics.lowering,
        crate::builtins::semantics::BuiltinLowering::Runtime(
            crate::ir::RuntimeCallTarget::Function(crate::ir::RuntimeFnId::ArrayPush)))
    { return None; }
    let (index, place) = args.iter().enumerate().find_map(|(index, arg)| {
        let (parameter, place) = ref_param_binding(sig, index, arg)?;
        (parameter == 0 && is_candidate_place_shape(place)).then_some((index, place))
    })?;
    let mixed_place = static_place_type(ctx, place)
        .is_some_and(|ty| ty.codegen_repr() == PhpType::Mixed);
    if (!is_array_place(ctx, place, &sig.params[0].1) && !mixed_place)
        || !addressable(ctx, place) { return None; }
    begin_call_argument_evaluation(ctx);
    let mut snapshots = Vec::new();
    let frozen = freeze_place(ctx, place, &mut snapshots);
    let (alias, _) = prepare_scoped_addressable_ref_array_receiver(ctx, &frozen)
        .expect("preflighted native array place must be addressable");
    if let ExprKind::Variable(name) = &alias.kind {
        if ctx.ref_cell_owner_slot(name).is_some() {
            // The managed cell outlives its selected object and index snapshots independently.
            // Clear their values while leaving the outer unwind records balanced below the lease.
            for snapshot in snapshots.iter().rev() {
                let slot = ctx.local_slots[snapshot];
                ctx.emit_void(Op::ReleaseLocalSlot, Vec::new(), Some(Immediate::LocalSlot(slot)),
                    Op::ReleaseLocalSlot.default_effects(), Some(expr.span));
            }
        }
    }
    let mut rewritten = args.to_vec();
    rewritten[index] = match &args[index].kind {
        ExprKind::NamedArg { name, .. } => Expr::new(ExprKind::NamedArg {
            name: name.clone(), value: Box::new(alias),
        }, args[index].span),
        _ => alias,
    };
    let result = lower_function_call(ctx, name, &rewritten, expr);
    Some(array_push_spread::finish_scope(ctx, result))
}

/// Proves that reference preparation can bind the complete place without a value-fetch fallback.
fn addressable(ctx: &LoweringContext<'_, '_>, place: &Expr) -> bool {
    match &place.kind {
        ExprKind::Variable(_) => true,
        ExprKind::PropertyAccess { object, property } =>
            container_property_slot_is_addressable(ctx, object, property),
        ExprKind::StaticPropertyAccess { .. } => true,
        ExprKind::ArrayAccess { array, .. } => addressable(ctx, array),
        _ => false,
    }
}

/// Captures object identity and index values at their original argument evaluation point.
fn freeze_place(ctx: &mut LoweringContext<'_, '_>, place: &Expr, snapshots: &mut Vec<String>) -> Expr {
    match &place.kind {
        ExprKind::PropertyAccess { object, property } => {
            let temp = array_push_spread::stage_value(ctx, object);
            snapshots.push(temp.clone());
            Expr::new(ExprKind::PropertyAccess {
                object: Box::new(Expr::new(ExprKind::Variable(temp), object.span)),
                property: property.clone(),
            }, place.span)
        }
        ExprKind::ArrayAccess { array, index } => {
            let array = freeze_place(ctx, array, snapshots);
            let temp = array_push_spread::stage_value(ctx, index);
            snapshots.push(temp.clone());
            Expr::new(ExprKind::ArrayAccess {
                array: Box::new(array),
                index: Box::new(Expr::new(ExprKind::Variable(temp), index.span)),
            }, place.span)
        }
        _ => place.clone(),
    }
}
