//! Purpose:
//! Integration tests for the SOURCE LOCATION an exception raised inside a builtin class reports.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - php reports the CALL SITE for an exception an internal method throws, because the `new`
//!   lives in php-src and has no php line of its own. elephc's builtin classes are compiled from
//!   SYNTHESIZED bodies, whose `new` carries no span either — so `getLine()` answered `0` and the
//!   uncaught report dropped both its ` in FILE:LINE` suffix and its `  thrown in` tail.
//! - MEASURED on `php -n` 8.5.6: `new DirectoryIterator("nope")` reports the line of the `new`,
//!   and `(new SplFileInfo("nope"))->getSize()` the line of the `getSize()` call.
//! - Only calls into a class whose bodies are SYNTHETIC publish the line, so an ordinary method
//!   call on a user class costs nothing.

use crate::support::*;

/// Verifies a caught SPL exception carries the line of the call that raised it.
#[test]
fn a_caught_spl_exception_reports_the_call_site() {
    let out = compile_and_run_capture(
        r#"<?php
try { new DirectoryIterator("nope"); }
catch (Throwable $e) { echo "ctor ", $e->getLine(), " ", basename($e->getFile()), "\n"; }

$info = new SplFileInfo("nope.txt");
try { $info->getSize(); }
catch (Throwable $e) { echo "getter ", $e->getLine(), "\n"; }

try { new SplFileObject("nope.txt"); }
catch (Throwable $e) { echo "open ", $e->getLine(), "\n"; }

$own = new RuntimeException("mine");
echo "user ", $own->getLine(), "\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "ctor 2 test.php\n\
         getter 6\n\
         open 9\n\
         user 12\n"
    );
}

/// Verifies the uncaught report carries the same location.
///
/// It reads the line out of the throwable's payload, so the two answers come from one value —
/// but the report also prints the ` thrown in` tail from it, which was missing entirely.
#[test]
fn an_uncaught_spl_exception_names_its_location() {
    let out = compile_and_run_capture(
        r#"<?php
$info = new SplFileInfo("nope.txt");
$info->getSize();
"#,
    );
    assert_eq!(out.exit_code, Some(255));
    assert!(
        out.located_diagnostics.contains("stat failed for nope.txt")
            && out.located_diagnostics.contains(":3"),
        "expected the call's line in the report, got {}",
        out.located_diagnostics
    );
}
