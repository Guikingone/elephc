//! Purpose:
//! Declares named SPL backing-object property lookup snapshots through the shared builtin contract.
//!
//! Called from:
//! - Checker, EIR and ownership consumers through the builtin registry.
//!
//! Key details:
//! - Integer buckets are excluded; named properties retain PHP offset-key conversion.
//! - The result is a fresh associative array of independently retained Mixed values.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "__elephc_spl_object_vars_named",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcSplObjectVarsNamed,
    ),
}

/// Requires an object source and returns a hash of independently retained named properties.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !matches!(ty.codegen_repr(), PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)) {
        return Err(CompileError::new(cx.span, "SPL backing property snapshot requires an object"));
    }
    Ok(PhpType::AssocArray { key: Box::new(PhpType::Mixed), value: Box::new(PhpType::Mixed) })
}
