//! Purpose:
//! Home of the PHP `get_loaded_extensions` builtin: its single-source registry declaration and
//! semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` narrows the declared `Mixed` return to `Array<Str>` and, when the optional
//!   `$zend_extensions` flag is present, requires it to be a bool/int: a literal picks the list at
//!   compile time, a dynamic bool/int picks between the two baked lists at runtime.
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
/// Both candidate lists of extension names are known at compile time, so the `$zend_extensions`
/// flag only has to be a bool or an int: a literal selects one list during lowering, and any other
/// bool/int expression selects between the two baked lists at runtime. Other types are rejected
/// because the lowering has no runtime truthiness conversion for them. A missing argument uses the
/// default (regular extension list). Returns `Array<Str>`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(flag) = cx.args.first() {
        let flag_ty = cx.checker.infer_type(flag, cx.env)?;
        if !matches!(flag.kind, ExprKind::BoolLiteral(_) | ExprKind::IntLiteral(_))
            && !matches!(
                flag_ty.codegen_repr(),
                PhpType::Bool | PhpType::False | PhpType::Int
            )
        {
            return Err(CompileError::new(
                cx.span,
                "get_loaded_extensions() argument must be a bool or int in AOT mode",
            ));
        }
    }
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
