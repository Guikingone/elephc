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
/// to accept a `Mixed`/union receiver was investigated and rejected as unsafe. The two Symfony
/// call sites — `ContainerBuilder::prependExtensionConfig` and `FileLoader`'s importing branch,
/// both `array_unshift($this->extensionConfigs[$name], $config)` — expose TWO independent
/// blockers:
///  (a) the prepended value is an ARRAY (`$config`), whereas the EIR dynamic path
///      (`crate::codegen::lower_inst::builtins::arrays::unshift::lower_array_unshift_dynamic`)
///      only handles `Int`/`Bool` prepended values; a value-generalized prepend (boxing the
///      value as a Mixed cell, mirroring the landed `array_shift` / `array_combine` Mixed paths)
///      is solvable, BUT
///  (b) the RECEIVER is a nested property-array-element (`$this->extensionConfigs[$name]`), not a
///      plain local. The by-ref writeback used by `array_pop` / `array_shift` relies on
///      `source_load_local_slot`, which only targets a direct `LoadLocal` (returns `None` here),
///      so the shared-cell divergence branch would publish a fresh Mixed cell with NOWHERE to
///      store it — the mutation to `$this->extensionConfigs[$name]` would be silently dropped (a
///      write-through false-green, the exact failure the [[refprop-nested-append-writethrough]]
///      gap describes; it blocks PassConfig/PhpDumper). Accepting the receiver would move the
///      diagnostic from the counted checker phase to a dropped writeback, so the gap is kept loud
///      pending nested property-element write-through support plus a value-generalized dynamic
///      path.
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
