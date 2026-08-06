//! Purpose:
//! Declares the internal dynamic constructor-existence predicate used by PDO hydration.
//!
//! Called from:
//! - The generated PDO prelude after allocating and hydrating a `FETCH_CLASS` object.
//!
//! Key details:
//! - The result is derived from the AOT class table and includes inherited constructors.
//! - `internal: true` keeps this compiler primitive out of PHP-visible builtin catalogs.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    name: "__elephc_class_has_constructor",
    area: System,
    params: [class: Str],
    returns: Bool,
    semantics: internal_eir_semantics(lower, Effects::PURE, BuiltinResultOwnership::NonHeap),
    summary: "Reports whether a dynamically named AOT class has a constructor.",
    internal: true
}

/// Lowers the class-name predicate to the dedicated AOT metadata EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicClassHasConstructor,
        vec![call.operand(0)?],
        None,
        call.result_type.clone(),
        Op::DynamicClassHasConstructor.default_effects(),
        Some(call.span),
    ))
}
