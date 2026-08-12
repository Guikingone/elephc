//! Purpose:
//! The locked v1 `MYSQLI_*` constant surface, as an elephc-PHP source fragment.
//! Only the constants the v1 subset actually consumes are declared, so
//! `defined('MYSQLI_…')` stays honest about what this build supports.
//!
//! Called from:
//! - `crate::mysqli_prelude::fragments::source_for_version` (concatenated into
//!   the injected prelude).
//!
//! Key details:
//! - Values match php-src's mysqli exactly (fetch modes, report flags, client
//!   flags, option ids, transaction flags).
//! - `MYSQLI_CLIENT_SSL` is declared but rejected at connect time with a clear
//!   error: elephc's mysqli does not support the SSL client flag (documented
//!   divergence; PDO MySQL TLS attributes are the supported TLS path).

/// The `MYSQLI_*` constant declarations (fragment without a `<?php` header).
pub(super) const SRC: &str = r#"
// -- mysqli constants (locked v1 subset; values match php-src) --

// Fetch modes for mysqli_result::fetch_array / fetch_all.
const MYSQLI_ASSOC = 1;
const MYSQLI_NUM = 2;
const MYSQLI_BOTH = 3;

// Result modes for mysqli::query. MYSQLI_USE_RESULT is accepted but still
// buffered (documented divergence: true unbuffered use_result is out of scope).
const MYSQLI_STORE_RESULT = 0;
const MYSQLI_USE_RESULT = 1;

// mysqli_report() flags. PHP 8.1+ defaults to ERROR|STRICT; PHP 8.0 to OFF.
const MYSQLI_REPORT_OFF = 0;
const MYSQLI_REPORT_ERROR = 1;
const MYSQLI_REPORT_STRICT = 2;
const MYSQLI_REPORT_INDEX = 4;
const MYSQLI_REPORT_ALL = 255;

// real_connect() client flags. MYSQLI_CLIENT_SSL is declared so programs that
// pass it compile, but connect rejects it with a clear error (no TLS support
// in elephc's mysqli surface).
const MYSQLI_CLIENT_COMPRESS = 32;
const MYSQLI_CLIENT_FOUND_ROWS = 2;
const MYSQLI_CLIENT_IGNORE_SPACE = 256;
const MYSQLI_CLIENT_INTERACTIVE = 1024;
const MYSQLI_CLIENT_SSL = 2048;

// mysqli_options() option ids (the three the v1 subset honors).
const MYSQLI_OPT_CONNECT_TIMEOUT = 0;
const MYSQLI_INIT_COMMAND = 3;
const MYSQLI_SET_CHARSET_NAME = 7;

// begin_transaction() flags.
const MYSQLI_TRANS_START_WITH_CONSISTENT_SNAPSHOT = 1;
const MYSQLI_TRANS_START_READ_WRITE = 2;
const MYSQLI_TRANS_START_READ_ONLY = 4;
"#;
