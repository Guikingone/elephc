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

/// The core result-identity guarantee: `query()` returns a `mysqli_result`
/// that OWNS its rows, so a later query on the same connection leaves an
/// earlier result fully usable (`data_seek`, `fetch_assoc`, `num_rows`).
#[test]
#[ignore]
fn test_mysqli_query_assoc_and_independent_result() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mj");
$db->query("CREATE TABLE mj (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(32))");
$db->query("INSERT INTO mj (name) VALUES ('Ada'), ('Ben')");
$r1 = $db->query("SELECT name FROM mj ORDER BY id");
$r2 = $db->query("SELECT COUNT(*) AS c FROM mj");
$row = $r2->fetch_assoc();
echo $row["c"], "|";
$r1->data_seek(1);
$second = $r1->fetch_assoc();
echo $second["name"], "|";
echo $r1->num_rows;
$db->query("DROP TABLE mj");
"#,
    ));
    assert_eq!(out, "2|Ben|2");
}

/// The fetch family over one buffered result: fetch_row / fetch_array modes /
/// fetch_object / fetch_all / fetch_column / foreach, plus field metadata.
#[test]
#[ignore]
fn test_mysqli_fetch_family_and_foreach() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mf");
$db->query("CREATE TABLE mf (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(32), score DOUBLE)");
$db->query("INSERT INTO mf (name, score) VALUES ('Ada', 1.5), ('Ben', 2.25)");
$r = $db->query("SELECT id, name, score FROM mf ORDER BY id");
if (!($r instanceof mysqli_result)) {
    echo "no-result";
    exit(1);
}
$row = $r->fetch_row();
echo $row[1], "|";
$both = $r->fetch_array(MYSQLI_BOTH);
echo $both[1], "=", $both["name"], "|";
$r->data_seek(0);
$obj = $r->fetch_object();
echo $obj->name, ":", $obj->score, "|";
$r->data_seek(0);
$all = $r->fetch_all(MYSQLI_ASSOC);
echo count($all), "|";
$r->data_seek(0);
echo $r->fetch_column(1), "|";
$names = "";
foreach ($r as $i => $frow) {
    $names = $names . $i . $frow["name"];
}
echo $names, "|";
$f = $r->fetch_field();
echo $f->name, ":", $f->type == MYSQLI_TYPE_LONG || $f->type == MYSQLI_TYPE_LONGLONG ? "int" : "other";
echo "|", $r->field_count, "|", mysqli_num_rows($r);
$db->query("DROP TABLE mf");
"#,
    ));
    // fetch_row consumes row 0 (Ada), fetch_array then sees row 1 (Ben); each
    // data_seek(0) rewinds before the next family member.
    assert_eq!(out, "Ada|Ben=Ben|Ada:1.5|2|Ada|0Ada1Ben|id:int|3|2");
}

/// Non-select statements return `true` and refresh `affected_rows` /
/// `insert_id`; `real_query` + `store_result` picks up the pending result.
#[test]
#[ignore]
fn test_mysqli_non_select_and_store_result() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mn");
$db->query("CREATE TABLE mn (id INT PRIMARY KEY AUTO_INCREMENT, v INT)");
$ok = $db->query("INSERT INTO mn (v) VALUES (10)");
echo $ok === true ? "T" : "F";
echo "|", $db->affected_rows, "|", $db->insert_id;
$db->query("INSERT INTO mn (v) VALUES (20), (30)");
echo "|", $db->affected_rows;
echo "|", $db->real_query("SELECT v FROM mn ORDER BY id") ? "rq" : "no";
$r = $db->store_result();
echo "|", $r === false ? "F" : $r->num_rows;
echo "|", $db->store_result() === false ? "empty" : "again";
$db->query("DROP TABLE mn");
"#,
    ));
    assert_eq!(out, "T|1|1|2|rq|3|empty");
}

/// Prepared statements: `bind_param` + `execute` + `get_result`, with a
/// re-executable statement. Bound values are captured at bind_param time
/// (documented divergence from PHP's read-at-execute references), so fresh
/// values come from re-binding or `execute($params)`.
#[test]
#[ignore]
fn test_mysqli_stmt_bind_param_and_get_result() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS ms");
$db->query("CREATE TABLE ms (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(32), score DOUBLE)");
$ins = $db->prepare("INSERT INTO ms (name, score) VALUES (?, ?)");
echo $ins->param_count, "|";
$name = "Ada";
$score = 1.5;
$ins->bind_param("sd", $name, $score);
$ins->execute();
$name = "Ben";
$score = 2.25;
$ins->bind_param("sd", $name, $score);
$ins->execute();
echo $ins->affected_rows, "|";
$sel = $db->prepare("SELECT name, score FROM ms WHERE name = ?");
$who = "Ben";
$sel->bind_param("s", $who);
$sel->execute();
$r = $sel->get_result();
if (!($r instanceof mysqli_result)) {
    echo "no-result";
    exit(1);
}
$row = $r->fetch_assoc();
echo $row["name"], ":", $row["score"], "|";
$who = "Ada";
$sel->bind_param("s", $who);
$sel->execute();
$r2 = $sel->get_result();
if (!($r2 instanceof mysqli_result)) {
    echo "no-result2";
    exit(1);
}
$row2 = $r2->fetch_assoc();
echo $row2["name"], "|";
$sel->close();
$ins->close();
$db->query("DROP TABLE ms");
"#,
    ));
    assert_eq!(out, "2|1|Ben:2.25|Ada|");
}

/// `execute($params)` binds an array per execution (PHP 8.1+ shape), and
/// `store_result` makes `num_rows` valid on the statement.
#[test]
#[ignore]
fn test_mysqli_stmt_execute_params_and_store_result() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mb");
$db->query("CREATE TABLE mb (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(32))");
$ins = $db->prepare("INSERT INTO mb (name) VALUES (?)");
$ins->execute(["Ada"]);
$ins->execute(["Ben"]);
echo $ins->insert_id, "|";
$sel = $db->prepare("SELECT id, name FROM mb ORDER BY id");
$sel->execute();
$sel->store_result();
echo $sel->num_rows, "|";
$sel->execute();
$r = $sel->get_result();
if (!($r instanceof mysqli_result)) {
    echo "no-result";
    exit(1);
}
$out = "";
foreach ($r as $row) {
    $out = $out . $row["id"] . ":" . $row["name"] . ",";
}
echo $out;
$sel->close();
$ins->close();
$db->query("DROP TABLE mb");
"#,
    ));
    assert_eq!(out, "2|2|1:Ada,2:Ben,");
}

/// `execute_query` (PHP 8.2+) is prepare + execute + get_result in one call,
/// and the procedural statement pipeline works end to end.
#[test]
#[ignore]
fn test_mysqli_execute_query_and_procedural_stmt() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mq");
$db->query("CREATE TABLE mq (id INT PRIMARY KEY AUTO_INCREMENT, v INT)");
echo $db->execute_query("INSERT INTO mq (v) VALUES (?)", [7]) === true ? "ins" : "no";
$r = $db->execute_query("SELECT v FROM mq");
if (!($r instanceof mysqli_result)) {
    echo "no-result";
    exit(1);
}
echo "|", $r->fetch_column(0);
$stmt = mysqli_prepare($db, "SELECT v + ? FROM mq");
$delta = 5;
echo "|", mysqli_stmt_bind_param($stmt, "i", $delta) ? "bp" : "no";
echo "|", mysqli_stmt_execute($stmt) ? "ex" : "no";
$pr = mysqli_stmt_get_result($stmt);
if (!($pr instanceof mysqli_result)) {
    echo "no-presult";
    exit(1);
}
echo "|", $pr->fetch_column(0);
echo "|", mysqli_stmt_param_count($stmt);
mysqli_stmt_close($stmt);
$db->query("DROP TABLE mq");
"#,
    ));
    assert_eq!(out, "ins|7|bp|ex|12|1");
}

/// `multi_query` sends the whole batch in one round-trip; `store_result`
/// returns each set, `more_results`/`next_result` walk the retained sets, and
/// a non-select statement in the batch updates `affected_rows` instead.
#[test]
#[ignore]
fn test_mysqli_multi_query_traverses_result_sets() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS mm");
$db->query("CREATE TABLE mm (v INT)");
echo $db->multi_query("SELECT 1 AS a; INSERT INTO mm (v) VALUES (5), (6); SELECT 2 AS b") ? "mq" : "no";
$r1 = $db->store_result();
if (!($r1 instanceof mysqli_result)) {
    echo "no-r1";
    exit(1);
}
echo "|", $r1->fetch_column(0);
echo "|", $db->more_results() ? "more" : "done";
echo "|", $db->next_result() ? "next" : "no-next";
echo "|", $db->store_result() === false ? "F" : "set";
echo "|", $db->affected_rows;
echo "|", $db->more_results() ? "more" : "done";
echo "|", $db->next_result() ? "next" : "no-next";
$r2 = $db->store_result();
if (!($r2 instanceof mysqli_result)) {
    echo "no-r2";
    exit(1);
}
echo "|", $r2->fetch_column(0);
echo "|", $db->more_results() ? "more" : "done";
echo "|", $db->next_result() ? "next" : "no-next";
echo "|", $r1->num_rows;
$db->query("DROP TABLE mm");
"#,
    ));
    assert_eq!(
        out,
        "mq|1|more|next|F|2|more|next|2|done|no-next|1"
    );
}

/// The procedural multi_query pipeline mirrors the object surface.
#[test]
#[ignore]
fn test_mysqli_multi_query_procedural() {
    let out = compile_and_run(&my_program(
        r#"
echo mysqli_multi_query($db, "SELECT 7 AS a; SELECT 8 AS b") ? "mq" : "no";
$r = mysqli_store_result($db);
if (!($r instanceof mysqli_result)) {
    echo "no-r";
    exit(1);
}
echo "|", $r->fetch_column(0);
echo "|", mysqli_more_results($db) ? "more" : "done";
echo "|", mysqli_next_result($db) ? "next" : "no-next";
$r2 = mysqli_store_result($db);
if (!($r2 instanceof mysqli_result)) {
    echo "no-r2";
    exit(1);
}
echo "|", $r2->fetch_column(0);
echo "|", mysqli_more_results($db) ? "more" : "done";
"#,
    ));
    assert_eq!(out, "mq|7|more|next|8|done");
}

/// `real_connect` on an already-connected object closes the previous
/// connection first (php-src reconnect semantics): the new session has a new
/// thread id and the object stays fully usable.
#[test]
#[ignore]
fn test_mysqli_reconnect_closes_previous_connection() {
    let out = compile_and_run(&my_program(
        r#"
$tid1 = $db->thread_id;
echo $tid1 > 0 ? "tid" : "notid";
echo "|", $db->real_connect($host, $user, $pass, $dbname, $port) ? "re" : "no";
echo "|", $db->thread_id > 0 && $db->thread_id != $tid1 ? "newtid" : "same";
$r = $db->query("SELECT 42");
echo "|", ($r instanceof mysqli_result) ? $r->fetch_column(0) : "no";
"#,
    ));
    assert_eq!(out, "tid|re|newtid|42");
}

/// While multi_query result sets remain unconsumed, `query()` and `prepare()`
/// fail with errno 2014 (php-src's "Commands out of sync"); after the batch is
/// fully consumed the connection accepts statements again.
#[test]
#[ignore]
fn test_mysqli_out_of_sync_guard() {
    let out = compile_and_run(&my_program(
        r#"
mysqli_report(MYSQLI_REPORT_OFF);
$db->multi_query("SELECT 1 AS a; SELECT 2 AS b");
echo $db->query("SELECT 3") === false ? "F" : "ok";
echo "|", $db->errno;
$r1 = $db->store_result();
echo "|", $db->query("SELECT 3") === false ? "F" : "ok";
echo "|", $db->errno;
echo "|", $db->prepare("SELECT ?") === false ? "F" : "ok";
echo "|", $db->ping() === false ? "F" : "ok";
echo "|", $db->select_db("testdb") === false ? "F" : "ok";
echo "|", $db->begin_transaction() === false ? "F" : "ok";
echo "|", $db->errno;
$db->next_result();
$r2 = $db->store_result();
$q = $db->query("SELECT 3 AS c");
echo "|", ($q instanceof mysqli_result) ? $q->fetch_column(0) : "no";
echo "|", ($r1 instanceof mysqli_result) ? $r1->fetch_column(0) : "no";
"#,
    ));
    assert_eq!(out, "F|2014|F|2014|F|F|F|F|2014|3|1");
}

/// `query()` rejects a second top-level statement client-side (errno 1064) —
/// the anti-injection contract php-src enforces server-side — while a trailing
/// `;`, semicolons inside string literals, and `--`-commented tails stay
/// single statements, and `multi_query()` remains the batch path.
#[test]
#[ignore]
fn test_mysqli_query_rejects_multi_statement() {
    let out = compile_and_run(&my_program(
        r#"
mysqli_report(MYSQLI_REPORT_OFF);
echo $db->query("SELECT 1 AS a; SELECT 2 AS b") === false ? "F" : "ok";
echo "|", $db->errno;
$r = $db->query("SELECT 7;");
echo "|", ($r instanceof mysqli_result) ? $r->fetch_column(0) : "no";
$s = $db->query("SELECT '; not a statement' AS s");
echo "|", ($s instanceof mysqli_result) ? "str-ok" : "no";
$c = $db->query("SELECT 8 -- tail; SELECT 9");
echo "|", ($c instanceof mysqli_result) ? $c->fetch_column(0) : "no";
echo "|", $db->multi_query("SELECT 5 AS a; SELECT 6 AS b") ? "mq" : "no";
$m1 = $db->store_result();
$db->next_result();
$m2 = $db->store_result();
echo "|", ($m2 instanceof mysqli_result) ? $m2->fetch_column(0) : "no";
echo "|", $db->query("CREATE TABLE mms (i INT); DROP TABLE mms") === false ? "F" : "ok";
echo "|", $db->errno;
"#,
    ));
    assert_eq!(out, "F|1064|7|str-ok|8|mq|6|F|1064");
}

/// Compound-body DDL is NOT exempt from the multi-statement scan: through
/// `query()` it fails 1064 (including the `... END; DROP ...` injection-tail
/// shape), while `multi_query()` runs it as one statement and `CALL` works.
#[test]
#[ignore]
fn test_mysqli_compound_ddl_requires_multi_query() {
    let out = compile_and_run(&my_program(
        r#"
mysqli_report(MYSQLI_REPORT_OFF);
$db->query("DROP PROCEDURE IF EXISTS mproc");
echo $db->query("CREATE PROCEDURE mproc() BEGIN SELECT 41; SELECT 42 AS v; END") === false ? "F" : "leaked";
echo "|", $db->errno;
echo "|", $db->query("CREATE PROCEDURE mproc() BEGIN SELECT 1; END; DROP TABLE users") === false ? "F" : "leaked";
echo "|", $db->errno;
echo "|", $db->multi_query("CREATE PROCEDURE mproc() BEGIN SELECT 41; SELECT 42 AS v; END") ? "ddl" : ("no:" . $db->errno);
echo "|", $db->multi_query("CALL mproc()") ? "call" : "no";
$r1 = $db->store_result();
echo "|", ($r1 instanceof mysqli_result) ? $r1->fetch_column(0) : "no";
while ($db->more_results()) {
    $db->next_result();
    $rn = $db->store_result();
    if ($rn instanceof mysqli_result) {
        echo "|", $rn->fetch_column(0);
    }
}
$db->query("DROP PROCEDURE mproc");
echo "|done";
"#,
    ));
    assert_eq!(out, "F|1064|F|1064|ddl|call|41|42|done");
}

/// `fetch_field_direct()->decimals` carries MySQL's wire "decimals" byte
/// (through the bridge's column_precision), so a DECIMAL(10,2) reports 2.
#[test]
#[ignore]
fn test_mysqli_fetch_field_decimals() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS md");
$db->query("CREATE TABLE md (price DECIMAL(10,2), label VARCHAR(8))");
$db->query("INSERT INTO md VALUES (12.34, 'x')");
$r = $db->query("SELECT price, label FROM md");
if (!($r instanceof mysqli_result)) {
    echo "no-result";
    exit(1);
}
$f = $r->fetch_field_direct(0);
echo $f->decimals;
$g = $r->fetch_field_direct(1);
echo "|", $g->decimals;
$db->query("DROP TABLE md");
"#,
    ));
    assert_eq!(out, "2|0");
}

/// Pins the documented bind_param divergence: values are captured at bind
/// time, so mutating the variable WITHOUT re-binding executes with the value
/// seen at bind (PHP's references would read the mutated value instead).
#[test]
#[ignore]
fn test_mysqli_stmt_bind_param_snapshot_divergence() {
    let out = compile_and_run(&my_program(
        r#"
$db->query("DROP TABLE IF EXISTS msnap");
$db->query("CREATE TABLE msnap (name VARCHAR(32))");
$ins = $db->prepare("INSERT INTO msnap (name) VALUES (?)");
$name = "Ada";
$ins->bind_param("s", $name);
$name = "Ben";
$ins->execute();
$r = $db->query("SELECT name FROM msnap");
echo ($r instanceof mysqli_result) ? $r->fetch_column(0) : "no";
$db->query("DROP TABLE msnap");
"#,
    ));
    assert_eq!(out, "Ada");
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
