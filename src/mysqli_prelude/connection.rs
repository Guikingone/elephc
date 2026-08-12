//! Purpose:
//! The `mysqli` connection class, as an elephc-PHP source fragment: DSN
//! construction over `elephc_pdo_open_persistent`, connect-time and per-op
//! error bookkeeping (`connect_errno`/`errno`/`error_list`), `mysqli_report`
//! dispatch, escaping, charset, `select_db`, ping, and transactions.
//!
//! Called from:
//! - `crate::mysqli_prelude::fragments::source_for_version` (concatenated into
//!   the injected prelude).
//!
//! Key details:
//! - `$conn = -1` means "not connected" (`mysqli_init()` / argument-less
//!   `new mysqli()`); a successful `real_connect` stores the bridge handle. The
//!   DSN prefix is always forced to `mysql:` and a successful open whose
//!   `elephc_pdo_driver_name` is not "mysql" is rejected (belt-and-braces).
//! - Failure dispatch follows `mysqli::$reportMode` (see `exception.rs`):
//!   STRICT throws `mysqli_sql_exception`, ERROR writes to STDERR and the
//!   caller returns `false`, OFF is silent — mirroring PHP 8.1+'s default.
//!   `PDOException` is never thrown.
//! - `real_escape_string` reuses the MySQL branch of `PDO::quote()` minus the
//!   wrapping quotes and the `_binary` introducer, including the
//!   `NO_BACKSLASH_ESCAPES` quote-doubling fallback (mysqlnd's own behavior).
//! - Method-local variables use the `$_` prefix (same checker clash rule as the
//!   PDO prelude). Public properties are refreshed after operations; writes to
//!   them stick (documented divergence, no write barriers in v1).

/// The `mysqli` class and its connection-level behavior (fragment without a
/// `<?php` header).
pub(super) const SRC: &str = r#"
// -- mysqli connection surface --

class mysqli {
    // Opaque elephc_pdo bridge connection handle; -1 = not connected.
    public int $conn = -1;

    // Process-wide mysqli_report() mode. The literal default is rewritten at
    // injection time for --php-version 8.0 (OFF); see fragments.rs.
    public static int $reportMode = 3;

    // Process-wide last-connect failure, read by the no-argument procedural
    // mysqli_connect_errno() / mysqli_connect_error() exactly like PHP's
    // globals; updated by every construct / real_connect attempt.
    public static int $lastConnectErrno = 0;
    public static string $lastConnectError = "";

    // Public properties refreshed after operations (writes stick; documented
    // divergence — no write barriers in v1).
    public int $affected_rows = 0;
    public int $connect_errno = 0;
    public ?string $connect_error = null;
    public int $errno = 0;
    public string $error = "";
    public array $error_list = [];
    public int $field_count = 0;
    public string $client_info = "";
    public int $client_version = 0;
    public string $host_info = "";
    public int $protocol_version = 10;
    public string $server_info = "";
    public int $server_version = 0;
    public string $info = "";
    public int $insert_id = 0;
    public string $sqlstate = "00000";
    public int $thread_id = 0;
    public int $warning_count = 0;

    // mysqli_options() values collected before real_connect applies them.
    private int $optConnectTimeout = 0;
    private string $optInitCommand = "";
    private string $optCharsetName = "";
    // Connection charset, lazily read from the server on first ask.
    private string $currentCharset = "";

    public function __construct(
        ?string $hostname = null,
        ?string $username = null,
        ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null
    ) {
        // The argument-less constructor is mysqli_init(): no connection attempt
        // until real_connect() (php-src behavior).
        if ($hostname === null && $username === null && $password === null
            && $database === null && $port === null && $socket === null) {
            return;
        }
        $this->real_connect($hostname, $username, $password, $database, $port, $socket, 0);
    }

    public function real_connect(
        ?string $hostname = null,
        ?string $username = null,
        ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null,
        int $flags = 0
    ): bool {
        if (($flags & 2048) != 0) {
            // MYSQLI_CLIENT_SSL is declared but unsupported: fail loudly rather
            // than silently connecting in cleartext.
            return $this->connectFailure(2054, "elephc mysqli does not support MYSQLI_CLIENT_SSL; use PDO MySQL TLS attributes", "HY000");
        }
        $_host = $hostname === null ? "localhost" : $hostname;
        $_persistent = 0;
        if (str_starts_with($_host, "p:")) {
            // php-src: a host beginning with `p:` selects a persistent
            // connection; the real host is the remainder.
            $_persistent = 1;
            $_host = substr($_host, 2);
        }
        $_port = $port === null ? 3306 : $port;
        $_socket = $socket === null ? "" : $socket;
        // php-src mysqli honors the socket only when the host is empty or
        // exactly "localhost"; any other host goes over TCP.
        $_dsn = "mysql:";
        if ($_socket !== "" && ($_host === "" || $_host === "localhost")) {
            $_dsn = $_dsn . "unix_socket=" . $_socket;
        } else {
            $_dsn = $_dsn . "host=" . ($_host === "" ? "localhost" : $_host) . ";port=" . $_port;
        }
        if ($database !== null && $database !== "") {
            $_dsn = $_dsn . ";dbname=" . $database;
        }
        if ($username !== null && $username !== "") {
            $_dsn = $_dsn . ";user=" . $username;
        }
        if ($password !== null && $password !== "") {
            $_dsn = $_dsn . ";password=" . $password;
        }
        if ($this->optConnectTimeout > 0) {
            $_dsn = $_dsn . ";connect_timeout=" . $this->optConnectTimeout;
        }
        // Client flags map onto the bridge's packed driver config (same format
        // PDO packs for $my_driver_config); FOUND_ROWS rides its own argument.
        $_driverConfig = "";
        if (($flags & 32) != 0) {
            $_driverConfig = $_driverConfig . "compress=1;";
        }
        if (($flags & 256) != 0) {
            $_driverConfig = $_driverConfig . "ignore=1;";
        }
        $_foundRows = (($flags & 2) != 0) ? 1 : 0;
        $_conn = elephc_pdo_open_persistent($_dsn, $_persistent, 0, $this->optInitCommand, "", $_foundRows, "", $_driverConfig);
        if ($_conn < 0) {
            $_message = elephc_pdo_last_open_error();
            $_state = elephc_pdo_last_open_sqlstate();
            if ($_state === "") {
                // Connect-time network failure default, same class the PDO
                // mysql driver falls back to.
                $_state = "HY000";
            }
            $_code = elephc_pdo_last_open_native_code();
            if ($_code == 0) {
                // CR_CONN_HOST_ERROR: generic client-side connect failure.
                $_code = 2002;
            }
            return $this->connectFailure($_code, $_message, $_state);
        }
        // Cannot happen while the DSN prefix is forced to mysql:, but keep the
        // guard: a non-mysql handle must never become a mysqli connection.
        if (elephc_pdo_driver_name($_conn) !== "mysql") {
            elephc_pdo_release($_conn, 0);
            return $this->connectFailure(2002, "elephc mysqli opened a non-mysql bridge connection", "HY000");
        }
        $this->conn = $_conn;
        $this->connect_errno = 0;
        $this->connect_error = null;
        mysqli::$lastConnectErrno = 0;
        mysqli::$lastConnectError = "";
        $this->clearError();
        // Connection information, refreshed once per connect.
        $this->host_info = elephc_pdo_connection_status($_conn);
        $this->server_info = elephc_pdo_server_version($_conn);
        $this->server_version = $this->versionStringToInt($this->server_info);
        $this->client_info = elephc_pdo_client_version($_conn);
        $this->client_version = $this->versionStringToInt($this->client_info);
        $this->thread_id = $this->fetchIntScalar("SELECT CONNECTION_ID()");
        $this->warning_count = elephc_pdo_warning_count($_conn);
        if ($this->optCharsetName !== "") {
            // MYSQLI_SET_CHARSET_NAME collected before connect applies now.
            $this->set_charset($this->optCharsetName);
        }
        return true;
    }

    public function connect(
        ?string $hostname = null,
        ?string $username = null,
        ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null
    ): bool {
        // Alias of the constructor's connect path (php-src keeps it for
        // backwards compatibility with the old procedural object).
        return $this->real_connect($hostname, $username, $password, $database, $port, $socket, 0);
    }

    public function close(): bool {
        if ($this->conn >= 0) {
            // Roll an open transaction back first (matching PHP and keeping a
            // persistent handle clean when it returns to the pool).
            if (elephc_pdo_in_transaction($this->conn) === 1) {
                elephc_pdo_rollback($this->conn);
            }
            elephc_pdo_release($this->conn, 0);
            $this->conn = -1;
        }
        return true;
    }

    public function __destruct() {
        $this->close();
    }

    public function ping(): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        // A cheap round-trip; see the plan's elephc_pdo_ping escape hatch if
        // this ever eats a pending multi_query result.
        if (elephc_pdo_exec($this->conn, "SELECT 1") < 0) {
            return $this->opFailed();
        }
        $this->clearError();
        return true;
    }

    public function select_db(string $database): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        $_ident = str_replace("`", "``", $database);
        if (elephc_pdo_exec($this->conn, "USE `" . $_ident . "`") < 0) {
            return $this->opFailed();
        }
        $this->clearError();
        return true;
    }

    public function set_charset(string $charset): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        // Same [A-Za-z0-9_] identifier filter the PDO DSN charset key uses: a
        // charset name is an identifier, never quoted, so anything else is
        // rejected before it can smuggle SQL.
        if (!$this->charsetIdentIsValid($charset)) {
            $this->syntheticFailure(2019, "Invalid characterset or character set not supported", "HY000");
            return false;
        }
        if (elephc_pdo_exec($this->conn, "SET NAMES " . $charset) < 0) {
            return $this->opFailed();
        }
        $this->currentCharset = $charset;
        $this->clearError();
        return true;
    }

    public function character_set_name(): string {
        if ($this->conn < 0) {
            return "";
        }
        if ($this->currentCharset === "") {
            $this->currentCharset = $this->fetchStringScalar("SELECT @@character_set_client");
        }
        return $this->currentCharset;
    }

    public function real_escape_string(string $string): string {
        // The MySQL branch of PDO::quote() minus the wrapping quotes and the
        // `_binary` introducer. Never calls quote(): mysqli's contract is the
        // escaped payload WITHOUT surrounding quotes.
        if ($this->conn >= 0 && elephc_pdo_no_backslash_escapes($this->conn) != 0) {
            // SECURITY: under NO_BACKSLASH_ESCAPES a backslash is a literal, so
            // backslash-escaping is unsafe (an "escaped" quote breaks out of
            // the literal); mysqlnd switches to quote-doubling only.
            return str_replace("'", "''", $string);
        }
        $_s = str_replace("\\", "\\\\", $string);
        $_s = str_replace("'", "\\'", $_s);
        $_s = str_replace("\"", "\\\"", $_s);
        $_s = str_replace(chr(0), "\\0", $_s);
        $_s = str_replace(chr(10), "\\n", $_s);
        $_s = str_replace(chr(13), "\\r", $_s);
        $_s = str_replace(chr(26), "\\Z", $_s);
        return $_s;
    }

    public function escape_string(string $string): string {
        return $this->real_escape_string($string);
    }

    public function begin_transaction(int $flags = 0, ?string $name = null): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        if (($flags & 4) != 0) {
            // MYSQLI_TRANS_START_READ_ONLY (best effort: report on failure).
            if (elephc_pdo_exec($this->conn, "SET TRANSACTION READ ONLY") < 0) {
                return $this->opFailed();
            }
        } elseif (($flags & 2) != 0) {
            // MYSQLI_TRANS_START_READ_WRITE.
            if (elephc_pdo_exec($this->conn, "SET TRANSACTION READ WRITE") < 0) {
                return $this->opFailed();
            }
        }
        if (($flags & 1) != 0) {
            // MYSQLI_TRANS_START_WITH_CONSISTENT_SNAPSHOT needs the explicit
            // START TRANSACTION form; the bridge's begin is a plain BEGIN.
            if (elephc_pdo_exec($this->conn, "START TRANSACTION WITH CONSISTENT SNAPSHOT") < 0) {
                return $this->opFailed();
            }
        } else {
            if (elephc_pdo_begin($this->conn) != 1) {
                return $this->opFailed();
            }
        }
        if ($name !== null) {
            if ($name === "") {
                throw new ValueError("mysqli::begin_transaction(): Argument #2 (\$name) cannot be empty");
            }
            if (elephc_pdo_exec($this->conn, "SAVEPOINT `" . str_replace("`", "``", $name) . "`") < 0) {
                return $this->opFailed();
            }
        }
        $this->clearError();
        return true;
    }

    public function commit(int $flags = 0, ?string $name = null): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        if ($name !== null) {
            if ($name === "") {
                throw new ValueError("mysqli::commit(): Argument #2 (\$name) cannot be empty");
            }
            if (elephc_pdo_exec($this->conn, "RELEASE SAVEPOINT `" . str_replace("`", "``", $name) . "`") < 0) {
                return $this->opFailed();
            }
            $this->clearError();
            return true;
        }
        if (elephc_pdo_commit($this->conn) != 1) {
            return $this->opFailed();
        }
        $this->clearError();
        return true;
    }

    public function rollback(int $flags = 0, ?string $name = null): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        if ($name !== null) {
            if ($name === "") {
                throw new ValueError("mysqli::rollback(): Argument #2 (\$name) cannot be empty");
            }
            if (elephc_pdo_exec($this->conn, "ROLLBACK TO `" . str_replace("`", "``", $name) . "`") < 0) {
                return $this->opFailed();
            }
            $this->clearError();
            return true;
        }
        if (elephc_pdo_rollback($this->conn) != 1) {
            return $this->opFailed();
        }
        $this->clearError();
        return true;
    }

    public function autocommit(bool $enable): bool {
        if (!$this->requireConnection()) {
            return false;
        }
        if (elephc_pdo_set_autocommit($this->conn, $enable ? 1 : 0) != 1) {
            return $this->opFailed();
        }
        $this->clearError();
        return true;
    }

    public function options(int $option, mixed $value): bool {
        // The three locked option ids; anything else is unsupported and fails
        // loudly with `false` (php-src also returns false for unknown options).
        if ($option == 0) {
            // MYSQLI_OPT_CONNECT_TIMEOUT -> DSN connect_timeout at connect.
            $this->optConnectTimeout = (int) $value;
            return true;
        }
        if ($option == 3) {
            // MYSQLI_INIT_COMMAND -> $my_init_command at connect.
            $this->optInitCommand = (string) $value;
            return true;
        }
        if ($option == 7) {
            // MYSQLI_SET_CHARSET_NAME -> SET NAMES after connect.
            $this->optCharsetName = (string) $value;
            return true;
        }
        return false;
    }

    public function set_opt(int $option, mixed $value): bool {
        return $this->options($option, $value);
    }

    public function get_server_info(): string {
        return $this->server_info;
    }

    public function get_client_info(): string {
        return $this->client_info;
    }

    public function get_host_info(): string {
        return $this->host_info;
    }

    public function get_proto_info(): int {
        return $this->protocol_version;
    }

    public function get_server_version(): int {
        return $this->server_version;
    }

    public function get_client_version(): int {
        return $this->client_version;
    }

    public function stat(): string|false {
        if (!$this->requireConnection()) {
            return false;
        }
        // The bridge's server_info is MySQL's own "Uptime: … Questions: …"
        // statistics line — exactly what mysqli::stat() returns.
        $_stat = elephc_pdo_server_info($this->conn);
        if ($_stat === "") {
            return false;
        }
        return $_stat;
    }

    // -- internal helpers ($_-prefixed locals; same checker rule as PDO) --

    // Guards every operation that needs a live connection: an unconnected
    // object records CR_SERVER_GONE_ERROR and reports it (false under
    // ERROR/OFF, throw under STRICT), matching the locked "fail loudly" rule.
    private function requireConnection(): bool {
        if ($this->conn >= 0) {
            return true;
        }
        $this->syntheticFailure(2006, "MySQL server has gone away", "HY000");
        return false;
    }

    // Records a client-side failure that has no live bridge error state
    // (unconnected handle, invalid charset) and dispatches mysqli_report.
    private function syntheticFailure(int $errno, string $message, string $sqlstate): void {
        $this->errno = $errno;
        $this->error = $message;
        $this->sqlstate = $sqlstate;
        $this->error_list = [["errno" => $errno, "sqlstate" => $sqlstate, "error" => $message]];
        $this->report($message, $errno, $sqlstate);
    }

    // Records a connect-time failure on both the instance (connect_errno /
    // connect_error, distinct from errno/error) and the process-wide statics
    // behind mysqli_connect_errno()/mysqli_connect_error(), then dispatches
    // mysqli_report. Always returns false so connect paths can tail-call it.
    private function connectFailure(int $errno, string $message, string $sqlstate): bool {
        $this->connect_errno = $errno;
        $this->connect_error = $message;
        $this->errno = $errno;
        $this->error = $message;
        $this->sqlstate = $sqlstate;
        $this->error_list = [["errno" => $errno, "sqlstate" => $sqlstate, "error" => $message]];
        mysqli::$lastConnectErrno = $errno;
        mysqli::$lastConnectError = $message;
        $this->report($message, $errno, $sqlstate);
        return false;
    }

    // Refreshes errno/error/sqlstate/error_list from the live bridge error
    // state after a failed operation, then dispatches mysqli_report. Always
    // returns false so callers can tail-call it.
    private function opFailed(): bool {
        $this->errno = elephc_pdo_errcode($this->conn);
        $this->error = elephc_pdo_errmsg($this->conn);
        $this->sqlstate = elephc_pdo_sqlstate($this->conn);
        if ($this->sqlstate === "") {
            $this->sqlstate = "HY000";
        }
        $this->error_list = [["errno" => $this->errno, "sqlstate" => $this->sqlstate, "error" => $this->error]];
        $this->report($this->error, $this->errno, $this->sqlstate);
        return false;
    }

    // Clears the per-operation error state after a successful operation.
    private function clearError(): void {
        $this->errno = 0;
        $this->error = "";
        $this->sqlstate = "00000";
        $this->error_list = [];
    }

    // mysqli_report dispatch: STRICT throws mysqli_sql_exception (never
    // PDOException), ERROR alone writes the message to STDERR, OFF is silent.
    private function report(string $message, int $errno, string $sqlstate): void {
        if ((mysqli::$reportMode & 2) != 0) {
            $_e = new mysqli_sql_exception($message, $errno);
            $_e->sqlstate = $sqlstate;
            throw $_e;
        }
        if ((mysqli::$reportMode & 1) != 0) {
            fwrite(STDERR, "mysqli error: " . $message . "\n");
        }
    }

    // Runs a one-row one-column SELECT and returns its integer value (0 on any
    // failure). Backs thread_id (SELECT CONNECTION_ID()).
    private function fetchIntScalar(string $sql): int {
        $_stmt = elephc_pdo_prepare($this->conn, $sql, 1);
        if ($_stmt < 0) {
            return 0;
        }
        $_value = 0;
        if (elephc_pdo_step($_stmt) == 1) {
            $_value = elephc_pdo_column_int($_stmt, 0);
        }
        elephc_pdo_finalize($_stmt);
        return $_value;
    }

    // Runs a one-row one-column SELECT and returns its text value ("" on any
    // failure). Length-counted copy so embedded NUL bytes would survive.
    private function fetchStringScalar(string $sql): string {
        $_stmt = elephc_pdo_prepare($this->conn, $sql, 1);
        if ($_stmt < 0) {
            return "";
        }
        $_value = "";
        if (elephc_pdo_step($_stmt) == 1) {
            $_len = elephc_pdo_column_data_len($_stmt, 0);
            if ($_len > 0) {
                $_value = __elephc_ptr_read_string(elephc_pdo_column_data_ptr($_stmt, 0), $_len);
            }
        }
        elephc_pdo_finalize($_stmt);
        return $_value;
    }

    // "8.0.36-log" -> 80036, php-src's major*10000 + minor*100 + patch. The
    // bridge's client string is "mysql x.y.z"; strip that prefix first.
    private function versionStringToInt(string $version): int {
        if (str_starts_with($version, "mysql ")) {
            $version = substr($version, 6);
        }
        $_parts = explode(".", $version);
        $_major = (int) $_parts[0];
        $_minor = count($_parts) > 1 ? (int) $_parts[1] : 0;
        $_patch = count($_parts) > 2 ? (int) $_parts[2] : 0;
        return $_major * 10000 + $_minor * 100 + $_patch;
    }

    // [A-Za-z0-9_] identifier filter shared by set_charset, same rule the PDO
    // DSN charset key enforces bridge-side.
    private function charsetIdentIsValid(string $charset): bool {
        $_len = strlen($charset);
        if ($_len == 0) {
            return false;
        }
        for ($_i = 0; $_i < $_len; $_i++) {
            $_c = ord(substr($charset, $_i, 1));
            $_ok = ($_c >= 48 && $_c <= 57) || ($_c >= 65 && $_c <= 90) || ($_c >= 97 && $_c <= 122) || $_c == 95;
            if (!$_ok) {
                return false;
            }
        }
        return true;
    }
}
"#;
