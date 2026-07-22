//! Purpose:
//! Home of the PHP `phpversion` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `phpversion()` returns the compiler package version `Str`; `phpversion($extension)`
//!   queries an extension version and, because elephc has no loadable extensions, is the
//!   concrete PHP `false` (`Bool`). The `check` hook selects the return type by arity so the
//!   backend's `lower_phpversion` (which emits the bool in the 1-arg form) stays type-coherent.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "phpversion",
    area: System,
    params: [extension: Mixed = crate::builtins::spec::DefaultSpec::Null],
    returns: Str,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Phpversion,
    ),
    summary: "Returns the current PHP / elephc compiler version string.",
}

/// Type-checks `phpversion()` / `phpversion($extension)` and returns its arity-dependent type.
///
/// The zero-argument form is the compiler version `Str`; any one-argument extension query is the
/// concrete PHP `false` (`Bool`) since elephc has no loadable extensions. Returning `Bool` here
/// keeps the checker in step with `lower_phpversion`, which emits the boolean for the 1-arg form.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if let Some(arg) = cx.args.first() {
        cx.checker.infer_type(arg, cx.env)?;
        return Ok(PhpType::Bool);
    }
    Ok(PhpType::Str)
}
