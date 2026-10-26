//! Purpose:
//! Acquires live native storage cells for synthesized SPL array-container mutations.
//!
//! Called from:
//! - `super::property_array_writes::lower_property_array_assign_with_diagnosed_key()`.
//!
//! Key details:
//! - Only builtin ArrayObject/ArrayIterator method bodies may lease this private cell.
//! - Updating its payload preserves debug-view identity across COW, growth and promotion.
//! - Ordinary PHP array copies still use the shared Mixed writer's payload COW boundary.

use super::*;

/// Acquires and guards the native storage cell without copying or replacing its identity.
pub(in crate::ir_lower) fn live_storage_cell(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: LoweredValue,
    property: &str,
    span: Span,
) -> Option<LoweredValue> {
    let class = ctx.current_class.as_deref()?;
    if property != "storage" || !matches!(class, "ArrayObject" | "ArrayIterator") {
        return None;
    }
    if !matches!(ctx.builder.value_php_type(receiver.value).codegen_repr(),
        PhpType::Object(name) if name == class)
    {
        return None;
    }
    if !matches!(object_property_type(ctx, receiver.value, property), Some(PhpType::Mixed)) {
        return None;
    }
    let data = ctx.intern_string(property);
    let cell = ctx.emit_owned_value(
        Op::PropGet,
        vec![receiver.value],
        Some(Immediate::Data(data)),
        PhpType::Mixed,
        Op::PropGet.default_effects(),
        Some(span),
    );
    ctx.begin_argument_guard_scope();
    ctx.guard_call_argument(cell, 0, span);
    Some(cell)
}

/// Retires the temporary property-read owner after the normal mutation path.
pub(in crate::ir_lower) fn release_live_storage_cell(
    ctx: &mut LoweringContext<'_, '_>, cell: LoweredValue, span: Span,
) {
    ctx.unguard_call_argument(cell.value, span);
    ctx.end_argument_guard_scope();
    crate::ir_lower::ownership::release_if_owned(ctx, cell, Some(span));
}

/// Identifies the direct builtin storage access accepted by internal unset bodies.
pub(in crate::ir_lower) fn is_live_storage_access(
    ctx: &LoweringContext<'_, '_>, array: &Expr,
) -> bool {
    matches!(&array.kind, ExprKind::PropertyAccess { object, property }
        if property == "storage"
            && matches!(&object.kind, ExprKind::This)
            && matches!(ctx.current_class.as_deref(), Some("ArrayObject" | "ArrayIterator")))
}
