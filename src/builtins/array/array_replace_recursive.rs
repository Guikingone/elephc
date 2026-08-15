//! Purpose:
//! Home of the PHP `array_replace_recursive` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `fixed(&["array", "replacements"])` (two required
//!   params, no variadic), matching the registry signature. The
//!   param-derived bounds already require exactly 2 arguments, so no `min_args`/
//!   `max_args` override is needed; `check_arity` owns the arity contract.
//! - `check` accepts every PHP array representation. Gradual inputs use a Mixed result and a
//!   runtime-validated boundary; statically known arrays preserve a precise merged hash type.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_replace_recursive",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayReplaceRecursive,
    ),
}

/// Validates both arguments are hash-compatible arrays and returns the merged hash type.
///
/// Arity (exactly 2 args) is pre-validated by `check_arity`. Both arguments are re-inferred here
/// to drive the return type; the registry already inferred every argument once for side effects.
/// Statically known arrays keep their precise merged hash type. Gradual inputs return `Mixed` so
/// lowering can validate their runtime array shapes.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty1 = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let ty2 = cx.checker.infer_type(&cx.args[1], cx.env)?;
    let accepted = |t: &PhpType| {
        matches!(
            t,
            PhpType::Array(_)
                | PhpType::AssocArray { .. }
                | PhpType::Mixed
                | PhpType::Union(_)
        )
    };
    if !accepted(&ty1) || !accepted(&ty2) {
        return Err(CompileError::new(
            cx.span,
            &format!("{}() arguments must be arrays", cx.name),
        ));
    }
    let requires_gradual = |t: &PhpType| matches!(t, PhpType::Mixed | PhpType::Union(_));
    if requires_gradual(&ty1) || requires_gradual(&ty2) {
        return Ok(PhpType::Mixed);
    }
    Ok(PhpType::two_input_hash_result(&ty1, &ty2))
}
