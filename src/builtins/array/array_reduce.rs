//! Purpose:
//! Home of the PHP `array_reduce` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP signature accepts two or three arguments, with a `null` initial carry by default.
//! - `check` validates the callback with the inferred initial and array-element types.
//!   The return type is `PhpType::Int`, matching the legacy arm.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "array_reduce",
    area: Array,
    params: [array: Mixed, callback: Mixed, initial: Mixed = DefaultSpec::Null],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayReduce,
    ),
    summary: "Iteratively reduces an array to a single value using a callback function.",
    php_manual: "https://www.php.net/manual/en/function.array-reduce.php",
}

/// Validates the callback for an `array_reduce` call and returns `PhpType::Int`.
///
/// Uses the initial-value and array-element types as the two callback parameter contexts. A
/// missing initial value contributes PHP `null` (`Void`) to the callback carry contract.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let initial_ty = match cx.args.get(2) {
        Some(initial) => cx.checker.infer_type(initial, cx.env)?,
        None => PhpType::Void,
    };
    let callback_arg_types = [
        initial_ty,
        crate::types::checker::builtins::array_element_type(&arr_ty),
    ];
    crate::types::checker::builtins::check_array_callback_builtin_call(
        cx.checker,
        &cx.args[1],
        &callback_arg_types,
        cx.span,
        cx.env,
        "array_reduce() callback",
    )?;
    Ok(PhpType::Int)
}
