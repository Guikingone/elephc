//! Purpose:
//! Adapts the RETURN position onto the shared checked-downcast emitter
//! (`crate::ir_lower::checked_downcast`). When a `return`'s value is statically only known to be
//! an ANCESTOR (base class or implemented interface) of a declared return-type arm — the
//! base→derived relaxation accepted by
//! `crate::types::checker::type_compat::object_types::Checker::checked_downcast_guardable` — the
//! shared emitter inserts a chain of `Op::InstanceOf` checks that passes through on a match and
//! throws a catchable `\TypeError` (naming the ACTUAL runtime class) when every arm mismatches.
//!
//! Called from:
//! - `crate::ir_lower::stmt::lower_return`.
//!
//! Key details:
//! - The chain itself, its fast paths, the class-hierarchy walk and the message formatting all
//!   live in the shared module now, so the return, argument and property-store positions cannot
//!   drift apart. This file exists only to name the position and hand over the declared type.
//! - The return position is the ONLY one whose throw releases the mismatched value
//!   (`Op::ThrowCheckedReturnTypeError`): a return value the caller never receives has no other
//!   owner. That policy is selected by the position, in the shared emitter.

use crate::ir_lower::checked_downcast::{DowncastPosition, emit_checked_downcast};
use crate::ir_lower::context::{LoweredValue, LoweringContext};
use crate::span::Span;

/// Inserts the checked-downcast-on-return guard around `value` (the already-lowered return
/// expression result) when needed, and returns the value to continue lowering with. A no-op
/// (returns `value` unchanged) whenever the value isn't statically an `Object`, the declared
/// return type has no named `Object` arm, or the value is already a proven subtype of a
/// declared arm.
pub(crate) fn emit_checked_downcast_return_guard(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    let declared = ctx.return_php_type.clone();
    let owner = ctx.owner_name().to_string();
    emit_checked_downcast(
        ctx,
        value,
        &declared,
        DowncastPosition::Return { owner: &owner },
        Some(span),
    )
}
