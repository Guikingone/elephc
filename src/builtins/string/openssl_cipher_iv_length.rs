//! Purpose:
//! Home of PHP's `openssl_cipher_iv_length` builtin and typed bridge target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through the builtin registry.
//!
//! Key details:
//! - Unknown ciphers produce `false`, so the checked result uses an `int|false` union.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "openssl_cipher_iv_length",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::OpensslCipherIvLength,
    ),
}

/// Returns the PHP `int|false` result contract for IV-length lookup.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Int, PhpType::False]))
}
