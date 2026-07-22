//! Purpose:
//! Home of the PHP `asort` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(fixed(["array"]))`: exactly 1 argument,
//!   the `array` param is by-reference. The `ref` marker is mandatory — it is what makes
//!   by-reference mutation lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - `check` requires the argument be an Array or AssocArray, returning Void.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "asort",
    area: Array,
    params: [ref array: Mixed, flags: Int = crate::builtins::spec::DefaultSpec::Int(0)],
    returns: Void,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Asort,
    ),
    summary: "Sorts an array and maintains index association.",
    php_manual: "https://www.php.net/manual/en/function.asort.php",
}

/// Validates the argument type for an `asort` call.
///
/// Requires the argument be an indexed or associative array. Arity (exactly 1) is
/// pre-validated by the registry. Returns `Ok(PhpType::Void)` on success.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    // Accepted under the gradual boundary (concrete array, `Mixed`, or a union containing
    // an array); a concretely non-array argument stays a compile error.
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
        return Err(CompileError::new(cx.span, &format!("{}() argument must be array", cx.name)));
    }
    Ok(PhpType::Void)
}
