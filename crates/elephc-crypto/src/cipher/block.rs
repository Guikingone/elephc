//! Purpose:
//! AES CBC, ECB, and CTR primitives for the PHP-compatible cipher bridge.
//!
//! Called from:
//! - `crate::cipher::encrypt()` and `crate::cipher::decrypt()`.
//!
//! Key details:
//! - CBC/ECB implement PKCS#7 locally so decrypt failures map to the bridge's
//!   stable status codes rather than panicking inside padding helpers.
//! - CTR uses the RustCrypto big-endian 128-bit counter flavor used by OpenSSL.

use super::CipherError;
use aes::{Aes128, Aes192, Aes256};
use cipher::{
    consts::U16, generic_array::GenericArray, BlockDecrypt, BlockEncrypt, BlockSizeUser, KeyInit,
    KeyIvInit, StreamCipher,
};

const BLOCK_LEN: usize = 16;

/// Pads plaintext for CBC/ECB or rejects unaligned zero-padding input.
fn padded_plaintext(data: &[u8], zero_padding: bool) -> Result<Vec<u8>, CipherError> {
    if zero_padding {
        if data.len() % BLOCK_LEN != 0 {
            return Err(CipherError::BadPlaintextLength);
        }
        return Ok(data.to_vec());
    }

    let padding = BLOCK_LEN - data.len() % BLOCK_LEN;
    let mut output = Vec::with_capacity(data.len() + padding);
    output.extend_from_slice(data);
    output.resize(output.len() + padding, padding as u8);
    Ok(output)
}

/// Removes and validates PKCS#7 padding, or preserves bytes in zero-padding mode.
fn unpad_plaintext(mut data: Vec<u8>, zero_padding: bool) -> Result<Vec<u8>, CipherError> {
    if zero_padding {
        return Ok(data);
    }
    let padding = usize::from(*data.last().ok_or(CipherError::DecryptFailed)?);
    if padding == 0 || padding > BLOCK_LEN || padding > data.len() {
        return Err(CipherError::DecryptFailed);
    }
    if !data[data.len() - padding..].iter().all(|byte| usize::from(*byte) == padding) {
        return Err(CipherError::DecryptFailed);
    }
    data.truncate(data.len() - padding);
    Ok(data)
}

/// Encrypts already-padded blocks independently with one AES key size.
fn ecb_encrypt_with<C>(key: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, CipherError>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    for chunk in data.chunks_exact_mut(BLOCK_LEN) {
        cipher.encrypt_block(GenericArray::from_mut_slice(chunk));
    }
    Ok(data)
}

/// Decrypts block-aligned ECB ciphertext with one AES key size.
fn ecb_decrypt_with<C>(key: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, CipherError>
where
    C: BlockDecrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    for chunk in data.chunks_exact_mut(BLOCK_LEN) {
        cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
    }
    Ok(data)
}

/// Encrypts already-padded CBC blocks with one AES key size.
fn cbc_encrypt_with<C>(key: &[u8], iv: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, CipherError>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    let mut previous = [0u8; BLOCK_LEN];
    previous.copy_from_slice(iv);
    for chunk in data.chunks_exact_mut(BLOCK_LEN) {
        for (byte, prior) in chunk.iter_mut().zip(previous) {
            *byte ^= prior;
        }
        cipher.encrypt_block(GenericArray::from_mut_slice(chunk));
        previous.copy_from_slice(chunk);
    }
    Ok(data)
}

/// Decrypts block-aligned CBC ciphertext with one AES key size.
fn cbc_decrypt_with<C>(key: &[u8], iv: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, CipherError>
where
    C: BlockDecrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    let mut previous = [0u8; BLOCK_LEN];
    previous.copy_from_slice(iv);
    for chunk in data.chunks_exact_mut(BLOCK_LEN) {
        let mut ciphertext = [0u8; BLOCK_LEN];
        ciphertext.copy_from_slice(chunk);
        cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
        for (byte, prior) in chunk.iter_mut().zip(previous) {
            *byte ^= prior;
        }
        previous = ciphertext;
    }
    Ok(data)
}

/// Encrypts raw bytes in ECB mode with PHP's selected padding behavior.
pub(crate) fn ecb_encrypt(
    key: &[u8],
    data: &[u8],
    zero_padding: bool,
) -> Result<Vec<u8>, CipherError> {
    let padded = padded_plaintext(data, zero_padding)?;
    match key.len() {
        16 => ecb_encrypt_with::<Aes128>(key, padded),
        24 => ecb_encrypt_with::<Aes192>(key, padded),
        32 => ecb_encrypt_with::<Aes256>(key, padded),
        _ => Err(CipherError::BadKey),
    }
}

/// Decrypts ECB ciphertext and validates/removes PKCS#7 when enabled.
pub(crate) fn ecb_decrypt(
    key: &[u8],
    data: &[u8],
    zero_padding: bool,
) -> Result<Vec<u8>, CipherError> {
    if data.len() % BLOCK_LEN != 0 {
        return Err(CipherError::DecryptFailed);
    }
    let decrypted = match key.len() {
        16 => ecb_decrypt_with::<Aes128>(key, data.to_vec()),
        24 => ecb_decrypt_with::<Aes192>(key, data.to_vec()),
        32 => ecb_decrypt_with::<Aes256>(key, data.to_vec()),
        _ => Err(CipherError::BadKey),
    }?;
    unpad_plaintext(decrypted, zero_padding)
}

/// Encrypts raw bytes in CBC mode with PHP's selected padding behavior.
pub(crate) fn cbc_encrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    zero_padding: bool,
) -> Result<Vec<u8>, CipherError> {
    let padded = padded_plaintext(data, zero_padding)?;
    match key.len() {
        16 => cbc_encrypt_with::<Aes128>(key, iv, padded),
        24 => cbc_encrypt_with::<Aes192>(key, iv, padded),
        32 => cbc_encrypt_with::<Aes256>(key, iv, padded),
        _ => Err(CipherError::BadKey),
    }
}

/// Decrypts CBC ciphertext and validates/removes PKCS#7 when enabled.
pub(crate) fn cbc_decrypt(
    key: &[u8],
    iv: &[u8],
    data: &[u8],
    zero_padding: bool,
) -> Result<Vec<u8>, CipherError> {
    if data.len() % BLOCK_LEN != 0 {
        return Err(CipherError::DecryptFailed);
    }
    let decrypted = match key.len() {
        16 => cbc_decrypt_with::<Aes128>(key, iv, data.to_vec()),
        24 => cbc_decrypt_with::<Aes192>(key, iv, data.to_vec()),
        32 => cbc_decrypt_with::<Aes256>(key, iv, data.to_vec()),
        _ => Err(CipherError::BadKey),
    }?;
    unpad_plaintext(decrypted, zero_padding)
}

/// Applies AES-CTR encryption/decryption using a big-endian 128-bit counter.
pub(crate) fn ctr_crypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>, CipherError> {
    let mut output = data.to_vec();
    macro_rules! apply_ctr {
        ($cipher:ty) => {{
            let mut cipher = ctr::Ctr128BE::<$cipher>::new_from_slices(key, iv)
                .map_err(|_| CipherError::InvalidArgument)?;
            cipher.apply_keystream(&mut output);
            Ok(output)
        }};
    }
    match key.len() {
        16 => apply_ctr!(Aes128),
        24 => apply_ctr!(Aes192),
        32 => apply_ctr!(Aes256),
        _ => Err(CipherError::BadKey),
    }
}
