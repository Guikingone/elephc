//! Purpose:
//! End-to-end AOT tests for OpenSSL constants, helpers, and non-AEAD AES modes.
//!
//! Called from:
//! - `cargo test --test codegen_tests openssl` through the Rust test harness.
//!
//! Key details:
//! - Ciphertext expectations come from the checked-in PHP 8.4/OpenSSL 3.6 fixtures.
//! - GCM is intentionally excluded until the phase-3 by-reference tag path is wired.

use super::*;

/// Verifies raw CBC, CTR, and ECB encryption matches the locked PHP ciphertexts.
#[test]
fn test_openssl_encrypt_non_aead_php_goldens() {
    let out = compile_and_run(
        r#"<?php
$pt = "The quick brown fox jumps over the lazy dog";
$iv = str_repeat("i", 16);
echo bin2hex(openssl_encrypt($pt, "aes-128-cbc", str_repeat("k", 16), OPENSSL_RAW_DATA, $iv)), "|";
echo bin2hex(openssl_encrypt($pt, "aes-256-ctr", str_repeat("k", 32), OPENSSL_RAW_DATA, $iv)), "|";
echo bin2hex(openssl_encrypt($pt, "aes-192-ecb", str_repeat("k", 24), OPENSSL_RAW_DATA)), "|";
echo bin2hex(openssl_encrypt($pt, "aes-256-cbc", str_repeat("k", 32), OPENSSL_RAW_DATA, "")), "|";
echo bin2hex(openssl_encrypt($pt, "aes-256-ctr", str_repeat("k", 32), OPENSSL_RAW_DATA, ""));
"#,
    );
    assert_eq!(
        out,
        concat!(
            "835efdd4ef3b4970b9f3db30d9127055f882c14e350482e2dbb2d280a3a88f6d",
            "b7268c07781eaef8c5bb68f682c1ccfe|",
            "4ed947c17245c2403b7f3bbc62aa3ea7c495c675da80902be67c78d801659339c",
            "0c7d00344365121b8fbf0|",
            "d3040c8b0ca3c82b183939dcc25b1556acf172ad850be4639d2e78c1902697284",
            "ee72046aa5ca058b5c0dc44ef422d1f|",
            "e17e9c1f4520d16e0ec019104b5ea547f0f3dfcfa25a8b813c3abf951dd8506e",
            "462b38a791415c8fcc3f56e357502a55|",
            "8aa92548f4b92874dbd7c62cc707e5c01ba5680094fb75f3a91357dd6a4d0e1a",
            "880bd0c5a6d56d1a4b7175",
        )
    );
}

/// Verifies default base64 output and raw/default decrypt round trips.
#[test]
fn test_openssl_non_aead_base64_and_raw_roundtrip() {
    let out = compile_and_run(
        r#"<?php
$pt = "The quick brown fox jumps over the lazy dog";
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
$encoded = openssl_encrypt($pt, "AES-256-CBC", $key, 0, $iv);
$raw = openssl_encrypt($pt, "aes-256-ctr", $key, OPENSSL_RAW_DATA, $iv);
echo $encoded, "|";
echo openssl_decrypt($encoded, "aes-256-cbc", $key, 0, $iv), "|";
echo openssl_decrypt($raw, "AES-256-CTR", $key, OPENSSL_RAW_DATA, $iv);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "vjHwei9me/NHVOjsRbsmgr8x9DDcswOrCJp60TzpOCYLVV1y+KcUFF5Dr3y1CiHh|",
            "The quick brown fox jumps over the lazy dog|",
            "The quick brown fox jumps over the lazy dog",
        )
    );
}

/// Verifies zero padding succeeds for aligned plaintext and rejects partial blocks.
#[test]
fn test_openssl_zero_padding_non_aead() {
    let out = compile_and_run(
        r#"<?php
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
$aligned = "0123456789abcdef0123456789abcdef";
$ciphertext = openssl_encrypt($aligned, "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv);
echo bin2hex($ciphertext), "|";
echo openssl_decrypt($ciphertext, "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv), "|";
echo openssl_encrypt("short", "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv) === false ? "false" : "bad";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "471e64cc510482e98305e3091b78fdbd91dbec72fc540174e38d251b11e15285|",
            "0123456789abcdef0123456789abcdef|false",
        )
    );
}

/// Verifies constants, IV lengths, unknown-cipher false, and the exact 12-name inventory.
#[test]
fn test_openssl_constants_and_cipher_helpers() {
    let out = compile_and_run(
        r#"<?php
namespace CryptoFixture;
$methods = openssl_get_cipher_methods();
echo OPENSSL_RAW_DATA, ":", OPENSSL_ZERO_PADDING, ":", OPENSSL_DONT_ZERO_PAD_KEY, "|";
echo OpEnSsL_CiPhEr_Iv_LeNgTh("aes-128-cbc"), ":", openssl_cipher_iv_length("AES-256-GCM"), ":", openssl_cipher_iv_length("aes-192-ecb"), "|";
echo openssl_cipher_iv_length("not-a-cipher") === false ? "false" : "bad", "|";
echo count($methods), ":", in_array("aes-128-cbc", $methods) ? "yes" : "no", ":", in_array("aes-256-gcm", $methods) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "1:2:4|16:12:0|false|12:yes:yes");
}

/// Verifies unknown ciphers and invalid padded ciphertext return false in both directions.
#[test]
fn test_openssl_non_aead_failures_return_false() {
    let out = compile_and_run(
        r#"<?php
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
echo openssl_encrypt("data", "unknown", $key, OPENSSL_RAW_DATA, $iv) === false ? "e" : "bad";
echo openssl_decrypt("not-a-block", "aes-256-cbc", $key, OPENSSL_RAW_DATA, $iv) === false ? "d" : "bad";
echo openssl_encrypt("data", "aes-256-cbc", "short", OPENSSL_RAW_DATA | OPENSSL_DONT_ZERO_PAD_KEY, $iv) === false ? "k" : "bad";
echo openssl_decrypt("%%%", "aes-256-cbc", $key, 0, $iv) === false ? "b" : "bad";
"#,
    );
    assert_eq!(out, "edkb");
}
