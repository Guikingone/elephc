//! Purpose:
//! Publishes pending operand owners only across EIR that can unwind.
//!
//! Called from:
//! - Concatenation and array-write operand lowering.
//!
//! Key details:
//! - Lower first to inspect authoritative EIR effects, then place ownership preparation before the window.
//! - Non-throwing scalar paths introduce no ownership slots or optimizer barriers.

use super::*;

/// Lowers an operand window and protects earlier owners when that window can throw.
/// Fresh transient strings become owned before publication; both steps precede the window.
pub(crate) fn lower_with_pending_owners<'m, 'f, T>(
    ctx: &mut LoweringContext<'m, 'f>,
    operands: &mut [LoweredValue],
    span: Span,
    lower: impl FnOnce(&mut LoweringContext<'m, 'f>) -> T,
) -> (T, Vec<PinnedInFlightOwner>) {
    let anchor = ctx.builder.insertion_block().expect("pending operand insertion block");
    let position = ctx.builder.function().block(anchor).unwrap().instructions.len();
    let first_instruction = ctx.builder.function().instructions.len();
    let first_block = ctx.builder.function().blocks.len();
    let result = lower(ctx);
    let function = ctx.builder.function();
    let can_throw = function.instructions[first_instruction..].iter()
        .any(|inst| inst.effects.contains(crate::ir::Effects::MAY_THROW))
        || matches!(function.block(anchor).unwrap().terminator, Some(crate::ir::Terminator::Throw { .. }))
        || function.blocks[first_block..].iter().any(|block| {
            matches!(block.terminator, Some(crate::ir::Terminator::Throw { .. }))
        });
    if !can_throw { return (result, Vec::new()); }
    let source = ctx.builder.insertion_block().expect("pending owner publication block");
    let start = ctx.builder.function().block(source).unwrap().instructions.len();
    for operand in operands.iter_mut() {
        let transient = ctx.builder.value_defining_instruction(operand.value)
            .is_some_and(crate::ir::Instruction::creates_transient_string);
        if transient && operand.ir_type == IrType::Str {
            *operand = ctx.emit_value(
                Op::StrPersist, vec![operand.value], None, PhpType::Str,
                Op::StrPersist.default_effects(), Some(span),
            );
        }
    }
    let values = operands.iter().map(|operand| operand.value).collect::<Vec<_>>();
    let pins = pin_in_flight_owners(ctx, &values, span);
    if ctx.builder.function().block(source).unwrap().instructions.len() != start {
        ctx.builder.move_trailing_instructions(source, start, anchor, position);
    }
    (result, pins)
}
