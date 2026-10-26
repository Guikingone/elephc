//! Purpose:
//! Lowers boolean conditions directly to their true and false control-flow edges.
//!
//! Called from:
//! - Statement conditionals and loops, and full ternary expression lowering.
//!
//! Key details:
//! - Logical operators preserve lazy source-order evaluation without boolean merge slots.
//! - Same-operator chains allocate operand blocks in source order for fallthrough edges.
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
        ExprKind::BinaryOp { left, op, right } if matches!(op, BinOp::And | BinOp::Or) =>
            lower_logical_condition(ctx, left, op, right, then_target, else_target),
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

/// Emits a same-operator chain with its continuation blocks in source evaluation order.
fn lower_logical_condition(
    ctx: &mut LoweringContext<'_, '_>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
    then_target: BlockId,
    else_target: BlockId,
) {
    let mut operands = Vec::new();
    collect_logical_operands(left, op, 0, &mut operands);
    collect_logical_operands(right, op, 0, &mut operands);
    let is_and = matches!(op, BinOp::And);
    let name = if is_and { "condition.and.rhs" } else { "condition.or.rhs" };
    let blocks: Vec<_> = (1..operands.len())
        .map(|_| ctx.builder.create_named_block(name, Vec::new()))
        .collect();
    for (index, (operand, extra_depth)) in operands.into_iter().enumerate() {
        if index > 0 {
            ctx.builder.position_at_end(blocks[index - 1]);
        }
        let next = blocks.get(index).copied();
        // Flattening removes logical AST ancestors, but their depth still scopes operand
        // owners and reference-return call staging inside the original expression tree.
        ctx.expression_depth += extra_depth;
        lower_condition(
            ctx,
            operand,
            if is_and { next.unwrap_or(then_target) } else { then_target },
            if is_and { else_target } else { next.unwrap_or(else_target) },
        );
        ctx.expression_depth -= extra_depth;
    }
}

/// Collects chain leaves and the number of flattened ancestors below the outer operator.
fn collect_logical_operands<'a>(
    expr: &'a Expr,
    op: &BinOp,
    extra_depth: usize,
    operands: &mut Vec<(&'a Expr, usize)>,
) {
    if let ExprKind::BinaryOp { left, op: nested_op, right } = &expr.kind {
        if nested_op == op {
            collect_logical_operands(left, op, extra_depth + 1, operands);
            collect_logical_operands(right, op, extra_depth + 1, operands);
            return;
        }
    }
    operands.push((expr, extra_depth));
}
