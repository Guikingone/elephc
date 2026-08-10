//! Purpose:
//! Home of the PHP `range` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - PHP's signature is `range($start, $end, int|float $step = 1)`; `step` works positionally and
//!   as a named argument.
//! - `check` infers every argument and always returns `Array(Int)`: the supported endpoints and
//!   step are integers, so the produced range is an indexed integer array.
//! - The three `ValueError`s php-src raises for a bad `$step` (zero, negative on an increasing
//!   range, wider than the spanned interval) are runtime guards emitted by `lower_range`, because
//!   the endpoints and the step can all be runtime values.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "range",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Range,
    ),
}

/// Infers every argument and returns `Array(Int)`.
///
/// The registry's `check_arity` handles arity enforcement (2 or 3 arguments).
/// All arguments are inferred for side-effect tracking; the return type is always
/// an indexed integer array matching the runtime emitter's output shape.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for index in 0..cx.args.len() {
        cx.checker.infer_type(&cx.args[index], cx.env)?;
    }
    Ok(PhpType::Array(Box::new(PhpType::Int)))
}
