//! Purpose:
//! Home of the PHP `mb_strtolower` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - PHP signature: `mb_strtolower(string $string, ?string $encoding = null): string`.
//! - The conversion runs in the `elephc-iconv` bridge (`crate::case` there), which the
//!   Magician backend links too, so both backends apply the same full Unicode case
//!   mapping, final-sigma rule, invalid-byte substitution, and mbstring encoding-name
//!   validation.
//! - An unknown `$encoding` throws a catchable `ValueError` at run time, never a compile
//!   error: the name is usually a runtime value.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "mb_strtolower",
    check: check,
    lazy_check: true,
    semantics: semantics(),
}

/// Runs through the shared iconv bridge target, keeping a runtime `null` encoding intact.
const fn semantics() -> crate::builtins::semantics::BuiltinSemantics {
    let mut semantics =
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::MbStrtolower);
    semantics.argument_lowering =
        crate::builtins::semantics::BuiltinArgumentLowering::NullableStringOperands;
    semantics
}

/// Validates `mb_strtolower()`'s arguments and returns `PhpType::Str`.
///
/// The hook infers every argument itself so a container passed where PHP declares a
/// string is rejected here instead of reaching the backend.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    super::iconv_strlen::check_string_argument(cx, 0, "mb_strtolower", "string")?;
    super::iconv_strlen::check_nullable_string_argument(cx, 1, "mb_strtolower", "encoding")?;
    Ok(PhpType::Str)
}
