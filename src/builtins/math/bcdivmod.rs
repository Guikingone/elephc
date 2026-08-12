//! Purpose:
//! Declares PHP `bcdivmod()` and refines its result to an indexed string array.
//!
//! Called from:
//! - The builtin registry for checker, EIR, optimizer, callable, and documentation consumers.
//!
//! Key details:
//! - The runtime returns a fresh two-element array containing quotient and remainder strings.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "bcdivmod",
    area: Math,
    params: [num1: Str, num2: Str, scale: Int = DefaultSpec::Null],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcDivmod,
    ),
    summary: "Returns the quotient and remainder of arbitrary-precision division.",
    php_manual: "https://www.php.net/manual/en/function.bcdivmod.php",
}

/// Returns the indexed two-string array type produced by `bcdivmod()`.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
