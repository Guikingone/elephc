//! Purpose:
//! Lowers boolean conditions directly to their true and false control-flow edges.
//!
//! Called from:
//! - Statement conditionals and loops, and full ternary expression lowering.
//!
//! Key details:
//! - Logical operators preserve lazy source-order evaluation without boolean merge slots.
//! - Leaf operands use ordinary expression lowering and consuming PHP truthiness.

use super::*;

/// Branches on a condition without materializing intermediate logical results.
///
/// Expression nesting matches `lower_expr`, including for reference-returning operands.
/// Value-producing consumers keep using `lower_expr` instead of this branch-only entry point.
pub(crate) fn lower_condition(
    ctx: &mut LoweringContext<'_, '_>,
    condition: &Expr,
    then_target: BlockId,
    else_target: BlockId,
) {
    ctx.expression_depth += 1;
    match &condition.kind {
        ExprKind::BinaryOp { left, op: BinOp::And, right } => {
            let rhs = ctx.builder.create_named_block("condition.and.rhs", Vec::new());
            lower_condition(ctx, left, rhs, else_target);
            ctx.builder.position_at_end(rhs);
            lower_condition(ctx, right, then_target, else_target);
        }
        ExprKind::BinaryOp { left, op: BinOp::Or, right } => {
            let rhs = ctx.builder.create_named_block("condition.or.rhs", Vec::new());
            lower_condition(ctx, left, then_target, rhs);
            ctx.builder.position_at_end(rhs);
            lower_condition(ctx, right, then_target, else_target);
        }
        ExprKind::Not(inner) => lower_condition(ctx, inner, else_target, then_target),
        _ => {
            let value = lower_expr_inner(ctx, condition);
            let value = ctx.truthy_consuming(value, Some(condition.span));
            ctx.builder.terminate(Terminator::CondBr {
                cond: value.value,
                then_target,
                then_args: Vec::new(),
                else_target,
                else_args: Vec::new(),
            });
        }
    }
    ctx.expression_depth -= 1;
}
