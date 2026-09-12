//! Purpose:
//! The OPcache accelerator's own diagnostic channel — php-src's `zend_accel_error`.
//! It is NOT PHP's error reporting: it writes a timestamped, pid-tagged line to
//! stderr (or to `opcache.error_log`), it is gated by `opcache.log_verbosity_level`
//! rather than by `error_reporting`, and a FATAL exits the process outright.
//!
//! Called from:
//! - `crate::script_cache::store` for the file-cache directory refusal.
//! - `crate::ffi::context::__elephc_eval_configure_opcache()` to install the gate.
//!
//! Key details:
//! - The line shape is `zend_accelerator_debug.c`'s, reproduced field for field:
//!   `asctime(localtime(t))` truncated at 24 bytes, ` (<pid>): `, a level word with a
//!   TRAILING SPACE (`Fatal Error `, `Error `, `Warning `, `Message `, `Debug `), the
//!   message, a newline, flushed. VERIFIED against reference PHP 8.5.6:
//!   `Sat Sep 12 08:33:47 2026 (79737): Fatal Error opcache.file_cache must be a full
//!   path of an accessible directory`.
//! - `localtime`/`asctime` come from libc, the same two functions php-src calls, so
//!   the timestamp cannot drift from reference's spelling in any locale or zone.
//! - The gate is `level <= log_verbosity_level`, and the DEFAULT level is 1. FATAL(0)
//!   and ERROR(1) therefore always print; WARNING(2) and below only with an explicit
//!   raise — which is why reference PHP looks silent about `opcache.preload_user`
//!   until you pass `-d opcache.log_verbosity_level=2` (VERIFIED both ways).
//! - Error handling happens EVEN WHEN THE LINE IS NOT LOGGED: php-src runs the
//!   `switch (type)` outside the verbosity guard, so a FATAL at verbosity 0 still
//!   exits. `exit(-2)` is an exit STATUS of 254, which is what reference returns.

use std::cell::RefCell;
use std::ffi::CStr;
use std::io::Write;

/// php-src's `ACCEL_LOG_*` levels, with their exact numeric values: the gate is a
/// `<=` against `opcache.log_verbosity_level`, so the ORDER and the NUMBERS are the
/// contract, not an internal detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum AccelLogLevel {
    /// Logged at every verbosity, and exits the process with status 254.
    Fatal = 0,
    /// Logged at every verbosity.
    #[allow(dead_code)]
    Error = 1,
    /// Needs `opcache.log_verbosity_level >= 2`.
    #[allow(dead_code)]
    Warning = 2,
    /// Needs `>= 3`. Spelled `Message ` in the output, not `Info `.
    #[allow(dead_code)]
    Info = 3,
    /// Needs `>= 4`.
    #[allow(dead_code)]
    Debug = 4,
}

impl AccelLogLevel {
    /// The level word php-src prints, INCLUDING its trailing space.
    const fn label(self) -> &'static str {
        match self {
            Self::Fatal => "Fatal Error ",
            Self::Error => "Error ",
            Self::Warning => "Warning ",
            Self::Info => "Message ",
            Self::Debug => "Debug ",
        }
    }
}

/// The two directives that decide where an accelerator diagnostic goes and whether it
/// is written at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccelLogConfig {
    /// `opcache.log_verbosity_level`. php-src's default is 1.
    pub(crate) verbosity: i32,
    /// `opcache.error_log`. Empty, or the literal `stderr`, means stderr — php-src
    /// tests all three of those cases before opening a file.
    pub(crate) error_log: String,
}

impl AccelLogConfig {
    /// The configuration a binary without generated OPcache wiring observes: php-src's
    /// own defaults, so a harness linking this archive directly behaves like reference.
    pub(crate) const fn defaults() -> Self {
        Self {
            verbosity: 1,
            error_log: String::new(),
        }
    }
}

thread_local! {
    static ACCEL_LOG_CONFIG: RefCell<AccelLogConfig> = RefCell::new(AccelLogConfig::defaults());
}

/// Installs the compile-time accelerator log configuration for the current thread.
pub(crate) fn set_config(config: AccelLogConfig) {
    ACCEL_LOG_CONFIG.with(|cell| *cell.borrow_mut() = config);
}

/// Returns a copy of the configuration active on the current thread.
pub(crate) fn config() -> AccelLogConfig {
    ACCEL_LOG_CONFIG.with(|cell| cell.borrow().clone())
}

/// Emits one accelerator diagnostic, then performs the level's own error handling.
///
/// Mirrors `zend_accel_error_va_args`: the write is gated by the verbosity, the error
/// handling is NOT. A `Fatal` therefore terminates the process whether or not anything
/// was printed — with status 254, php-src's `exit(-2)` as the shell sees it.
pub(crate) fn accel_error(level: AccelLogLevel, message: &str) -> ! {
    accel_log(level, message);
    // Only `Fatal` reaches this function's `!` return; every other level goes through
    // `accel_log` directly. Keeping the exit here rather than inside `accel_log` is what
    // lets the non-fatal levels return normally.
    std::process::exit(254);
}

/// Emits one accelerator diagnostic and returns, doing no error handling.
///
/// Silently does nothing when `level` is above the configured verbosity. A log file that
/// cannot be opened falls back to stderr rather than being dropped, exactly as php-src does.
pub(crate) fn accel_log(level: AccelLogLevel, message: &str) {
    let config = config();
    if (level as i32) > config.verbosity {
        return;
    }
    let line = format!(
        "{} ({}): {}{}\n",
        local_time_string(),
        std::process::id(),
        level.label(),
        message
    );
    if config.error_log.is_empty() || config.error_log == "stderr" {
        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(line.as_bytes());
        let _ = stderr.flush();
        return;
    }
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.error_log)
    {
        Ok(mut file) => {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        Err(_) => {
            // php-src falls back to stderr on a failed `fopen` rather than losing the line.
            let mut stderr = std::io::stderr();
            let _ = stderr.write_all(line.as_bytes());
            let _ = stderr.flush();
        }
    }
}

/// Returns the 24-character local timestamp php-src prints.
///
/// `asctime(localtime(&t))` with `time_string[24] = 0` — that is the fixed-width C form
/// `Sat Sep 12 08:33:47 2026`, with the trailing newline `asctime` appends chopped off by
/// the truncation. Calling the same two libc functions rather than formatting by hand is
/// what guarantees the day and month abbreviations, the space-padded day of month and the
/// local zone offset all match reference byte for byte.
fn local_time_string() -> String {
    // SAFETY: `time` accepts a null pointer and returns the timestamp; `localtime` returns a
    // pointer to a static `struct tm` and `asctime` a pointer to a static buffer of at least
    // 26 bytes. Both statics are libc's own and are read before any other libc call on this
    // thread can overwrite them. Either can answer null (a timestamp `localtime` cannot
    // represent), which is why both are checked rather than dereferenced blind.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let tm = libc::localtime(&now);
        if tm.is_null() {
            return String::new();
        }
        let text = libc::asctime(tm);
        if text.is_null() {
            return String::new();
        }
        let bytes = CStr::from_ptr(text).to_bytes();
        String::from_utf8_lossy(&bytes[..bytes.len().min(24)]).into_owned()
    }
}
