//! Purpose:
//! Ownership coverage for issue #1049: spreading an INDEXED array into a hash destination must
//! read its source, not consume it.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `Op::ArrayToHash` CONSUMES its operand: the promote path abandons the source indexed array
//!   for a freshly built hash and decrefs it. The spread lowering handed it a borrowed local, so
//!   the promotion released the caller's only reference.
//! - The symptom was quiet: the source read back as `array(0) {}` while its elements still read
//!   correctly through the freed block, so a fixture that checked only the RESULT passed.
//! - These live here rather than beside the array-semantics fixtures because they measure
//!   lifetimes (AGENTS.md:383).

use crate::support::*;

/// Verifies spreading an indexed array leaves the SOURCE untouched.
///
/// `Op::ArrayToHash` consumes its operand: its promote path abandons the source indexed array
/// for a freshly built hash and decrefs it. The spread lowering handed it a borrowed local, so
/// the promotion released the caller's only reference -- `$idx` read back as `array(0) {}` with
/// its elements still intact, and spreading it twice crashed.
///
/// This shape needs NO explicit key, so it reached the same promotion on `main` long before
/// mixed literals existed; it is pinned here because the mixed-literal work is what made it
/// common enough to hit.
#[test]
fn test_spreading_an_indexed_array_leaves_the_source_intact() {
    let out = compile_and_run(
        r#"<?php
$idx = [3, 4];
$assoc = ["x" => 1];
$once = [...$idx, ...$assoc];
$twice = [...$idx, "c" => 8];
echo count($once), count($twice), "|";
foreach ($idx as $k => $v) { echo $k, "=", $v, ","; }
echo "|", count($idx);
"#,
    );
    assert_eq!(out, "33|0=3,1=4,|2");
}

/// Verifies a mixed literal allocates nothing it does not free.
///
/// The promotion's acquire has to be balanced by the release of the promoted hash. Getting that
/// ledger wrong leaks the source array once per iteration, which only a repeated fixture shows.
#[test]
fn test_a_mixed_literal_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$total = 0;
for ($i = 0; $i < 100; $i++) {
    $src = [$i, $i + 1];
    $a = ["head" => "h", ...$src, "tail" => "t"];
    $total += count($a) + count($src);
}
echo $total;
"#,
    );
    assert_eq!(out.stdout, "600", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "a mixed literal must not leak its promoted source: {}",
        out.stderr
    );
}
