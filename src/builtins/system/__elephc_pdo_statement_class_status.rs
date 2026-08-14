//! Purpose:
//! Declares the internal PDO statement-class validator used by the generated PDO prelude.
//!
//! Called from:
//! - `PDO::setAttribute()` and `PDO::prepare()` for `PDO::ATTR_STATEMENT_CLASS`.
//!
//! Key details:
//! - The integer result distinguishes unknown classes, wrong ancestry, public constructors,
//!   concrete valid classes, and abstract valid classes from the AOT class table.
//! - `internal: true` keeps this compiler primitive out of PHP-visible builtin catalogs.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    contract: "__elephc_pdo_statement_class_status",
    semantics: internal_eir_semantics(lower, Effects::PURE, BuiltinResultOwnership::NonHeap),
}

/// Lowers PDO statement-class validation to the dedicated AOT metadata EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicPdoStatementClassStatus,
        vec![call.operand(0)?],
        None,
        call.result_type.clone(),
        Op::DynamicPdoStatementClassStatus.default_effects(),
        Some(call.span),
    ))
}
