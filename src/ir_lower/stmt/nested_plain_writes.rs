//! Purpose:
//! Lowers plain nested stores and deep appends through attached writable parent cells.
//!
//! Called from:
//! - Nested array assignment dispatch and recognized parser-generated deep append groups.
//!
//! Key details:
//! - Computed inputs run before the RHS; write-context traversal runs after it.
//! - False conversion is published before its deprecation callback.
//! - Parent, key and RHS owners remain rooted across callbacks and retirement.

use super::*;

/// Writes a leaf only after evaluating its computed dimensions and RHS.
pub(super) fn lower_store(
    ctx: &mut LoweringContext<'_, '_>, target: &Expr, value: &Expr, span: Span,
) {
    let mut sources = Vec::new();
    let target = super::nested_write_sources::capture_target(ctx, target, &mut sources);
    let value = lower_expr(ctx, value);
    let (value, value_owner) = super::nested_array_writes::root_static_update_receiver(ctx, value, span);
    let target = super::nested_array_writes::bind_concrete_static_update_root(ctx, &target, &target)
        .map_or(target, |(_, write)| write);
    let ExprKind::ArrayAccess { array, index } = &target.kind else { unreachable!() };
    let parent = super::nested_array_writes::lower_nested_assign_parent_for_store(ctx, array, span);
    let parent = resolve_parent_for_write(ctx, parent, span);
    let (parent, parent_owner) = super::nested_array_writes::root_static_update_receiver(ctx, parent, span);
    let key = lower_expr(ctx, index);
    let (key, key_owner) = super::nested_array_writes::root_static_update_receiver(ctx, key, span);
    let pending = super::nested_false_errors::capture(ctx, parent, span);
    super::nested_false_errors::handle_pending(ctx, pending, Some(key), span, |ctx| {
        emit_store(ctx, parent, key, value, index, span);
    });
    emit_store(ctx, parent, key, value, index, span);
    retire_owner(ctx, key_owner, span);
    retire_owner(ctx, parent_owner, span);
    retire_owner(ctx, value_owner, span);
    super::nested_write_sources::retire(ctx, sources, span);
}

/// Appends directly into a stored parent instead of cloning a detached read and writing it back.
pub(super) fn lower_append(
    ctx: &mut LoweringContext<'_, '_>, target: &Expr, value: &Expr, span: Span,
) {
    let mut sources = Vec::new();
    let target = super::nested_write_sources::capture_target(ctx, target, &mut sources);
    let value = lower_expr(ctx, value);
    let (value, value_owner) = super::nested_array_writes::root_static_update_receiver(ctx, value, span);
    let target = super::nested_array_writes::bind_concrete_static_update_root(ctx, &target, &target)
        .map_or(target, |(_, write)| write);
    let parent = super::nested_array_writes::lower_nested_assign_parent_for_store(ctx, &target, span);
    let parent = resolve_parent_for_write(ctx, parent, span);
    let (parent, parent_owner) = super::nested_array_writes::root_static_update_receiver(ctx, parent, span);
    let pending = super::nested_false_errors::capture(ctx, parent, span);
    super::nested_false_errors::handle_pending(ctx, pending, None, span, |ctx| {
        emit_append(ctx, parent, value, span);
    });
    emit_append(ctx, parent, value, span);
    retire_owner(ctx, parent_owner, span);
    retire_owner(ctx, value_owner, span);
    super::nested_write_sources::retire(ctx, sources, span);
}

/// Resolves boxed caller-reference markers before conversion or mutation of the selected cell.
fn resolve_parent_for_write(
    ctx: &mut LoweringContext<'_, '_>, parent: LoweredValue, span: Span,
) -> LoweredValue {
    if ctx.builder.value_php_type(parent.value).codegen_repr() != PhpType::Mixed {
        return parent;
    }
    let resolved = ctx.emit_owned_value(
        Op::RuntimeCall,
        vec![parent.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::MixedCellResolveForWrite)),
        PhpType::Mixed,
        effects_lookup::runtime_effects(),
        Some(span),
    );
    crate::ir_lower::ownership::release_if_owned(ctx, parent, Some(span));
    resolved
}

/// Preserves string-offset writes and scalar errors after optional false conversion.
fn emit_store(
    ctx: &mut LoweringContext<'_, '_>, parent: LoweredValue,
    key: LoweredValue, value: LoweredValue, index: &Expr, span: Span,
) {
    if super::nested_write_guards::guard_nested_write_parent(ctx, parent, Some(index), span) {
        super::nested_scalar_writes::lower_nested_set(ctx, parent, key, value, span);
    }
}

/// Rejects string and scalar append parents before dispatching the boxed append.
fn emit_append(
    ctx: &mut LoweringContext<'_, '_>, parent: LoweredValue,
    value: LoweredValue, span: Span,
) {
    if super::nested_write_guards::guard_nested_write_parent(ctx, parent, None, span) {
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![parent.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
    }
}

/// Fetches one child with balanced, unwind-visible input and result owners.
pub(super) fn fetch_parent(
    ctx: &mut LoweringContext<'_, '_>, receiver: LoweredValue, index: &Expr, span: Span,
) -> LoweredValue {
    let (receiver, receiver_owner) =
        super::nested_array_writes::root_static_update_receiver(ctx, receiver, span);
    let key = lower_expr(ctx, index);
    let (key, key_owner) = super::nested_array_writes::root_static_update_receiver(ctx, key, span);
    let pending = super::nested_false_errors::capture(ctx, receiver, span);
    super::nested_false_errors::handle_pending(ctx, pending, Some(key), span, |ctx| {
        let null = lower_expr(ctx, &Expr::new(ExprKind::Null, span));
        ctx.emit_void(Op::RuntimeCall, vec![receiver.value, key.value, null.value], None,
            effects_lookup::runtime_effects(), Some(span));
    });
    if !guard_fetched_parent(ctx, receiver, span) {
        return receiver;
    }
    let child = ctx.emit_value(Op::RuntimeCall, vec![receiver.value, key.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
        PhpType::Mixed, effects_lookup::runtime_effects(), Some(span));
    let selected = ctx.declare_owned_hidden_temp(PhpType::Mixed);
    let slot = ctx.local_slots[&selected];
    ctx.store_call_argument_local(&selected, child, PhpType::Mixed, Some(span));
    // Publish the child before a retired input can throw, keeping every older owner reachable.
    if let Some(owner) = key_owner { pop_owner(ctx, owner, span); }
    push_owner(ctx, slot, span);
    if let Some(owner) = key_owner { release_owner(ctx, owner, span); }
    if let Some(owner) = receiver_owner {
        pop_owner(ctx, slot, span);
        pop_owner(ctx, owner, span);
        push_owner(ctx, slot, span);
        release_owner(ctx, owner, span);
    }
    let child = ctx.load_local(&selected, Some(span));
    ctx.clear_owned_hidden_temp(&selected, Some(span));
    pop_owner(ctx, slot, span);
    child
}

/// Rejects strings and scalar values selected as intermediate write parents.
fn guard_fetched_parent(
    ctx: &mut LoweringContext<'_, '_>, parent: LoweredValue, span: Span,
) -> bool {
    let parent_ty = ctx.builder.value_php_type(parent.value).codegen_repr();
    if parent_ty == PhpType::Str {
        lower_throw_access_error(ctx, "Cannot use string offset as an array", span);
        return false;
    }
    if matches!(parent_ty, PhpType::Int | PhpType::Float | PhpType::Resource(_)) {
        lower_throw_access_error(ctx, "Cannot use a scalar value as an array", span);
        return false;
    }
    if !matches!(parent_ty, PhpType::Mixed | PhpType::Bool) {
        return true;
    }
    let string = ctx.emit_value(
        Op::TypePredicate,
        vec![parent.value],
        Some(Immediate::TypePredicate(crate::ir::PhpTypePredicate::String)),
        PhpType::Bool,
        Op::TypePredicate.default_effects(),
        Some(span),
    );
    let invalid_string = ctx.builder.create_named_block("nested.store.string.error", Vec::new());
    let scalar_check = ctx.builder.create_named_block("nested.store.scalar.check", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: string.value,
        then_target: invalid_string,
        then_args: Vec::new(),
        else_target: scalar_check,
        else_args: Vec::new(),
    });
    ctx.builder.position_at_end(invalid_string);
    lower_throw_access_error(ctx, "Cannot use string offset as an array", span);
    ctx.builder.position_at_end(scalar_check);
    let mut scalar = ctx.emit_value(
        Op::TypePredicate,
        vec![parent.value],
        Some(Immediate::TypePredicate(crate::ir::PhpTypePredicate::Bool)),
        PhpType::Bool,
        Op::TypePredicate.default_effects(),
        Some(span),
    );
    for predicate in [
        crate::ir::PhpTypePredicate::Int,
        crate::ir::PhpTypePredicate::Float,
        crate::ir::PhpTypePredicate::Resource,
    ] {
        let other = ctx.emit_value(
            Op::TypePredicate,
            vec![parent.value],
            Some(Immediate::TypePredicate(predicate)),
            PhpType::Bool,
            Op::TypePredicate.default_effects(),
            Some(span),
        );
        scalar = ctx.emit_value(
            Op::IBitOr,
            vec![scalar.value, other.value],
            None,
            PhpType::Bool,
            Op::IBitOr.default_effects(),
            Some(span),
        );
    }
    let invalid_scalar = ctx.builder.create_named_block("nested.store.scalar.error", Vec::new());
    let valid = ctx.builder.create_named_block("nested.store.parent", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: scalar.value,
        then_target: invalid_scalar,
        then_args: Vec::new(),
        else_target: valid,
        else_args: Vec::new(),
    });
    ctx.builder.position_at_end(invalid_scalar);
    lower_throw_access_error(ctx, "Cannot use a scalar value as an array", span);
    ctx.builder.position_at_end(valid);
    true
}

/// Removes one topmost guard without invoking its value's destructor.
fn pop_owner(ctx: &mut LoweringContext<'_, '_>, slot: LocalSlotId, span: Span) {
    ctx.emit_void(Op::PopCallOperandOwner, Vec::new(), Some(Immediate::LocalSlot(slot)),
        Op::PopCallOperandOwner.default_effects(), Some(span));
}

/// Publishes one initialized result or input owner.
fn push_owner(ctx: &mut LoweringContext<'_, '_>, slot: LocalSlotId, span: Span) {
    ctx.emit_void(Op::PushCallOperandOwner, Vec::new(), Some(Immediate::LocalSlot(slot)),
        Op::PushCallOperandOwner.default_effects(), Some(span));
}

/// Clears storage before releasing an owner whose destructor may throw.
fn release_owner(ctx: &mut LoweringContext<'_, '_>, slot: LocalSlotId, span: Span) {
    ctx.emit_void(Op::ReleaseLocalSlot, Vec::new(), Some(Immediate::LocalSlot(slot)),
        Op::ReleaseLocalSlot.default_effects(), Some(span));
}

/// Retires an optional topmost owner after its consumer has retained or copied the value.
fn retire_owner(ctx: &mut LoweringContext<'_, '_>, owner: Option<LocalSlotId>, span: Span) {
    if let Some(owner) = owner { crate::ir_lower::expr::retire_owned_call_operand(ctx, owner, span); }
}
