//! Purpose:
//! Shared symmetric-cipher catalog and PHP-compatible dispatch for the crypto
//! bridge's OpenSSL surface.
//!
//! Called from:
//! - `crate::cipher::abi` for compiled-program and magician C ABI calls.
//! - `cargo test -p elephc-crypto` through the public ABI.
//!
//! Key details:
//! - Cipher lookup is ASCII-case-insensitive and restricted to the documented
//!   AES CBC/CTR/ECB/GCM matrix.
//! - The bridge always consumes and produces raw bytes. PHP base64 behavior is
//!   owned by the AOT/magician glue; option bit 1 is accepted but ignored here.
//! - CBC/CTR IVs are zero-padded or truncated, GCM accepts any non-empty IV,
//!   and short keys are zero-padded unless option bit 4 rejects the call.

mod abi;
mod block;
mod gcm;

pub use abi::{
    elephc_crypto_cipher_iv_length, elephc_crypto_cipher_methods, elephc_crypto_decrypt,
    elephc_crypto_encrypt,
};

/// Successful cipher ABI operation.
pub const CIPHER_OK: i32 = 0;
/// The requested cipher name is outside the supported matrix.
pub const CIPHER_ERR_UNKNOWN: i32 = -1;
/// A short key was supplied with `OPENSSL_DONT_ZERO_PAD_KEY`.
pub const CIPHER_ERR_BAD_KEY: i32 = -2;
/// GCM received an empty IV.
pub const CIPHER_ERR_BAD_IV: i32 = -3;
/// Zero-padding encryption received non-block-aligned plaintext.
pub const CIPHER_ERR_BAD_PLAINTEXT_LENGTH: i32 = -4;
/// Decryption failed because of length, padding, or authentication.
pub const CIPHER_ERR_DECRYPT_FAILED: i32 = -5;
/// GCM encryption received a tag length outside `1..=16`.
pub const CIPHER_ERR_BAD_TAG_LENGTH: i32 = -6;
/// A caller-provided output buffer is too small.
pub const CIPHER_ERR_OUTPUT_TOO_SMALL: i32 = -7;
/// Options or C ABI pointer arguments are invalid.
pub const CIPHER_ERR_INVALID_ARGUMENT: i32 = -8;

pub(crate) const OPENSSL_RAW_DATA: u32 = 1;
pub(crate) const OPENSSL_ZERO_PADDING: u32 = 2;
pub(crate) const OPENSSL_DONT_ZERO_PAD_KEY: u32 = 4;
const SUPPORTED_OPTIONS: u32 =
    OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING | OPENSSL_DONT_ZERO_PAD_KEY;

/// Cipher mode selected by a catalog entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Cbc,
    Ctr,
    Ecb,
    Gcm,
}

/// One supported cipher's fixed key/IV shape and mode.
#[derive(Clone, Copy, Debug)]
struct CipherSpec {
    name: &'static str,
    key_len: usize,
    iv_len: usize,
    mode: Mode,
}

macro_rules! define_cipher_catalog {
    ($(($name:literal, $key_len:literal, $iv_len:literal, $mode:ident)),+ $(,)?) => {
        const SPECS: &[CipherSpec] = &[
            $(CipherSpec { name: $name, key_len: $key_len, iv_len: $iv_len, mode: Mode::$mode }),+
        ];

        /// Stable lowercase method inventory returned by the bridge.
        pub const CIPHER_METHODS: &[&str] = &[$($name),+];

        /// Trailing-NUL packed representation of [`CIPHER_METHODS`] for the C ABI.
        pub(crate) const CIPHER_METHODS_PACKED: &[u8] =
            concat!($($name, "\0"),+).as_bytes();
    };
}

define_cipher_catalog!(
    ("aes-128-cbc", 16, 16, Cbc),
    ("aes-128-ctr", 16, 16, Ctr),
    ("aes-128-ecb", 16, 0, Ecb),
    ("aes-128-gcm", 16, 12, Gcm),
    ("aes-192-cbc", 24, 16, Cbc),
    ("aes-192-ctr", 24, 16, Ctr),
    ("aes-192-ecb", 24, 0, Ecb),
    ("aes-192-gcm", 24, 12, Gcm),
    ("aes-256-cbc", 32, 16, Cbc),
    ("aes-256-ctr", 32, 16, Ctr),
    ("aes-256-ecb", 32, 0, Ecb),
    ("aes-256-gcm", 32, 12, Gcm),
);

/// Error category returned by the internal cipher engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CipherError {
    UnknownCipher,
    BadKey,
    BadIv,
    BadPlaintextLength,
    DecryptFailed,
    BadTagLength,
    OutputTooSmall,
    InvalidArgument,
}

impl CipherError {
    /// Converts an internal error into the stable negative C ABI status code.
    pub(crate) fn status(self) -> i32 {
        match self {
            Self::UnknownCipher => CIPHER_ERR_UNKNOWN,
            Self::BadKey => CIPHER_ERR_BAD_KEY,
            Self::BadIv => CIPHER_ERR_BAD_IV,
            Self::BadPlaintextLength => CIPHER_ERR_BAD_PLAINTEXT_LENGTH,
            Self::DecryptFailed => CIPHER_ERR_DECRYPT_FAILED,
            Self::BadTagLength => CIPHER_ERR_BAD_TAG_LENGTH,
            Self::OutputTooSmall => CIPHER_ERR_OUTPUT_TOO_SMALL,
            Self::InvalidArgument => CIPHER_ERR_INVALID_ARGUMENT,
        }
    }
}

/// Raw encryption result plus the optional GCM authentication tag.
pub(crate) struct EncryptOutput {
    pub(crate) ciphertext: Vec<u8>,
    pub(crate) tag: Vec<u8>,
}

/// Finds a supported cipher without allocating or accepting non-ASCII aliases.
fn find_spec(name: &[u8]) -> Result<CipherSpec, CipherError> {
    SPECS
        .iter()
        .copied()
        .find(|spec| name.eq_ignore_ascii_case(spec.name.as_bytes()))
        .ok_or(CipherError::UnknownCipher)
}

/// Rejects unknown option bits while accepting the PHP raw-data bit as a no-op.
fn validate_options(options: u32) -> Result<(), CipherError> {
    if options & !SUPPORTED_OPTIONS == 0 {
        Ok(())
    } else {
        Err(CipherError::InvalidArgument)
    }
}

/// Applies PHP's short-key zero padding and long-key truncation rules.
fn normalize_key(spec: CipherSpec, key: &[u8], options: u32) -> Result<Vec<u8>, CipherError> {
    if key.len() < spec.key_len && options & OPENSSL_DONT_ZERO_PAD_KEY != 0 {
        return Err(CipherError::BadKey);
    }
    let mut normalized = vec![0; spec.key_len];
    let copied = key.len().min(spec.key_len);
    normalized[..copied].copy_from_slice(&key[..copied]);
    Ok(normalized)
}

/// Applies mode-specific PHP IV handling before cryptographic dispatch.
fn normalize_iv(spec: CipherSpec, iv: &[u8]) -> Result<Vec<u8>, CipherError> {
    match spec.mode {
        Mode::Ecb => Ok(Vec::new()),
        Mode::Cbc | Mode::Ctr => {
            let mut normalized = vec![0; spec.iv_len];
            let copied = iv.len().min(spec.iv_len);
            normalized[..copied].copy_from_slice(&iv[..copied]);
            Ok(normalized)
        }
        Mode::Gcm if iv.is_empty() => Err(CipherError::BadIv),
        Mode::Gcm => Ok(iv.to_vec()),
    }
}

/// Returns the PHP-visible default IV length for one supported cipher.
pub(crate) fn cipher_iv_length(name: &[u8]) -> Result<usize, CipherError> {
    Ok(find_spec(name)?.iv_len)
}

/// Encrypts raw bytes under one supported cipher and returns raw ciphertext/tag.
pub(crate) fn encrypt(
    name: &[u8],
    data: &[u8],
    key: &[u8],
    iv: &[u8],
    options: u32,
    aad: &[u8],
    tag_len: usize,
) -> Result<EncryptOutput, CipherError> {
    let spec = find_spec(name)?;
    validate_options(options)?;
    let key = normalize_key(spec, key, options)?;
    let iv = normalize_iv(spec, iv)?;
    let zero_padding = options & OPENSSL_ZERO_PADDING != 0;

    match spec.mode {
        Mode::Cbc => Ok(EncryptOutput {
            ciphertext: block::cbc_encrypt(&key, &iv, data, zero_padding)?,
            tag: Vec::new(),
        }),
        Mode::Ctr => Ok(EncryptOutput {
            ciphertext: block::ctr_crypt(&key, &iv, data)?,
            tag: Vec::new(),
        }),
        Mode::Ecb => Ok(EncryptOutput {
            ciphertext: block::ecb_encrypt(&key, data, zero_padding)?,
            tag: Vec::new(),
        }),
        Mode::Gcm => {
            if !(1..=16).contains(&tag_len) {
                return Err(CipherError::BadTagLength);
            }
            let (ciphertext, full_tag) = gcm::encrypt(&key, &iv, aad, data)?;
            Ok(EncryptOutput { ciphertext, tag: full_tag[..tag_len].to_vec() })
        }
    }
}

/// Decrypts raw ciphertext, validating PKCS#7 padding or the supplied GCM tag.
pub(crate) fn decrypt(
    name: &[u8],
    data: &[u8],
    key: &[u8],
    iv: &[u8],
    options: u32,
    aad: &[u8],
    tag: &[u8],
) -> Result<Vec<u8>, CipherError> {
    let spec = find_spec(name)?;
    validate_options(options)?;
    let key = normalize_key(spec, key, options)?;
    let iv = normalize_iv(spec, iv)?;
    let zero_padding = options & OPENSSL_ZERO_PADDING != 0;

    match spec.mode {
        Mode::Cbc => block::cbc_decrypt(&key, &iv, data, zero_padding),
        Mode::Ctr => block::ctr_crypt(&key, &iv, data),
        Mode::Ecb => block::ecb_decrypt(&key, data, zero_padding),
        Mode::Gcm => {
            if tag.is_empty() || tag.len() > 16 {
                return Err(CipherError::DecryptFailed);
            }
            gcm::decrypt(&key, &iv, aad, data, tag)
        }
    }
}
