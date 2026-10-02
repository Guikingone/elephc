//! Purpose:
//! Home of the PHP `array_product` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` computes the actual return type (Int or Float) based on the element type
//!   of the argument array. The declared `returns: Int` is only used as the FCC type.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "array_product",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayProduct,
    ),
}

/// Computes the return type (Int or Float) based on the array element type.
///
/// The registry's `check_arity` handles arity enforcement (exactly 1 argument).
/// A float-element array yields Float; integer or mixed-element arrays yield Int.
/// Non-array arguments are rejected.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    // Float elements fold to a float and int or bool elements to an int; any other element can
    // produce either, which only `mixed` represents. Typing those `int` truncated `[1, 1.5]` to
    // `2`. A value not known to be an array is still refused, as before.
    let element = match ty.codegen_repr() {
        PhpType::Array(element) => Some(element.codegen_repr()),
        PhpType::AssocArray { value, .. } => Some(value.codegen_repr()),
        _ => None,
    };
    match element {
        Some(PhpType::Float) => Ok(PhpType::Float),
        Some(PhpType::Int | PhpType::Bool | PhpType::False | PhpType::Never | PhpType::Void) => {
            Ok(PhpType::Int)
        }
        Some(_) => Ok(PhpType::Mixed),
        None => Err(CompileError::new(
            cx.span,
            "array_product() argument must be array",
        )),
    }
}
