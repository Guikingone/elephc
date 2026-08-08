//! Purpose:
//! Home of the PHP `key` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Returns `null` once the internal pointer has run past either end of the array.
//! - The receiver's internal pointer lives in a compiler-allocated cursor slot beside the
//!   array local, so the argument must be a plain variable. Both that rule and the
//!   argument-type rule are shared with the other five pointer builtins in
//!   `crate::builtins::array::internal_pointer`.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "key",
    area: Array,
    params: [array: Mixed],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::array_pointer_semantics(
        crate::builtins::semantics::ArrayPointerOp::Key,
        crate::ir::RuntimeFnId::ArrayPtrKey,
    ),
    summary: "Returns the key of the element under the array's internal pointer.",
    php_manual: "https://www.php.net/manual/en/function.key.php",
}

/// Validates the receiver shape and type for `key()` and returns `Mixed`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    super::internal_pointer::check_array_pointer_call(cx, "key")
}
