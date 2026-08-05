<?php

// OpenSSL-compatible symmetric encryption backed by elephc's pure-Rust
// elephc-crypto bridge. Fixed keys and IVs keep this demo reproducible; use
// securely generated, unique values in production.

$plaintext = "elephc keeps PHP crypto portable";
$key = str_repeat("k", 32);

echo "--- AES-256-CBC ---\n";
$cbcIv = str_repeat("i", openssl_cipher_iv_length("aes-256-cbc"));
$encoded = openssl_encrypt($plaintext, "aes-256-cbc", $key, 0, $cbcIv);
echo "ciphertext (base64): " . $encoded . "\n";
echo "decrypted: " . openssl_decrypt($encoded, "aes-256-cbc", $key, 0, $cbcIv) . "\n";

echo "\n--- AES-256-GCM ---\n";
$gcmIv = str_repeat("i", openssl_cipher_iv_length("aes-256-gcm"));
$aad = "example-context";
$ciphertext = openssl_encrypt(
    $plaintext,
    "aes-256-gcm",
    $key,
    OPENSSL_RAW_DATA,
    $gcmIv,
    $tag,
    $aad,
);
echo "ciphertext (hex): " . bin2hex($ciphertext) . "\n";
echo "tag length: " . strlen($tag) . "\n";
echo "decrypted: " . openssl_decrypt(
    $ciphertext,
    "aes-256-gcm",
    $key,
    OPENSSL_RAW_DATA,
    $gcmIv,
    $tag,
    $aad,
) . "\n";

echo "\nsupported ciphers: " . count(openssl_get_cipher_methods()) . "\n";
