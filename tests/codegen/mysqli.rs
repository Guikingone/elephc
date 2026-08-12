//! Purpose:
//! Offline mysqli prelude tests that do not need a live MySQL server: surface
//! injection, no PDO class leak, and (in later tasks) connect-failure paths and
//! escaping that need no live row.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the test harness.
//!
//! Key details:
//! - Live query/fetch fixtures live in `mysqli_mysql.rs` and are `#[ignore]`d
//!   (they need `ELEPHC_MY_DSN`, same as `pdo_mysql.rs`).
//! - The class-leak assertions are the point: a mysqli-only program must declare
//!   `mysqli` but not `PDO`, and a PDO-only program must not grow `mysqli`.

use crate::support::*;

/// A mysqli-only program declares the mysqli surface (class, procedural alias,
/// constants) and does NOT leak the PDO classes. The `new mysqli()` is the
/// detection trigger: capability probes are string literals and deliberately
/// never inject (same rule as PDO; `--with-mysqli` forces probe-only programs).
#[test]
fn test_mysqli_class_exists_and_does_not_leak_pdo() {
    let out = compile_and_run(
        r#"<?php
$db = new mysqli();
echo class_exists('mysqli') ? '1' : '0';
echo class_exists('PDO') ? '1' : '0';
echo function_exists('mysqli_connect') ? '1' : '0';
echo defined('MYSQLI_ASSOC') ? '1' : '0';
"#,
    );
    assert_eq!(out, "1011");
}

/// A PDO-only program does not grow the mysqli surface.
#[test]
fn test_pdo_program_does_not_grow_mysqli() {
    let out = compile_and_run(
        r#"<?php
$db = new PDO("sqlite::memory:");
echo class_exists('mysqli') ? '1' : '0';
echo class_exists('PDO') ? '1' : '0';
"#,
    );
    assert_eq!(out, "01");
}

/// The mysqli exception hierarchy is mysqli's own: `mysqli_sql_exception`
/// extends `RuntimeException`, and the locked `MYSQLI_*` constants carry
/// php-src's values.
#[test]
fn test_mysqli_exception_and_constants() {
    let out = compile_and_run(
        r#"<?php
$e = new mysqli_sql_exception("boom");
echo $e instanceof RuntimeException ? 'rt' : 'no';
echo '|', MYSQLI_ASSOC, MYSQLI_NUM, MYSQLI_BOTH;
echo '|', MYSQLI_REPORT_OFF, MYSQLI_REPORT_ERROR, MYSQLI_REPORT_STRICT;
echo '|', MYSQLI_CLIENT_SSL;
echo '|', $e->sqlstate;
"#,
    );
    assert_eq!(out, "rt|123|012|2048|00000");
}
