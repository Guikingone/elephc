//! Purpose:
//! Home of the PHP `get_object_vars` builtin and its object-only checker contract.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Runtime-typed `Mixed` values remain accepted because `unserialize()` can
//!   produce an object through that storage shape.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "get_object_vars",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetObjectVars,
    ),
}

/// Requires an object-shaped value and returns a string-keyed Mixed array.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !matches!(ty.codegen_repr(), PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)) {
        return Err(CompileError::new(
            cx.span,
            "get_object_vars() argument must be an object",
        ));
    }
    Ok(PhpType::AssocArray {
        key: Box::new(PhpType::Str),
        value: Box::new(PhpType::Mixed),
    })
}
