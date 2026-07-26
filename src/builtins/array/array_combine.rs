//! Purpose:
//! Home of the PHP `array_combine` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` reproduces the legacy rule: the result is an associative array whose key
//!   type is derived from the keys-array element type (via
//!   `array_key_type_from_value_type`) and whose value type is the values-array element
//!   type. Both arguments must be indexed arrays. A check hook is required because the
//!   return type depends on the two inferred argument types.
//! - Arity (exactly 2 arguments) is validated by the registry's `check_arity` before
//!   the hook fires; the inline arity check from the legacy arm is not reproduced here.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::{array_key_type_from_value_type, PhpType};

builtin! {
    name: "array_combine",
    area: Array,
    params: [keys: Mixed, values: Mixed],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ArrayCombine,
    ),
    summary: "Creates an array by using one array for keys and another for values.",
    php_manual: "https://www.php.net/manual/en/function.array-combine.php",
}

/// Returns the combined associative-array type for an `array_combine` call.
///
/// The key type is derived from the keys-array element type via
/// `array_key_type_from_value_type`, and the value type is the values-array element
/// type. A concrete `Array(elem)` operand yields its element type; a Mixed /
/// array-containing-union operand is accepted under the gradual-typing boundary and
/// contributes `Mixed` (codegen's `lower_array_combine` routes such calls through the
/// runtime-tag `__rt_array_combine_mixed` helper, which normalizes both operands to hashes
/// and pairs their values positionally). A concretely non-array operand is rejected so
/// genuine type errors keep being reported.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let keys_ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let vals_ty = cx.checker.infer_type(&cx.args[1], cx.env)?;
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&keys_ty) {
        return Err(CompileError::new(
            cx.span,
            "array_combine() first argument must be array",
        ));
    }
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&vals_ty) {
        return Err(CompileError::new(
            cx.span,
            "array_combine() second argument must be array",
        ));
    }
    let key_elem = match keys_ty {
        PhpType::Array(elem) => *elem,
        _ => PhpType::Mixed,
    };
    let val_elem = match vals_ty {
        PhpType::Array(elem) => *elem,
        _ => PhpType::Mixed,
    };
    Ok(PhpType::AssocArray {
        key: Box::new(array_key_type_from_value_type(key_elem)),
        value: Box::new(val_elem),
    })
}
