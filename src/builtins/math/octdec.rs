//! Purpose:
//! Home of the PHP `octdec` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `octdec(string $octal_string): int|float`. The union return
//!   type cannot be spelled in the `builtin!` `returns:` field, so the declared type is
//!   `Mixed` (the shared codegen representation of a union) and the `check` hook supplies the
//!   precise `int|float` contract.
//! - Characters that are not octal digits are IGNORED rather than ending the scan, and the
//!   result widens to `float` once it would exceed `PHP_INT_MAX`; both behaviours live in the
//!   shared `__rt_base_to_number` runtime helper.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "octdec",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Octdec,
    ),
}

/// Returns `PhpType::Union([Int, Float])` for a `octdec` call.
///
/// The `builtin!` macro cannot express a union return type inline, so the precise
/// `int|float` contract is supplied here. A value that fits `PHP_INT_MAX` is an `int`;
/// anything larger widens to `float`, which is only decidable at runtime.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Union(vec![PhpType::Int, PhpType::Float]))
}
