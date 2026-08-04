//! Purpose:
//! Schema and coverage checks for PHP-generated OpenSSL cipher fixtures.
//!
//! Called from:
//! - `cargo test -p elephc-crypto` through Rust's test harness.
//!
//! Key details:
//! - These tests do not require PHP or the future cipher ABI; they keep the
//!   checked-in baseline complete and consumable on every CI host.

#[path = "fixtures/openssl_php_fixtures.rs"]
mod fixtures;

use fixtures as fx;

/// Validates a fixture hex string and returns its decoded byte length.
fn hex_byte_len(value: &str) -> usize {
    assert_eq!(value.len() % 2, 0, "odd-length fixture hex: {value}");
    assert!(
        value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "non-hex fixture value: {value}"
    );
    value.len() / 2
}

/// Pins complete golden coverage for all twelve ciphers in the planned matrix.
#[test]
fn openssl_php_golden_covers_cipher_matrix() {
    let expected = [
        ("aes-128-cbc", 16, 16, 1),
        ("aes-192-cbc", 24, 16, 1),
        ("aes-256-cbc", 32, 16, 1),
        ("aes-128-ecb", 16, 0, 1),
        ("aes-192-ecb", 24, 0, 1),
        ("aes-256-ecb", 32, 0, 1),
        ("aes-128-ctr", 16, 16, 1),
        ("aes-192-ctr", 24, 16, 1),
        ("aes-256-ctr", 32, 16, 1),
        ("aes-128-gcm", 16, 12, 2),
        ("aes-192-gcm", 24, 12, 2),
        ("aes-256-gcm", 32, 12, 2),
    ];

    assert!(!fx::FIXTURE_PHP_VERSION.is_empty());
    assert!(fx::FIXTURE_OPENSSL_VERSION.starts_with("OpenSSL "));
    assert_eq!(fx::IV_LENGTHS.len(), expected.len());
    assert_eq!(fx::CIPHER_VECTORS.len(), 15);

    for (cipher, key_len, iv_len, vector_count) in expected {
        assert_eq!(
            fx::IV_LENGTHS.iter().find(|(name, _)| *name == cipher),
            Some(&(cipher, iv_len))
        );
        let vectors: Vec<_> = fx::CIPHER_VECTORS
            .iter()
            .filter(|vector| vector.cipher == cipher)
            .collect();
        assert_eq!(vectors.len(), vector_count, "fixture count for {cipher}");

        for vector in vectors {
            assert_eq!(hex_byte_len(vector.key_hex), key_len);
            assert_eq!(hex_byte_len(vector.iv_hex), iv_len);
            hex_byte_len(vector.aad_hex);
            let ciphertext_len = hex_byte_len(vector.ciphertext_hex);
            if cipher.ends_with("-cbc") || cipher.ends_with("-ecb") {
                assert_eq!(ciphertext_len, 48);
            } else {
                assert_eq!(ciphertext_len, fx::PT43.len());
            }
            if cipher.ends_with("-gcm") {
                assert_eq!(hex_byte_len(vector.tag_hex), vector.tag_length);
            } else {
                assert!(vector.tag_hex.is_empty());
            }
        }
    }
}

/// Locks the edge and failure fixture inventory before the cipher ABI exists.
#[test]
fn openssl_php_golden_covers_edges_and_failures() {
    assert_eq!(hex_byte_len(fx::PT16_HEX), 16);
    assert_eq!(hex_byte_len(fx::EMPTY_PT_CBC_CIPHERTEXT_HEX), 16);
    assert!(fx::EMPTY_PT_CTR_CIPHERTEXT_HEX.is_empty());
    assert!(fx::EMPTY_PT_GCM_CIPHERTEXT_HEX.is_empty());
    assert_eq!(hex_byte_len(fx::EMPTY_PT_GCM_TAG_HEX), 16);
    for value in [
        fx::SHORT_KEY_CIPHERTEXT_HEX,
        fx::LONG_KEY_CIPHERTEXT_HEX,
        fx::EMPTY_IV_CIPHERTEXT_HEX,
        fx::SHORT_IV_CIPHERTEXT_HEX,
        fx::LONG_IV_CIPHERTEXT_HEX,
        fx::UPPERCASE_NAME_CIPHERTEXT_HEX,
    ] {
        assert_eq!(hex_byte_len(value), 48);
    }
    assert_eq!(hex_byte_len(fx::ZERO_PAD_CBC_CIPHERTEXT_HEX), 32);
    assert_eq!(hex_byte_len(fx::ZERO_PAD_ECB_CIPHERTEXT_HEX), 32);
    assert_eq!(fx::CBC_BASE64.len(), 64);

    let tag_lengths: Vec<_> = fx::GCM_TAG_LENGTH_VECTORS
        .iter()
        .map(|vector| {
            assert_eq!(hex_byte_len(vector.ciphertext_hex), fx::PT43.len());
            assert_eq!(hex_byte_len(vector.tag_hex), vector.tag_length);
            vector.tag_length
        })
        .collect();
    assert_eq!(tag_lengths, [1, 4, 12]);

    let gcm_iv_lengths: Vec<_> = fx::GCM_NON_DEFAULT_IV_VECTORS
        .iter()
        .map(|vector| {
            assert_eq!(hex_byte_len(vector.ciphertext_hex), fx::PT43.len());
            assert_eq!(hex_byte_len(vector.tag_hex), 16);
            hex_byte_len(vector.iv_hex)
        })
        .collect();
    assert_eq!(gcm_iv_lengths, [1, 16, 20]);

    let expected_iv_edges = [
        ("aes-256-cbc", "empty", 0, 48),
        ("aes-256-cbc", "short", 3, 48),
        ("aes-256-cbc", "long", 20, 48),
        ("aes-256-ctr", "empty", 0, fx::PT43.len()),
        ("aes-256-ctr", "short", 3, fx::PT43.len()),
        ("aes-256-ctr", "long", 20, fx::PT43.len()),
    ];
    assert_eq!(fx::IV_NORMALIZATION_VECTORS.len(), expected_iv_edges.len());
    for (cipher, case, iv_len, ciphertext_len) in expected_iv_edges {
        let vector = fx::IV_NORMALIZATION_VECTORS
            .iter()
            .find(|vector| vector.cipher == cipher && vector.case == case)
            .unwrap_or_else(|| panic!("missing IV normalization fixture: {cipher}/{case}"));
        assert_eq!(hex_byte_len(vector.key_hex), 32);
        assert_eq!(hex_byte_len(vector.iv_hex), iv_len);
        assert_eq!(hex_byte_len(vector.plaintext_hex), fx::PT43.len());
        assert_eq!(hex_byte_len(vector.ciphertext_hex), ciphertext_len);
        assert!(vector.encrypt_warning.starts_with("openssl_encrypt():"));
        if case == "empty" {
            assert!(vector.decrypt_warning.is_empty());
        } else {
            assert!(vector.decrypt_warning.starts_with("openssl_decrypt():"));
        }
    }
    let cbc_empty = fx::IV_NORMALIZATION_VECTORS
        .iter()
        .find(|vector| vector.cipher == "aes-256-cbc" && vector.case == "empty")
        .expect("missing CBC empty-IV fixture");
    assert_eq!(cbc_empty.ciphertext_hex, fx::EMPTY_IV_CIPHERTEXT_HEX);

    let expected_failures = [
        "unknown_cipher_encrypt",
        "unknown_cipher_decrypt",
        "unknown_cipher_iv_length",
        "short_key_dont_zero_pad_encrypt",
        "zero_padding_unaligned_encrypt",
        "zero_padding_misaligned_decrypt",
        "invalid_pkcs7_decrypt",
        "gcm_empty_iv_encrypt",
        "gcm_wrong_tag_decrypt",
        "gcm_missing_tag_decrypt",
        "gcm_tag_length_0_encrypt",
        "gcm_tag_length_17_encrypt",
        "invalid_base64_decrypt",
    ];
    assert_eq!(fx::FAILURE_VECTORS.len(), expected_failures.len());
    for case in expected_failures {
        let vector = fx::FAILURE_VECTORS
            .iter()
            .find(|vector| vector.case == case)
            .unwrap_or_else(|| panic!("missing failure fixture: {case}"));
        assert!(matches!(vector.operation, "encrypt" | "decrypt" | "iv_length"));
        assert!(!vector.cipher.is_empty());
        hex_byte_len(vector.data_hex);
        hex_byte_len(vector.key_hex);
        hex_byte_len(vector.iv_hex);
        hex_byte_len(vector.aad_hex);
        hex_byte_len(vector.tag_hex);
        assert!(vector.options <= 7);
        assert!(vector.tag_length <= 17);
        assert!(vector.warning.is_empty() || vector.warning.starts_with("openssl_"));
    }
}
