//! Purpose:
//! Pins that a `return` inside an included file ends THAT FILE — wherever it sits in the file's
//! control flow — and never the caller, with the include's value and `finally` blocks intact.
//!
//! Called from:
//! - `cargo test --test include_scoped_return_tests` through Rust's test harness.
//!
//! Key details:
//! - elephc inlines an include's body into the caller, so a `return` in it would return from the
//!   CALLER. The resolver confines it: a nested one becomes `break n` out of a
//!   `do { … } while (false)` wrapping the body (`resolver::engine_includes::confine_nested_returns`).
//! - Every expected output here was MEASURED on reference PHP 8.5 with the same files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

/// Creates an isolated temp dir unique across parallel test threads/processes.
fn make_test_dir(prefix: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{:?}_{}",
        prefix,
        std::process::id(),
        std::thread::current().id(),
        id
    ));
    fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

/// Resolves the elephc CLI binary path (cargo env var, fallback next to the test binary).
fn elephc_bin() -> String {
    std::env::var("CARGO_BIN_EXE_elephc").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().expect("failed to resolve current test binary");
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.join("elephc").to_string_lossy().into_owned()
    })
}

/// Writes `files` into a fresh directory, compiles `main.php`, runs it and returns its stdout.
fn run_program(prefix: &str, files: &[(&str, &str)]) -> String {
    let dir = make_test_dir(prefix);
    for (name, source) in files {
        fs::write(dir.join(name), source).unwrap();
    }
    let output = Command::new(elephc_bin())
        .env("XDG_CACHE_HOME", dir.join("cache-root"))
        .current_dir(&dir)
        .arg(dir.join("main.php"))
        .output()
        .expect("failed to spawn elephc");
    assert!(
        output.status.success(),
        "compilation failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    run(&dir.join("main"))
}

/// Runs a compiled binary, asserting a clean exit, and returns its stdout.
fn run(bin: &Path) -> String {
    let output = Command::new(bin).output().expect("failed to run binary");
    assert!(
        output.status.success(),
        "binary failed ({:?}):\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The classic include guard — `if (...) { return; }` — ends the file, not the caller.
#[test]
fn an_include_guard_return_ends_only_the_included_file() {
    let out = run_program(
        "inc_ret_guard",
        &[
            (
                "guard.php",
                "<?php\necho \"G1 \";\nif (!defined('NEVER_DEFINED')) { return; }\necho \"G-NEVER \";\n",
            ),
            (
                "main.php",
                "<?php\necho \"start \";\ninclude __DIR__ . '/guard.php';\necho \"after\\n\";\n",
            ),
        ],
    );
    assert_eq!(out, "start G1 after\n");
}

/// A `return` inside a loop and a `switch` leaves every one of them, and only the file.
#[test]
fn a_return_nested_in_loops_and_a_switch_ends_only_the_file() {
    let out = run_program(
        "inc_ret_loops",
        &[
            (
                "loops.php",
                "<?php\nforeach ([1, 2, 3] as $i) {\n    while (true) {\n        switch ($i) {\n            case 2:\n                echo \"stop \";\n                return;\n        }\n        echo \"L$i \";\n        break;\n    }\n}\necho \"NEVER \";\n",
            ),
            (
                "main.php",
                "<?php\ninclude __DIR__ . '/loops.php';\necho \"after\\n\";\n",
            ),
        ],
    );
    assert_eq!(out, "L1 stop after\n");
}

/// A `return` inside `try` still runs the `finally`, then ends the file.
#[test]
fn a_return_in_try_runs_the_finally_and_ends_only_the_file() {
    let out = run_program(
        "inc_ret_finally",
        &[
            (
                "finally.php",
                "<?php\ntry {\n    echo \"try \";\n    if (true) { return; }\n} finally {\n    echo \"finally \";\n}\necho \"NEVER \";\n",
            ),
            (
                "main.php",
                "<?php\ninclude __DIR__ . '/finally.php';\necho \"after\\n\";\n",
            ),
        ],
    );
    assert_eq!(out, "try finally after\n");
}

/// A function declared in the included file keeps its own `return`.
#[test]
fn a_function_in_the_included_file_keeps_its_own_return() {
    let out = run_program(
        "inc_ret_function",
        &[
            (
                "func.php",
                "<?php\nfunction inner_helper() { if (true) { return \"fn-ret\"; } return \"fn-late\"; }\necho inner_helper(), \" \";\nif (true) { echo \"cond \"; }\n",
            ),
            (
                "main.php",
                "<?php\ninclude __DIR__ . '/func.php';\necho \"after\\n\";\n",
            ),
        ],
    );
    assert_eq!(out, "fn-ret cond after\n");
}

/// A nested include's `return` ends the nested file; the outer file's own ends the outer one.
#[test]
fn nested_includes_each_confine_their_own_return() {
    let out = run_program(
        "inc_ret_nested",
        &[
            ("guard.php", "<?php\necho \"G \";\nif (true) { return; }\necho \"G-NEVER \";\n"),
            (
                "outer.php",
                "<?php\necho \"O1 \";\ninclude __DIR__ . '/guard.php';\necho \"O2 \";\nif (true) { return; }\necho \"O-NEVER \";\n",
            ),
            (
                "main.php",
                "<?php\ninclude __DIR__ . '/outer.php';\necho \"after\\n\";\n",
            ),
        ],
    );
    assert_eq!(out, "O1 G O2 after\n");
}

/// The VALUE of an include is whichever `return` ran — conditional, in a loop, or last — and a
/// bare `return;` makes it NULL, not the default `1`.
#[test]
fn the_include_value_comes_from_whichever_return_ran() {
    let out = run_program(
        "inc_ret_value",
        &[
            (
                "value.php",
                "<?php\nif ($flag) { return \"early\"; }\nforeach ([1] as $x) { if ($flag2) { return \"in-loop\"; } }\nreturn \"late\";\n",
            ),
            ("bare.php", "<?php\nif (true) { return; }\n"),
            (
                "main.php",
                "<?php\n$flag = true; $flag2 = false; $v = include __DIR__ . '/value.php'; var_dump($v);\n$flag = false; $flag2 = true; $v = include __DIR__ . '/value.php'; var_dump($v);\n$flag = false; $flag2 = false; $v = include __DIR__ . '/value.php'; var_dump($v);\n$v = include __DIR__ . '/bare.php'; var_dump($v);\n",
            ),
        ],
    );
    assert_eq!(
        out,
        "string(5) \"early\"\nstring(7) \"in-loop\"\nstring(4) \"late\"\nNULL\n"
    );
}
