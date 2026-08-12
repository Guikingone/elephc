//! Purpose:
//! Integration tests for the mysqli surface against a live MySQL / MariaDB
//! server. Each fixture compiles a PHP program that drives the server through
//! `mysqli` / `mysqli_result` / `mysqli_stmt` and asserts the produced stdout.
//!
//! Called from:
//! - `cargo test` through Rust's test harness. These tests are `#[ignore]`d
//!   because they require a running MySQL/MariaDB server. Run them opt-in with
//!   the DSN in the `ELEPHC_MY_DSN` environment variable (same variable as
//!   `pdo_mysql.rs`), e.g.:
//!     docker run -d --name my -e MARIADB_ROOT_PASSWORD=rootpw \
//!         -e MARIADB_DATABASE=testdb -e MARIADB_USER=test \
//!         -e MARIADB_PASSWORD=test -p 33060:3306 mariadb:11
//!     ELEPHC_MY_DSN='mysql:host=127.0.0.1;port=33060;dbname=testdb;user=test;password=test' \
//!         cargo test --test codegen_tests -- --ignored mysqli_mysql
//!
//! Key details:
//! - `my_program` parses the PDO-style `mysql:` DSN into `new mysqli(...)`
//!   arguments inside the PHP fixture, so CI keeps one env var for both PDO and
//!   mysqli live jobs.
//! - Fixtures use `DROP TABLE IF EXISTS` on fixture-specific tables so reruns
//!   are idempotent, and connect under `MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`
//!   so a broken connection fails the test loudly.

use crate::support::*;

/// Wraps a PHP body with a header that parses `ELEPHC_MY_DSN` (a PDO-style
/// `mysql:` DSN) and opens `$db` as a `mysqli` connection, so each fixture only
/// writes the mysqli logic under test.
fn my_program(body: &str) -> String {
    format!(
        r#"<?php
mysqli_report(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT);
$dsn = (string) getenv("ELEPHC_MY_DSN");
$host = "";
$port = 3306;
$dbname = "";
$user = "";
$pass = "";
$socket = "";
foreach (explode(";", substr($dsn, 6)) as $pair) {{
    $eq = strpos($pair, "=");
    if ($eq === false) {{
        continue;
    }}
    $key = substr($pair, 0, $eq);
    $value = substr($pair, $eq + 1);
    if ($key === "host") {{
        $host = $value;
    }} elseif ($key === "port") {{
        $port = (int) $value;
    }} elseif ($key === "dbname") {{
        $dbname = $value;
    }} elseif ($key === "user") {{
        $user = $value;
    }} elseif ($key === "password") {{
        $pass = $value;
    }} elseif ($key === "unix_socket") {{
        $socket = $value;
    }}
}}
if ($socket === "") {{
    $db = new mysqli($host, $user, $pass, $dbname, $port);
}} else {{
    $db = new mysqli($host, $user, $pass, $dbname, $port, $socket);
}}
{}
"#,
        body
    )
}

/// `real_escape_string` returns the escaped payload WITHOUT wrapping quotes,
/// with live `NO_BACKSLASH_ESCAPES` detection (default mode backslash-escapes).
#[test]
#[ignore]
fn test_mysqli_escape_and_roundtrip() {
    let out = compile_and_run(&my_program(
        r#"
echo $db->real_escape_string("a'b\\c");
"#,
    ));
    assert_eq!(out, r"a\'b\\c");
}

/// Connection information, ping, charset, autocommit, and a commit round-trip
/// against the live server.
#[test]
#[ignore]
fn test_mysqli_connection_info_and_transactions() {
    let out = compile_and_run(&my_program(
        r#"
echo $db->ping() ? "ping" : "dead";
echo "|", $db->server_version >= 50000 ? "ver" : "old";
echo "|", strlen($db->host_info) > 0 ? "host" : "nohost";
echo "|", $db->thread_id > 0 ? "tid" : "notid";
echo "|", $db->set_charset("utf8mb4") ? "cs" : "nocs";
echo "|", $db->character_set_name();
echo "|", $db->autocommit(true) ? "ac" : "noac";
$db->begin_transaction();
echo "|", $db->commit() ? "tx" : "notx";
$db->begin_transaction(0, "sp1");
echo "|", $db->rollback(0, "sp1") ? "sp" : "nosp";
echo "|", $db->rollback() ? "rb" : "norb";
echo "|", strlen((string) $db->stat()) > 0 ? "stat" : "nostat";
$db->close();
echo "|closed";
"#,
    ));
    assert_eq!(
        out,
        "ping|ver|host|tid|cs|utf8mb4|ac|tx|sp|rb|stat|closed"
    );
}
