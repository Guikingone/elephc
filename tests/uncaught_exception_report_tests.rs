//! Purpose:
//! End-to-end tests for the fatal diagnostic an UNCAUGHT `throw` prints, and the process exit
//! status that follows it.
//!
//! Before this, every uncaught exception — of any class, with any message — printed one fixed
//! 32-byte string and exited `1`:
//!
//! ```text
//! Fatal error: uncaught exception
//! ```
//!
//! The class and the message were both discarded, so a production crash told you nothing about
//! what had been thrown. `_exc_value` already held the Throwable at that point
//! (`lower_throw_value` publishes it immediately before `__rt_throw_current`); the uncaught arm
//! simply never read it.
//!
//! Reference PHP 8.5.6, measured with `php -d xdebug.mode=off`:
//!
//! ```text
//! Fatal error: Uncaught RuntimeException: boom detail in /path/e.php:2
//! Fatal error: Uncaught MyErr: custom text in /path/m2.php:3
//! Fatal error: Uncaught Exception in /path/m1.php:2        <- EMPTY message: no colon
//! ```
//!
//! elephc now emits the class, the message, the ` in <file>:<line>` suffix, and exits `255` like
//! PHP.
//!
//! Called from:
//! - `cargo test --test uncaught_exception_report_tests` through Rust's test harness.
//!
//! Key details:
//! - THE LOCATION IS THE CONSTRUCTION SITE, NOT THE THROW SITE, because that is what PHP reports:
//!   a `new RuntimeException(...)` on line 2 stored in a variable and thrown on line 5 prints
//!   line 2 in both engines. The line is stamped into the Throwable payload when it is allocated,
//!   which is precisely the `new`; `separates_construction_from_throw_site` is the test that would
//!   fail if the line were taken from the `throw` instead, and it is the only one of these tests
//!   where the two differ.
//! - THE STACK TRACE IS STILL ABSENT, so the tests assert a PREFIX up to the location rather than
//!   full equality. `getTrace()`/`getTraceAsString()` remain synthetic in `lower_inst.rs` — an
//!   empty array and an empty string — because elephc keeps no call stack. Asserting equality
//!   would freeze that gap as intended behaviour.
//! - A Throwable with NO user `new` behind it — a `DivisionByZeroError` raised by `intdiv($n, 0)`
//!   — carries line `0`, and the suffix is omitted entirely rather than printed as `:0`.
//!   `synthesized_error_omits_the_location_and_still_exits_255` pins both halves of that: the
//!   omission, and the exit status, which travels a SEPARATE code path
//!   (`codegen::lower_inst::exceptions`) that never reaches `__rt_report_uncaught_exception`.
//! - The EXIT STATUS is asserted separately from the text. It moved from `1` to `255`; a script
//!   that branched on `$?` saw the wrong value before, and that is invisible in stdout/stderr.
//! - The empty-message case is the one that would silently pass with a naive implementation:
//!   writing `": "` unconditionally still looks right in every other test.
//! - A USER SUBCLASS is covered because the class name comes from the runtime
//!   `_class_name_entries` table rather than a compile-time literal, so a name that only exists in
//!   user code is the case that proves the table lookup works.
//! - Host-target only in execution; the emitter change is pinned on both architectures by the unit
//!   tests in `src/codegen_support/runtime/exceptions/uncaught_report.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

/// Creates an isolated temp dir unique across parallel test threads/processes.
fn make_test_dir(prefix: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::SeqCst);
    let tid = std::thread::current().id();
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("{}_{}_{:?}_{}", prefix, pid, tid, id));
    fs::create_dir_all(&dir).unwrap();
    dir
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

/// Compiles `source` and returns the executable path.
fn compile(dir: &Path, source: &str, stem: &str) -> PathBuf {
    let php = dir.join(format!("{}.php", stem));
    fs::write(&php, source).unwrap();
    let output = Command::new(elephc_bin())
        .env("XDG_CACHE_HOME", dir.join("cache-root"))
        .current_dir(dir)
        .arg(&php)
        .output()
        .expect("failed to spawn elephc");
    assert!(
        output.status.success(),
        "elephc compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    dir.join(stem)
}

/// Compiles and runs `source`, returning `(stderr, exit_code)`.
///
/// The temp directory is NOT removed before returning, because the location suffix names the
/// script by its canonical path and the assertions have to rebuild that path to compare.
fn run_uncaught(prefix: &str, source: &str) -> (String, Option<i32>, PathBuf) {
    let dir = make_test_dir(prefix);
    let bin = compile(&dir, source, prefix);
    let output = Command::new(&bin).output().expect("failed to run compiled binary");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code();
    (stderr, code, dir)
}

/// Returns the ` in <file>:<line>` suffix reference PHP prints for `script` in `dir`.
///
/// Built from the CANONICALIZED directory, the same normalization
/// `crate::magic_constants::file_pass` applies when it bakes `__FILE__` and `_script_source_file`.
/// On macOS `std::env::temp_dir()` hands back a `/var/...` symlink to `/private/var/...`, so a
/// naive `dir.join(...)` would produce a path that never matches.
fn location_suffix(dir: &Path, script: &str, line: u32) -> String {
    let canonical = dir
        .canonicalize()
        .expect("failed to canonicalize the test directory");
    format!(" in {}:{}", canonical.join(script).display(), line)
}

/// A built-in exception subclass reports its own class name, its message and its location.
#[test]
fn uncaught_builtin_subclass_reports_class_and_message() {
    let prefix = "elephc_uncaught_builtin";
    let (stderr, code, dir) = run_uncaught(
        prefix,
        "<?php\nthrow new RuntimeException(\"boom detail\");\n",
    );
    let expected = format!(
        "Fatal error: Uncaught RuntimeException: boom detail{}",
        location_suffix(&dir, &format!("{prefix}.php"), 2)
    );

    assert!(
        stderr.starts_with(&expected),
        "stderr must name the class, the message and the location;\n  expected prefix: {expected:?}\n  got:             {stderr:?}"
    );
    assert_eq!(code, Some(255), "PHP exits 255 for an uncaught exception");
    let _ = fs::remove_dir_all(&dir);
}

/// A USER-DECLARED subclass is named from the runtime class table, not a compile-time literal.
#[test]
fn uncaught_user_subclass_reports_its_own_name() {
    let prefix = "elephc_uncaught_user";
    let (stderr, code, dir) = run_uncaught(
        prefix,
        "<?php\nclass MyErr extends Exception {}\nthrow new MyErr(\"custom text\");\n",
    );
    let expected = format!(
        "Fatal error: Uncaught MyErr: custom text{}",
        location_suffix(&dir, &format!("{prefix}.php"), 3)
    );

    assert!(
        stderr.starts_with(&expected),
        "a user subclass must be named from the runtime class table;\n  expected prefix: {expected:?}\n  got:             {stderr:?}"
    );
    assert_eq!(code, Some(255));
    let _ = fs::remove_dir_all(&dir);
}

/// An EMPTY message drops the `": "` separator but KEEPS the location, exactly as PHP does.
///
/// This is the case a naive implementation gets wrong while still looking correct everywhere
/// else, because writing the separator unconditionally is invisible whenever a message follows.
/// It is also where the location is easiest to lose: the empty-message branch skips forward, and
/// skipping one label too far would drop the suffix along with the separator.
#[test]
fn uncaught_empty_message_omits_the_separator() {
    let prefix = "elephc_uncaught_empty";
    let (stderr, code, dir) = run_uncaught(prefix, "<?php\nthrow new Exception(\"\");\n");
    let expected = format!(
        "Fatal error: Uncaught Exception{}\n",
        location_suffix(&dir, &format!("{prefix}.php"), 2)
    );

    assert_eq!(
        stderr, expected,
        "an empty message must not be preceded by a colon, yet must still carry the location"
    );
    assert_eq!(code, Some(255));
    let _ = fs::remove_dir_all(&dir);
}

/// The reported line is where the exception was CONSTRUCTED, not where it was thrown.
///
/// This is the one test whose expected value would change if the line came from the `throw`
/// terminator instead of the `new`: the two sit on different lines, and in different functions.
/// Reference PHP 8.5.6 prints line 2 here.
#[test]
fn uncaught_reports_the_construction_site_not_the_throw_site() {
    let prefix = "elephc_uncaught_construction_site";
    let (stderr, code, dir) = run_uncaught(
        prefix,
        "<?php\nfunction make() { return new LogicException(\"made here\"); }\n$e = make();\necho \"still running\\n\";\nthrow $e;\n",
    );
    let expected = format!(
        "Fatal error: Uncaught LogicException: made here{}",
        location_suffix(&dir, &format!("{prefix}.php"), 2)
    );

    assert!(
        stderr.starts_with(&expected),
        "the location must be the `new` on line 2, not the `throw` on line 5;\n  expected prefix: {expected:?}\n  got:             {stderr:?}"
    );
    assert_eq!(code, Some(255));
    let _ = fs::remove_dir_all(&dir);
}

/// A Throwable with no user `new` behind it omits the location and STILL exits 255.
///
/// `intdiv($n, 0)` raises a `DivisionByZeroError` synthesized by a codegen guard, which writes its
/// own fatal diagnostic in `codegen::lower_inst::exceptions` and never reaches
/// `__rt_report_uncaught_exception`. Two distinct regressions hide here: printing `:0` for a line
/// the compiler does not know, and letting the two uncaught paths disagree on `$?` — that second
/// path exited `1` while `throw new ...` exited `255`, so a script branching on the status saw a
/// different answer depending on which kind of exception escaped.
#[test]
fn synthesized_error_omits_the_location_and_still_exits_255() {
    let prefix = "elephc_uncaught_synthesized";
    let (stderr, code, dir) = run_uncaught(
        prefix,
        "<?php\n$n = 1;\n$d = 0;\necho intdiv($n, $d);\n",
    );

    assert_eq!(
        stderr, "Fatal error: Uncaught DivisionByZeroError: Division by zero\n",
        "an error with no construction site must omit the location, never print `:0`"
    );
    assert_eq!(
        code,
        Some(255),
        "both uncaught paths must leave the same exit status"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// An exception that IS caught prints nothing and exits cleanly — the report is uncaught-only.
///
/// Without this the reporting path could fire on every throw and still pass the tests above.
#[test]
fn caught_exception_prints_no_fatal_report() {
    let dir = make_test_dir("elephc_uncaught_caught");
    let bin = compile(
        &dir,
        "<?php\ntry {\n    throw new RuntimeException(\"handled\");\n} catch (RuntimeException $e) {\n    echo \"caught:\", $e->getMessage(), \"\\n\";\n}\n",
        "elephc_uncaught_caught",
    );
    let output = Command::new(&bin).output().expect("failed to run compiled binary");

    assert!(output.status.success(), "a caught exception must exit cleanly");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "caught:handled\n",
        "the catch body must run normally"
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("Fatal error"),
        "a caught exception must print no fatal report"
    );
    let _ = fs::remove_dir_all(&dir);
}
