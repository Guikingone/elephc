//! Purpose:
//! Home of the PHP `spl_autoload_functions` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - A `check` hook is required because the return type `Array<Mixed>` cannot be
//!   expressed as a plain `TypeSpec` ident in the `builtin!` macro.
//! - The function takes no arguments; arity is enforced by the registry.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "spl_autoload_functions",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplAutoloadFunctions,
    ),
}

/// Returns `Array<Mixed>` as the precise return type for `spl_autoload_functions()`.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Mixed)))
}
