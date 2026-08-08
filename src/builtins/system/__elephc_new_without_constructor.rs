//! Purpose:
//! Declares the internal object-allocation builtin used by PDO fetch hydration.
//!
//! Called from:
//! - The generated PDO prelude when `FETCH_CLASS` uses PHP's default hydration order.
//!
//! Key details:
//! - Allocation initializes declared property defaults but deliberately does not invoke `__construct`.
//! - `internal: true` keeps this compiler primitive out of PHP-visible builtin catalogs.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    name: "__elephc_new_without_constructor",
    area: System,
    params: [class: Str],
    returns: Mixed,
    semantics: internal_eir_semantics(
        lower,
        Effects::READS_HEAP.union(Effects::ALLOC_HEAP).union(Effects::MAY_DEOPT),
        BuiltinResultOwnership::Fresh,
    ),
    summary: "Allocates a dynamically named object without invoking its constructor.",
    internal: true
}

/// Lowers constructorless allocation to the dedicated dynamic-object EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicObjectNewWithoutConstructorMixed,
        vec![call.operand(0)?],
        None,
        call.result_type.clone(),
        Op::DynamicObjectNewWithoutConstructorMixed.default_effects(),
        Some(call.span),
    ))
}
