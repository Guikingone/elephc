//! Purpose:
//! AES-GCM authenticated encryption with PHP-compatible runtime IV and tag
//! lengths.
//!
//! Called from:
//! - `crate::cipher::encrypt()` and `crate::cipher::decrypt()` for GCM entries.
//!
//! Key details:
//! - AES block encryption and GHASH come from RustCrypto crates.
//! - J0 derivation supports every non-empty IV length, including PHP's
//!   non-12-byte IV behavior.
//! - Decrypt authenticates the supplied 1..=16-byte tag prefix in constant time
//!   before exposing plaintext.

use super::CipherError;
use aes::{Aes128, Aes192, Aes256};
use cipher::{
    consts::U16, generic_array::GenericArray, BlockEncrypt, BlockSizeUser, KeyInit,
};
use ghash::{universal_hash::UniversalHash, GHash};
use subtle::ConstantTimeEq;

const BLOCK_LEN: usize = 16;

/// Encrypts one AES block without mutating the caller's input.
fn encrypt_block<C>(cipher: &C, input: [u8; BLOCK_LEN]) -> [u8; BLOCK_LEN]
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16>,
{
    let mut block = GenericArray::from(input);
    cipher.encrypt_block(&mut block);
    block.into()
}

/// Computes GHASH over two independently padded byte strings plus bit lengths.
fn ghash(
    hash_subkey: [u8; BLOCK_LEN],
    first: &[u8],
    second: &[u8],
) -> Result<[u8; BLOCK_LEN], CipherError> {
    let first_bits = u64::try_from(first.len())
        .ok()
        .and_then(|len| len.checked_mul(8))
        .ok_or(CipherError::InvalidArgument)?;
    let second_bits = u64::try_from(second.len())
        .ok()
        .and_then(|len| len.checked_mul(8))
        .ok_or(CipherError::InvalidArgument)?;
    let mut hasher = GHash::new(GenericArray::from_slice(&hash_subkey));
    hasher.update_padded(first);
    hasher.update_padded(second);
    let mut lengths = ghash::Block::default();
    lengths[..8].copy_from_slice(&first_bits.to_be_bytes());
    lengths[8..].copy_from_slice(&second_bits.to_be_bytes());
    hasher.update(&[lengths]);
    Ok(hasher.finalize().into())
}

/// Derives the GCM pre-counter block J0 for standard and arbitrary IV lengths.
fn derive_j0(hash_subkey: [u8; BLOCK_LEN], iv: &[u8]) -> Result<[u8; 16], CipherError> {
    if iv.len() == 12 {
        let mut j0 = [0u8; BLOCK_LEN];
        j0[..12].copy_from_slice(iv);
        j0[15] = 1;
        Ok(j0)
    } else {
        ghash(hash_subkey, &[], iv)
    }
}

/// Increments the low 32 bits of a GCM counter block modulo 2^32.
fn increment_counter(counter: &mut [u8; BLOCK_LEN]) {
    let value = u32::from_be_bytes([counter[12], counter[13], counter[14], counter[15]]);
    counter[12..].copy_from_slice(&value.wrapping_add(1).to_be_bytes());
}

/// Applies GCM's counter-mode transform starting from J0 + 1.
fn gctr<C>(cipher: &C, j0: [u8; BLOCK_LEN], input: &[u8]) -> Vec<u8>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16>,
{
    let mut output = Vec::with_capacity(input.len());
    let mut counter = j0;
    for chunk in input.chunks(BLOCK_LEN) {
        increment_counter(&mut counter);
        let keystream = encrypt_block(cipher, counter);
        output.extend(chunk.iter().zip(keystream).map(|(byte, mask)| byte ^ mask));
    }
    output
}

/// Computes the full 16-byte GCM authentication tag over AAD and ciphertext.
fn authentication_tag<C>(
    cipher: &C,
    hash_subkey: [u8; BLOCK_LEN],
    j0: [u8; BLOCK_LEN],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<[u8; BLOCK_LEN], CipherError>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16>,
{
    let auth = ghash(hash_subkey, aad, ciphertext)?;
    let mask = encrypt_block(cipher, j0);
    let mut tag = [0u8; BLOCK_LEN];
    for index in 0..BLOCK_LEN {
        tag[index] = auth[index] ^ mask[index];
    }
    Ok(tag)
}

/// Encrypts with one concrete AES key size and produces a full GCM tag.
fn encrypt_with<C>(
    key: &[u8],
    iv: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; BLOCK_LEN]), CipherError>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    let hash_subkey = encrypt_block(&cipher, [0u8; BLOCK_LEN]);
    let j0 = derive_j0(hash_subkey, iv)?;
    let ciphertext = gctr(&cipher, j0, plaintext);
    let tag = authentication_tag(&cipher, hash_subkey, j0, aad, &ciphertext)?;
    Ok((ciphertext, tag))
}

/// Decrypts with one AES key size after constant-time tag-prefix validation.
fn decrypt_with<C>(
    key: &[u8],
    iv: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    supplied_tag: &[u8],
) -> Result<Vec<u8>, CipherError>
where
    C: BlockEncrypt + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let cipher = C::new_from_slice(key).map_err(|_| CipherError::BadKey)?;
    let hash_subkey = encrypt_block(&cipher, [0u8; BLOCK_LEN]);
    let j0 = derive_j0(hash_subkey, iv)?;
    let expected_tag = authentication_tag(&cipher, hash_subkey, j0, aad, ciphertext)?;
    if expected_tag[..supplied_tag.len()].ct_eq(supplied_tag).unwrap_u8() != 1 {
        return Err(CipherError::DecryptFailed);
    }
    Ok(gctr(&cipher, j0, ciphertext))
}

/// Encrypts with AES-GCM for any supported AES key size.
pub(crate) fn encrypt(
    key: &[u8],
    iv: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<(Vec<u8>, [u8; BLOCK_LEN]), CipherError> {
    match key.len() {
        16 => encrypt_with::<Aes128>(key, iv, aad, plaintext),
        24 => encrypt_with::<Aes192>(key, iv, aad, plaintext),
        32 => encrypt_with::<Aes256>(key, iv, aad, plaintext),
        _ => Err(CipherError::BadKey),
    }
}

/// Decrypts with AES-GCM for any supported AES key size.
pub(crate) fn decrypt(
    key: &[u8],
    iv: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> Result<Vec<u8>, CipherError> {
    match key.len() {
        16 => decrypt_with::<Aes128>(key, iv, aad, ciphertext, tag),
        24 => decrypt_with::<Aes192>(key, iv, aad, ciphertext, tag),
        32 => decrypt_with::<Aes256>(key, iv, aad, ciphertext, tag),
        _ => Err(CipherError::BadKey),
    }
}
