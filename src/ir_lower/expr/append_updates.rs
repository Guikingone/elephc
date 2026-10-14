//! Purpose:
//! Lowers compiler-generated append updates through captured ordinary Mixed cells.
//! Keeps the selected place stable across diagnostic callbacks without PHP reference flags.
//!
//! Called from:
//! - `crate::ir_lower::expr::assignments::lower_assignment_expr()`.
//! - `LoweringContext::store_local_impl()` for stores through an active cell alias.
//!
//! Key details:
//! - RefAssign marker names are an unforgeable parser/EIR protocol, not reference-cell ABI pointers.
//! - Alias slots own stored Mixed boxes and publish cleanup records on normal/exceptional paths.

use super::*;
use super::call_operand_owners::{register_owned_call_operand, retire_owned_call_operand};

/// Processes an append prelude while every captured stored cell stays rooted for unwinding.
pub(super) fn lower(
    ctx: &mut LoweringContext<'_, '_>, result: &Expr, prelude: &[Stmt],
) -> Option<LoweredValue> {
    if !prelude.iter().any(|stmt| matches!(&stmt.kind,
        StmtKind::RefAssign { target, .. } if crate::names::append_update_alias_kind(target).is_some()
    )) { return None; }
    let mut aliases = Vec::new();
    for stmt in prelude {
        if let StmtKind::RefAssign { target, source } = &stmt.kind {
            if crate::names::append_update_alias_kind(target).is_some() {
                let cell = crate::ir_lower::stmt::lower_nested_assign_parent(ctx, source, source.span);
                debug_assert_eq!(ctx.builder.value_php_type(cell.value).codegen_repr(), PhpType::Mixed);
                debug_assert!(!ctx.append_update_cell_aliases.contains(target));
                // A normal Mixed pointer slot owns the box itself. PHP references instead
                // contain an address of a boxed-pointer slot and use a different ABI.
                ctx.store_local(target, cell, PhpType::Mixed, Some(source.span));
                let slot = ctx.local_slots[target];
                register_owned_call_operand(ctx, slot, source.span);
                ctx.append_update_cell_aliases.insert(target.clone());
                aliases.push((target.clone(), slot, source.span));
                continue;
            }
        }
        crate::ir_lower::stmt::lower_stmt(ctx, stmt);
    }
    for (name, slot, span) in aliases.into_iter().rev() {
        retire_owned_call_operand(ctx, slot, span);
        ctx.append_update_cell_aliases.remove(&name);
    }
    Some(lower_expr(ctx, result))
}

/// Publishes a value into the selected box before retiring its prior payload and source owner.
pub(crate) fn store_alias(
    ctx: &mut LoweringContext<'_, '_>, name: &str, value: LoweredValue,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    let span = span.unwrap_or_else(crate::span::Span::synthetic);
    let target = ctx.load_local(name, Some(span));
    let source = if matches!(ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)) {
        value
    } else {
        ctx.box_value_as_mixed(value, PhpType::Mixed, Some(span))
    };
    let pins = super::pin_in_flight_owners(ctx, &[source.value], span);
    ctx.emit_void(Op::RuntimeCall, vec![target.value, source.value], None,
        crate::ir_lower::effects_lookup::runtime_effects(), Some(span));
    super::unpin_in_flight_owners(ctx, pins, span);
    crate::ir_lower::ownership::release_if_owned(ctx, source, Some(span));
    ctx.load_local(name, Some(span))
}
