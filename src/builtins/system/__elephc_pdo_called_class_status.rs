//! Purpose:
//! Declares the internal late-static PDO factory class classifier.
//!
//! Called from:
//! - `PDO::connect()` in the generated PHP 8.4+ PDO prelude.
//!
//! Key details:
//! - The result distinguishes base PDO, each driver hierarchy, and generic PDO subclasses.
//! - `internal: true` keeps this compiler primitive out of PHP-visible builtin catalogs.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    name: "__elephc_pdo_called_class_status",
    area: System,
    params: [class: Str],
    returns: Int,
    semantics: internal_eir_semantics(lower, Effects::PURE, BuiltinResultOwnership::NonHeap),
    summary: "Classifies PDO::connect's late-static called class by driver hierarchy.",
    internal: true
}

/// Lowers the late-static PDO classifier to the dedicated AOT metadata EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicPdoCalledClassStatus,
        vec![call.operand(0)?],
        None,
        call.result_type.clone(),
        Op::DynamicPdoCalledClassStatus.default_effects(),
        Some(call.span),
    ))
}
