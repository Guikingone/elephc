//! Purpose:
//! Home of the PHP `class_alias` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The check hook accepts calls whose class strings are statically resolvable after
//!   name resolution. Dynamic class strings remain unsupported in the AOT class table.
//! - Arguments are pre-inferred by the registry common path before the hook runs.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "class_alias",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ClassAlias,
    ),
}

/// Accepts statically resolved alias calls and rejects runtime-dependent class strings.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some((_, alias)) = crate::autoload::resolved_class_alias_args(cx.args) {
        if cx.checker.classes.contains_key(&alias) {
            return Ok(PhpType::Bool);
        }
    }
    Err(CompileError::new(
        cx.span,
        "class_alias() requires statically resolvable class names",
    ))
}
