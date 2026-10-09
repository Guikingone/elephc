//! Purpose:
//! Owns a fresh closure descriptor while its immediate static invocation is prepared.
//!
//! Called from:
//! - `closure_calls::lower_expr_call()` for an eligible freshly created closure.
//!
//! Key details:
//! - The descriptor is not a static call operand, so argument cleanup cannot release it.
//! - Both preparation failure and body failure release this separate owner before rethrow.
//! - Results are stabilized before releasing captures owned by the descriptor.

use super::*;

pub(super) fn invoke(
    ctx: &mut LoweringContext<'_, '_>,
    descriptor: LoweredValue,
    target: StaticCallableBinding,
    args: &[Expr],
    expr: &Expr,
) -> LoweredValue {
    debug_assert_eq!(ctx.builder.value_defining_op(descriptor.value), Some(Op::ClosureNew));
    let ty = ctx.builder.value_php_type(descriptor.value);
    let owner = ctx.declare_owned_hidden_temp(ty.clone());
    ctx.store_local(&owner, descriptor, ty, Some(expr.span));
    let handler = ctx.builder.create_named_block("closure.immediate_cleanup", Vec::new());
    let after = ctx.builder.create_named_block("closure.immediate_after", Vec::new());
    ctx.emit_void(
        Op::TryPushHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
        Op::TryPushHandler.default_effects(), Some(expr.span),
    );
    let result = lower_static_callable_call(ctx, target, args, expr)
        .expect("eligible static closure binding remains invokable");
    let result_slot = if result.ir_type != IrType::Void {
        let ty = ctx.builder.value_php_type(result.value);
        let slot = ctx.declare_owned_hidden_temp(ty.clone());
        store_value_into_temp(ctx, &slot, ty, result, expr.span);
        Some(slot)
    } else { None };
    pop_handler(ctx, handler, expr.span);
    release_descriptor(ctx, &owner, expr.span);
    branch_to(ctx, after);

    ctx.builder.position_at_end(handler);
    ctx.clear_static_callable_locals();
    pop_handler(ctx, handler, expr.span);
    release_descriptor(ctx, &owner, expr.span);
    let current = ctx.emit_owned_value(
        Op::CatchBind, Vec::new(), None, PhpType::Object("Throwable".to_string()),
        Op::CatchBind.default_effects(), Some(expr.span),
    );
    ctx.builder.terminate(Terminator::Throw { value: current.value });
    ctx.builder.position_at_end(after);
    ctx.clear_static_callable_locals();
    result_slot.map(|slot| take_owned_temp(ctx, &slot, expr.span)).unwrap_or(result)
}

fn release_descriptor(ctx: &mut LoweringContext<'_, '_>, slot: &str, span: Span) {
    let descriptor = ctx.load_local(slot, Some(span));
    crate::ir_lower::ownership::release_if_owned(ctx, descriptor, Some(span));
    ctx.clear_owned_hidden_temp(slot, Some(span));
}

fn pop_handler(ctx: &mut LoweringContext<'_, '_>, handler: BlockId, span: Span) {
    ctx.emit_void(
        Op::TryPopHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
        Op::TryPopHandler.default_effects(), Some(span),
    );
}
