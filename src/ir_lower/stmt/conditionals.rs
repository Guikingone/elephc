//! Purpose:
//! If-chain lowering and loop-entry storage contracts.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers an `if` / `elseif` / `else` chain and terminates unreachable merge blocks explicitly.
pub(super) fn lower_if(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    then_body: &[Stmt],
    elseif_clauses: &[(Expr, Vec<Stmt>)],
    else_body: Option<&[Stmt]>,
    span: Span,
) {
    let merge = ctx.builder.create_named_block("if.merge", Vec::new());
    let merge_reachable = lower_if_chain(
        ctx,
        condition,
        then_body,
        elseif_clauses,
        else_body,
        merge,
        span,
    );
    ctx.builder.position_at_end(merge);
    if !merge_reachable {
        ctx.builder.terminate(Terminator::Unreachable);
    }
    ctx.clear_static_callable_locals();
}

/// Recursively emits one condition node in an `if` chain and reports whether the merge is reachable.
pub(super) fn lower_if_chain(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    then_body: &[Stmt],
    elseif_clauses: &[(Expr, Vec<Stmt>)],
    else_body: Option<&[Stmt]>,
    merge: BlockId,
    span: Span,
) -> bool {
    let cond_value = lower_expr(ctx, condition);
    let cond_value = ctx.truthy_consuming(cond_value, Some(condition.span));
    let split_initialized = ctx.initialized_slots_snapshot();
    let then_block = ctx.builder.create_named_block("if.then", Vec::new());
    let else_block = ctx.builder.create_named_block("if.else", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: cond_value.value,
        then_target: then_block,
        then_args: Vec::new(),
        else_target: else_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(then_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    lower_block(ctx, then_body);
    let then_initialized = ctx.initialized_slots_snapshot();
    let mut merge_reachable = false;
    let then_reachable = !ctx.builder.insertion_block_is_terminated();
    if then_reachable {
        merge_reachable = true;
        branch_to(ctx, merge);
    }

    ctx.clear_static_callable_locals();
    ctx.builder.position_at_end(else_block);
    ctx.restore_initialized_slots(split_initialized.clone());
    let else_reachable =
        if let Some(((next_condition, next_body), rest)) = elseif_clauses.split_first() {
            lower_if_chain(ctx, next_condition, next_body, rest, else_body, merge, span)
        } else if let Some(else_body) = else_body {
            lower_block(ctx, else_body);
            if !ctx.builder.insertion_block_is_terminated() {
                branch_to(ctx, merge);
                true
            } else {
                false
            }
        } else {
            lower_noop(ctx, span);
            if !ctx.builder.insertion_block_is_terminated() {
                branch_to(ctx, merge);
                true
            } else {
                false
            }
        };
    merge_reachable |= else_reachable;
    let else_initialized = ctx.initialized_slots_snapshot();
    ctx.restore_initialized_slots(merge_initialized_slots(
        &split_initialized,
        then_initialized,
        then_reachable,
        else_initialized,
        else_reachable,
    ));
    merge_reachable
}

/// Merges definitely-initialized locals from the reachable branches of an `if`.
pub(super) fn merge_initialized_slots(
    split_initialized: &HashSet<LocalSlotId>,
    then_initialized: HashSet<LocalSlotId>,
    then_reachable: bool,
    else_initialized: HashSet<LocalSlotId>,
    else_reachable: bool,
) -> HashSet<LocalSlotId> {
    match (then_reachable, else_reachable) {
        (true, true) => then_initialized
            .intersection(&else_initialized)
            .copied()
            .collect(),
        (true, false) => then_initialized,
        (false, true) => else_initialized,
        (false, false) => split_initialized.clone(),
    }
}

/// Lowers a residual `ifdef`; normally the conditional pass removes these first.
pub(super) fn lower_ifdef(
    ctx: &mut LoweringContext<'_, '_>,
    _symbol: &str,
    then_body: &[Stmt],
    else_body: Option<&[Stmt]>,
    _span: Span,
) {
    if !then_body.is_empty() {
        lower_block(ctx, then_body);
    } else if let Some(else_body) = else_body {
        lower_block(ctx, else_body);
    }
    ctx.clear_static_callable_locals();
}

/// Materializes the checker-recorded storage contract before entering a loop.
///
/// Indexed and associative arrays are promoted in place so existing elements use boxed payload
/// cells. A whole-value `Mixed` contract uses the ordinary retaining store, allowing loop-carried
/// container-kind changes to share the same fixed frame representation.
pub(super) fn apply_loop_storage_contracts(
    ctx: &mut LoweringContext<'_, '_>,
    loop_span: Span,
    span: Option<Span>,
) {
    let contracts = ctx
        .loop_storage_types
        .get(&(ctx.loop_storage_scope.clone(), loop_span))
        .cloned()
        .unwrap_or_default();
    for (name, target_ty) in contracts {
        if !ctx.local_slots.contains_key(&name) {
            continue;
        }
        let source_ty = ctx.local_type(&name).codegen_repr();
        if source_ty == target_ty.codegen_repr() {
            ctx.set_local_type(&name, target_ty);
            continue;
        }
        let source = ctx.load_local(&name, span);
        let target_repr = target_ty.codegen_repr();
        match (&source_ty, &target_repr) {
            (PhpType::Array(_), PhpType::AssocArray { .. }) => {
                let converted = ctx.emit_value(
                    Op::ArrayToHash,
                    vec![source.value],
                    None,
                    target_ty.clone(),
                    Op::ArrayToHash.default_effects(),
                    span,
                );
                ctx.store_mutated_local(&name, converted, target_ty, span);
            }
            (PhpType::Array(_), PhpType::Array(target_element))
                if target_element.codegen_repr() == PhpType::Mixed =>
            {
                let converted = ctx.emit_value(
                    Op::ArrayToMixed,
                    vec![source.value],
                    None,
                    target_ty.clone(),
                    Op::ArrayToMixed.default_effects(),
                    span,
                );
                ctx.store_mutated_local(&name, converted, target_ty, span);
            }
            (
                PhpType::AssocArray { .. },
                PhpType::AssocArray {
                    value: target_value,
                    ..
                },
            ) if target_value.codegen_repr() == PhpType::Mixed => {
                let converted = ctx.emit_value(
                    Op::HashToMixed,
                    vec![source.value],
                    None,
                    target_ty.clone(),
                    Op::HashToMixed.default_effects(),
                    span,
                );
                ctx.store_mutated_local(&name, converted, target_ty, span);
            }
            (_, PhpType::Mixed) => {
                let converted = ctx.box_value_as_mixed(source, target_ty.clone(), span);
                ctx.store_local(&name, converted, target_ty, span);
            }
            // The contract cannot be materialized for the representation this local actually
            // holds — the only remaining shapes disagree on container kind (an `AssocArray`
            // local against an `Array(Mixed)` contract, or a non-container local). Re-declaring
            // the type here would leave the slot holding a hash while every later read is typed
            // `array<mixed>`, so the write-site promotion reads storage it has already released.
            // Leave the local alone and let its own assignment path convert it, mirroring the
            // heap-kind guard the pre-fixed-point widening applied before promoting.
            _ => {}
        }
    }
}

