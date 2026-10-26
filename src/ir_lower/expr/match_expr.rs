//! Purpose:
//! Lazy match-expression lowering.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers a match expression with lazy arm-result evaluation.
pub(super) fn lower_match(
    ctx: &mut LoweringContext<'_, '_>,
    subject: &Expr,
    arms: &[(Vec<Expr>, Expr)],
    default: Option<&Expr>,
    expr: &Expr,
) -> LoweredValue {
    let subject = lower_expr(ctx, subject);
    // An owning temporary subject (`match (strtolower($t))`) is compared by every arm and then
    // dropped: it leaked one value per evaluation. It is parked in a synthetic local, which the
    // frame's cleanup also covers when an arm throws, and released after the merge.
    if ctx.value_is_owning_temporary(subject) {
        let subject_ty = ctx.builder.value_php_type(subject.value);
        let parked = ctx.declare_synthetic_php_local(subject_ty.clone());
        ctx.store_local(&parked, subject, subject_ty.clone(), Some(expr.span));
        let loaded = ctx.load_local(&parked, Some(expr.span));
        let value = lower_match_on_subject(ctx, loaded, arms, default, expr);
        let slot = ctx.declare_local(&parked, subject_ty);
        ctx.release_stored_local_value(&parked, slot, Some(expr.span));
        ctx.emit_void(
            Op::UnsetLocal,
            Vec::new(),
            Some(Immediate::LocalSlot(slot)),
            Op::UnsetLocal.default_effects(),
            Some(expr.span),
        );
        return value;
    }
    lower_match_on_subject(ctx, subject, arms, default, expr)
}

/// [`lower_match`] once the subject is a value the match may read as often as it needs.
fn lower_match_on_subject(
    ctx: &mut LoweringContext<'_, '_>,
    subject: LoweredValue,
    arms: &[(Vec<Expr>, Expr)],
    default: Option<&Expr>,
    expr: &Expr,
) -> LoweredValue {
    let result_type = match_merge_result_type(ctx, arms, default, expr);
    let temp_name = ctx.declare_owned_hidden_temp(result_type.clone());
    let merge = ctx.builder.create_named_block("match.merge", Vec::new());

    for (conditions, result) in arms {
        let result_block = ctx.builder.create_named_block("match.result", Vec::new());
        let mut fallthrough = ctx.builder.insertion_block();
        for condition in conditions {
            let next_test = ctx.builder.create_named_block("match.next", Vec::new());
            let condition = lower_expr(ctx, condition);
            let matched = ctx.emit_value(
                Op::StrictEq,
                vec![subject.value, condition.value],
                None,
                PhpType::Bool,
                Op::StrictEq.default_effects(),
                Some(expr.span),
            );
            ctx.builder.terminate(Terminator::CondBr {
                cond: matched.value,
                then_target: result_block,
                then_args: Vec::new(),
                else_target: next_test,
                else_args: Vec::new(),
            });
            ctx.builder.position_at_end(next_test);
            fallthrough = Some(next_test);
        }
        ctx.builder.position_at_end(result_block);
        super::lazy_branches::store_branch_expr_into_temp(
            ctx,
            &temp_name,
            result_type.clone(),
            result,
            expr.span,
        );
        branch_to(ctx, merge);
        if let Some(fallthrough) = fallthrough {
            ctx.builder.position_at_end(fallthrough);
        }
    }
    if let Some(default) = default {
        super::lazy_branches::store_branch_expr_into_temp(
            ctx,
            &temp_name,
            result_type.clone(),
            default,
            expr.span,
        );
        branch_to(ctx, merge);
    } else if !ctx.builder.insertion_block_is_terminated() {
        let message = ctx.intern_string("Fatal error: unhandled match case\n");
        ctx.builder.terminate(Terminator::Fatal { message });
    }
    ctx.builder.position_at_end(merge);
    take_owned_temp(ctx, &temp_name, expr.span)
}

