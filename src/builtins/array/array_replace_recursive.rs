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
//! - `check` accepts associative and indexed arrays, including nested indexed values,
//!   and the result is the two-input hash result type. A
//!   check hook is required because the return type depends on the inferred arguments.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_replace_recursive",
    area: Array,
    params: [array: Mixed],
    variadic: "replacements",
    arity_error: "array_replace_recursive() requires at least 1 argument",
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayReplaceRecursive,
    ),
    summary: "Replaces elements from passed arrays into the first array recursively.",
    php_manual: "https://www.php.net/manual/en/function.array-replace-recursive.php",
}

/// Validates every argument is a hash-compatible array and returns the merged hash type.
///
/// Arity is pre-validated by `check_arity`. Arguments are re-inferred here to drive
/// the return type; the registry already inferred every argument once for side effects.
/// The indexed-to-hash runtime conversion preserves nested array payloads, so nested
/// indexed arrays are accepted alongside associative and gradual array operands.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    // PHP: `array_replace(array $array, array ...$replacements): array` — 1+ arguments.
    // Each argument must be an associative array or an indexed array of scalars, accepted
    // under the gradual boundary (`Mixed`/union-containing-array operands defer to the
    // runtime guard).
    // The indexed-to-hash runtime path reads the packed value-type header and retains
    // heap-backed entries, including nested indexed arrays. Non-concrete gradual operands
    // (`Mixed` or a union containing an array) defer the check to runtime.
    let accepted = |t: &PhpType| {
        matches!(t, PhpType::AssocArray { .. } | PhpType::Array(_))
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
