//! Purpose:
//! Home of the PHP `php_uname` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates that the optional `mode` argument, when present, is a string type.
//! - `arity_error` overrides the default "takes at most 1 argument" message to match
//!   the legacy phrasing "takes 0 or 1 arguments".

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "php_uname",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::PhpUname,
    ),
}

/// Validates that the optional `mode` argument is a string when present.
///
/// Returns `PhpType::Str` unconditionally; the error path fires when an argument
/// is provided but does not infer as `PhpType::Str`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(arg) = cx.args.first() {
        let ty = cx.checker.infer_type(arg, cx.env)?;
        if ty != PhpType::Str {
            return Err(CompileError::new(
                cx.span,
                "php_uname() argument must be string",
            ));
        }
    }
    Ok(PhpType::Str)
}
