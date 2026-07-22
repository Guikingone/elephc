//! Purpose:
//! Home of the PHP `defined` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - A literal name is folded to a static boolean by `lower_static_defined_call`; a
//!   non-literal (runtime) name lowers through the typed `RuntimeFnId::Defined` registry
//!   lookup (`__rt_defined`), mirroring `constant()`'s `__rt_constant` fallback.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "defined",
    area: System,
    params: [constant_name: Str],
    returns: Bool,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Defined,
    ),
    summary: "Checks whether the given named constant exists.",
}

/// Type-checks `defined($name)` and returns `PhpType::Bool`.
///
/// The name may be any string expression: a literal call folds to a static boolean in
/// `lower_static_defined_call`, and a non-literal name is resolved at runtime through
/// the `RuntimeFnId::Defined` registry lookup, so no string-literal restriction applies.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(PhpType::Bool)
}
