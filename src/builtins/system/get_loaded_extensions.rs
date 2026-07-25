//! Purpose:
//! Home of the PHP `get_loaded_extensions` builtin: its single-source registry declaration and
//! semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` narrows the declared `Mixed` return to `Array<Str>` and, when the optional
//!   `$zend_extensions` flag is present, requires it to be a literal bool/int (AOT requirement:
//!   the list is selected at compile time).
//! - The regular and Zend extension lists are resolved at codegen against compile-time-known sets.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "get_loaded_extensions",
    area: System,
    params: [zend_extensions: Bool = DefaultSpec::Bool(false)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetLoadedExtensions,
    ),
    summary: "Returns an array with the names of all loaded modules.",
    php_manual: "function.get-loaded-extensions",
}

/// Validates the optional flag argument and narrows the return type to `Array<Str>`.
///
/// The list of extension names is selected at compile time, so the `$zend_extensions` flag,
/// when written explicitly, must be a literal bool or int. A missing argument uses the default
/// (regular extension list). Returns `Array<Str>`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(flag) = cx.args.first() {
        cx.checker.infer_type(flag, cx.env)?;
        if !matches!(flag.kind, ExprKind::BoolLiteral(_) | ExprKind::IntLiteral(_)) {
            return Err(CompileError::new(
                cx.span,
                "get_loaded_extensions() argument must be a literal bool or int in AOT mode",
            ));
        }
    }
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
