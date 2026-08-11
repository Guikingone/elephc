//! Purpose:
//! Eval registry entry and bridge lookup for `openssl_cipher_iv_length`.
//!
//! Called from:
//! - The shared OpenSSL direct and evaluated-argument hooks.
//!
//! Key details:
//! - Supported ciphers return their PHP-visible IV length; unknown names return `false`.

eval_builtin! {
    name: "openssl_cipher_iv_length",
    area: String,
    params: [cipher_algo],
    direct: Openssl,
    values: Openssl,
}

use super::super::super::*;

/// Evaluates positional `openssl_cipher_iv_length()` expressions.
pub(in crate::interpreter) fn eval_builtin_openssl_cipher_iv_length(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [cipher] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let cipher = eval_expr(cipher, context, scope, values)?;
    eval_openssl_cipher_iv_length_values_result(&[cipher], values)
}

/// Returns the bridge-reported IV length for one evaluated cipher name.
pub(in crate::interpreter) fn eval_openssl_cipher_iv_length_values_result(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [cipher] = evaluated_args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let cipher = values.string_bytes(*cipher)?;
    let length = unsafe {
        elephc_crypto::elephc_crypto_cipher_iv_length(cipher.as_ptr(), cipher.len())
    };
    if length < 0 {
        values.bool_value(false)
    } else {
        values.int(i64::try_from(length).map_err(|_| EvalStatus::RuntimeFatal)?)
    }
}
