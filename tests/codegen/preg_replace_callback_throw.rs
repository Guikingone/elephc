//! Purpose:
//! Regression coverage for `preg_replace_callback` releasing its private buffers
//! when the callback throws, including after the output block has grown.
//!
//! Called from:
//! - `cargo test --test codegen_tests preg_replace_callback_throw` through Rust's test harness.
//!
//! Key details:
//! - Expected stdout was measured on PHP 8.5.9. PHP 8.2–8.4 share this behavior:
//!   a throw abandons the replacement, and an invalid pattern or `/e` yields null.
//! - Repeated catches must not accumulate helper-owned blocks. The 65536-byte heap
//!   is the size that previously died after a handful of caught throws.

use crate::support::*;

/// Compiles and runs one fixture at a chosen heap size, optionally with heap debugging.
fn run_preg_callback(source: &str, heap_size: usize, heap_debug: bool) -> ProgramOutput {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "elephc_preg_cb_{}_{:?}_{}",
        std::process::id(),
        std::thread::current().id(),
        id
    ));
    fs::create_dir_all(&dir).unwrap();
    let (user_asm, runtime_asm, requirements) =
        compile_source_to_asm_with_options(source, &dir, heap_size, false, heap_debug);
    let runtime_obj = runtime_obj_for_asm(&runtime_asm);
    let output = assemble_and_run_capture(
        &user_asm,
        &runtime_obj,
        &dir,
        &requirements,
        &default_link_paths(),
        &[],
    );
    let _ = fs::remove_dir_all(&dir);
    output
}

/// Reads the `live_blocks=` count from a `--heap-debug` leak summary.
fn live_blocks(stderr: &str) -> usize {
    let summary = stderr
        .lines()
        .find(|line| line.contains("leak summary"))
        .unwrap_or_else(|| panic!("no leak summary in:\n{stderr}"));
    if summary.contains("clean") {
        return 0;
    }
    summary
        .split("live_blocks=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("no live_blocks in: {summary}"))
}

/// Builds the fixture. Subjects and callback names depend on `$argc` so the calls
/// survive constant folding. `iterations` repeats both the first-match throw and
/// the grow-then-throw call.
fn fixture(iterations: usize) -> String {
    format!(
        r#"<?php
function keep(array $m): string {{ return "Z"; }}
function boom(array $m): string {{ throw new Exception("x"); }}
function wide(array $m): string {{ return str_repeat("b", 5000); }}
function grow(array $m): string {{
    global $phase;
    if ($phase == 0) {{
        $phase = 1;
        return str_repeat("b", 5000);
    }}
    throw new Exception("g");
}}
function shortq(array $m): string {{ return "Q"; }}
$one = $argc > 0 ? "a" : "b";
$two = $argc > 0 ? "aa" : "bb";
$xa = $argc > 0 ? "xa" : "ya";
$keep = $argc > 0 ? "keep" : "keep";
$boom = $argc > 0 ? "boom" : "boom";
$wide = $argc > 0 ? "wide" : "wide";
$grow = $argc > 0 ? "grow" : "grow";
$shortq = $argc > 0 ? "shortq" : "shortq";
echo preg_replace_callback("/a/", $keep, $one), "\n";
echo strlen(preg_replace_callback("/a/", $wide, $one)), "\n";
echo preg_replace_callback("/a/", $shortq, $xa), "\n";
$dots = "";
for ($i = 0; $i < {iterations}; $i++) {{
    try {{
        preg_replace_callback("/a/", $boom, $one);
        $dots .= "!";
    }} catch (Exception $e) {{
        $dots .= ".";
    }}
}}
echo $dots, "\n";
$w = "";
for ($i = 0; $i < {iterations}; $i++) {{
    $s = preg_replace_callback("/a/", $wide, $one);
    $w .= (strlen($s) == 5000 ? "W" : "w");
}}
echo $w, "\n";
$marks = "";
for ($i = 0; $i < {iterations}; $i++) {{
    $phase = 0;
    try {{
        preg_replace_callback("/a/", $grow, $two);
        $marks .= "!";
    }} catch (Exception $e) {{
        $marks .= "g";
    }}
}}
echo $marks, "\n";
echo (preg_replace_callback("/(/", $keep, "subject") ?? "null"), "\n";
echo (preg_replace_callback("/a/e", $keep, $one) ?? "bad"), "\n";
$kept = null;
try {{
    preg_replace_callback("/(a)/", function($m) use (&$kept) {{
        $kept = $m[0];
        throw new Exception("k");
    }}, $one);
}} catch (Exception $e) {{
    echo $kept, "\n";
}}
$kept_arr = ["z"];
try {{
    preg_replace_callback("/(a)/", function($m) use (&$kept_arr) {{
        $kept_arr = $m;
        throw new Exception("k");
    }}, $one);
}} catch (Exception $e) {{
    echo $kept_arr[0], "\n";
}}
"#
    )
}

/// Stdout measured on PHP 8.5.9 for the same fixture.
fn expected(iterations: usize) -> String {
    format!(
        "Z\n5000\nxQ\n{}\n{}\n{}\nnull\nbad\na\na\n",
        ".".repeat(iterations),
        "W".repeat(iterations),
        "g".repeat(iterations),
    )
}

/// Caught throws, including throws after the output block grows, must match PHP
/// and must not leak the helper's subject or output blocks as the loop repeats.
#[test]
fn test_preg_replace_callback_throw_releases_private_buffers() {
    assert_throw_cleanup_does_not_scale(true);
}

/// The same release holds with the EIR optimizer forced off. The runtime helper
/// is not rewritten by that pass, but the caller's fresh-result cleanup is.
#[test]
fn test_preg_replace_callback_throw_releases_private_buffers_without_ir_opt() {
    without_ir_opt(|| assert_throw_cleanup_does_not_scale(false));
}

/// Runs the fixture once and twenty times. Live blocks must not grow with the
/// iteration count. The twenty-iteration run also has to finish in a 64 KiB heap.
fn assert_throw_cleanup_does_not_scale(check_small_heap: bool) {
    let once = run_preg_callback(&fixture(1), 8_388_608, true);
    assert!(
        once.success,
        "one iteration failed\nstdout:\n{}\nstderr:\n{}",
        once.stdout, once.stderr
    );
    assert_eq!(once.stdout, expected(1), "stderr:\n{}", once.stderr);

    let many = run_preg_callback(&fixture(20), 8_388_608, true);
    assert!(
        many.success,
        "twenty iterations failed\nstdout:\n{}\nstderr:\n{}",
        many.stdout, many.stderr
    );
    assert_eq!(many.stdout, expected(20), "stderr:\n{}", many.stderr);
    let (once_blocks, many_blocks) = (live_blocks(&once.stderr), live_blocks(&many.stderr));
    assert_eq!(
        once_blocks, many_blocks,
        "live blocks scale with caught throws, so a helper buffer is leaking:\n\
         1 iteration -> {once_blocks}\n20 iterations -> {many_blocks}\n\n{}",
        many.stderr
    );

    if check_small_heap {
        let small = run_preg_callback(&fixture(20), 65_536, true);
        assert!(
            small.success,
            "65536-byte heap exhausted\nstdout:\n{}\nstderr:\n{}",
            small.stdout, small.stderr
        );
        assert_eq!(small.stdout, expected(20), "stderr:\n{}", small.stderr);
    }
}
