//! Purpose:
//! Home of the PHP `prev` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Returns `false` when the pointer steps before the first element, and the pointer stays invalid until `reset()`/`end()`.
//! - The receiver's internal pointer lives in a compiler-allocated cursor slot beside the
//!   array local, so the argument must be a plain variable. Both that rule and the
//!   argument-type rule are shared with the other five pointer builtins in
//!   `crate::builtins::array::internal_pointer`.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "prev",
    check: check,
    semantics: crate::builtins::semantics::array_pointer_semantics(
        crate::builtins::semantics::ArrayPointerOp::Prev,
        crate::ir::RuntimeFnId::ArrayPtrSeek,
    ),
}

/// Validates the receiver shape and type for `prev()` and returns `Mixed`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    super::internal_pointer::check_array_pointer_call(cx, "prev")
}
