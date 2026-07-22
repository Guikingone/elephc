//! Purpose:
//! Home of the PHP `array_replace` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `fixed(&["array", "replacements"])` (two required
//!   params, no variadic), matching the registry signature. The
//!   param-derived bounds already require exactly 2 arguments, so no `min_args`/
//!   `max_args` override is needed; `check_arity` owns the arity contract.
//! - `check` enforces that both arguments are associative arrays or
//!   indexed arrays of scalars, and the result is the two-input hash result type. A
//!   check hook is required because the return type depends on the inferred arguments.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_replace",
    area: Array,
    params: [array: Mixed],
    variadic: "replacements",
    arity_error: "array_replace() requires at least 1 argument",
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayReplace,
    ),
    summary: "Replaces elements from passed arrays into the first array.",
    php_manual: "https://www.php.net/manual/en/function.array-replace.php",
}

/// Validates both arguments are hash-compatible arrays and returns the merged hash type.
///
/// Arity (exactly 2 args) is pre-validated by `check_arity`. Both arguments are
/// re-inferred here to drive the return type; the registry already inferred every
/// argument once for side effects. Each operand must be an associative array or an
/// indexed array of scalars; the result widens key/value to `Mixed` when the operands
/// disagree, via `PhpType::two_input_hash_result`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    // PHP: `array_replace(array $array, array ...$replacements): array` — 1+ arguments.
    // Each argument must be an associative array or an indexed array of scalars, accepted
    // under the gradual boundary (`Mixed`/union-containing-array operands defer to the
    // runtime guard).
    // Concrete arrays must be associative or scalar-indexed (string/heap-element indexed
    // inputs stay a loud error until the runtime supports them); only non-concrete gradual
    // operands (`Mixed` or a union containing an array) defer the check to runtime.
    let accepted = |t: &PhpType| {
        matches!(t, PhpType::AssocArray { .. })
            || t.is_scalar_indexed_array()
            || (matches!(t, PhpType::Mixed | PhpType::Union(_))
                && crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(t))
    };
    let mut result = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !accepted(&result) {
        return Err(CompileError::new(
            cx.span,
            &format!(
                "{}() arguments must be associative arrays or indexed arrays of scalars",
                cx.name
            ),
        ));
    }
    for arg in &cx.args[1..] {
        let ty = cx.checker.infer_type(arg, cx.env)?;
        if !accepted(&ty) {
            return Err(CompileError::new(
                cx.span,
                &format!(
                    "{}() arguments must be associative arrays or indexed arrays of scalars",
                    cx.name
                ),
            ));
        }
        result = PhpType::two_input_hash_result(&result, &ty);
    }
    Ok(result)
}
