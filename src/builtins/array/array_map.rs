//! Purpose:
//! Home of the PHP `array_map` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `variadic(&["callback","array"], "arrays")` (two
//!   required params plus a variadic `arrays`). The legacy CHECK arm required exactly
//!   2 arguments, so `min_args: 2, max_args: 2` reproduce that enforcement in
//!   `check_arity` only; `function_sig` and the parity gate keep the variadic shape.
//! - `check` validates that the second argument is an indexed array and infers the
//!   callback return element type; the mapped array uses that return type, not the
//!   input array's element type.
//! - Refcounted object elements use the same pointer-sized callback ABI as other
//!   8-byte array slots; they do not require a checker-only rejection.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemantics,
};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_map",
    area: Array,
    params: [callback: Mixed, array: Mixed],
    variadic: "arrays",
    min_args: 2,
    max_args: 2,
    returns: Mixed,
    check: check,
    semantics: array_map_semantics(),
    summary: "Applies a callback to the elements of an array.",
    php_manual: "https://www.php.net/manual/en/function.array-map.php",
}

/// Returns the result type of an `array_map()` call whose array argument is a gradual boundary.
///
/// Single source of truth for the two layers that must agree about it: the `check` hook below,
/// and `crate::ir_lower`'s rewrite of the gradual call into
/// `array_combine(array_keys($a), array_map($cb, array_values($a)))`, whose `array_combine` result
/// is exactly this keyed type. PHP's single-array `array_map()` preserves its source's keys, so
/// this must NOT be a list type: a list source simply yields keys `0..n-1`, which is the same PHP
/// array, while a hash source keeps its own keys instead of being silently reindexed.
pub(crate) fn gradual_result_type() -> PhpType {
    PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    }
}

/// Builds semantics that reuse the callback-sensitive result recorded by the checker.
const fn array_map_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::ArrayMap);
    semantics.result_type = BuiltinResultType::Checked;
    semantics
}

/// Returns the mapped array type for an `array_map` call.
///
/// Validates that the second argument is an indexed array, checks the callback
/// with its contextual element type, and derives the result element type from the callback
/// return type. Arity (exactly 2 args) is pre-validated by `check_arity`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[1], cx.env)?;
    match arr_ty {
        PhpType::Array(elem_ty) => {
            let callback_arg_types = [elem_ty.as_ref().clone()];
            let callback_ret_ty =
                crate::types::checker::builtins::check_array_callback_builtin_call(
                    cx.checker,
                    &cx.args[0],
                    &callback_arg_types,
                    cx.span,
                    cx.env,
                    "array_map() callback",
                )?;
            Ok(PhpType::Array(Box::new(callback_ret_ty)))
        }
        // Gradual boundary: a `Mixed` or union-containing-array argument is accepted; the element
        // type is unknown, so the callback is checked against a `Mixed` element.
        //
        // The RESULT must not promise a list. PHP's single-array `array_map()` PRESERVES its
        // source's keys, so a gradual source that turns out to hold a string-keyed hash produces a
        // hash, which an `array<mixed>` slot cannot represent — promising one would leave the
        // backend no faithful option but a silent key-losing reindex, the same shape-promise
        // defect fixed for `array_keys`. `crate::ir_lower` rewrites the gradual call into
        // `array_combine(array_keys($a), array_map($cb, array_values($a)))`, whose result is
        // exactly this keyed type, and answers it from the same helper so the two cannot drift.
        t if crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&t) => {
            let callback_arg_types = [PhpType::Mixed];
            crate::types::checker::builtins::check_array_callback_builtin_call(
                cx.checker,
                &cx.args[0],
                &callback_arg_types,
                cx.span,
                cx.env,
                "array_map() callback",
            )?;
            Ok(gradual_result_type())
        }
        _ => Err(CompileError::new(
            cx.span,
            "array_map() second argument must be array",
        )),
    }
}
