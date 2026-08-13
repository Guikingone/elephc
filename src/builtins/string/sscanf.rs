//! Purpose:
//! Home of the PHP `sscanf` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts required `string` and `format` params plus a variadic `vars` list.
//! - `check` returns `PhpType::Array(Box::new(PhpType::Str))` because the macro
//!   `returns:` field cannot express a parameterized array type inline.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "sscanf",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Sscanf,
    ),
}

/// Returns `PhpType::Array(Box::new(PhpType::Str))` for a `sscanf` call.
///
/// A check hook is required because the `builtin!` macro cannot express a
/// parameterized array return type inline.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
