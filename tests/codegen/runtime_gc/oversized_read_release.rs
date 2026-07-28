//! Purpose:
//! Regression tests: a read larger than the concat arena's free space takes an owned
//! heap block instead of overrunning the arena, and that block must be reclaimed.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each fixture runs under `--heap-debug` and asserts `leak summary: clean`.
//! - The leak needed two independent omissions to appear, so a fixture that exercises
//!   only one of them passes either way: the EIR already emitted a `release` for these
//!   values but the backend discarded it (`value_is_scratch_string` treated any
//!   non-`Fresh` runtime call as arena scratch), *and* the block carried heap kind 0,
//!   which every `__rt_decref_any` in the runtime's own read paths skips as raw.
//! - The fixtures use a bare relative filename rather than `sys_get_temp_dir()`. That
//!   helper leaks a small block on Windows (one 54-byte path survived a run that had
//!   already released every 100 KB read block), which is a separate defect: dragging it
//!   in would make this file fail for a reason that has nothing to do with what it tests.
//! - Sizes straddle the 64 KiB arena deliberately: a read that fits returns a borrowed
//!   arena slice and must NOT be freed, which is what the small-read fixture pins.

use crate::support::compile_and_run_with_heap_debug;

/// Asserts the program printed `expected` and left a clean heap under heap debug.
fn assert_clean(out: crate::support::ProgramOutput, expected: &str) {
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Draining a file with reads larger than the arena must not strand one block per read.
#[test]
fn test_oversized_fread_releases_its_block_every_iteration() {
    // Each iteration asks for 100000 bytes, well past what the 64 KiB arena can hold,
    // so every read takes its own block. The result is consumed by strlen() and never
    // stored, which is the shape that leaked: five blocks and half a megabyte survived
    // a 500 KB file, and they accumulated for the life of the process.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$path = "elephc-oversized-read-release.bin";
$sink = fopen($path, "w");
$chunk = str_repeat("Q", 1000);
for ($i = 0; $i < 500; $i++) { fwrite($sink, $chunk); }
fclose($sink);

$source = fopen($path, "r");
$total = 0;
while (!feof($source)) { $total += strlen(fread($source, 100000)); }
fclose($source);
unlink($path);
echo $total;
"#,
    );
    assert_clean(out, "500000");
}

/// Boxing a read-all result into a Mixed cell must not strand its accumulation buffer.
#[test]
fn test_stream_get_contents_mixed_box_releases_its_accumulation_buffer() {
    // `stream_get_contents()` is typed `string|false`, so its result is boxed into a Mixed
    // cell inside the lowering itself. The boxing copies rather than adopts --
    // `__rt_mixed_from_value` tag 1 goes through `__rt_str_persist`, which always allocates --
    // so the accumulation block the read owned was left with no owner and no EIR value naming
    // it. A 500 KB stream outgrows the 64 KiB arena, so each iteration stranded one
    // 524288-byte block: four blocks and two megabytes survived this program.
    //
    // The str_starts_with/str_ends_with pair is the ordering guard, not decoration. Releasing
    // the source before boxing copied it would still produce the right *length*, so a
    // length-only assertion passes either way; reading the bytes is what pins copy-then-free.
    // They are also the reason this fixture holds no large expected-value local: a 500 KB
    // top-level string is itself never reclaimed at exit, which would mask the result.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$path = "elephc-sgc-mixed-box-release.bin";
$sink = fopen($path, "w");
for ($i = 0; $i < 500; $i++) { fwrite($sink, str_repeat("Q", 1000)); }
fclose($sink);

$total = 0;
$intact = 0;
for ($i = 0; $i < 4; $i++) {
    $source = fopen($path, "r");
    $body = stream_get_contents($source);
    $total += strlen($body);
    if (str_starts_with($body, "QQQ") && str_ends_with($body, "QQQ")) { $intact++; }
    fclose($source);
}
unlink($path);
echo $total, ":", $intact;
"#,
    );
    assert_clean(out, "2000000:4");
}

/// A read-all that still fits the arena is borrowed storage and must not be released.
#[test]
fn test_arena_sized_stream_get_contents_box_stays_borrowed() {
    // The negative control for the fixture above. The release added there runs on every
    // read-all, including this one, where the result is a borrowed `_concat_buf` slice --
    // freeing that would be a wild free rather than a leak. Nothing heuristic protects it:
    // `_concat_buf` and `_heap_buf` are separate `.comm` objects, so an arena pointer can
    // never satisfy `__rt_heap_free_safe`'s managed-heap window test. This fixture is what
    // notices if that ever stops holding.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$path = "elephc-sgc-arena-box.bin";
$sink = fopen($path, "w");
fwrite($sink, str_repeat("R", 4096));
fclose($sink);

$total = 0;
$intact = 0;
for ($i = 0; $i < 4; $i++) {
    $source = fopen($path, "r");
    $body = stream_get_contents($source);
    $total += strlen($body);
    if (str_starts_with($body, "RRR") && str_ends_with($body, "RRR")) { $intact++; }
    fclose($source);
}
unlink($path);
echo $total, ":", $intact;
"#,
    );
    assert_clean(out, "16384:4");
}

/// A read that still fits the arena is borrowed storage and must not be released.
#[test]
fn test_arena_sized_fread_is_borrowed_and_stays_clean() {
    // The negative control for the fixture above. Releasing an arena slice would be a
    // wild free rather than a leak, and the guard against it is not a heuristic:
    // `_concat_buf` and `_heap_buf` are separate `.comm` objects, so an arena pointer
    // can never satisfy `__rt_heap_free_safe`'s managed-heap window test. If that ever
    // stops holding, this fixture is what notices.
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$path = "elephc-arena-read-release.bin";
$sink = fopen($path, "w");
fwrite($sink, str_repeat("R", 4096));
fclose($sink);

$source = fopen($path, "r");
$total = 0;
while (!feof($source)) { $total += strlen(fread($source, 512)); }
fclose($source);
unlink($path);
echo $total;
"#,
    );
    assert_clean(out, "4096");
}
