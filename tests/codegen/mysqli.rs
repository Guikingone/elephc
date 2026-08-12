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

/// A failed constructor connect under `MYSQLI_REPORT_OFF` leaves a usable
/// object with `connect_errno` / `connect_error` populated (no exception, no
/// PDO types). Port 1 on localhost refuses immediately, so this needs no
/// server and cannot hang.
#[test]
fn test_mysqli_connect_failure_sets_connect_errno() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
$db = @new mysqli("127.0.0.1", "nope", "nope", "nope", 1);
echo $db->connect_errno > 0 ? "err" : "ok";
echo "|";
echo $db->connect_error !== "" ? "msg" : "empty";
"#,
    );
    assert_eq!(out, "err|msg");
}

/// Procedural `mysqli_connect()` returns `false` on failure under REPORT_OFF,
/// and the no-argument `mysqli_connect_errno()` / `mysqli_connect_error()`
/// read the process-wide last-connect failure.
#[test]
fn test_mysqli_connect_procedural_failure_returns_false() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
$db = mysqli_connect("127.0.0.1", "nope", "nope", "nope", 1);
echo $db === false ? "F" : "obj";
echo "|", mysqli_connect_errno() > 0 ? "err" : "ok";
echo "|", mysqli_connect_error() !== null ? "msg" : "null";
"#,
    );
    assert_eq!(out, "F|err|msg");
}

/// Under `MYSQLI_REPORT_STRICT` a failed connect throws `mysqli_sql_exception`
/// — never `PDOException` — with the SQLSTATE on the public property.
#[test]
fn test_mysqli_connect_failure_strict_throws_mysqli_sql_exception() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT);
try {
    new mysqli("127.0.0.1", "nope", "nope", "nope", 1);
    echo "no-throw";
} catch (mysqli_sql_exception $e) {
    echo "caught|", strlen($e->getMessage()) > 0 ? "msg" : "empty";
    echo "|", $e->sqlstate !== "" ? "state" : "none";
}
"#,
    );
    assert_eq!(out, "caught|msg|state");
}

/// Operations on an unconnected handle fail loudly but silently-with-`false`
/// under REPORT_OFF, recording CR_SERVER_GONE_ERROR (2006) — and
/// `real_escape_string` still escapes with the default (backslash) rules.
#[test]
fn test_mysqli_unconnected_ops_and_offline_escape() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
$db = mysqli_init();
echo $db->ping() ? "T" : "F";
echo "|", $db->errno;
echo "|", $db->select_db("nope") ? "T" : "F";
echo "|", $db->real_escape_string("a'b\\c");
echo "|", $db->options(MYSQLI_OPT_CONNECT_TIMEOUT, 1) ? "T" : "F";
echo "|", $db->options(99, 1) ? "T" : "F";
"#,
    );
    assert_eq!(out, r"F|2006|F|a\'b\\c|T|F");
}

/// An empty query string throws `ValueError` (php-src's message), and a query
/// on an unconnected handle under REPORT_OFF returns `false` with `errno` set.
#[test]
fn test_mysqli_query_empty_string_and_unconnected() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
$db = mysqli_init();
try {
    $db->query("");
    echo "no";
} catch (ValueError $e) {
    echo "ve";
}
echo "|", $db->query("SELECT 1") === false ? "F" : "ok";
echo "|", $db->errno;
"#,
    );
    assert_eq!(out, "ve|F|2006");
}

/// Procedural aliases validate their link/result argument at runtime with a
/// `TypeError` naming the expected class (PHP's own behavior), so the classic
/// `mysqli_query(...)` → `mysqli_num_rows(...)` pipeline fails loudly on a
/// `false` result instead of reading garbage.
#[test]
fn test_mysqli_procedural_alias_type_errors() {
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
try {
    mysqli_num_rows(false);
    echo "no";
} catch (TypeError $e) {
    echo strpos($e->getMessage(), "mysqli_result") !== false ? "te-result" : "te-?";
}
try {
    mysqli_ping("not-a-link");
    echo "|no";
} catch (TypeError $e) {
    echo "|", strpos($e->getMessage(), "must be of type mysqli") !== false ? "te-link" : "te-?";
}
"#,
    );
    assert_eq!(out, "te-result|te-link");
}

/// `bind_param` validates offline: a type character outside `i`/`d`/`s`/`b`
/// throws `ValueError`; a types-vs-variables count mismatch reports and
/// returns `false` under REPORT_OFF; execute on an unprepared statement fails
/// with `errno` set.
#[test]
fn test_mysqli_stmt_bind_param_validation() {
    // The statement comes from a `mysqli_stmt|false`-typed helper, the same
    // union shape `mysqli::prepare` returns: statement method calls dispatch
    // dynamically on the union receiver (a concretely-typed receiver would
    // instead hit the checker's by-ref storage rule at compile time — loud,
    // and documented).
    let out = compile_and_run(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
function make_stmt(): mysqli_stmt|false {
    return new mysqli_stmt();
}
$stmt = make_stmt();
$v = 1;
$w = 2;
try {
    $stmt->bind_param("x", $v);
    echo "no";
} catch (ValueError $e) {
    echo "ve";
}
echo "|", $stmt->bind_param("is", $v) ? "T" : "F";
echo "|", $stmt->bind_param("i", $v, $w) ? "T" : "F";
echo "|", $stmt->execute() ? "T" : "F";
echo "|", $stmt->errno;
"#,
    );
    assert_eq!(out, "ve|F|F|F|2006");
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
