//! Purpose:
//! Home of the PHP `dirname` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates that the optional `levels` argument, when a static integer literal,
//!   is greater than or equal to 1 (PHP requirement).
//! - The registry pre-infers arguments before calling the hook; the hook does not
//!   call `infer_type` again.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    contract: "dirname",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Dirname,
    ),
}

/// Returns `Str`, rejecting static integer `levels` arguments less than 1.
///
/// The registry pre-infers arguments before calling this hook. The hook checks
/// whether the optional `levels` argument is a compile-time integer literal less
/// than 1 and emits a diagnostic if so; otherwise returns `PhpType::Str`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if matches!(
        cx.args.get(1).map(|arg| &arg.kind),
        Some(ExprKind::IntLiteral(levels)) if *levels < 1
    ) {
        return Err(CompileError::new(
            cx.span,
            "dirname() levels must be greater than or equal to 1",
        ));
    }
    Ok(PhpType::Str)
}
