//! Purpose:
//! Security regression tests for hostile native PHAR size fields.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Allocation probes run in exact-filtered child processes to isolate the global counter.

use super::*;

/// Verifies user-facing PHAR documentation discloses both decompression
/// ceilings that may reject archives accepted by reference PHP.
#[test]
fn phar_documentation_discloses_decompression_safety_limits() {
    let streams_docs = include_str!("../../../../docs/php/streams.md");
    assert!(
        streams_docs.contains("1024") && streams_docs.contains("64 MiB"),
        "PHAR docs must disclose the 1024x ratio and 64 MiB decompression ceilings"
    );
}

/// Verifies user-facing PHAR documentation discloses the OpenSSL public-key
/// sidecar contract and the fail-closed behavior for unauthenticated archives.
#[test]
fn phar_documentation_discloses_openssl_verification_contract() {
    let streams_docs = include_str!("../../../../docs/php/streams.md");
    assert!(
        streams_docs.contains("<archive>.pubkey")
            && streams_docs.contains("fail closed")
            && streams_docs.contains("does not create this sidecar"),
        "PHAR docs must disclose the OpenSSL sidecar and fail-closed contract"
    );
}

/// Runs one allocation-sensitive test in a fresh exact-filtered child so
/// unrelated parallel PHAR tests cannot contaminate the global counter.
fn run_allocation_probe_in_child(test_name: &str) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .env("ELEPHC_PHAR_ALLOC_PROBE", test_name)
        .output()
        .expect("spawn isolated PHAR allocation probe");
    assert!(
        output.status.success(),
        "isolated PHAR allocation probe failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Rejects an impossible manifest file count before reserving capacity
/// proportional to the attacker-controlled `u32` field.
#[test]
fn native_phar_manifest_count_does_not_preallocate_untrusted_capacity() {
    const TEST_NAME: &str =
        "tests::security::native_phar_manifest_count_does_not_preallocate_untrusted_capacity";
    if std::env::var("ELEPHC_PHAR_ALLOC_PROBE").as_deref() != Ok(TEST_NAME) {
        run_allocation_probe_in_child(TEST_NAME);
        return;
    }

    let mut archive = b"<?php __HALT_COMPILER();".to_vec();
    archive.extend_from_slice(&18u32.to_le_bytes());
    archive.extend_from_slice(&100_000u32.to_le_bytes());
    archive.extend_from_slice(&[0x11, 0x00]);
    archive.extend_from_slice(&0u32.to_le_bytes());
    archive.extend_from_slice(&0u32.to_le_bytes());
    archive.extend_from_slice(&0u32.to_le_bytes());

    PHAR_TEST_LARGEST_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    PHAR_TEST_TRACK_ALLOCATIONS.store(true, std::sync::atomic::Ordering::Relaxed);
    let parsed = parse_native_phar_archive(&archive);
    PHAR_TEST_TRACK_ALLOCATIONS.store(false, std::sync::atomic::Ordering::Relaxed);
    let largest = PHAR_TEST_LARGEST_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert!(parsed.is_none(), "the truncated hostile manifest must be rejected");
    assert!(
        largest <= 1024 * 1024,
        "manifest validation reserved an attacker-sized allocation of {largest} bytes"
    );
}

/// Rejects a tiny compressed payload with an implausible uncompressed-size
/// claim before reserving the entire claimed output buffer.
#[test]
fn native_phar_payload_claim_does_not_preallocate_untrusted_capacity() {
    const TEST_NAME: &str =
        "tests::security::native_phar_payload_claim_does_not_preallocate_untrusted_capacity";
    if std::env::var("ELEPHC_PHAR_ALLOC_PROBE").as_deref() != Ok(TEST_NAME) {
        run_allocation_probe_in_child(TEST_NAME);
        return;
    }

    let stored = deflate_payload(b"tiny");
    let archive = build_native_phar_with_flags(&[(
        "bomb.txt",
        &stored,
        16 * 1024 * 1024,
        PHAR_FILE_MODE_0644 | PHAR_FLAG_GZIP,
    )]);

    PHAR_TEST_LARGEST_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    PHAR_TEST_TRACK_ALLOCATIONS.store(true, std::sync::atomic::Ordering::Relaxed);
    let parsed = parse_native_phar_archive(&archive);
    PHAR_TEST_TRACK_ALLOCATIONS.store(false, std::sync::atomic::Ordering::Relaxed);
    let largest = PHAR_TEST_LARGEST_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert!(parsed.is_none(), "the inconsistent compressed entry must be rejected");
    assert!(
        largest <= 1024 * 1024,
        "payload validation reserved an attacker-sized allocation of {largest} bytes"
    );
}

/// Rejects a deflate stream whose real output exceeds its tiny manifest
/// claim without first allocating the attacker-controlled expansion.
#[test]
fn native_phar_actual_deflate_output_is_bounded_by_claim() {
    const TEST_NAME: &str =
        "tests::security::native_phar_actual_deflate_output_is_bounded_by_claim";
    if std::env::var("ELEPHC_PHAR_ALLOC_PROBE").as_deref() != Ok(TEST_NAME) {
        run_allocation_probe_in_child(TEST_NAME);
        return;
    }

    let expanded = vec![b'A'; 8 * 1024 * 1024];
    let stored = deflate_payload(&expanded);
    drop(expanded);
    PHAR_TEST_LARGEST_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    PHAR_TEST_TRACK_ALLOCATIONS.store(true, std::sync::atomic::Ordering::Relaxed);
    let decoded = decode_phar_payload(&stored, PHAR_FLAG_GZIP, 64);
    PHAR_TEST_TRACK_ALLOCATIONS.store(false, std::sync::atomic::Ordering::Relaxed);
    let largest = PHAR_TEST_LARGEST_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert!(decoded.is_none(), "a stream larger than its claim must be rejected");
    assert!(
        largest <= 1024 * 1024,
        "decoder allocated {largest} bytes for a 64-byte claim"
    );
}

/// Rejects a ZIP central-directory size claim before allocating the declared
/// output capacity, applying the same policy as native PHAR entry decoding.
#[test]
fn zip_payload_claim_does_not_preallocate_untrusted_capacity() {
    const TEST_NAME: &str =
        "tests::security::zip_payload_claim_does_not_preallocate_untrusted_capacity";
    if std::env::var("ELEPHC_PHAR_ALLOC_PROBE").as_deref() != Ok(TEST_NAME) {
        run_allocation_probe_in_child(TEST_NAME);
        return;
    }

    let mut archive = build_zip(&[("bomb.txt", b"tiny", true)]);
    let (_, central_offset) = zip_eocd_info(&archive).unwrap();
    archive[central_offset + 24..central_offset + 28]
        .copy_from_slice(&(16 * 1024 * 1024u32).to_le_bytes());

    PHAR_TEST_LARGEST_ALLOCATION.store(0, std::sync::atomic::Ordering::Relaxed);
    PHAR_TEST_TRACK_ALLOCATIONS.store(true, std::sync::atomic::Ordering::Relaxed);
    let decoded = extract_entry_bytes(&archive, b"bomb.txt");
    PHAR_TEST_TRACK_ALLOCATIONS.store(false, std::sync::atomic::Ordering::Relaxed);
    let largest = PHAR_TEST_LARGEST_ALLOCATION.load(std::sync::atomic::Ordering::Relaxed);

    assert!(decoded.is_none(), "the implausible ZIP size claim must be rejected");
    assert!(
        largest <= 1024 * 1024,
        "ZIP validation reserved an attacker-sized allocation of {largest} bytes"
    );
}

/// Verifies targeted ZIP reads do not decode a hostile unrelated entry before
/// locating and returning the requested payload.
#[test]
fn zip_targeted_extraction_skips_unrelated_bomb() {
    let mut archive = build_zip(&[
        ("bomb.txt", b"tiny", true),
        ("wanted.txt", b"requested payload", true),
    ]);
    let (_, central_offset) = zip_eocd_info(&archive).unwrap();
    archive[central_offset + 24..central_offset + 28]
        .copy_from_slice(&(16 * 1024 * 1024u32).to_le_bytes());

    assert_eq!(extract_entry_bytes(&archive, b"bomb.txt"), None);
    assert_eq!(
        extract_entry_bytes(&archive, b"wanted.txt").as_deref(),
        Some(&b"requested payload"[..])
    );
}

/// Verifies byte extraction dispatches a ZIP PHAR by magic and therefore cannot
/// bypass its signature check through native-PHAR fallback parsing.
#[test]
fn zip_byte_extraction_rejects_tampered_signed_archive() {
    let path = std::env::temp_dir().join(format!(
        "elephc-phar-byte-dispatch-{}.zip",
        std::process::id()
    ));
    let path_bytes = path.to_string_lossy();
    let payload = b"authenticated byte API payload";
    assert_eq!(
        put_entry_bytes(path_bytes.as_bytes(), b"payload.txt", payload),
        Some(payload.len())
    );
    assert_eq!(sign_archive_hash(path_bytes.as_bytes(), 3), Some(()));
    let mut archive = std::fs::read(&path).unwrap();
    let offset = archive
        .windows(payload.len())
        .position(|window| window == payload)
        .expect("locate signed ZIP payload");
    archive[offset] ^= 1;

    assert_eq!(extract_entry_bytes(&archive, b"payload.txt"), None);
    std::fs::remove_file(path).ok();
}

/// Verifies ordinary unsigned entry data ending in the PHAR trailer magic is
/// not mistaken for a malformed signature trailer.
#[test]
fn native_phar_payload_ending_in_gbmb_remains_readable() {
    let payload = b"ordinary payload GBMB";
    let archive = build_native_phar(&[("payload.txt", payload)]);
    assert_eq!(
        extract_entry_bytes(&archive, b"payload.txt").as_deref(),
        Some(&payload[..])
    );
}

/// Verifies a native PHAR that declares the signature header flag cannot be
/// opened without a complete, recognized, and valid signature trailer.
#[test]
fn native_phar_signature_flag_requires_valid_trailer() {
    let mut archive = build_native_phar(&[("payload.txt", b"signed by declaration")]);
    let halt = find_subslice(&archive, b"__HALT_COMPILER();").unwrap();
    let mut manifest_start = halt + b"__HALT_COMPILER();".len();
    for &byte in &[b' ', b'?', b'>', b'\r', b'\n'] {
        if archive.get(manifest_start) == Some(&byte) {
            manifest_start += 1;
        }
    }
    archive[manifest_start + 10..manifest_start + 14]
        .copy_from_slice(&PHAR_HDR_SIGNATURE.to_le_bytes());

    assert!(
        parse_native_phar_archive(&archive).is_none(),
        "the signed-header flag must not be accepted without a trailer"
    );

    archive.extend_from_slice(&0x7fff_ffffu32.to_le_bytes());
    archive.extend_from_slice(b"GBMB");
    assert!(
        parse_native_phar_archive(&archive).is_none(),
        "an unknown signature type must not satisfy the signed-header flag"
    );
}

/// Verifies a recognizable signature trailer is rejected when the native
/// manifest does not declare the signature header flag.
#[test]
fn native_phar_signature_trailer_requires_header_flag() {
    let mut archive = build_native_phar(&[("payload.txt", b"unsigned manifest")]);
    append_sha1_signature(&mut archive);
    assert!(
        parse_native_phar_archive(&archive).is_none(),
        "signature framing and the native manifest flag must agree"
    );
}
