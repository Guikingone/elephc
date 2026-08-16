//! Purpose:
//! Eval registry entry and bridge-backed implementation for `openssl_decrypt`.
//!
//! Called from:
//! - The shared OpenSSL direct and evaluated-argument hooks.
//!
//! Key details:
//! - Default-mode input is decoded from Base64 before reaching the raw crypto bridge.
//! - GCM authentication failures and all stable bridge errors return PHP `false`.

eval_builtin! {
    contract: "openssl_decrypt",
    area: String,
    direct: Openssl,
    values: Openssl,
}

use super::super::super::*;

const OPENSSL_RAW_DATA: u32 = 1;

/// Evaluates positional `openssl_decrypt()` expressions in source order.
pub(in crate::interpreter) fn eval_builtin_openssl_decrypt(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut evaluated_args = Vec::with_capacity(args.len());
    for arg in args {
        evaluated_args.push(eval_expr(arg, context, scope, values)?);
    }
    eval_openssl_decrypt_values_result(&evaluated_args, values)
}

/// Decrypts already evaluated PHP arguments through the raw `elephc-crypto` ABI.
pub(in crate::interpreter) fn eval_openssl_decrypt_values_result(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !(3..=7).contains(&evaluated_args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let mut data = values.string_bytes(evaluated_args[0])?;
    let cipher = values.string_bytes(evaluated_args[1])?;
    let passphrase = values.string_bytes(evaluated_args[2])?;
    let options = evaluated_args
        .get(3)
        .map_or(Ok(0), |value| eval_int_value(*value, values))? as u32;
    let iv = evaluated_args
        .get(4)
        .map_or_else(|| Ok(Vec::new()), |value| values.string_bytes(*value))?;
    let tag = evaluated_args
        .get(5)
        .map_or_else(|| Ok(Vec::new()), |value| values.string_bytes(*value))?;
    let aad = evaluated_args
        .get(6)
        .map_or_else(|| Ok(Vec::new()), |value| values.string_bytes(*value))?;
    if options & OPENSSL_RAW_DATA == 0 {
        data = super::base64_decode::eval_base64_decode_bytes(&data);
    }
    let mut output = vec![0_u8; data.len()];
    let mut output_length = 0_usize;
    let status = unsafe {
        elephc_crypto::elephc_crypto_decrypt(
            cipher.as_ptr(),
            cipher.len(),
            data.as_ptr(),
            data.len(),
            passphrase.as_ptr(),
            passphrase.len(),
            iv.as_ptr(),
            iv.len(),
            options,
            aad.as_ptr(),
            aad.len(),
            tag.as_ptr(),
            tag.len(),
            output.as_mut_ptr(),
            output.len(),
            &mut output_length,
        )
    };
    if status != elephc_crypto::CIPHER_OK {
        return values.bool_value(false);
    }
    if output_length > output.len() {
        return Err(EvalStatus::RuntimeFatal);
    }
    output.truncate(output_length);
    values.string_bytes_value(&output)
}
