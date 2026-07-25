//! Purpose:
//! Defines PHP stream / I/O-adjacent constants exposed as integer constants.
//! Single source of truth for `STREAM_*`, `PSFS_*`, `FILE_*`, `FILEINFO_*`, output-buffer,
//! INI-scanner, and `GLOB_*` values.
//!
//! Called from:
//! - `crate::types::checker::driver::init` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//! - `crate::name_resolver::names` when recognizing builtin constant names.
//!
//! Key details:
//! - Values must match PHP 8.x exactly (`php -r 'echo CONST;'`) for parity.
//! - Only target-INVARIANT constants live here. `LOCK_*` and `FNM_*` are
//!   registered elsewhere (and `FNM_*` is target-sensitive). `STREAM_PF_INET6`
//!   is target-divergent (AF_INET6: 30 on macOS, 10 on Linux) and is registered
//!   target-sensitively when the socket layer lands.
//! - `GLOB_ERR`/`GLOB_MARK`/`GLOB_NOCHECK`/`GLOB_NOSORT`/`GLOB_BRACE`/
//!   `GLOB_NOESCAPE` are native libc `glob()` bit flags forwarded straight into
//!   the `glob()` syscall wrapper — NOT POSIX-portable (contrary to a prior
//!   comment here): BSD/Darwin (macOS) and glibc (Linux) assign different bit
//!   positions to the same flag name. They live in `GLOB_PLATFORM_CONSTANTS`
//!   below, mirroring `crate::types::pcntl_constants::PCNTL_PLATFORM_SIGNALS`.
//!   `GLOB_ONLYDIR` is the one exception: PHP defines it as its OWN portable
//!   sentinel (`1 << 30`) on every platform — libc's native `GLOB_ONLYDIR` hint
//!   (where one exists) is documented as unreliable, so PHP (and elephc) always
//!   strip the bit before calling `glob()` and post-filter matches with `stat()`
//!   instead. That is why `GLOB_ONLYDIR` stays in the flat, target-invariant
//!   `STREAM_INT_CONSTANTS` table below.

pub(crate) const STREAM_INT_CONSTANTS: &[(&str, i64)] = &[
    // Core I/O-adjacent extension flags.
    ("FILEINFO_MIME_TYPE", 16),
    ("INI_SCANNER_RAW", 1),
    ("PHP_OUTPUT_HANDLER_CLEANABLE", 16),
    ("PHP_OUTPUT_HANDLER_FLUSHABLE", 32),
    ("PHP_OUTPUT_HANDLER_REMOVABLE", 64),
    // Client / server connection flags.
    ("STREAM_CLIENT_PERSISTENT", 1),
    ("STREAM_CLIENT_ASYNC_CONNECT", 2),
    ("STREAM_CLIENT_CONNECT", 4),
    ("STREAM_SERVER_BIND", 4),
    ("STREAM_SERVER_LISTEN", 8),
    // Shutdown directions for stream_socket_shutdown().
    ("STREAM_SHUT_RD", 0),
    ("STREAM_SHUT_WR", 1),
    ("STREAM_SHUT_RDWR", 2),
    // Out-of-band / peek flags for stream_socket_recvfrom().
    ("STREAM_OOB", 1),
    ("STREAM_PEEK", 2),
    // Stream filter chain direction.
    ("STREAM_FILTER_READ", 1),
    ("STREAM_FILTER_WRITE", 2),
    ("STREAM_FILTER_ALL", 3),
    // TLS crypto methods (client side).
    ("STREAM_CRYPTO_METHOD_SSLv2_CLIENT", 3),
    ("STREAM_CRYPTO_METHOD_SSLv3_CLIENT", 5),
    ("STREAM_CRYPTO_METHOD_SSLv23_CLIENT", 57),
    ("STREAM_CRYPTO_METHOD_TLS_CLIENT", 121),
    ("STREAM_CRYPTO_METHOD_TLSv1_0_CLIENT", 9),
    ("STREAM_CRYPTO_METHOD_TLSv1_1_CLIENT", 17),
    ("STREAM_CRYPTO_METHOD_TLSv1_2_CLIENT", 33),
    ("STREAM_CRYPTO_METHOD_TLSv1_3_CLIENT", 65),
    ("STREAM_CRYPTO_METHOD_ANY_CLIENT", 127),
    // TLS crypto methods (server side).
    ("STREAM_CRYPTO_METHOD_SSLv2_SERVER", 2),
    ("STREAM_CRYPTO_METHOD_SSLv3_SERVER", 4),
    ("STREAM_CRYPTO_METHOD_SSLv23_SERVER", 120),
    ("STREAM_CRYPTO_METHOD_TLS_SERVER", 120),
    ("STREAM_CRYPTO_METHOD_TLSv1_0_SERVER", 8),
    ("STREAM_CRYPTO_METHOD_TLSv1_1_SERVER", 16),
    ("STREAM_CRYPTO_METHOD_TLSv1_2_SERVER", 32),
    ("STREAM_CRYPTO_METHOD_TLSv1_3_SERVER", 64),
    ("STREAM_CRYPTO_METHOD_ANY_SERVER", 126),
    // TLS crypto protocol aliases.
    ("STREAM_CRYPTO_PROTO_SSLv3", 4),
    ("STREAM_CRYPTO_PROTO_TLSv1_0", 8),
    ("STREAM_CRYPTO_PROTO_TLSv1_1", 16),
    ("STREAM_CRYPTO_PROTO_TLSv1_2", 32),
    ("STREAM_CRYPTO_PROTO_TLSv1_3", 64),
    // Socket-pair domain / type / protocol (target-invariant values only;
    // STREAM_PF_INET6 is target-divergent and registered with the socket layer).
    ("STREAM_PF_INET", 2),
    ("STREAM_PF_UNIX", 1),
    ("STREAM_SOCK_STREAM", 1),
    ("STREAM_SOCK_DGRAM", 2),
    ("STREAM_SOCK_RAW", 3),
    ("STREAM_SOCK_RDM", 4),
    ("STREAM_SOCK_SEQPACKET", 5),
    ("STREAM_IPPROTO_IP", 0),
    ("STREAM_IPPROTO_TCP", 6),
    ("STREAM_IPPROTO_UDP", 17),
    ("STREAM_IPPROTO_ICMP", 1),
    ("STREAM_IPPROTO_RAW", 255),
    // Notification codes / severities for stream context notifiers.
    ("STREAM_NOTIFY_RESOLVE", 1),
    ("STREAM_NOTIFY_CONNECT", 2),
    ("STREAM_NOTIFY_AUTH_REQUIRED", 3),
    ("STREAM_NOTIFY_MIME_TYPE_IS", 4),
    ("STREAM_NOTIFY_FILE_SIZE_IS", 5),
    ("STREAM_NOTIFY_REDIRECTED", 6),
    ("STREAM_NOTIFY_PROGRESS", 7),
    ("STREAM_NOTIFY_COMPLETED", 8),
    ("STREAM_NOTIFY_FAILURE", 9),
    ("STREAM_NOTIFY_AUTH_RESULT", 10),
    ("STREAM_NOTIFY_SEVERITY_INFO", 0),
    ("STREAM_NOTIFY_SEVERITY_WARN", 1),
    ("STREAM_NOTIFY_SEVERITY_ERR", 2),
    // Wrapper / cast / meta / option / buffer / URL-stat flags.
    ("STREAM_IS_URL", 1),
    ("STREAM_USE_PATH", 1),
    ("STREAM_REPORT_ERRORS", 8),
    ("STREAM_CAST_FOR_SELECT", 3),
    ("STREAM_CAST_AS_STREAM", 0),
    ("STREAM_META_TOUCH", 1),
    ("STREAM_META_OWNER_NAME", 2),
    ("STREAM_META_OWNER", 3),
    ("STREAM_META_GROUP_NAME", 4),
    ("STREAM_META_GROUP", 5),
    ("STREAM_META_ACCESS", 6),
    ("STREAM_MKDIR_RECURSIVE", 1),
    ("STREAM_OPTION_BLOCKING", 1),
    ("STREAM_OPTION_READ_BUFFER", 2),
    ("STREAM_OPTION_WRITE_BUFFER", 3),
    ("STREAM_OPTION_READ_TIMEOUT", 4),
    ("STREAM_BUFFER_NONE", 0),
    ("STREAM_BUFFER_LINE", 1),
    ("STREAM_BUFFER_FULL", 2),
    ("STREAM_URL_STAT_LINK", 1),
    ("STREAM_URL_STAT_QUIET", 2),
    ("STREAM_MUST_SEEK", 16),
    ("STREAM_IGNORE_URL", 2),
    // User stream-filter return values / flags.
    ("PSFS_ERR_FATAL", 0),
    ("PSFS_FEED_ME", 1),
    ("PSFS_PASS_ON", 2),
    ("PSFS_FLAG_NORMAL", 0),
    ("PSFS_FLAG_FLUSH_INC", 1),
    ("PSFS_FLAG_FLUSH_CLOSE", 2),
    // file() / file_put_contents() flags.
    ("FILE_USE_INCLUDE_PATH", 1),
    ("FILE_IGNORE_NEW_LINES", 2),
    ("FILE_SKIP_EMPTY_LINES", 4),
    ("FILE_APPEND", 8),
    ("FILE_NO_DEFAULT_CONTEXT", 16),
    // glob() ONLYDIR is PHP's own portable sentinel (elephc post-filters with
    // stat() instead of relying on libc's unreliable native ONLYDIR hint) — see
    // the module doc for why this one glob() flag is NOT in `GLOB_PLATFORM_CONSTANTS`.
    ("GLOB_ONLYDIR", 1073741824),
    // scandir() sort orders (php-verified, target-invariant — these are PHP's
    // own enum values, not forwarded to any libc call).
    ("SCANDIR_SORT_ASCENDING", 0),
    ("SCANDIR_SORT_DESCENDING", 1),
    ("SCANDIR_SORT_NONE", 2),
];

/// `(name, macos_value, linux_value)` for `glob()` bit flags forwarded directly
/// into libc `glob()`. BSD/Darwin (macOS) and glibc (Linux) `<glob.h>` assign
/// different bit positions to the same flag names, so these values differ by
/// compile target (selected from the `Platform` the same way as
/// `PHP_RUNTIME_PLATFORM_CONSTANTS`, not `cfg(target_os)`, since elephc
/// cross-compiles).
///
/// macOS values php-verified locally (`php -n -r 'echo GLOB_MARK,",",GLOB_NOSORT,",",GLOB_BRACE;'`
/// → `8,32,128`) and cross-checked against Darwin's `<glob.h>`. Linux values are
/// glibc's well-established, decades-stable `<glob.h>` bit positions
/// (`GLOB_ERR=1<<0`, `GLOB_MARK=1<<1`, `GLOB_NOSORT=1<<2`, `GLOB_NOCHECK=1<<4`,
/// `GLOB_NOESCAPE=1<<6`, `GLOB_BRACE=1<<10`, a GNU extension supported by both
/// glibc targets elephc ships — `linux-x86_64` and `linux-aarch64`).
pub(crate) const GLOB_PLATFORM_CONSTANTS: &[(&str, i64, i64)] = &[
    ("GLOB_ERR", 4, 1),
    ("GLOB_MARK", 8, 2),
    ("GLOB_NOSORT", 32, 4),
    ("GLOB_NOCHECK", 16, 16),
    ("GLOB_NOESCAPE", 4096, 64),
    ("GLOB_BRACE", 128, 1024),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the stream constant invariant for stream filter all is three.
    #[test]
    fn stream_filter_all_is_three() {
        let entry = STREAM_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "STREAM_FILTER_ALL")
            .expect("STREAM_FILTER_ALL defined");
        assert_eq!(entry.1, 3);
    }

    /// Verifies the stream constant invariant for stream client connect is four.
    #[test]
    fn stream_client_connect_is_four() {
        let entry = STREAM_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "STREAM_CLIENT_CONNECT")
            .expect("STREAM_CLIENT_CONNECT defined");
        assert_eq!(entry.1, 4);
    }

    /// Verifies Symfony-demanded I/O-adjacent flags against PHP 8.5.6.
    #[test]
    fn io_adjacent_flags_match_php() {
        let value_of = |name: &str| {
            STREAM_INT_CONSTANTS
                .iter()
                .find(|(constant_name, _)| *constant_name == name)
                .unwrap_or_else(|| panic!("{name} defined"))
                .1
        };
        assert_eq!(value_of("FILEINFO_MIME_TYPE"), 16);
        assert_eq!(value_of("INI_SCANNER_RAW"), 1);
        assert_eq!(value_of("PHP_OUTPUT_HANDLER_CLEANABLE"), 16);
        assert_eq!(value_of("PHP_OUTPUT_HANDLER_FLUSHABLE"), 32);
        assert_eq!(value_of("PHP_OUTPUT_HANDLER_REMOVABLE"), 64);
    }

    /// Verifies the stream constant invariant for no duplicate constant names.
    #[test]
    fn no_duplicate_constant_names() {
        let mut names: Vec<&str> = STREAM_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate stream constant name");
    }

    /// Verifies `GLOB_PLATFORM_CONSTANTS` has no duplicate names, and no overlap
    /// with the flat `STREAM_INT_CONSTANTS` table (in particular `GLOB_ONLYDIR`
    /// must stay a single portable value, not be duplicated here).
    #[test]
    fn glob_platform_constants_no_duplicates_and_no_overlap() {
        let mut names: Vec<&str> = GLOB_PLATFORM_CONSTANTS.iter().map(|(n, _, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(len_before, names.len(), "duplicate GLOB_PLATFORM_CONSTANTS name");
        for (name, _, _) in GLOB_PLATFORM_CONSTANTS {
            assert!(
                !STREAM_INT_CONSTANTS.iter().any(|(n, _)| n == name),
                "{name} must not be registered in both tables",
            );
        }
    }

    /// php-verified (macOS 8.5.6 local): `GLOB_MARK,GLOB_NOSORT,GLOB_BRACE` = `8,32,128`.
    #[test]
    fn glob_platform_constants_macos_values_match_php_verify() {
        let get = |name: &str| {
            GLOB_PLATFORM_CONSTANTS
                .iter()
                .find(|(n, _, _)| *n == name)
                .unwrap_or_else(|| panic!("{name} defined"))
                .1
        };
        assert_eq!(get("GLOB_MARK"), 8);
        assert_eq!(get("GLOB_NOSORT"), 32);
        assert_eq!(get("GLOB_BRACE"), 128);
        assert_eq!(get("GLOB_ERR"), 4);
        assert_eq!(get("GLOB_NOCHECK"), 16);
        assert_eq!(get("GLOB_NOESCAPE"), 4096);
    }

    /// Linux (glibc) `<glob.h>` bit positions differ from Darwin/BSD for these flags.
    #[test]
    fn glob_platform_constants_linux_values_are_glibc_bits() {
        let get = |name: &str| {
            GLOB_PLATFORM_CONSTANTS
                .iter()
                .find(|(n, _, _)| *n == name)
                .unwrap_or_else(|| panic!("{name} defined"))
                .2
        };
        assert_eq!(get("GLOB_MARK"), 2);
        assert_eq!(get("GLOB_NOSORT"), 4);
        assert_eq!(get("GLOB_BRACE"), 1024);
        assert_eq!(get("GLOB_ERR"), 1);
        assert_eq!(get("GLOB_NOCHECK"), 16);
        assert_eq!(get("GLOB_NOESCAPE"), 64);
    }

    /// Verifies the stream constant invariant for does not redeclare lock or fnmatch constants.
    #[test]
    fn does_not_redeclare_lock_or_fnmatch_constants() {
        // LOCK_* and FNM_* are registered elsewhere — keep them out of this table.
        for (name, _) in STREAM_INT_CONSTANTS {
            assert!(
                !name.starts_with("LOCK_") && !name.starts_with("FNM_"),
                "{name} must not be registered here",
            );
        }
    }
}
