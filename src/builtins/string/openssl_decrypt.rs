//! Purpose:
//! Home of the PHP `openssl_decrypt` builtin and its typed crypto runtime target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through the builtin registry.
//!
//! Key details:
//! - The checked result is `string|false`; phase 2 handles CBC, CTR, and ECB.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "openssl_decrypt",
    area: String,
    params: [
        data: Str,
        cipher_algo: Str,
        passphrase: Str,
        options: Int = DefaultSpec::Int(0),
        iv: Str = DefaultSpec::Str(""),
        tag: Mixed = DefaultSpec::Null,
        aad: Str = DefaultSpec::Str("")
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::OpensslDecrypt,
    ),
    summary: "Decrypts data with a supported AES cipher.",
    php_manual: "https://www.php.net/manual/en/function.openssl-decrypt.php",
}

/// Returns the PHP `string|false` result contract for decryption.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
