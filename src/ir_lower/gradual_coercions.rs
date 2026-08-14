//! Purpose:
//! Narrows boxed gradual values at concrete PHP call and return boundaries.
//!
//! Called from:
//! - `crate::ir_lower::expr` and `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Runtime checks precede every representation change, and replacement temporaries own their
//!   retained or converted payload independently of the source boxed cell.

use crate::ir::{Immediate, IrType, Op};
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::ir_lower::ownership;
use crate::span::Span;
use crate::types::PhpType;

/// Narrows a boxed gradual value into a concrete boundary representation when supported.
pub(super) fn coerce_gradual_value_to_boundary(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    target: &PhpType,
    span: Option<Span>,
) -> LoweredValue {
    let source = ctx.builder.value_php_type(value.value).codegen_repr();
    if !matches!(source, PhpType::Mixed) {
        return value;
    }
    let target = target.codegen_repr();
    let narrowed = match &target {
        PhpType::Int => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::I64)),
            target.clone(),
            Op::Cast.default_effects(),
            span,
        ),
        PhpType::Float => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::F64)),
            target.clone(),
            Op::Cast.default_effects(),
            span,
        ),
        PhpType::Bool => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::I64)),
            target.clone(),
            Op::Cast.default_effects(),
            span,
        ),
        PhpType::Str => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::Str)),
            target.clone(),
            Op::Cast.default_effects(),
            span,
        ),
        PhpType::Object(class_name) => {
            let data = ctx.intern_class_name(class_name);
            ctx.emit_value(
                Op::MixedUnbox,
                vec![value.value],
                Some(Immediate::Data(data)),
                target.clone(),
                Op::MixedUnbox.default_effects(),
                span,
            )
        }
        PhpType::Callable => ctx.emit_value(
            Op::MixedUnbox,
            vec![value.value],
            Some(Immediate::I64(10)),
            target.clone(),
            Op::MixedUnbox.default_effects(),
            span,
        ),
        PhpType::Array(element) if element.codegen_repr() == PhpType::Mixed => ctx.emit_value(
            Op::MixedToHash,
            vec![value.value],
            None,
            target.clone(),
            Op::MixedToHash.default_effects(),
            span,
        ),
        PhpType::AssocArray { key, value: element }
            if key.codegen_repr() == PhpType::Mixed
                && element.codegen_repr() == PhpType::Mixed =>
        {
            ctx.emit_value(
                Op::MixedToHash,
                vec![value.value],
                None,
                target.clone(),
                Op::MixedToHash.default_effects(),
                span,
            )
        }
        _ => return value,
    };
    if ctx.value_is_owning_temporary(value) {
        ownership::release_if_owned(ctx, value, span);
    }
    narrowed
}
