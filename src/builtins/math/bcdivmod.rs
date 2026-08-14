//! Purpose:
//! Declares PHP `bcdivmod()` and refines its result to an indexed string array.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The runtime returns a fresh two-element array containing quotient and remainder strings.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "bcdivmod",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcDivmod,
    ),
}

/// Returns the indexed two-string array type produced by `bcdivmod()`.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
