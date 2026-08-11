//! Purpose:
//! Home of PHP's `openssl_get_cipher_methods` builtin and supported-method inventory target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through the builtin registry.
//!
//! Key details:
//! - The returned indexed array contains only ciphers implemented by elephc-crypto.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "openssl_get_cipher_methods",
    area: String,
    params: [aliases: Bool = DefaultSpec::Bool(false)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::OpensslGetCipherMethods,
    ),
    summary: "Returns the supported OpenSSL cipher method names.",
    php_manual: "https://www.php.net/manual/en/function.openssl-get-cipher-methods.php",
}

/// Returns an indexed string-array type for the method inventory.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
