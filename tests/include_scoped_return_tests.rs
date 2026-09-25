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

/// Compiles `main.php` among `files` with `args`, returning the compiler's output unchecked.
fn compile_with(
    prefix: &str,
    files: &[(&str, &str)],
    args: &[&str],
) -> (PathBuf, std::process::Output) {
    let dir = make_test_dir(prefix);
    for (name, source) in files {
        fs::write(dir.join(name), source).unwrap();
    }
    let output = Command::new(elephc_bin())
        .env("XDG_CACHE_HOME", dir.join("cache-root"))
        .current_dir(&dir)
        .args(args)
        .arg(dir.join("main.php"))
        .output()
        .expect("failed to spawn elephc");
    (dir, output)
}

/// Returns the `live_blocks=` count of a `--heap-debug` binary's leak summary.
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

/// A `return` inside a `finally` ends the included file too — in both include forms, and
/// overriding a `return` the `try` already made.
///
/// Round 13's rewrite turned it into a `break` out of the `finally`, which the checker refuses
/// as "Cannot jump out of a finally block": these programs stopped compiling. Before that, the
/// `return` ended the whole program. MEASURED on reference PHP 8.5.10: `try after:7`,
/// `try after`, `after:2`, `t1 after:5`.
#[test]
fn a_return_in_finally_ends_only_the_file() {
    let lib = "<?php\ntry { echo \"try \"; }\nfinally { return 7; }\n";
    let value = run_program(
        "inc_ret_fin_value",
        &[
            ("lib.php", lib),
            (
                "main.php",
                "<?php\n$v = include __DIR__ . '/lib.php';\necho \"after:\", $v, \"\\n\";\n",
            ),
        ],
    );
    assert_eq!(value, "try after:7\n");

    let statement = run_program(
        "inc_ret_fin_stmt",
        &[
            ("lib.php", lib),
            ("main.php", "<?php\ninclude __DIR__ . '/lib.php';\necho \"after\\n\";\n"),
        ],
    );
    assert_eq!(statement, "try after\n");

    let overrides = run_program(
        "inc_ret_fin_override",
        &[
            ("lib.php", "<?php\ntry { return 1; }\nfinally { return 2; }\n"),
            (
                "main.php",
                "<?php\n$v = include __DIR__ . '/lib.php';\necho \"after:\", $v, \"\\n\";\n",
            ),
        ],
    );
    assert_eq!(overrides, "after:2\n");

    let in_loop = run_program(
        "inc_ret_fin_loop",
        &[
            (
                "lib.php",
                "<?php\nforeach ([1, 2] as $i) {\n  try { echo \"t$i \"; }\n  finally { if ($i === 1) { return 5; } }\n}\necho \"NEVER \";\n",
            ),
            (
                "main.php",
                "<?php\n$v = include __DIR__ . '/lib.php';\necho \"after:\", $v, \"\\n\";\n",
            ),
        ],
    );
    assert_eq!(in_loop, "t1 after:5\n");
}

/// A `return` in `finally` DISCARDS the exception in flight, and the program continues after
/// the include; one that does not return lets it propagate to the caller's `catch`.
///
/// The optimizer read `try { throw … }` as a function exit without noticing that the `finally`
/// could break out, and pruned everything after the include: elephc printed nothing. MEASURED on
/// reference PHP 8.5.10: `after:9`, then `caught:boom`. That the discarded exception is also
/// RELEASED is `a_discarded_exception_is_released`.
#[test]
fn a_return_in_finally_discards_the_exception_in_flight() {
    let discarded = run_program(
        "inc_ret_fin_discard",
        &[
            (
                "lib.php",
                "<?php\ntry { throw new Exception(\"boom\"); }\nfinally { return 9; }\n",
            ),
            (
                "main.php",
                "<?php\n$v = include __DIR__ . '/lib.php';\necho \"after:\", $v, \"\\n\";\n",
            ),
        ],
    );
    assert_eq!(discarded, "after:9\n");

    let propagated = run_program(
        "inc_ret_fin_propagate",
        &[
            (
                "lib.php",
                "<?php\ntry { throw new Exception(\"boom\"); }\nfinally { if ($argc > 5) { return 9; } }\n",
            ),
            (
                "main.php",
                "<?php\ntry { $v = include __DIR__ . '/lib.php'; echo \"NEVER\\n\"; }\ncatch (Exception $e) { echo \"caught:\", $e->getMessage(), \"\\n\"; }\n",
            ),
        ],
    );
    assert_eq!(propagated, "caught:boom\n");
}

/// The exception a `return` in `finally` discards is RELEASED — in an included file and in a
/// function alike.
///
/// The finalizer used to peek at the in-flight exception and rely on rethrowing it; a `return`
/// skipped the rethrow and the exception was never released, one leaked object per call. It now
/// takes the exception first, as `catch (Throwable $t) { …; throw $t; }` would. Measured as a
/// SLOPE — 20 against 200 iterations — so a live value at exit cannot pass for a leak.
#[test]
fn a_discarded_exception_is_released() {
    for (label, main) in [
        (
            "include",
            "<?php\nfor ($i = 0; $i < N; $i++) { $v = include __DIR__ . '/lib.php'; }\necho $v, \"\\n\";\n",
        ),
        (
            "function",
            "<?php\nfunction f() { try { throw new Exception(\"boom\"); } finally { return 9; } }\nfor ($i = 0; $i < N; $i++) { $v = f(); }\necho $v, \"\\n\";\n",
        ),
    ] {
        let mut counts = Vec::new();
        for iterations in [20, 200] {
            let source = main.replace('N', &iterations.to_string());
            let (dir, output) = compile_with(
                &format!("inc_ret_fin_leak_{label}"),
                &[
                    (
                        "lib.php",
                        "<?php\ntry { throw new Exception(\"boom\"); }\nfinally { return 9; }\n",
                    ),
                    ("main.php", &source),
                ],
                &["--heap-debug"],
            );
            assert!(
                output.status.success(),
                "compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let run = Command::new(dir.join("main")).output().expect("failed to run binary");
            assert_eq!(String::from_utf8_lossy(&run.stdout), "9\n", "{label}");
            counts.push(live_blocks(&String::from_utf8_lossy(&run.stderr)));
        }
        assert_eq!(
            counts[0], counts[1],
            "{label}: live blocks grow with the iteration count, so the discarded exception leaks"
        );
    }
}

/// Only the `break` a `return` becomes may leave a `finally`. One written by hand is still
/// refused, in the entry script and in an included file alike, as reference refuses it with
/// "jump out of a finally block is disallowed".
#[test]
fn a_handwritten_break_out_of_finally_is_still_refused() {
    let body = "for (;;) { try { echo \"a\"; } finally { break; } }\n";
    let entry = format!("<?php\n{body}echo \"b\\n\";\n");
    let lib = format!("<?php\n{body}");
    let cases: [(&str, Vec<(&str, &str)>); 2] = [
        ("main", vec![("main.php", entry.as_str())]),
        (
            "include",
            vec![
                ("lib.php", lib.as_str()),
                ("main.php", "<?php\ninclude __DIR__ . '/lib.php';\necho \"b\\n\";\n"),
            ],
        ),
    ];
    for (label, files) in cases {
        let (_, output) = compile_with(&format!("inc_fin_break_{label}"), &files, &[]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{label}: a hand-written break out of finally compiled");
        assert!(
            stderr.contains("Cannot jump out of a finally block"),
            "{label}: {stderr}"
        );
    }
}

/// An exception that passes THROUGH a `finally` able to return — the `return` not taken — leaves
/// the frame intact and reaches the caller's `catch`, and is released once caught.
///
/// Such a finalizer takes the in-flight exception and rethrows it. Rethrowing with `throw $temp`
/// handed the temp's reference on without clearing the slot, so unwinding the function released
/// it under the in-flight exception: MEASURED, the caller's `catch` missed and the program died on
/// "Uncaught" with a garbage line. The frame must actually unwind to show it — a `catch` in the
/// same frame hides it — hence the function, in both forms. MEASURED on reference PHP 8.5.10:
/// every exception is caught, `caught:20` for 20 calls.
#[test]
fn an_exception_passing_through_a_returning_finally_reaches_the_caller() {
    let lib = "<?php\ntry { throw new Exception(\"boom\"); }\nfinally { if ($x > 5) { return 1; } }\n";
    for (label, function) in [
        (
            "function",
            "function f($x) { try { throw new Exception(\"boom\"); } finally { if ($x > 5) { return 1; } } }\n",
        ),
        ("include", "function f($x) { return include __DIR__ . '/lib.php'; }\n"),
    ] {
        let mut counts = Vec::new();
        for iterations in [20, 200] {
            let main = format!(
                "<?php\n{function}$seen = 0;\nfor ($i = 0; $i < {iterations}; $i++) {{\n  try {{ f($argc); }} catch (Exception $e) {{ if ($e->getMessage() === 'boom') {{ $seen++; }} }}\n}}\necho \"caught:\", $seen, \"\\n\";\n"
            );
            let (dir, output) = compile_with(
                &format!("inc_fin_passthrough_{label}"),
                &[("lib.php", lib), ("main.php", &main)],
                &["--heap-debug"],
            );
            assert!(
                output.status.success(),
                "compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let run = Command::new(dir.join("main")).output().expect("failed to run binary");
            assert!(
                run.status.success(),
                "{label}: the binary died:\n{}",
                String::from_utf8_lossy(&run.stdout)
            );
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                format!("caught:{iterations}\n"),
                "{label}"
            );
            counts.push(live_blocks(&String::from_utf8_lossy(&run.stderr)));
        }
        assert_eq!(counts[0], counts[1], "{label}: the rethrown exception leaks");
    }
}

/// The exception's DESTRUCTOR runs when reference runs it — the sharpest oracle for who owns the
/// reference a `finally` able to return takes and rethrows.
///
/// A second release of the rethrown exception ran `__destruct` during the unwind, before the
/// caller's `catch`; a missing one never ran it at all. MEASURED on reference PHP 8.5.10, with
/// `--heap-debug` clean here: `caught:boom`, `end`, then `[destruct]` at shutdown when the
/// exception is caught (across a frame and within one); `[destruct]` BEFORE the returned value
/// is printed when the `finally` discards it — `main` printed `got:7` and never destructed it.
#[test]
fn a_rethrown_or_discarded_exception_is_destructed_on_time() {
    let class = "class E extends Exception { public function __destruct() { echo \"[destruct]\"; } }\n";
    for (label, body, expected) in [
        (
            "cross_frame",
            "function g($x) { try { throw new E(\"boom\"); } finally { if ($x) { return; } } }\ntry { g(false); } catch (E $e) { echo \"caught:\", $e->getMessage(), \"\\n\"; }\necho \"end\\n\";\n",
            "caught:boom\nend\n[destruct]",
        ),
        (
            "same_frame",
            "try { try { throw new E(\"boom\"); } finally { if ($argc > 5) { return; } } }\ncatch (E $e) { echo \"caught:\", $e->getMessage(), \"\\n\"; }\necho \"end\\n\";\n",
            "caught:boom\nend\n[destruct]",
        ),
        (
            "discarded",
            "function g($x) { try { throw new E(\"boom\"); } finally { if ($x) { return 7; } } }\necho \"got:\", g(true), \"\\n\";\necho \"end\\n\";\n",
            "got:[destruct]7\nend\n",
        ),
    ] {
        let main = format!("<?php\n{class}{body}");
        let (dir, output) = compile_with(
            &format!("fin_destruct_{label}"),
            &[("main.php", &main)],
            &["--heap-debug"],
        );
        assert!(
            output.status.success(),
            "compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let run = Command::new(dir.join("main")).output().expect("failed to run binary");
        assert!(run.status.success(), "{label}: the binary died");
        assert_eq!(String::from_utf8_lossy(&run.stdout), expected, "{label}");
        assert_eq!(live_blocks(&String::from_utf8_lossy(&run.stderr)), 0, "{label}: leaked");
    }
}
