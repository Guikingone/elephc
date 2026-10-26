//! Purpose:
//! Declares numeric insertion into SPL backing-object property tables.
//!
//! Called from:
//! - The shared builtin registry and synthesized SPL storage methods.
//!
//! Key details:
//! - The receiver is borrowed; the table keeps an independent owner of the value.
//! - Integer buckets bypass named-property access, visibility and numeric-string conversion.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "__elephc_spl_object_append",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcSplObjectAppend,
    ),
}

/// Validates the internal receiver while keeping dynamically typed object storage available.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !matches!(ty.codegen_repr(), PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)) {
        return Err(CompileError::new(cx.span, "SPL backing property append requires an object"));
    }
    cx.checker.infer_type(&cx.args[1], cx.env)?;
    Ok(PhpType::Bool)
}
