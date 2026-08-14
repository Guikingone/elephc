//! Purpose:
//! Home of the PHP `extension_loaded` builtin: its single-source registry declaration and
//! semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates that the argument is a string: a literal name const-folds at codegen, a
//!   dynamic `string` expression lowers to a baked case-insensitive membership test. The *set* of
//!   extensions is always statically known (AOT link set), only the name may be dynamic.
//! - Membership is resolved at codegen against a compile-time-known extension set; there is no
//!   runtime state in this increment.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "extension_loaded",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ExtensionLoaded,
    ),
}

/// Validates that the argument is a string (literal or dynamic).
///
/// AOT compilation knows the full set of loaded extensions at compile time (core extensions plus
/// the bridges actually linked), so only the *name* has to be a string: a literal const-folds,
/// while any other `string` expression lowers to a baked case-insensitive membership test.
/// A non-string argument is rejected because the lowering has no runtime string conversion.
/// Returns `PhpType::Bool` on success.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let extension_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !matches!(cx.args[0].kind, ExprKind::StringLiteral(_))
        && extension_ty.codegen_repr() != PhpType::Str
    {
        return Err(CompileError::new(
            cx.span,
            "extension_loaded() first argument must be a string in AOT mode",
        ));
    }
    Ok(PhpType::Bool)
}
