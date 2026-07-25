//! Purpose:
//! Home of the PHP `in_array` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` accepts a concrete or gradually typed array boundary and returns `Bool`.
//! - The optional `strict` (3rd) argument selects PHP `===` membership; omitted or
//!   false strictness uses PHP `==` semantics for the supported scalar/string paths.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "in_array",
    area: Array,
    params: [needle: Mixed, haystack: Mixed, strict: Bool = DefaultSpec::Bool(false)],
    returns: Bool,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::InArray,
    ),
    summary: "Checks if a value exists in an array.",
    php_manual: "https://www.php.net/manual/en/function.in-array.php",
}

/// Validates that the second argument can hold an array and returns `Bool`.
///
/// The registry's `check_arity` handles the 2-to-3 argument range. `Mixed` and unions
/// containing an array defer their concrete check to the EIR runtime boundary; concretely
/// non-array values remain compile errors.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    let arr_ty = cx.checker.infer_type(&cx.args[1], cx.env)?;
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&arr_ty) {
        return Err(CompileError::new(
            cx.span,
            "in_array() second argument must be array",
        ));
    }
    Ok(PhpType::Bool)
}
