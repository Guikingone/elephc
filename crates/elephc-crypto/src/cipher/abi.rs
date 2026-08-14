//! Purpose:
//! Panic-contained C ABI for the crypto bridge's symmetric-cipher operations.
//!
//! Called from:
//! - Compiled PHP runtime helpers through published function-pointer slots.
//! - `elephc-magician` and `cargo test -p elephc-crypto`.
//!
//! Key details:
//! - Input pointer/length pairs reject null pointers when the length is nonzero.
//! - Output lengths are reset on entry and report required sizes on `-7`.
//! - Every public entry point contains Rust panics and converts them to `-8`.

use super::{
    cipher_iv_length, decrypt, encrypt, CipherError, EncryptOutput, CIPHER_ERR_INVALID_ARGUMENT,
    CIPHER_ERR_OUTPUT_TOO_SMALL, CIPHER_METHODS, CIPHER_METHODS_PACKED, CIPHER_OK,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Returns a checked byte slice for one C pointer/length pair.
unsafe fn input_slice<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], CipherError> {
    if len == 0 {
        Ok(&[])
    } else if ptr.is_null() {
        Err(CipherError::InvalidArgument)
    } else {
        Ok(std::slice::from_raw_parts(ptr, len))
    }
}

/// Writes an output length when the pointer is non-null.
unsafe fn write_len(ptr: *mut usize, value: usize) {
    if !ptr.is_null() {
        *ptr = value;
    }
}

/// Publishes ciphertext and tag into caller-owned buffers.
unsafe fn publish_encrypt(
    result: EncryptOutput,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
    tag_out_ptr: *mut u8,
    tag_out_cap: usize,
    tag_out_len: *mut usize,
) -> i32 {
    write_len(out_len, result.ciphertext.len());
    write_len(tag_out_len, result.tag.len());
    let output_invalid = result.ciphertext.len() > out_cap
        || (!result.ciphertext.is_empty() && out_ptr.is_null());
    let tag_invalid = result.tag.len() > tag_out_cap || (!result.tag.is_empty() && tag_out_ptr.is_null());
    if output_invalid || tag_invalid {
        return CIPHER_ERR_OUTPUT_TOO_SMALL;
    }
    if !result.ciphertext.is_empty() {
        std::ptr::copy_nonoverlapping(result.ciphertext.as_ptr(), out_ptr, result.ciphertext.len());
    }
    if !result.tag.is_empty() {
        std::ptr::copy_nonoverlapping(result.tag.as_ptr(), tag_out_ptr, result.tag.len());
    }
    CIPHER_OK
}

/// Publishes plaintext into a caller-owned buffer.
unsafe fn publish_decrypt(
    plaintext: Vec<u8>,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    write_len(out_len, plaintext.len());
    if plaintext.len() > out_cap || (!plaintext.is_empty() && out_ptr.is_null()) {
        return CIPHER_ERR_OUTPUT_TOO_SMALL;
    }
    if !plaintext.is_empty() {
        std::ptr::copy_nonoverlapping(plaintext.as_ptr(), out_ptr, plaintext.len());
    }
    CIPHER_OK
}

/// Returns the default IV length for a supported cipher, or a negative status.
///
/// # Safety
/// `name_ptr` must be valid for `name_len` bytes unless `name_len` is zero.
#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_cipher_iv_length(
    name_ptr: *const u8,
    name_len: usize,
) -> isize {
    match catch_unwind(AssertUnwindSafe(|| {
        let name = input_slice(name_ptr, name_len)?;
        cipher_iv_length(name)
    })) {
        Ok(Ok(length)) => length as isize,
        Ok(Err(error)) => error.status() as isize,
        Err(_) => CIPHER_ERR_INVALID_ARGUMENT as isize,
    }
}

/// Writes all supported lowercase cipher names as trailing-NUL packed strings.
/// Returns the method count on success. `aliases` currently produces the same
/// canonical list for both zero and nonzero values.
///
/// On `-7`, `out_len` still receives the required packed byte length.
///
/// # Safety
/// `out_len` must be writable. `out_ptr` must hold `out_cap` bytes when non-null.
#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_cipher_methods(
    _aliases: i32,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> isize {
    if out_len.is_null() {
        return CIPHER_ERR_INVALID_ARGUMENT as isize;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        write_len(out_len, CIPHER_METHODS_PACKED.len());
        if CIPHER_METHODS_PACKED.len() > out_cap || out_ptr.is_null() {
            return Err(CipherError::OutputTooSmall);
        }
        std::ptr::copy_nonoverlapping(
            CIPHER_METHODS_PACKED.as_ptr(),
            out_ptr,
            CIPHER_METHODS_PACKED.len(),
        );
        Ok(CIPHER_METHODS.len() as isize)
    })) {
        Ok(Ok(count)) => count,
        Ok(Err(error)) => error.status() as isize,
        Err(_) => CIPHER_ERR_INVALID_ARGUMENT as isize,
    }
}

/// Encrypts raw bytes and writes raw ciphertext plus an optional GCM tag.
/// Returns zero on success or a stable negative status code.
///
/// `OPENSSL_RAW_DATA` is accepted but ignored because base64 belongs to the PHP
/// glue layer. `out_len` and `tag_out_len` receive required sizes on `-7`.
///
/// # Safety
/// Every input pointer must be valid for its stated nonzero length. Output
/// pointers must hold their advertised capacities; both length pointers must be
/// writable.
#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_encrypt(
    cipher_ptr: *const u8,
    cipher_len: usize,
    data_ptr: *const u8,
    data_len: usize,
    key_ptr: *const u8,
    key_len: usize,
    iv_ptr: *const u8,
    iv_len: usize,
    options: u32,
    aad_ptr: *const u8,
    aad_len: usize,
    tag_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
    tag_out_ptr: *mut u8,
    tag_out_cap: usize,
    tag_out_len: *mut usize,
) -> i32 {
    if out_len.is_null() || tag_out_len.is_null() {
        return CIPHER_ERR_INVALID_ARGUMENT;
    }
    write_len(out_len, 0);
    write_len(tag_out_len, 0);
    match catch_unwind(AssertUnwindSafe(|| {
        let cipher = input_slice(cipher_ptr, cipher_len)?;
        let data = input_slice(data_ptr, data_len)?;
        let key = input_slice(key_ptr, key_len)?;
        let iv = input_slice(iv_ptr, iv_len)?;
        let aad = input_slice(aad_ptr, aad_len)?;
        encrypt(cipher, data, key, iv, options, aad, tag_len)
    })) {
        Ok(Ok(result)) => publish_encrypt(
            result,
            out_ptr,
            out_cap,
            out_len,
            tag_out_ptr,
            tag_out_cap,
            tag_out_len,
        ),
        Ok(Err(error)) => error.status(),
        Err(_) => CIPHER_ERR_INVALID_ARGUMENT,
    }
}

/// Decrypts raw bytes and writes authenticated/unpadded plaintext.
/// Returns zero on success or a stable negative status code.
///
/// # Safety
/// Every input pointer must be valid for its stated nonzero length. `out_ptr`
/// must hold `out_cap` bytes and `out_len` must be writable.
#[no_mangle]
pub unsafe extern "C" fn elephc_crypto_decrypt(
    cipher_ptr: *const u8,
    cipher_len: usize,
    data_ptr: *const u8,
    data_len: usize,
    key_ptr: *const u8,
    key_len: usize,
    iv_ptr: *const u8,
    iv_len: usize,
    options: u32,
    aad_ptr: *const u8,
    aad_len: usize,
    tag_ptr: *const u8,
    tag_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
    out_len: *mut usize,
) -> i32 {
    if out_len.is_null() {
        return CIPHER_ERR_INVALID_ARGUMENT;
    }
    write_len(out_len, 0);
    match catch_unwind(AssertUnwindSafe(|| {
        let cipher = input_slice(cipher_ptr, cipher_len)?;
        let data = input_slice(data_ptr, data_len)?;
        let key = input_slice(key_ptr, key_len)?;
        let iv = input_slice(iv_ptr, iv_len)?;
        let aad = input_slice(aad_ptr, aad_len)?;
        let tag = input_slice(tag_ptr, tag_len)?;
        decrypt(cipher, data, key, iv, options, aad, tag)
    })) {
        Ok(Ok(plaintext)) => publish_decrypt(plaintext, out_ptr, out_cap, out_len),
        Ok(Err(error)) => error.status(),
        Err(_) => CIPHER_ERR_INVALID_ARGUMENT,
    }
}
