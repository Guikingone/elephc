//! Purpose:
//! Home of the PHP `unserialize` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The check hook validates the data argument is string-compatible.
//!   The optional options argument is accepted without type restriction.
//!   Type errors are reported at the offending argument's span.
//! - `options` default is `DefaultSpec::EmptyArray` (matches legacy `ArrayLiteral([])`
//!   for parity gate comparison).

use crate::builtins::spec::BuiltinCheckCtx;
use crate::builtins::system::json_support;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "unserialize",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Unserialize,
    ),
}

/// Validates that the data argument is string-compatible.
///
/// The optional options argument and its `allowed_classes` policy are validated
/// by the runtime helpers so dynamic values cannot be interpreted as untyped words.
/// Reports type errors at the span of the offending argument, not the call span.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let data_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !json_support::is_json_string_arg_type(&data_ty) {
        return Err(CompileError::new(
            cx.args[0].span,
            "unserialize() data argument must be string-compatible",
        ));
    }
    Ok(PhpType::Mixed)
}
