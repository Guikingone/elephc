//! Purpose:
//! Converts a `buffer_new<T>()` length to the `I64` element count the allocation takes.
//!
//! Called from:
//! - `super::scoped_values::lower_buffer_new()`.
//!
//! Key details:
//! - An `int` length passes through unchanged. A boxed `mixed` / `int|float` length (checked int
//!   arithmetic such as `$n + 1` produces one) that holds an int is unboxed exactly.
//! - Any other length converts both to int and to float with PHP's conversions, and the int count
//!   is used only when it converts back to the same float and is not `PHP_INT_MAX`, where PHP's
//!   numeric-string conversion saturates. A fraction, NaN, infinity, or a value outside the int
//!   range becomes `-1`, which `__rt_buffer_new` rejects with its length diagnostic on every
//!   target.
//! - Counts stay exact at every magnitude: a zero-size packed element has stride 0, so any
//!   non-negative count allocates and `buffer_len()` would show a rounded one.

use super::*;

/// Lowers a `buffer_new<T>()` length to an `I64` element count.
///
/// A boxed length dispatches on its runtime tag: an int payload is converted directly, anything
/// else goes through the whole-number check of `select_checked_count`. A raw float uses `FToI`,
/// which raises no cast warning. The owning boxed temporary, if any, is released once every
/// conversion has read it.
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
    let selected = ctx
        .builder
        .create_named_block("buffer.length", vec![(IrType::I64, PhpType::Int)]);
    if matches!(value.ir_type, IrType::Heap(IrHeapKind::Mixed | IrHeapKind::Union)) {
        let is_int = ctx.emit_value(
            Op::TypePredicate,
            vec![value.value],
            Some(Immediate::TypePredicate(crate::ir::PhpTypePredicate::Int)),
            PhpType::Bool,
            Op::TypePredicate.default_effects(),
            Some(span),
        );
        let int_payload = ctx.builder.create_named_block("buffer.length.int", Vec::new());
        let other_payload = ctx.builder.create_named_block("buffer.length.convert", Vec::new());
        ctx.builder.terminate(Terminator::CondBr {
            cond: is_int.value,
            then_target: int_payload,
            then_args: Vec::new(),
            else_target: other_payload,
            else_args: Vec::new(),
        });
        ctx.builder.position_at_end(int_payload);
        let count = emit_cast(ctx, value, IrType::I64, PhpType::Int, span);
        ctx.builder.terminate(Terminator::Br { target: selected, args: vec![count.value] });
        ctx.builder.position_at_end(other_payload);
    }
    let (truncated, float) = if value.ir_type == IrType::F64 {
        let truncated = ctx.emit_value(
            Op::FToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::FToI.default_effects(),
            Some(span),
        );
        (truncated, value)
    } else {
        let truncated = emit_cast(ctx, value, IrType::I64, PhpType::Int, span);
        (truncated, emit_cast(ctx, value, IrType::F64, PhpType::Float, span))
    };
    select_checked_count(ctx, truncated, float, selected, span);
    ctx.builder.position_at_end(selected);
    release_coerced_source_if_owned(ctx, value, Some(span));
    LoweredValue { value: ctx.builder.block_param(selected, 0), ir_type: IrType::I64 }
}

/// Emits a PHP scalar `Cast` of `value` without releasing it, so both conversions can read it.
fn emit_cast(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    target: IrType,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    ctx.emit_value(
        Op::Cast,
        vec![value.value],
        Some(Immediate::CastTarget(target)),
        php_type,
        Op::Cast.default_effects(),
        Some(span),
    )
}

/// Branches to `selected` with `truncated` when it is the whole count `float` names, else with -1.
///
/// The count must convert back (`IToF`) to the same float, which rejects a fraction, NaN (an
/// unordered comparison is false on every target), infinity and a float outside the int range.
/// It must also differ from `PHP_INT_MAX`: PHP saturates an out-of-range numeric string there,
/// and its float is 2^63, so for a non-int payload that count cannot be told apart from overflow.
fn select_checked_count(
    ctx: &mut LoweringContext<'_, '_>,
    truncated: LoweredValue,
    float: LoweredValue,
    selected: BlockId,
    span: Span,
) {
    let restored = ctx.emit_value(
        Op::IToF,
        vec![truncated.value],
        None,
        PhpType::Float,
        Op::IToF.default_effects(),
        Some(span),
    );
    let round_trips = ctx.emit_value(
        Op::FCmp,
        vec![restored.value, float.value],
        Some(Immediate::CmpPredicate(CmpPredicate::Eq)),
        PhpType::Bool,
        Op::FCmp.default_effects(),
        Some(span),
    );
    let in_range = ctx.builder.create_named_block("buffer.length.range", Vec::new());
    let accept = ctx.builder.create_named_block("buffer.length.whole", Vec::new());
    let reject = ctx.builder.create_named_block("buffer.length.reject", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: round_trips.value,
        then_target: in_range,
        then_args: Vec::new(),
        else_target: reject,
        else_args: Vec::new(),
    });
    ctx.builder.position_at_end(in_range);
    let saturated = ctx.builder.emit_const_i64(i64::MAX);
    let below_saturation = ctx.emit_value(
        Op::ICmp,
        vec![truncated.value, saturated],
        Some(Immediate::CmpPredicate(CmpPredicate::Ne)),
        PhpType::Bool,
        Op::ICmp.default_effects(),
        Some(span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: below_saturation.value,
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
}
