//! Purpose:
//! Home of the PHP `array_filter` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP golden signature is `optional(&["array","callback","mode"], 1, &[null, 0])`.
//! - `check` validates the first argument is an array, derives callback argument types from the
//!   static mode value, and validates a non-null callback signature. Omitting the callback, or
//!   passing `null`, selects PHP truthiness filtering. The return type preserves the input
//!   container shape and element metadata.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinResultType, BuiltinSemanticInput, BuiltinSemantics,
};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_filter",
    check: check,
    semantics: array_filter_semantics(),
}

/// Builds semantics whose storage result follows the source container representation.
const fn array_filter_semantics() -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(crate::ir::RuntimeFnId::ArrayFilter);
    semantics.result_type = BuiltinResultType::Shared(eir_result_type);
    semantics
}

/// Returns a boxed result for gradual sources and preserves concrete array metadata.
fn eir_result_type(input: &BuiltinSemanticInput<'_>) -> PhpType {
    match input.arg_types.first().map(PhpType::codegen_repr) {
        Some(PhpType::Array(elem)) => PhpType::Array(elem),
        Some(PhpType::AssocArray { key, value }) => PhpType::AssocArray { key, value },
        _ => PhpType::Mixed,
    }
}

/// Returns the filtered array type for an `array_filter` call.
///
/// Validates the first argument is an array, derives callback argument types
/// from the optional mode argument, and validates a non-null callback. Arity is
/// pre-validated by the registry.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let arr_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if let Some(mode) = cx.args.get(2) {
        cx.checker.infer_type(mode, cx.env)?;
    }
    match arr_ty {
        arr_ty @ PhpType::Array(_) | arr_ty @ PhpType::AssocArray { .. } => {
            if let Some(callback) = cx.args.get(1) {
                if !matches!(callback.kind, crate::parser::ast::ExprKind::Null) {
                    let callback_arg_types =
                        crate::types::checker::builtins::array_filter_callback_arg_types(
                            &arr_ty,
                            cx.args.get(2),
                        );
                    crate::types::checker::builtins::check_array_callback_builtin_call(
                        cx.checker,
                        callback,
                        &callback_arg_types,
                        cx.span,
                        cx.env,
                        "array_filter() callback",
                    )?;
                }
            }
            Ok(arr_ty)
        }
        PhpType::Mixed | PhpType::Union(_) => Ok(PhpType::Mixed),
        _ => Err(CompileError::new(
            cx.span,
            "array_filter() first argument must be array",
        )),
    }
}
