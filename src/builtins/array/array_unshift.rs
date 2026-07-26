//! Purpose:
//! Home of the PHP `array_unshift` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(variadic(["array"], "values"))`: `array`
//!   by-ref plus a variadic `values` param. The legacy CHECK arm enforced exactly 2
//!   arguments, so `min_args: 2, max_args: 2` reproduce that enforcement in `check_arity`
//!   only; `function_sig` and the parity gate keep the variadic shape from the golden.
//! - The `ref` marker on `array` is mandatory — it is what makes by-reference mutation
//!   lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - Returns `Int` — the new number of elements in the array.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_unshift",
    area: Array,
    params: [ref array: Mixed],
    variadic: "values",
    min_args: 2,
    returns: Int,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayUnshift,
    ),
    summary: "Prepends one or more elements to the beginning of an array.",
    php_manual: "https://www.php.net/manual/en/function.array-unshift.php",
}

/// Validates the first argument is an array for an `array_unshift` call.
///
/// Arity (exactly 2 args) is pre-validated by `check_arity`. Both arguments are inferred
/// to produce any side effects; the first must be an indexed or associative array or the
/// call is rejected. Returns `Int` — the new element count.
///
/// Gradual-typing note: this hook intentionally stays STRICT (concrete array only). Relaxing it
/// to accept a `Mixed`/union receiver was investigated and rejected as unsafe — the EIR dynamic
/// path (`crate::codegen::lower_inst::builtins::arrays::unshift::lower_array_unshift_dynamic`)
/// (a) supports only `Int`/`Bool` prepended values (an array/`Mixed` value — as in Symfony's
/// `ContainerBuilder::prependExtensionConfig`'s `array_unshift($this->extensionConfigs[$name],
/// $config)` — hits an unsupported EIR backend error, producing no binary), and (b) corrupts the
/// refcount when the receiver is a direct function parameter (proven `bad refcount` under
/// `--heap-debug`). Accepting the receiver would move the diagnostic from the counted checker
/// phase to the uncounted backend phase without producing working code (a false-green), so the
/// gap is kept loud pending a value-generalized, ownership-correct dynamic path.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    cx.checker.infer_type(&cx.args[1], cx.env)?;
    if !matches!(arr_ty, PhpType::Array(_) | PhpType::AssocArray { .. }) {
        return Err(CompileError::new(
            cx.span,
            "array_unshift() first argument must be array",
        ));
    }
    Ok(PhpType::Int)
}
