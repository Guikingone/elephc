//! Purpose:
//! End-to-end C ABI tests for the elephc-crypto symmetric-cipher engine.
//!
//! Called from:
//! - `cargo test -p elephc-crypto --test openssl_php_fixtures`.
//!
//! Key details:
//! - PHP-generated fixtures are shared with the parent integration-test module.
//! - Every supported cipher is compared byte-for-byte and then decrypted.
//! - Published NIST vectors independently validate CBC and GCM composition.

use super::{fixtures as fx, hex_byte_len};
use elephc_crypto::*;
use std::ptr;

const RAW: u32 = 1;
const ZERO_PADDING: u32 = 2;
const DONT_ZERO_PAD_KEY: u32 = 4;

/// Decodes one lowercase/uppercase hexadecimal fixture into bytes.
fn decode_hex(value: &str) -> Vec<u8> {
    hex_byte_len(value);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).expect("validated hex");
            let low = char::from(pair[1]).to_digit(16).expect("validated hex");
            ((high << 4) | low) as u8
        })
        .collect()
}

/// Invokes the encrypt ABI with generously sized caller-owned buffers.
fn encrypt_raw(
    cipher: &str,
    data: &[u8],
    key: &[u8],
    iv: &[u8],
    options: u32,
    aad: &[u8],
    tag_len: usize,
) -> Result<(Vec<u8>, Vec<u8>), i32> {
    let mut output = vec![0u8; data.len() + 32];
    let mut tag = vec![0u8; 16];
    let mut output_len = 0usize;
    let mut tag_output_len = 0usize;
    let status = unsafe {
        elephc_crypto_encrypt(
            cipher.as_ptr(),
            cipher.len(),
            data.as_ptr(),
            data.len(),
            key.as_ptr(),
            key.len(),
            iv.as_ptr(),
            iv.len(),
            options,
            aad.as_ptr(),
            aad.len(),
            tag_len,
            output.as_mut_ptr(),
            output.len(),
            &mut output_len,
            tag.as_mut_ptr(),
            tag.len(),
            &mut tag_output_len,
        )
    };
    if status != CIPHER_OK {
        return Err(status);
    }
    output.truncate(output_len);
    tag.truncate(tag_output_len);
    Ok((output, tag))
}

/// Invokes the decrypt ABI with a caller-owned plaintext buffer.
fn decrypt_raw(
    cipher: &str,
    data: &[u8],
    key: &[u8],
    iv: &[u8],
    options: u32,
    aad: &[u8],
    tag: &[u8],
) -> Result<Vec<u8>, i32> {
    let mut output = vec![0u8; data.len() + 16];
    let mut output_len = 0usize;
    let status = unsafe {
        elephc_crypto_decrypt(
            cipher.as_ptr(),
            cipher.len(),
            data.as_ptr(),
            data.len(),
            key.as_ptr(),
            key.len(),
            iv.as_ptr(),
            iv.len(),
            options,
            aad.as_ptr(),
            aad.len(),
            tag.as_ptr(),
            tag.len(),
            output.as_mut_ptr(),
            output.len(),
            &mut output_len,
        )
    };
    if status != CIPHER_OK {
        return Err(status);
    }
    output.truncate(output_len);
    Ok(output)
}

/// Verifies all twelve ciphers against PHP ciphertext/tag bytes and round-trips.
#[test]
fn cipher_matrix_matches_php_goldens() {
    for vector in fx::CIPHER_VECTORS {
        let key = decode_hex(vector.key_hex);
        let iv = decode_hex(vector.iv_hex);
        let aad = decode_hex(vector.aad_hex);
        let expected_ciphertext = decode_hex(vector.ciphertext_hex);
        let expected_tag = decode_hex(vector.tag_hex);
        let (ciphertext, tag) = encrypt_raw(
            vector.cipher,
            fx::PT43.as_bytes(),
            &key,
            &iv,
            RAW,
            &aad,
            vector.tag_length,
        )
        .unwrap_or_else(|status| panic!("{} encrypt status {status}", vector.cipher));
        assert_eq!(ciphertext, expected_ciphertext, "{} ciphertext", vector.cipher);
        assert_eq!(tag, expected_tag, "{} tag", vector.cipher);
        assert_eq!(
            decrypt_raw(vector.cipher, &ciphertext, &key, &iv, RAW, &aad, &tag),
            Ok(fx::PT43.as_bytes().to_vec()),
            "{} decrypt",
            vector.cipher
        );
    }
}

/// Verifies PHP's non-default GCM IV lengths and truncated authentication tags.
#[test]
fn gcm_runtime_iv_and_tag_lengths_match_php() {
    let key = vec![b'k'; 32];
    let aad = b"fixture-aad";
    for vector in fx::GCM_NON_DEFAULT_IV_VECTORS {
        let iv = decode_hex(vector.iv_hex);
        let expected_ciphertext = decode_hex(vector.ciphertext_hex);
        let expected_tag = decode_hex(vector.tag_hex);
        let (ciphertext, tag) =
            encrypt_raw("aes-256-gcm", fx::PT43.as_bytes(), &key, &iv, RAW, aad, 16)
                .expect("non-default GCM IV must encrypt");
        assert_eq!(ciphertext, expected_ciphertext);
        assert_eq!(tag, expected_tag);
        assert_eq!(
            decrypt_raw("aes-256-gcm", &ciphertext, &key, &iv, RAW, aad, &tag),
            Ok(fx::PT43.as_bytes().to_vec())
        );
    }

    let iv = vec![b'i'; 12];
    for vector in fx::GCM_TAG_LENGTH_VECTORS {
        let (ciphertext, tag) = encrypt_raw(
            "aes-256-gcm",
            fx::PT43.as_bytes(),
            &key,
            &iv,
            RAW,
            aad,
            vector.tag_length,
        )
        .expect("truncated GCM tag must encrypt");
        assert_eq!(ciphertext, decode_hex(vector.ciphertext_hex));
        assert_eq!(tag, decode_hex(vector.tag_hex));
        assert_eq!(
            decrypt_raw("aes-256-gcm", &ciphertext, &key, &iv, RAW, aad, &tag),
            Ok(fx::PT43.as_bytes().to_vec())
        );
    }
}

/// Verifies CBC/CTR empty, short, and long IV normalization byte-for-byte.
#[test]
fn iv_normalization_matches_php_goldens() {
    for vector in fx::IV_NORMALIZATION_VECTORS {
        let key = decode_hex(vector.key_hex);
        let iv = decode_hex(vector.iv_hex);
        let plaintext = decode_hex(vector.plaintext_hex);
        let expected_ciphertext = decode_hex(vector.ciphertext_hex);
        let (ciphertext, tag) = encrypt_raw(vector.cipher, &plaintext, &key, &iv, RAW, &[], 16)
            .expect("normalized IV must encrypt");
        assert_eq!(ciphertext, expected_ciphertext, "{}/{}", vector.cipher, vector.case);
        assert!(tag.is_empty());
        assert_eq!(
            decrypt_raw(vector.cipher, &ciphertext, &key, &iv, RAW, &[], &[]),
            Ok(plaintext),
            "{}/{}",
            vector.cipher,
            vector.case
        );
    }
}

/// Verifies PHP key normalization, zero padding, and empty-plaintext behavior.
#[test]
fn key_padding_and_empty_edges_match_php() {
    let iv = vec![b'i'; 16];
    let short_key = b"shortkey";
    let (short_ciphertext, _) = encrypt_raw(
        "aes-256-cbc",
        fx::PT43.as_bytes(),
        short_key,
        &iv,
        RAW,
        &[],
        16,
    )
    .expect("short key must be zero padded");
    assert_eq!(short_ciphertext, decode_hex(fx::SHORT_KEY_CIPHERTEXT_HEX));
    assert_eq!(
        encrypt_raw(
            "aes-256-cbc",
            fx::PT43.as_bytes(),
            short_key,
            &iv,
            RAW | DONT_ZERO_PAD_KEY,
            &[],
            16,
        ),
        Err(CIPHER_ERR_BAD_KEY)
    );

    let long_key = vec![b'k'; 32];
    let (long_ciphertext, _) = encrypt_raw(
        "aes-128-cbc",
        fx::PT43.as_bytes(),
        &long_key,
        &iv,
        RAW,
        &[],
        16,
    )
    .expect("long key must be truncated");
    assert_eq!(long_ciphertext, decode_hex(fx::LONG_KEY_CIPHERTEXT_HEX));

    let aligned = b"0123456789abcdef0123456789abcdef";
    for (cipher, expected) in [
        ("aes-256-cbc", fx::ZERO_PAD_CBC_CIPHERTEXT_HEX),
        ("aes-256-ecb", fx::ZERO_PAD_ECB_CIPHERTEXT_HEX),
    ] {
        let cipher_iv = if cipher.ends_with("-ecb") { &[][..] } else { &iv };
        let (ciphertext, _) = encrypt_raw(
            cipher,
            aligned,
            &long_key,
            cipher_iv,
            RAW | ZERO_PADDING,
            &[],
            16,
        )
        .expect("aligned zero-padding input must encrypt");
        assert_eq!(ciphertext, decode_hex(expected));
        assert_eq!(
            decrypt_raw(
                cipher,
                &ciphertext,
                &long_key,
                cipher_iv,
                RAW | ZERO_PADDING,
                &[],
                &[],
            ),
            Ok(aligned.to_vec())
        );
    }

    let (empty_cbc, _) = encrypt_raw("aes-256-cbc", &[], &long_key, &iv, RAW, &[], 16)
        .expect("empty CBC input must pad");
    assert_eq!(empty_cbc, decode_hex(fx::EMPTY_PT_CBC_CIPHERTEXT_HEX));
    assert_eq!(decrypt_raw("aes-256-cbc", &empty_cbc, &long_key, &iv, RAW, &[], &[]), Ok(vec![]));

    let (empty_ctr, _) = encrypt_raw("aes-256-ctr", &[], &long_key, &iv, RAW, &[], 16)
        .expect("empty CTR input must encrypt");
    assert_eq!(empty_ctr, decode_hex(fx::EMPTY_PT_CTR_CIPHERTEXT_HEX));

    let gcm_iv = vec![b'i'; 12];
    let (empty_gcm, tag) = encrypt_raw(
        "aes-256-gcm",
        &[],
        &long_key,
        &gcm_iv,
        RAW,
        b"fixture-aad",
        16,
    )
    .expect("empty GCM input must authenticate");
    assert_eq!(empty_gcm, decode_hex(fx::EMPTY_PT_GCM_CIPHERTEXT_HEX));
    assert_eq!(tag, decode_hex(fx::EMPTY_PT_GCM_TAG_HEX));
    assert_eq!(
        decrypt_raw(
            "aes-256-gcm",
            &empty_gcm,
            &long_key,
            &gcm_iv,
            RAW,
            b"fixture-aad",
            &tag,
        ),
        Ok(vec![])
    );
}

/// Maps every PHP false-return fixture to the bridge's stable failure status.
#[test]
fn failure_vectors_return_stable_status_codes() {
    for vector in fx::FAILURE_VECTORS {
        let expected = match vector.case {
            "unknown_cipher_encrypt" | "unknown_cipher_decrypt" | "unknown_cipher_iv_length" => {
                CIPHER_ERR_UNKNOWN
            }
            "short_key_dont_zero_pad_encrypt" => CIPHER_ERR_BAD_KEY,
            "zero_padding_unaligned_encrypt" => CIPHER_ERR_BAD_PLAINTEXT_LENGTH,
            "gcm_empty_iv_encrypt" => CIPHER_ERR_BAD_IV,
            "gcm_tag_length_0_encrypt" | "gcm_tag_length_17_encrypt" => {
                CIPHER_ERR_BAD_TAG_LENGTH
            }
            _ => CIPHER_ERR_DECRYPT_FAILED,
        };
        if vector.operation == "iv_length" {
            let status = unsafe {
                elephc_crypto_cipher_iv_length(vector.cipher.as_ptr(), vector.cipher.len())
            };
            assert_eq!(status, expected as isize, "{}", vector.case);
            continue;
        }
        let data = decode_hex(vector.data_hex);
        let key = decode_hex(vector.key_hex);
        let iv = decode_hex(vector.iv_hex);
        let aad = decode_hex(vector.aad_hex);
        let tag = decode_hex(vector.tag_hex);
        let status = if vector.operation == "encrypt" {
            encrypt_raw(
                vector.cipher,
                &data,
                &key,
                &iv,
                vector.options,
                &aad,
                vector.tag_length,
            )
            .expect_err("failure fixture unexpectedly encrypted")
        } else {
            decrypt_raw(vector.cipher, &data, &key, &iv, vector.options, &aad, &tag)
                .expect_err("failure fixture unexpectedly decrypted")
        };
        assert_eq!(status, expected, "{}", vector.case);
    }
}

/// Verifies IV metadata, case-insensitive lookup, and the packed method ABI.
#[test]
fn cipher_metadata_abi_is_stable() {
    for (cipher, iv_len) in fx::IV_LENGTHS {
        let actual = unsafe { elephc_crypto_cipher_iv_length(cipher.as_ptr(), cipher.len()) };
        assert_eq!(actual, *iv_len as isize, "{cipher}");
    }
    let uppercase = b"AES-256-GCM";
    assert_eq!(
        unsafe { elephc_crypto_cipher_iv_length(uppercase.as_ptr(), uppercase.len()) },
        12
    );

    let mut required = 0usize;
    assert_eq!(
        unsafe { elephc_crypto_cipher_methods(0, ptr::null_mut(), 0, &mut required) },
        CIPHER_ERR_OUTPUT_TOO_SMALL as isize
    );
    let mut packed = vec![0u8; required];
    let count = unsafe {
        elephc_crypto_cipher_methods(1, packed.as_mut_ptr(), packed.len(), &mut required)
    };
    assert_eq!(count, CIPHER_METHODS.len() as isize);
    let methods: Vec<_> = packed
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| std::str::from_utf8(name).expect("ASCII method name"))
        .collect();
    assert_eq!(methods, CIPHER_METHODS);
}

/// Validates CBC and GCM against independent NIST known-answer vectors.
#[test]
fn nist_cbc_and_gcm_vectors_match() {
    let cbc_key = decode_hex("2b7e151628aed2a6abf7158809cf4f3c");
    let cbc_iv = decode_hex("000102030405060708090a0b0c0d0e0f");
    let cbc_plaintext = decode_hex("6bc1bee22e409f96e93d7e117393172a");
    let (cbc_ciphertext, _) = encrypt_raw(
        "aes-128-cbc",
        &cbc_plaintext,
        &cbc_key,
        &cbc_iv,
        RAW | ZERO_PADDING,
        &[],
        16,
    )
    .expect("NIST CBC vector must encrypt");
    assert_eq!(cbc_ciphertext, decode_hex("7649abac8119b246cee98e9b12e9197d"));

    let gcm_key = vec![0u8; 16];
    let gcm_iv = vec![0u8; 12];
    let gcm_plaintext = vec![0u8; 16];
    let (gcm_ciphertext, gcm_tag) = encrypt_raw(
        "aes-128-gcm",
        &gcm_plaintext,
        &gcm_key,
        &gcm_iv,
        RAW,
        &[],
        16,
    )
    .expect("NIST GCM vector must encrypt");
    assert_eq!(gcm_ciphertext, decode_hex("0388dace60b6a392f328c2b971b2fe78"));
    assert_eq!(gcm_tag, decode_hex("ab6e47d42cec13bdf53a67b21257bddf"));
}

/// Verifies output sizing, unknown options, and null-pointer rejection.
#[test]
fn cipher_abi_rejects_invalid_buffers_and_arguments() {
    let cipher = b"aes-256-cbc";
    let data = b"x";
    let key = [b'k'; 32];
    let iv = [b'i'; 16];
    let mut output_len = 99usize;
    let mut tag_len = 99usize;
    let status = unsafe {
        elephc_crypto_encrypt(
            cipher.as_ptr(),
            cipher.len(),
            data.as_ptr(),
            data.len(),
            key.as_ptr(),
            key.len(),
            iv.as_ptr(),
            iv.len(),
            RAW,
            ptr::null(),
            0,
            16,
            ptr::null_mut(),
            0,
            &mut output_len,
            ptr::null_mut(),
            0,
            &mut tag_len,
        )
    };
    assert_eq!(status, CIPHER_ERR_OUTPUT_TOO_SMALL);
    assert_eq!(output_len, 16);
    assert_eq!(tag_len, 0);

    let mut output = [0u8; 32];
    let invalid_pointer = unsafe {
        elephc_crypto_encrypt(
            ptr::null(),
            1,
            data.as_ptr(),
            data.len(),
            key.as_ptr(),
            key.len(),
            iv.as_ptr(),
            iv.len(),
            RAW,
            ptr::null(),
            0,
            16,
            output.as_mut_ptr(),
            output.len(),
            &mut output_len,
            ptr::null_mut(),
            0,
            &mut tag_len,
        )
    };
    assert_eq!(invalid_pointer, CIPHER_ERR_INVALID_ARGUMENT);
    assert_eq!(
        encrypt_raw("aes-256-cbc", data, &key, &iv, 8, &[], 16),
        Err(CIPHER_ERR_INVALID_ARGUMENT)
    );
    assert_eq!(
        unsafe { elephc_crypto_cipher_methods(0, ptr::null_mut(), 0, ptr::null_mut()) },
        CIPHER_ERR_INVALID_ARGUMENT as isize
    );
}
