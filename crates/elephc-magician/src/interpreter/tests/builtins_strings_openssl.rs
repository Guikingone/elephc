//! Purpose:
//! Interpreter tests for OpenSSL-compatible AES cipher builtins.
//!
//! Called from:
//! - `cargo test -p elephc-magician openssl` through Rust's test harness.
//!
//! Key details:
//! - Ciphertext and GCM tags are locked to the PHP 8.4/OpenSSL 3.6 fixtures.
//! - Direct, named, callable, by-reference, helper, and failure paths mirror AOT coverage.

use super::super::*;
use super::support::*;

/// Verifies every cipher advertised by magician round-trips through the shared bridge.
#[test]
fn execute_program_round_trips_full_openssl_cipher_matrix() {
    let program = parse_fragment(
        br#"$plaintext = "matrix plaintext";
$count = 0;
foreach (openssl_get_cipher_methods() as $method) {
    if (str_contains($method, "-128-")) {
        $key = str_repeat("k", 16);
    } elseif (str_contains($method, "-192-")) {
        $key = str_repeat("k", 24);
    } else {
        $key = str_repeat("k", 32);
    }
    $iv = str_repeat("i", openssl_cipher_iv_length($method));
    if (str_ends_with($method, "-gcm")) {
        $ciphertext = openssl_encrypt($plaintext, $method, $key, OPENSSL_RAW_DATA, $iv, $tag, "matrix-aad");
        $decrypted = openssl_decrypt($ciphertext, $method, $key, OPENSSL_RAW_DATA, $iv, $tag, "matrix-aad");
    } else {
        $ciphertext = openssl_encrypt($plaintext, $method, $key, OPENSSL_RAW_DATA, $iv);
        $decrypted = openssl_decrypt($ciphertext, $method, $key, OPENSSL_RAW_DATA, $iv);
    }
    if ($decrypted !== $plaintext) {
        return false;
    }
    $count++;
}
return $count === 12;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies CBC, CTR, and ECB PHP goldens plus raw/default decrypt round trips.
#[test]
fn execute_program_dispatches_openssl_non_aead_builtins() {
    let program = parse_fragment(
        br#"$pt = "The quick brown fox jumps over the lazy dog";
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
echo bin2hex(openssl_encrypt($pt, "aes-128-cbc", str_repeat("k", 16), OPENSSL_RAW_DATA, $iv)), "|";
echo bin2hex(openssl_encrypt($pt, "aes-256-ctr", $key, OPENSSL_RAW_DATA, $iv)), "|";
echo bin2hex(openssl_encrypt($pt, "aes-192-ecb", str_repeat("k", 24), OPENSSL_RAW_DATA)), "|";
$encoded = openssl_encrypt($pt, "AES-256-CBC", $key, 0, $iv);
echo $encoded, "|", openssl_decrypt($encoded, "aes-256-cbc", $key, 0, $iv), "|";
$raw = call_user_func("openssl_encrypt", $pt, "aes-256-ctr", $key, OPENSSL_RAW_DATA, $iv);
echo call_user_func_array("openssl_decrypt", [$raw, "AES-256-CTR", $key, OPENSSL_RAW_DATA, $iv]);
return function_exists("openssl_encrypt") && function_exists("openssl_decrypt");"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "835efdd4ef3b4970b9f3db30d9127055f882c14e350482e2dbb2d280a3a88f6d",
            "b7268c07781eaef8c5bb68f682c1ccfe|",
            "4ed947c17245c2403b7f3bbc62aa3ea7c495c675da80902be67c78d801659339c",
            "0c7d00344365121b8fbf0|",
            "d3040c8b0ca3c82b183939dcc25b1556acf172ad850be4639d2e78c1902697284",
            "ee72046aa5ca058b5c0dc44ef422d1f|",
            "vjHwei9me/NHVOjsRbsmgr8x9DDcswOrCJp60TzpOCYLVV1y+KcUFF5Dr3y1CiHh|",
            "The quick brown fox jumps over the lazy dog|",
            "The quick brown fox jumps over the lazy dog",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies constants, IV lengths, exact cipher inventory, zero padding, and stable failures.
#[test]
fn execute_program_dispatches_openssl_helpers_and_failures() {
    let program = parse_fragment(
        br#"$methods = openssl_get_cipher_methods();
echo OPENSSL_RAW_DATA, ":", OPENSSL_ZERO_PADDING, ":", OPENSSL_DONT_ZERO_PAD_KEY, "|";
echo openssl_cipher_iv_length("aes-128-cbc"), ":", openssl_cipher_iv_length("AES-256-GCM"), ":", openssl_cipher_iv_length("aes-192-ecb"), "|";
echo openssl_cipher_iv_length("not-a-cipher") === false ? "false" : "bad", "|";
echo count($methods), ":", implode(",", $methods), "|";
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
$aligned = "0123456789abcdef0123456789abcdef";
$ciphertext = openssl_encrypt($aligned, "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv);
echo bin2hex($ciphertext), ":", openssl_decrypt($ciphertext, "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv), "|";
echo openssl_encrypt("short", "aes-256-cbc", $key, OPENSSL_RAW_DATA | OPENSSL_ZERO_PADDING, $iv) === false ? "p" : "bad";
echo openssl_encrypt("data", "unknown", $key, OPENSSL_RAW_DATA, $iv) === false ? "e" : "bad";
echo openssl_decrypt("not-a-block", "aes-256-cbc", $key, OPENSSL_RAW_DATA, $iv) === false ? "d" : "bad";
echo openssl_encrypt("data", "aes-256-cbc", "short", OPENSSL_RAW_DATA | OPENSSL_DONT_ZERO_PAD_KEY, $iv) === false ? "k" : "bad";
return function_exists("openssl_cipher_iv_length") && function_exists("openssl_get_cipher_methods") && defined("OPENSSL_RAW_DATA") && defined("OPENSSL_ZERO_PADDING") && defined("OPENSSL_DONT_ZERO_PAD_KEY");"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "1:2:4|16:12:0|false|12:",
            "aes-128-cbc,aes-128-ctr,aes-128-ecb,aes-128-gcm,",
            "aes-192-cbc,aes-192-ctr,aes-192-ecb,aes-192-gcm,",
            "aes-256-cbc,aes-256-ctr,aes-256-ecb,aes-256-gcm|",
            "471e64cc510482e98305e3091b78fdbd91dbec72fc540174e38d251b11e15285:",
            "0123456789abcdef0123456789abcdef|pedk",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies all AES-GCM key sizes and supported truncated tags against PHP goldens.
#[test]
fn execute_program_dispatches_openssl_gcm_goldens_and_named_tag_writeback() {
    let program = parse_fragment(
        br#"$pt = "The quick brown fox jumps over the lazy dog";
$iv = str_repeat("i", 12);
$key256 = str_repeat("k", 32);
$cipher256 = openssl_encrypt($pt, "aes-256-gcm", $key256, OPENSSL_RAW_DATA, $iv, $tag256, "fixture-aad");
echo bin2hex($cipher256), ":", bin2hex($tag256), "|";
$short = openssl_encrypt(tag: $tag12, data: $pt, cipher_algo: "AES-256-GCM", passphrase: $key256, options: OPENSSL_RAW_DATA, iv: $iv, aad: "fixture-aad", tag_length: 12);
echo bin2hex($short), ":", bin2hex($tag12), "|";
$key128 = str_repeat("k", 16);
$cipher128 = openssl_encrypt($pt, "aes-128-gcm", $key128, OPENSSL_RAW_DATA, $iv, $tag128, "fixture-aad");
echo bin2hex($cipher128), ":", bin2hex($tag128), ":", openssl_decrypt($cipher128, "aes-128-gcm", $key128, OPENSSL_RAW_DATA, $iv, $tag128, "fixture-aad") === $pt ? "ok" : "bad", "|";
$key192 = str_repeat("k", 24);
$cipher192 = openssl_encrypt($pt, "aes-192-gcm", $key192, OPENSSL_RAW_DATA, $iv, $tag192, "fixture-aad");
echo bin2hex($cipher192), ":", bin2hex($tag192), ":", openssl_decrypt($cipher192, "aes-192-gcm", $key192, OPENSSL_RAW_DATA, $iv, $tag192, "fixture-aad") === $pt ? "ok" : "bad", "|";
$cipher1 = openssl_encrypt($pt, "aes-256-gcm", $key256, OPENSSL_RAW_DATA, $iv, $tag1, "fixture-aad", 1);
$cipher4 = openssl_encrypt($pt, "aes-256-gcm", $key256, OPENSSL_RAW_DATA, $iv, $tag4, "fixture-aad", 4);
echo bin2hex($tag1), ":", openssl_decrypt($cipher1, "aes-256-gcm", $key256, OPENSSL_RAW_DATA, $iv, $tag1, "fixture-aad") === $pt ? "ok" : "bad", "|";
echo bin2hex($tag4), ":", openssl_decrypt($cipher4, "aes-256-gcm", $key256, OPENSSL_RAW_DATA, $iv, $tag4, "fixture-aad") === $pt ? "ok" : "bad";
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "8eec9d54438dc8679074830af4604b47f4cb062a754edb3c561d21b76dbac62b",
            "6fd7b2c040ca67e3e5647a:9dc8b5e7b800bfed5c1be43fc4614f51|",
            "8eec9d54438dc8679074830af4604b47f4cb062a754edb3c561d21b76dbac62b",
            "6fd7b2c040ca67e3e5647a:9dc8b5e7b800bfed5c1be43f|",
            "498c432b6dc88827705706a39e646b6a0698aa89834fb084768c9c416f758018",
            "ccc5ae865bc43a1db8b91a:bec1dfa8e8781396a92dc401358ee08c:ok|",
            "4213f3f4708cef7d03ac167c8487ec43b14e740e0f0ea458b41ae4eca28cb9cb",
            "46d24ec873f24e40010cf5:f9ebd39bfa23886420d8a159d50f4dad:ok|",
            "9d:ok|9dc8b5e7:ok",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies GCM non-default IVs, empty plaintext, tag replacement, and authentication failures.
#[test]
fn execute_program_dispatches_openssl_gcm_edges_and_failures() {
    let program = parse_fragment(
        br#"$pt = "The quick brown fox jumps over the lazy dog";
$key = str_repeat("k", 32);
$iv = str_repeat("i", 16);
$tag = "old";
$ciphertext = openssl_encrypt($pt, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, $tag, "fixture-aad");
echo bin2hex($ciphertext), ":", bin2hex($tag), "|";
echo openssl_decrypt($ciphertext, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, $tag, "fixture-aad"), "|";
echo openssl_decrypt($ciphertext, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, $tag, "wrong-aad") === false ? "a" : "bad";
echo openssl_decrypt($ciphertext, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, str_repeat("x", 16), "fixture-aad") === false ? "w" : "bad";
echo openssl_decrypt($ciphertext, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv) === false ? "m" : "bad";
echo openssl_encrypt($pt, "aes-256-gcm", $key, OPENSSL_RAW_DATA, "", $empty_iv_tag, "fixture-aad") === false ? "i" : "bad";
echo openssl_encrypt($pt, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, $zero_tag, "fixture-aad", 0) === false ? "z" : "bad";
echo openssl_encrypt($pt, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv, $long_tag, "fixture-aad", 17) === false ? "l" : "bad";
echo openssl_encrypt($pt, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv) === false ? "t" : "bad", "|";
$iv12 = str_repeat("i", 12);
$empty = openssl_encrypt("", "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv12, $empty_tag, "fixture-aad");
echo strlen($empty), ":", bin2hex($empty_tag), ":", strlen(openssl_decrypt($empty, "aes-256-gcm", $key, OPENSSL_RAW_DATA, $iv12, $empty_tag, "fixture-aad")), "|";
$cbc_tag = "keep";
openssl_encrypt($pt, "aes-256-cbc", $key, OPENSSL_RAW_DATA, str_repeat("i", 16), $cbc_tag);
echo $cbc_tag;
return true;"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let result = execute_program(&program, &mut scope, &mut values).expect("execute eval ir");

    assert_eq!(
        values.output,
        concat!(
            "058c1967af117fc4142b6c851c3ca083c65b71f3c28d9041a8247fc5f3070afb",
            "513839a2043457948c44b6:69490efb2851c5a47142b09f074b143e|",
            "The quick brown fox jumps over the lazy dog|awmizlt|",
            "0:76a0e5a2ff64c6f1c8ee0f2c0f066c91:0|keep",
        )
    );
    assert_eq!(values.get(result), FakeValue::Bool(true));
}

/// Verifies an explicitly supplied encrypt tag argument must be writable even for CBC.
#[test]
fn execute_program_rejects_openssl_encrypt_literal_tag_target() {
    let program = parse_fragment(
        br#"openssl_encrypt("data", "aes-256-cbc", str_repeat("k", 32), OPENSSL_RAW_DATA, str_repeat("i", 16), "literal");"#,
    )
    .expect("parse eval fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();

    let status = execute_program(&program, &mut scope, &mut values)
        .expect_err("literal tag target must fail eval execution");

    assert_eq!(status, EvalStatus::RuntimeFatal);
}
