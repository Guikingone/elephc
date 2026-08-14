//! Purpose:
//! Declares the internal PDOStatement native-state initializer used by `PDO::prepare()`.
//!
//! Called from:
//! - The generated PDO prelude after allocating the configured statement subclass.
//!
//! Key details:
//! - The lowering invokes PDOStatement's private initializer directly, so subclasses are
//!   initialized before their user constructor without exposing a reset API to PHP code.

use crate::builtins::semantics::{
    internal_eir_semantics, BuiltinLoweringContext, BuiltinResultOwnership,
    LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::ir::{Effects, Op};

builtin! {
    contract: "__elephc_initialize_pdo_statement",
    semantics: internal_eir_semantics(
        lower,
        Effects::all().difference(Effects::REFCOUNT_OP),
        BuiltinResultOwnership::NonHeap,
    ),
}

/// Lowers private PDOStatement initialization to its direct-method EIR primitive.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, crate::builtins::semantics::BuiltinLoweringError> {
    Ok(ctx.emit_value(
        Op::DynamicPdoStatementInitialize,
        call.operands.to_vec(),
        None,
        call.result_type.clone(),
        Op::DynamicPdoStatementInitialize.default_effects(),
        Some(call.span),
    ))
}
