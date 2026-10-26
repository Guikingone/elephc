//! Purpose:
//! Captures array-write keys by value and protects borrowed write operands during evaluation.
//!
//! Called from:
//! - Shared array/hash key evaluation and nested write lowering in `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Mixed cells detach and string keys persist before later expressions can change storage.
//! - Writers borrow the captured views; one unwind-visible slot owns each managed temporary.

use super::*;

/// Captures the PHP key value and returns a borrowed view with an optional scoped owner.
pub(crate) fn capture_array_write_key(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<crate::ir::LocalSlotId>) {
    let ty = ctx.builder.value_php_type(value.value);
    if !Ownership::php_type_needs_lifetime_tracking(&ty)
        || ctx.builder.value_ownership(value.value) == Ownership::Persistent
    {
        return (value, None);
    }
    let captured = match ty.codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) => {
            crate::ir_lower::ownership::copy_assignment_value(ctx, value, Some(span))
        }
        PhpType::Str => {
            let stable = ctx.emit_owned_value(
                Op::StrPersist, vec![value.value], None, ty.clone(),
                Op::StrPersist.default_effects(), Some(span),
            );
            if ctx.value_needs_release_after_use(value) {
                crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
            }
            stable
        }
        _ => {
            let captured = crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, Some(span));
            if ctx.value_is_owning_temporary(value) {
                crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
            }
            captured
        }
    };
    let name = ctx.declare_owned_hidden_temp(ty.clone());
    ctx.store_local(&name, captured, ty.clone(), Some(span));
    let slot = ctx.local_slots[&name];
    register_owned_call_operand(ctx, slot, span);
    (load_published_container(ctx, slot, ty, span), Some(slot))
}

/// Moves an existing temporary lease into a scoped slot while preserving the value's identity.
pub(crate) fn root_borrowed_write_operand(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<crate::ir::LocalSlotId>) {
    let (rooted, owner) = root_owned_call_operand(ctx, value, span);
    match owner {
        Some(slot) => {
            let ty = ctx.builder.value_php_type(rooted.value);
            (load_published_container(ctx, slot, ty, span), Some(slot))
        }
        None => (value, None),
    }
}
