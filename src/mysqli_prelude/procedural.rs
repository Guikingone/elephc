//! Purpose:
//! The `mysqli_*` procedural aliases, as an elephc-PHP source fragment. Every
//! alias is an ordinary PHP function forwarding to the `mysqli` object, so
//! `function_exists('mysqli_query')` is true once the prelude is injected.
//!
//! Called from:
//! - `crate::mysqli_prelude::fragments::source_for_version` (concatenated into
//!   the injected prelude).
//!
//! Key details:
//! - elephc always requires the explicit link argument, including under
//!   `--php-version=8.0` (PHP 8.0's implicit last-link is a documented
//!   divergence; PHP 8.1+ requires the object anyway).
//! - `mysqli_connect_errno()` / `mysqli_connect_error()` take no link and read
//!   the process-wide last-connect statics on `mysqli`, exactly like PHP.
//! - `mysqli_report()` lives in `exception.rs` next to the flag store.

/// The procedural `mysqli_*` alias functions (fragment without a `<?php`
/// header).
pub(super) const SRC: &str = r#"
// -- mysqli procedural aliases --

function mysqli_connect(
    ?string $hostname = null,
    ?string $username = null,
    ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null
): mysqli|false {
    // Unlike the argument-less constructor, procedural mysqli_connect() always
    // attempts the connection (php-src behavior; null arguments take their
    // defaults inside real_connect).
    $_link = new mysqli();
    if ($_link->real_connect($hostname, $username, $password, $database, $port, $socket, 0)) {
        return $_link;
    }
    return false;
}

function mysqli_init(): mysqli {
    return new mysqli();
}

function mysqli_real_connect(
    mysqli $mysql,
    ?string $hostname = null,
    ?string $username = null,
    ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null,
    int $flags = 0
): bool {
    return $mysql->real_connect($hostname, $username, $password, $database, $port, $socket, $flags);
}

function mysqli_close(mysqli $mysql): bool {
    return $mysql->close();
}

function mysqli_ping(mysqli $mysql): bool {
    return $mysql->ping();
}

function mysqli_select_db(mysqli $mysql, string $database): bool {
    return $mysql->select_db($database);
}

function mysqli_set_charset(mysqli $mysql, string $charset): bool {
    return $mysql->set_charset($charset);
}

function mysqli_character_set_name(mysqli $mysql): string {
    return $mysql->character_set_name();
}

function mysqli_real_escape_string(mysqli $mysql, string $string): string {
    return $mysql->real_escape_string($string);
}

function mysqli_escape_string(mysqli $mysql, string $string): string {
    return $mysql->real_escape_string($string);
}

function mysqli_begin_transaction(mysqli $mysql, int $flags = 0, ?string $name = null): bool {
    return $mysql->begin_transaction($flags, $name);
}

function mysqli_commit(mysqli $mysql, int $flags = 0, ?string $name = null): bool {
    return $mysql->commit($flags, $name);
}

function mysqli_rollback(mysqli $mysql, int $flags = 0, ?string $name = null): bool {
    return $mysql->rollback($flags, $name);
}

function mysqli_autocommit(mysqli $mysql, bool $enable): bool {
    return $mysql->autocommit($enable);
}

function mysqli_options(mysqli $mysql, int $option, mixed $value): bool {
    return $mysql->options($option, $value);
}

function mysqli_set_opt(mysqli $mysql, int $option, mixed $value): bool {
    return $mysql->options($option, $value);
}

function mysqli_get_server_info(mysqli $mysql): string {
    return $mysql->get_server_info();
}

function mysqli_get_client_info(mysqli $mysql): string {
    return $mysql->get_client_info();
}

function mysqli_get_host_info(mysqli $mysql): string {
    return $mysql->get_host_info();
}

function mysqli_get_proto_info(mysqli $mysql): int {
    return $mysql->get_proto_info();
}

function mysqli_get_server_version(mysqli $mysql): int {
    return $mysql->get_server_version();
}

function mysqli_get_client_version(mysqli $mysql): int {
    return $mysql->get_client_version();
}

function mysqli_stat(mysqli $mysql): string|false {
    return $mysql->stat();
}

function mysqli_thread_id(mysqli $mysql): int {
    return $mysql->thread_id;
}

function mysqli_connect_errno(): int {
    return mysqli::$lastConnectErrno;
}

function mysqli_connect_error(): ?string {
    if (mysqli::$lastConnectErrno == 0) {
        return null;
    }
    return mysqli::$lastConnectError;
}

function mysqli_errno(mysqli $mysql): int {
    return $mysql->errno;
}

function mysqli_error(mysqli $mysql): string {
    return $mysql->error;
}

function mysqli_error_list(mysqli $mysql): array {
    return $mysql->error_list;
}

function mysqli_sqlstate(mysqli $mysql): string {
    return $mysql->sqlstate;
}

function mysqli_affected_rows(mysqli $mysql): int {
    return $mysql->affected_rows;
}

function mysqli_insert_id(mysqli $mysql): int {
    return $mysql->insert_id;
}

function mysqli_field_count(mysqli $mysql): int {
    return $mysql->field_count;
}

function mysqli_warning_count(mysqli $mysql): int {
    return $mysql->warning_count;
}

function mysqli_info(mysqli $mysql): ?string {
    if ($mysql->info === "") {
        return null;
    }
    return $mysql->info;
}
"#;
