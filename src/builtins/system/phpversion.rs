//! Purpose:
//! Home of the PHP `phpversion` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `phpversion()` returns the PHP LANGUAGE version elephc targets (`--php-version`,
//!   as `8.<minor>.0`), NOT the elephc compiler's own package version. See
//!   `crate::web_prelude::PhpVersion::version_string` for the rule and why the patch
//!   component is `0`.
//! - `phpversion($extension)` returns that same string for a loaded extension and `false`
//!   for anything else, so its checked type is `string|false` (backend repr: boxed
//!   `Mixed`) while the zero-argument form stays a plain `string`.
//! - The loaded-extension set is NOT decided here: `lower_phpversion` answers it with the
//!   exact predicate `extension_loaded()` uses, at the point in the pipeline where the
//!   linked-bridge set is known (see that function for why the decision cannot move
//!   earlier).

use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "phpversion",
    area: System,
    params: [extension: Str = DefaultSpec::Null],
    arity_error: "phpversion() takes 0 or 1 arguments",
    returns: Mixed,
    check: check,
    semantics: phpversion_semantics(),
    summary: "Returns the PHP version elephc targets, or an extension's version string.",
}

/// Builds semantics whose EIR result type matches the backend representation per arity.
const fn phpversion_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::Phpversion);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns the concrete backend layout the lowering produces for each arity.
///
/// Zero arguments always yield a static string. One argument yields either a string or
/// `false`, which the union type `string|false` represents as a boxed `Mixed` cell — so the
/// EIR result type must be `Mixed` there, matching `check`'s `Union(Str, False)` after
/// `codegen_repr()`.
///
/// ARITY IS READ FROM `arg_types`, NOT `args`: `semantics::lower_registry_call` re-resolves
/// this hook with `args: &[]` (it only has lowered EIR operands at that point, not AST
/// expressions) while `ir_lower::expr::registry_builtin_result_type` resolves it with the real
/// AST args. `arg_types` is derived from the operand list in BOTH, so it is the only field
/// that reports the same arity on both paths — keying off `args` silently answered `Str` for
/// `phpversion($ext)` during lowering while the checker answered `string|false`, and the
/// backend then stored a 16-byte string pair over a boxed cell (observed as
/// `phpversion('nope') === ""` instead of `false`).
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    if input.arg_types.is_empty() {
        PhpType::Str
    } else {
        PhpType::Mixed
    }
}

/// Returns `Str` for `phpversion()` and `Union(Str, False)` for `phpversion($extension)`.
///
/// Reference PHP 8.5.6 verified: `phpversion()` is `string`, `phpversion('Zend OPcache')` is
/// `'8.5.6'`, and `phpversion('nope_xyz')` is `false`. The union is returned for EVERY
/// one-argument call, including a literal known to name a loaded extension: the loaded set
/// includes bridges discovered by the type checker itself (`check_result.required_libraries`),
/// so a type computed here could not see them and would contradict the value
/// `lower_phpversion` later folds. Keeping ONE type for the whole arity makes that
/// contradiction impossible.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let Some(arg) = cx.args.first() else {
        return Ok(PhpType::Str);
    };
    let ty = cx.checker.infer_type(arg, cx.env)?;
    if ty.codegen_repr() != PhpType::Str {
        return Err(CompileError::new(
            cx.span,
            "phpversion() extension argument must be string",
        ));
    }
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
