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
