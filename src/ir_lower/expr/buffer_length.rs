//! Purpose:
//! Converts a `buffer_new<T>()` length to the `I64` element count the allocation takes.
//!
//! Called from:
//! - `super::scoped_values::lower_buffer_new()`.
//!
//! Key details:
//! - An `int` length passes through unchanged. Any other length the checker admits (the boxed
//!   `mixed` of checked int arithmetic such as `$n + 1`, or a `mixed` / `int|float` value)
//!   converts both to int and to float with PHP's conversions.
//! - The int count is used only when it converts back to the same float, so an int payload of
//!   any magnitude stays exact, while a fraction, NaN, infinity, or a float outside the int range
//!   becomes `-1`, which `__rt_buffer_new` rejects with its existing length diagnostic on every
//!   target. Exactness matters even above the payload limit: a zero-size packed element has
//!   stride 0, so any non-negative count allocates.

use super::*;

/// Lowers a `buffer_new<T>()` length to an `I64` element count.
///
/// A non-`int` length is converted to int first (PHP's conversion, `FToI` for a raw float so no
/// cast warning is raised) and then to float, which releases an owning boxed temporary once both
/// conversions have read it. The int count is selected when `IToF` gives the float back;
/// otherwise the count is `-1`, which the allocation rejects.
pub(super) fn lower_buffer_length(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if value.ir_type == IrType::I64
        && ctx.builder.value_php_type(value.value).codegen_repr() == PhpType::Int
    {
        return value;
    }
    let truncated = if value.ir_type == IrType::F64 {
        ctx.emit_value(
            Op::FToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::FToI.default_effects(),
            Some(span),
        )
    } else {
        ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::I64)),
            PhpType::Int,
            Op::Cast.default_effects(),
            Some(span),
        )
    };
    let float = coerce_to_float_at_span(ctx, value, Some(span));
    let restored = ctx.emit_value(
        Op::IToF,
        vec![truncated.value],
        None,
        PhpType::Float,
        Op::IToF.default_effects(),
        Some(span),
    );
    // An unordered (NaN) comparison is false on every target, so NaN takes the reject edge.
    let whole = ctx.emit_value(
        Op::FCmp,
        vec![restored.value, float.value],
        Some(Immediate::CmpPredicate(CmpPredicate::Eq)),
        PhpType::Bool,
        Op::FCmp.default_effects(),
        Some(span),
    );
    let selected = ctx
        .builder
        .create_named_block("buffer.length", vec![(IrType::I64, PhpType::Int)]);
    let accept = ctx.builder.create_named_block("buffer.length.whole", Vec::new());
    let reject = ctx.builder.create_named_block("buffer.length.reject", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: whole.value,
        then_target: accept,
        then_args: Vec::new(),
        else_target: reject,
        else_args: Vec::new(),
    });
    ctx.builder.position_at_end(accept);
    ctx.builder.terminate(Terminator::Br { target: selected, args: vec![truncated.value] });
    ctx.builder.position_at_end(reject);
    let rejected = ctx.builder.emit_const_i64(-1);
    ctx.builder.terminate(Terminator::Br { target: selected, args: vec![rejected] });
    ctx.builder.position_at_end(selected);
    LoweredValue { value: ctx.builder.block_param(selected, 0), ir_type: IrType::I64 }
}
