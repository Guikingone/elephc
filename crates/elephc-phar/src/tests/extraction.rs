//! Purpose:
//! Tests extraction and entry listing across supported PHAR container families.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Native PHAR, tar, stored ZIP, and deflated ZIP fixtures share canonical expectations.

use super::*;

/// Verifies native PHAR manifest extraction.
#[test]
pub(super) fn extracts_native_phar_entry() {
    let archive = build_native_phar(&[("a.txt", b"alpha"), ("dir/b.txt", b"bravo")]);
    assert_eq!(
        extract_entry_bytes(&archive, b"dir/b.txt").as_deref(),
        Some(&b"bravo"[..])
    );
}

/// Verifies tar container extraction.
#[test]
pub(super) fn extracts_tar_entry() {
    let archive = build_tar(&[("a.txt", b"alpha"), ("dir/b.txt", b"bravo")]);
    assert_eq!(
        extract_entry_bytes(&archive, b"dir/b.txt").as_deref(),
        Some(&b"bravo"[..])
    );
}

/// Verifies ZIP store and deflate extraction.
#[test]
pub(super) fn extracts_zip_entries() {
    let archive = build_zip(&[
        ("plain.txt", b"stored", false),
        ("deflated.txt", b"deflated payload", true),
    ]);
    assert_eq!(
        extract_entry_bytes(&archive, b"plain.txt").as_deref(),
        Some(&b"stored"[..])
    );
    assert_eq!(
        extract_entry_bytes(&archive, b"deflated.txt").as_deref(),
        Some(&b"deflated payload"[..])
    );
}

/// Verifies entry-name listing across supported archive families.
#[test]
pub(super) fn lists_entry_names_for_supported_archive_families() {
    let base = std::env::temp_dir().join(format!(
        "elephc_phar_list_{}_{}",
        std::process::id(),
        "unit"
    ));
    let phar_path = base.with_extension("phar");
    let tar_path = base.with_extension("tar");
    let zip_path = base.with_extension("zip");

    std::fs::write(
        &phar_path,
        build_native_phar(&[("one.txt", b"alpha"), ("dir/two.txt", b"bravo")]),
    )
    .unwrap();
    std::fs::write(
        &tar_path,
        build_tar(&[("tar.txt", b"tar"), ("dir/nested.txt", b"nested")]),
    )
    .unwrap();
    std::fs::write(
        &zip_path,
        build_zip(&[("zip.txt", b"zip", false), ("def.txt", b"def", true)]),
    )
    .unwrap();

    assert_eq!(
        entry_names_bytes(phar_path.to_string_lossy().as_bytes()).as_deref(),
        Some(serialized_names(&["one.txt", "dir/two.txt"]).as_slice())
    );
    assert_eq!(
        entry_names_bytes(tar_path.to_string_lossy().as_bytes()).as_deref(),
        Some(serialized_names(&["tar.txt", "dir/nested.txt"]).as_slice())
    );
    assert_eq!(
        entry_names_bytes(zip_path.to_string_lossy().as_bytes()).as_deref(),
        Some(serialized_names(&["zip.txt", "def.txt"]).as_slice())
    );

    std::fs::remove_file(&phar_path).ok();
    std::fs::remove_file(&tar_path).ok();
    std::fs::remove_file(&zip_path).ok();
}
