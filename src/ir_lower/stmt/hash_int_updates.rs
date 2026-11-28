//! Purpose:
//! Lowers `$hash[$key] = $hash[$key] + N` (and `- N`, `+= N`, `-= N`) on a local
//! `array<..., mixed>` through a guarded in-place integer add before the ordinary statement.
//!
//! Called from:
//! - `crate::ir_lower::stmt::array_write_core::lower_array_assign`.
//!
//! Key details:
//! - The ordinary statement reads the element (boxing a fresh Mixed copy), adds through the
//!   generic Mixed arithmetic (boxing the sum), and writes it back (releasing the old box): three
//!   heap round-trips and two hash probes per counter update.
//! - `__rt_hash_add_int_in_place` performs the update only in the case that is indistinguishable
//!   from the statement: an existing integer element of a uniquely owned hash, held inline or in
//!   a cell only that entry owns, with no overflow. Anything else returns 0 having touched
//!   nothing, and the ordinary statement runs unchanged on that path, warnings included.
//! - The call names the hash's frame slot; codegen reads the slot's raw word and picks the boxed
//!   or typed helper from the slot's FINAL storage (a later statement can widen it to Mixed), and
//!   answers 0 outright for a slot that ever holds a reference cell.
//! - Only a plain frame local with a pure key (a literal or a plain local `string`/`int`) takes
//!   the fast path, so evaluating the key once more on the fallback path cannot be observed.

use super::*;
use crate::parser::ast::BinOp;

/// Emits the guarded fast path and the ordinary statement as its fallback; returns false when
/// the statement does not have the recognized shape, leaving lowering to the caller.
pub(super) fn lower(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    ordinary: impl FnOnce(&mut LoweringContext<'_, '_>),
) -> Result<(), ()> {
    let Some(addend) = recognized_addend(array, index, value) else { return Err(()) };
    let mixed_hash = |ty: PhpType| matches!(ty.codegen_repr(),
        PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed);
    if !ctx.is_plain_frame_local(array)
        || !mixed_hash(ctx.local_type(array))
        || !mixed_hash(ctx.local_mutation_type(array))
        || !key_is_pure(ctx, index)
    {
        return Err(());
    }
    // The runtime guard counts owners, so it must inspect exactly what the frame holds. The
    // slot's storage (the hash itself, or a Mixed cell boxing it) is only final once the whole
    // function is lowered, so the call names the slot and codegen reads its raw word.
    let Some(slot) = ctx.plain_frame_local_slot(array) else { return Err(()) };
    let slot = ctx.emit_value(Op::ConstI64, Vec::new(), Some(Immediate::I64(i64::from(slot.as_raw()))),
        PhpType::Int, Op::ConstI64.default_effects(), Some(span));
    let key = lower_expr(ctx, index);
    let addend = ctx.emit_value(Op::ConstI64, Vec::new(), Some(Immediate::I64(addend)), PhpType::Int,
        Op::ConstI64.default_effects(), Some(span));
    let done_fast = ctx.emit_value(Op::RuntimeCall, vec![slot.value, key.value, addend.value],
        Some(Immediate::RuntimeCall(crate::ir::RuntimeCallTarget::HashAddIntInPlace)), PhpType::Bool,
        crate::ir::Effects::READS_LOCAL | crate::ir::Effects::READS_HEAP | crate::ir::Effects::WRITES_HEAP
            | crate::ir::Effects::REFCOUNT_OP, Some(span));
    crate::ir_lower::ownership::release_if_owned(ctx, key, Some(span));
    let fallback = ctx.builder.create_named_block("hash.add.ordinary", Vec::new());
    let done = ctx.builder.create_named_block("hash.add.done", Vec::new());
    let initialized = ctx.initialized_slots_snapshot();
    ctx.builder.terminate(Terminator::CondBr { cond: done_fast.value, then_target: done,
        then_args: Vec::new(), else_target: fallback, else_args: Vec::new() });
    ctx.builder.position_at_end(fallback);
    ordinary(ctx);
    let ordinary_slots = ctx.initialized_slots_snapshot();
    ctx.builder.terminate(Terminator::Br { target: done, args: Vec::new() });
    ctx.builder.position_at_end(done);
    ctx.restore_initialized_slots(initialized.intersection(&ordinary_slots).copied().collect());
    Ok(())
}

/// Returns the integer added by `$array[$index] +/- N`, or `None` for any other value shape.
fn recognized_addend(array: &str, index: &Expr, value: &Expr) -> Option<i64> {
    let ExprKind::BinaryOp { left, op, right } = &value.kind else { return None };
    let ExprKind::ArrayAccess { array: receiver, index: read_index } = &left.kind else { return None };
    if !matches!(&receiver.kind, ExprKind::Variable(name) if name == array) || read_index.kind != index.kind {
        return None;
    }
    let ExprKind::IntLiteral(literal) = right.kind else { return None };
    match op {
        BinOp::Add => Some(literal),
        BinOp::Sub => literal.checked_neg(),
        _ => None,
    }
}

/// Accepts keys whose second evaluation on the fallback path is unobservable and diagnostic-free.
fn key_is_pure(ctx: &LoweringContext<'_, '_>, index: &Expr) -> bool {
    match &index.kind {
        ExprKind::IntLiteral(_) | ExprKind::StringLiteral(_) => true,
        ExprKind::Variable(name) => ctx.is_plain_frame_local(name)
            && matches!(ctx.local_type(name).codegen_repr(), PhpType::Str | PhpType::Int),
        _ => false,
    }
}
