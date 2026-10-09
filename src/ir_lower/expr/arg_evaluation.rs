//! Purpose:
//! Holds argument snapshots until argument evaluation succeeds or unwinds.
//!
//! Called from:
//! - Signature-aware call argument lowering and its shared snapshot primitive.
//!
//! Key details:
//! - Nested calls use independent ledgers; rollback includes the ledger stack.
//! - Zero-initialized owned slots make a partially evaluated argument list safe to release.
//! - Success transfers owners to existing call cleanup, not to a second lifetime pin.

use super::*;

pub(super) fn protect_argument_evaluation(
    ctx: &mut LoweringContext<'_, '_>,
    span: Span,
    lower: impl FnOnce(&mut LoweringContext<'_, '_>) -> Vec<ValueId>,
) -> Vec<ValueId> {
    let handler = ctx.builder.create_named_block("args.eval_cleanup", Vec::new());
    let after = ctx.builder.create_named_block("args.eval_after", Vec::new());
    ctx.argument_snapshot_scopes.push(Vec::new());
    ctx.emit_void(
        Op::TryPushHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
        Op::TryPushHandler.default_effects(), Some(span),
    );
    let values = lower(ctx);
    let slots = ctx.argument_snapshot_scopes.pop().expect("argument scope is balanced");
    pop_handler(ctx, handler, span);
    for slot in &slots {
        // The SSA load already holds this pointer. Clear the backing owner without
        // releasing it so normal/exceptional call cleanup can consume it exactly once.
        ctx.clear_owned_hidden_temp(slot, Some(span));
    }
    branch_to(ctx, after);

    ctx.builder.position_at_end(handler);
    ctx.clear_static_callable_locals();
    pop_handler(ctx, handler, span);
    for slot in &slots {
        let value = ctx.load_local(slot, Some(span));
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
        ctx.clear_owned_hidden_temp(slot, Some(span));
    }
    let current = ctx.emit_owned_value(
        Op::CatchBind, Vec::new(), None, PhpType::Object("Throwable".to_string()),
        Op::CatchBind.default_effects(), Some(span),
    );
    ctx.builder.terminate(Terminator::Throw { value: current.value });
    ctx.builder.position_at_end(after);
    ctx.clear_static_callable_locals();
    values
}

fn pop_handler(ctx: &mut LoweringContext<'_, '_>, handler: BlockId, span: Span) {
    ctx.emit_void(
        Op::TryPopHandler, Vec::new(), Some(Immediate::I64(handler.as_raw() as i64)),
        Op::TryPopHandler.default_effects(), Some(span),
    );
}

pub(super) fn hold_argument_snapshot(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> ValueId {
    if ctx.argument_snapshot_scopes.is_empty() {
        return value.value;
    }
    let php_type = ctx.builder.value_php_type(value.value);
    let slot = ctx.declare_owned_hidden_temp(php_type.clone());
    // OwnedTemp stores move this owner's reference; unlike store_value_into_temp,
    // no extra retain is needed for a newly created MixedClone.
    ctx.store_local(&slot, value, php_type, Some(span));
    ctx.argument_snapshot_scopes.last_mut().unwrap().push(slot.clone());
    ctx.load_local(&slot, Some(span)).value
}
