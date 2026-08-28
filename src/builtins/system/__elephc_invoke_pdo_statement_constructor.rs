//! Purpose:
//! Declares the internal constructor invoker used for PDO custom statement classes.
//!
//! Called from:
//! - `PDO::prepare()` after native PDOStatement fields have been initialized.
//!
//! Key details:
//! - Dispatch uses AOT class metadata and deliberately bypasses userland visibility, matching
//!   php-src's internal call of protected/private PDOStatement subclass constructors.
//! - Constructor arguments remain a boxed runtime container so named arguments are preserved.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    contract: "__elephc_invoke_pdo_statement_constructor",
    semantics: internal_eir_semantics(
        lower,
        Effects::all().difference(Effects::REFCOUNT_OP),
        BuiltinResultOwnership::NonHeap,
    ),
}

/// Lowers internal PDO constructor invocation to its dynamic-dispatch EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicPdoStatementConstructorCall,
        call.operands.to_vec(),
        None,
        call.result_type.clone(),
        Op::DynamicPdoStatementConstructorCall.default_effects(),
        Some(call.span),
    ))
}
