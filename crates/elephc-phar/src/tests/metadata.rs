//! Purpose:
//! Tests global metadata, per-file metadata, stubs, and whole-archive compression.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Metadata representations differ by archive family but share round-trip contracts.

use super::*;

/// Stub used by the metadata/stub round-trip tests; ends with `?>\r\n` so the
/// native-PHAR `__HALT_COMPILER();` boundary scan round-trips it exactly.
const ROUND_TRIP_STUB: &[u8] = b"<?php Phar::mapPhar(); __HALT_COMPILER(); ?>\r\n";
const ROUND_TRIP_META: &[u8] = b"a:1:{s:3:\"ver\";s:5:\"1.2.3\";}";

/// Shared body: set metadata+stub, prove they survive a later entry write, and
/// that the reserved `.phar/*` control files stay hidden from the entry listing.
pub(super) fn check_metadata_stub_round_trip(ext: &str, tag: &str) {
    let path = std::env::temp_dir().join(format!(
        "elephc_phar_meta_{}_{}.{}",
        std::process::id(),
        tag,
        ext
    ));
    let pb = path.to_string_lossy();
    let pbytes = pb.as_bytes();
    assert_eq!(put_entry_bytes(pbytes, b"a.txt", b"alpha"), Some(5));
    assert_eq!(set_metadata_bytes(pbytes, ROUND_TRIP_META), Some(()));
    assert_eq!(set_stub_bytes(pbytes, ROUND_TRIP_STUB), Some(()));
    // A later entry write must preserve both metadata and stub.
    assert_eq!(put_entry_bytes(pbytes, b"b.txt", b"bravo"), Some(5));
    assert_eq!(get_metadata_bytes(pbytes).as_deref(), Some(ROUND_TRIP_META));
    assert_eq!(get_stub_bytes(pbytes).as_deref(), Some(ROUND_TRIP_STUB));
    let archive = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).ok();
    assert_eq!(
        extract_entry_bytes(&archive, b"a.txt").as_deref(),
        Some(&b"alpha"[..])
    );
    assert_eq!(
        extract_entry_bytes(&archive, b"b.txt").as_deref(),
        Some(&b"bravo"[..])
    );
    let (entries, _) = parse_archive_entries(&archive).unwrap();
    assert_eq!(entries.len(), 2, "{} entry count", tag);
    assert!(
        entries.iter().all(|e| !e.name.starts_with(b".phar/")),
        "{} leaked a .phar/ control entry",
        tag
    );
}

const ROUND_TRIP_FILE_META: &[u8] = b"a:1:{s:4:\"role\";s:5:\"first\";}";

/// Drives a per-file metadata round-trip for one archive family: set metadata on
/// one entry, confirm it survives a later entry write, and that only the targeted
/// entry carries metadata while `.phar/` control entries never leak.
pub(super) fn check_file_metadata_round_trip(ext: &str, tag: &str) {
    let path = std::env::temp_dir().join(format!(
        "elephc_phar_filemeta_{}_{}.{}",
        std::process::id(),
        tag,
        ext
    ));
    let pb = path.to_string_lossy();
    let pbytes = pb.as_bytes();
    assert_eq!(put_entry_bytes(pbytes, b"a.txt", b"alpha"), Some(5));
    assert_eq!(put_entry_bytes(pbytes, b"b.txt", b"bravo"), Some(5));
    assert_eq!(
        set_file_metadata_bytes(pbytes, b"a.txt", ROUND_TRIP_FILE_META),
        Some(())
    );
    // A later entry write must preserve the per-file metadata.
    assert_eq!(put_entry_bytes(pbytes, b"c.txt", b"charlie"), Some(7));
    assert_eq!(
        get_file_metadata_bytes(pbytes, b"a.txt").as_deref(),
        Some(ROUND_TRIP_FILE_META),
        "{} a.txt metadata",
        tag
    );
    // Untouched entries carry no metadata.
    assert_eq!(
        get_file_metadata_bytes(pbytes, b"b.txt").as_deref(),
        Some(&b""[..]),
        "{} b.txt metadata",
        tag
    );
    // Setting metadata on a missing entry fails.
    assert_eq!(
        set_file_metadata_bytes(pbytes, b"missing.txt", ROUND_TRIP_FILE_META),
        None
    );
    let archive = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).ok();
    let (entries, _) = parse_archive_entries(&archive).unwrap();
    assert_eq!(entries.len(), 3, "{} entry count", tag);
    assert!(
        entries.iter().all(|e| !e.name.starts_with(b".phar/")),
        "{} leaked a .phar/ control entry",
        tag
    );
}

/// Drives a whole-archive compression round-trip: build a tar, compress it with
/// `compressor`, confirm the returned compressed file parses transparently with
/// entries intact, then decompress it and confirm the entries survive again.
pub(super) fn check_archive_compression_round_trip(
    tag: &str,
    ext: &str,
    compressor: fn(&[u8]) -> Option<Vec<u8>>,
) {
    let dir = std::env::temp_dir();
    let src = dir.join(format!("elephc_phar_comp_{}_{}.tar", std::process::id(), tag));
    let sb = src.to_string_lossy();
    assert_eq!(put_entry_bytes(sb.as_bytes(), b"a.txt", b"alpha"), Some(5));
    assert_eq!(put_entry_bytes(sb.as_bytes(), b"b.txt", b"bravo"), Some(5));
    let comp_bytes = compressor(sb.as_bytes()).expect("compress");
    let comp = String::from_utf8(comp_bytes).unwrap();
    assert_eq!(comp, format!("{}.{}", sb, ext), "{} dest path", tag);
    // The compressed file parses transparently with entries intact.
    let (entries, _) = parse_archive_entries(&std::fs::read(&comp).unwrap()).unwrap();
    assert_eq!(entries.len(), 2, "{} compressed entry count", tag);
    assert_eq!(
        extract_entry_bytes(&std::fs::read(&comp).unwrap(), b"a.txt"),
        Some(b"alpha".to_vec())
    );
    // Decompressing reproduces a plain tar (the `.tar` base) with the same entries.
    let back_bytes = decompress_archive(comp.as_bytes()).expect("decompress");
    let back = String::from_utf8(back_bytes).unwrap();
    assert_eq!(back, sb.to_string(), "{} decompress dest path", tag);
    let plain = std::fs::read(&back).unwrap();
    assert_eq!(plain.get(257..262), Some(&b"ustar"[..]), "{} decompressed is tar", tag);
    assert_eq!(extract_entry_bytes(&plain, b"b.txt"), Some(b"bravo".to_vec()));
    for p in [src.to_string_lossy().to_string(), comp] {
        std::fs::remove_file(p).ok();
    }
}

/// A tar archive round-trips through whole-archive gzip compression.
#[test]
pub(super) fn tar_archive_gzip_compress_round_trip() {
    check_archive_compression_round_trip("gz", "gz", gzip_archive);
}

/// A tar archive round-trips through whole-archive bzip2 compression.
#[test]
pub(super) fn tar_archive_bzip2_compress_round_trip() {
    check_archive_compression_round_trip("bz", "bz2", bzip2_archive);
}

/// Per-file metadata round-trips through the native manifest per-entry field.
#[test]
pub(super) fn native_phar_file_metadata_round_trip() {
    check_file_metadata_round_trip("phar", "native");
}

/// Per-file metadata round-trips through `.phar/.metadata/<path>/.metadata.bin`.
#[test]
pub(super) fn tar_phar_file_metadata_round_trip() {
    check_file_metadata_round_trip("tar", "tar");
}

/// Per-file metadata round-trips through the zip central-directory file comment.
#[test]
pub(super) fn zip_phar_file_metadata_round_trip() {
    check_file_metadata_round_trip("zip", "zip");
}

/// Verifies native-PHAR global metadata and stub persist and survive entry writes.
#[test]
pub(super) fn native_phar_metadata_and_stub_round_trip() {
    check_metadata_stub_round_trip("phar", "native");
}

/// Verifies tar-based phar metadata/stub persist via `.phar/.metadata.bin` and
/// `.phar/stub.php`, and survive entry writes.
#[test]
pub(super) fn tar_phar_metadata_and_stub_round_trip() {
    check_metadata_stub_round_trip("tar", "tar");
}

/// Verifies zip-based phar metadata persists in the EOCD comment and the stub in
/// `.phar/stub.php`, and both survive entry writes.
#[test]
pub(super) fn zip_phar_metadata_and_stub_round_trip() {
    check_metadata_stub_round_trip("zip", "zip");
}

/// Verifies `set_stub_bytes` rejects a stub without the `__HALT_COMPILER();` marker.
#[test]
pub(super) fn set_stub_requires_halt_compiler() {
    let path = std::env::temp_dir().join(format!(
        "elephc_phar_badstub_{}.phar",
        std::process::id()
    ));
    let pb = path.to_string_lossy();
    assert_eq!(put_entry_bytes(pb.as_bytes(), b"a.txt", b"alpha"), Some(5));
    assert_eq!(set_stub_bytes(pb.as_bytes(), b"<?php echo 1;"), None);
    std::fs::remove_file(&path).ok();
}
