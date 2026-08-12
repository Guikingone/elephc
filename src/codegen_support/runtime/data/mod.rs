//! Purpose:
//! Collects runtime data-section emitters and shared diagnostic string constants.
//! The module separates cacheable fixed data from user-program metadata emitted during compilation.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emit_runtime_data_fixed()` and `crate::codegen_support::runtime::emit_runtime_data_user()`.
//!
//! Key details:
//! - Symbol names and table layouts are link-time ABI shared with generated code and runtime helper labels.

mod fixed;
/// Also home of `escaped_bytes()`, the crate's single assembler-string escaper:
/// reachable outside this module so non-runtime emitters (`crate::debug_info`)
/// escape quoted directive operands the same way.
pub(crate) mod instanceof;
mod user;

pub(crate) use fixed::emit_runtime_data_fixed;
/// Emit fixed runtime data section (heap globals, fatal/assertion messages, lookup tables, builtin callable metadata).
pub(crate) use user::emit_runtime_data_user;
pub(crate) use user::{is_user_filter_contract_method, is_user_wrapper_contract_method};

/// Fatal error message when `php_uname()` receives a `$mode` argument whose length is not exactly 1.
pub(crate) const PHP_UNAME_MODE_LEN_MSG: &str =
    "Fatal error: php_uname(): Argument #1 ($mode) must be a single character\n";
/// Fatal error message when `php_uname()` receives a `$mode` argument that is not one of the supported single-character values.
pub(crate) const PHP_UNAME_MODE_VALUE_MSG: &str =
    "Fatal error: php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\"\n";
/// Fatal error message when `dirname()` receives a `$levels` argument less than 1.
/// ob_* PHP-parity diagnostics shared by the fixed data section and the
/// output-buffering runtime emitters (which need the exact byte lengths).
/// PHP's Notice when `stream_wrapper_restore()` is given a built-in scheme that was never
/// unregistered. Completed at run time with the scheme name and the suffix below.
pub(crate) const SWR_NTC_PREFIX: &str = "Notice: stream_wrapper_restore(): ";

/// PHP's Warning when `stream_wrapper_restore()` is given a scheme that never existed.
pub(crate) const SWR_WRN_PREFIX: &str = "Warning: stream_wrapper_restore(): ";

/// Tail of the never-unregistered Notice, after the scheme name.
pub(crate) const SWR_NEVER_CHANGED: &str = ":// was never changed, nothing to restore\n";

/// Tail of the unknown-scheme Warning, after the scheme name.
pub(crate) const SWR_NEVER_EXISTED: &str = ":// never existed, nothing to restore\n";

/// PHP's Warning when a socket builtin cannot open its endpoint. The address and the reason are
/// only known at run time, so each of these is a prefix the runtime completes.
/// PHP uses the same "Unable to connect to" wording for a failed bind, which reads oddly but is
/// what `stream_socket_server()` prints.
pub(crate) const SOCKET_FAILED_CLIENT_PREFIX: &str = "Warning: stream_socket_client(): ";

/// The `stream_socket_server()` form of [`SOCKET_FAILED_CLIENT_PREFIX`].
pub(crate) const SOCKET_FAILED_SERVER_PREFIX: &str = "Warning: stream_socket_server(): ";

/// The `fsockopen()` form of [`SOCKET_FAILED_CLIENT_PREFIX`]. Its address is a bare host, so the
/// runtime appends `:` and the port to reach PHP's `host:port` spelling.
pub(crate) const SOCKET_FAILED_FSOCKOPEN_PREFIX: &str = "Warning: fsockopen(): ";

/// Head of the connect-failure line, after the `Warning: <fn>(): ` the three prefixes carry.
pub(crate) const SOCKET_FAILED_UNABLE: &str = "Unable to connect to ";

/// Head of the message PHP reports when a socket address names a host that does not resolve.
///
/// php-src composes this itself rather than using an `errno`, which is why the `&$error_code` of
/// such a failure is `0` and only `&$error_message` says anything.
pub(crate) const GAI_MSG_PREFIX: &str = "php_network_getaddresses: getaddrinfo for ";

/// Middle of the unresolvable-host message, between the host name and the resolver's own text.
pub(crate) const GAI_MSG_MIDDLE: &str = " failed: ";

/// Bytes reserved for the composed unresolvable-host message.
///
/// Prefix, a host clamped to the 255 bytes a DNS name can hold, the middle, and the resolver text
/// clamped to 200 all fit. The buffer is static so the message the caller receives needs no
/// allocation and cannot dangle; a second failed resolution overwrites it.
pub(crate) const SOCKET_GAI_MSG_CAPACITY: usize = 512;

/// Longest host name copied into the composed message.
pub(crate) const SOCKET_GAI_HOST_CLAMP: i64 = 255;

/// Longest resolver text copied into the composed message.
pub(crate) const SOCKET_GAI_REASON_CLAMP: i64 = 200;

/// Separator between the socket warning's address and the reason for the failure.
pub(crate) const SOCKET_FAILED_REASON_OPEN: &str = " (";

/// What the warning says when the failure carries no reason at all.
///
/// php-src prints `errstr == NULL ? "Unknown error" : errstr`, so a failure no syscall described —
/// a datagram transport asked to listen, for one — still names something inside the parentheses.
/// Only the warning substitutes it: the caller's `&$error_message` stays the empty string PHP
/// leaves there, so this cannot come from `__rt_socket_strerror`, which feeds both.
pub(crate) const SOCKET_FAILED_REASON_UNKNOWN: &str = "Unknown error";

/// Tail of the socket warning, after the reason.
pub(crate) const SOCKET_FAILED_REASON_CLOSE: &str = ")\n";

/// The `count()` TypeError texts, indexed the way `__rt_count_reject_index` answers.
///
/// PHP names the type with the VALUE's own spelling — a boolean reports `true` or `false`, not
/// `bool` — so these are read off `php -n` 8.5.6 rather than derived from the tag names. A
/// non-`Countable` object is absent on purpose: PHP names the class there, which needs a lookup
/// and a composed string.
pub(crate) const COUNT_TYPE_ERROR_MESSAGES: [&str; 7] = [
    "count(): Argument #1 ($value) must be of type Countable|array, int given",
    "count(): Argument #1 ($value) must be of type Countable|array, string given",
    "count(): Argument #1 ($value) must be of type Countable|array, float given",
    "count(): Argument #1 ($value) must be of type Countable|array, true given",
    "count(): Argument #1 ($value) must be of type Countable|array, false given",
    "count(): Argument #1 ($value) must be of type Countable|array, null given",
    "count(): Argument #1 ($value) must be of type Countable|array, resource given",
];

/// A lone newline, for a diagnostic written out in fragments.
pub(crate) const DIAG_NEWLINE: &str = "\n";

/// Head of the warning `disk_free_space()` prints for a path it cannot stat.
///
/// PHP names the function and the reason and nothing else here — no path, unlike the failed-open
/// warning — so the runtime only has to append `strerror` and a newline.
pub(crate) const DISK_FREE_SPACE_WARNING: &str = "Warning: disk_free_space(): ";

/// The `disk_total_space()` form of [`DISK_FREE_SPACE_WARNING`].
pub(crate) const DISK_TOTAL_SPACE_WARNING: &str = "Warning: disk_total_space(): ";

pub(crate) const OB_NTC_NO_END_FLUSH: &str =
    "Notice: ob_end_flush(): Failed to delete and flush buffer. No buffer to delete or flush\n";
/// ob_get_flush() no-buffer notice line.
pub(crate) const OB_NTC_NO_GET_FLUSH: &str =
    "Notice: ob_get_flush(): Failed to delete and flush buffer. No buffer to delete or flush\n";
/// ob_end_clean() no-buffer notice line.
pub(crate) const OB_NTC_NO_END_CLEAN: &str =
    "Notice: ob_end_clean(): Failed to delete buffer. No buffer to delete\n";
/// ob_flush() no-buffer notice line.
pub(crate) const OB_NTC_NO_FLUSH: &str =
    "Notice: ob_flush(): Failed to flush buffer. No buffer to flush\n";
/// ob_clean() no-buffer notice line.
pub(crate) const OB_NTC_NO_CLEAN: &str =
    "Notice: ob_clean(): Failed to delete buffer. No buffer to delete\n";
/// ob_clean() flags-gated notice prefix (completed with "NAME (LEVEL)\n").
pub(crate) const OB_NTC_G_CLEAN: &str = "Notice: ob_clean(): Failed to delete buffer of ";
/// ob_flush() flags-gated notice prefix.
pub(crate) const OB_NTC_G_FLUSH: &str = "Notice: ob_flush(): Failed to flush buffer of ";
/// ob_end_clean() flags-gated notice prefix.
pub(crate) const OB_NTC_G_END_CLEAN: &str =
    "Notice: ob_end_clean(): Failed to discard buffer of ";
/// ob_get_clean() flags-gated notice prefix.
pub(crate) const OB_NTC_G_GET_CLEAN: &str =
    "Notice: ob_get_clean(): Failed to discard buffer of ";
/// ob_end_flush() flags-gated notice prefix.
pub(crate) const OB_NTC_G_END_FLUSH: &str =
    "Notice: ob_end_flush(): Failed to send buffer of ";
/// ob_get_flush() flags-gated notice prefix.
pub(crate) const OB_NTC_G_GET_FLUSH: &str =
    "Notice: ob_get_flush(): Failed to send buffer of ";
/// ob_start() invalid-callback warning prefix (completed with the name + suffix).
pub(crate) const OB_WARN_BAD_CALLBACK_PREFIX: &str = "Warning: ob_start(): function \"";
/// ob_start() invalid-callback warning suffix.
pub(crate) const OB_WARN_BAD_CALLBACK_SUFFIX: &str =
    "\" not found or invalid function name\n";
/// ob_start() invalid-callback warning for non-string, non-callable values.
pub(crate) const OB_WARN_BAD_CALLBACK_GENERIC: &str =
    "Warning: ob_start(): no array or string given\n";
/// ob_start() failed-create notice line.
pub(crate) const OB_NTC_CREATE_FAIL: &str = "Notice: ob_start(): Failed to create buffer\n";
/// ob_start()-inside-a-handler fatal line.
pub(crate) const OB_FATAL_IN_HANDLER: &str =
    "Fatal error: ob_start(): Cannot use output buffering in output buffering display handlers\n";
/// PHP's default output-handler display name.
pub(crate) const OB_DEFAULT_HANDLER_NAME: &str = "default output handler";
/// PHP's closure / first-class-callable handler display name.
pub(crate) const OB_CLOSURE_INVOKE_NAME: &str = "Closure::__invoke";

pub(crate) const DIRNAME_LEVELS_MSG: &str =
    "Fatal error: dirname(): Argument #2 ($levels) must be greater than or equal to 1\n";
/// Fatal error message written by `__rt_stack_overflow` when a function prologue finds the
/// stack pointer below `_stack_limit`. PHP 8.3+ reports the same condition as
/// `Fatal error: Uncaught Error: Maximum call stack size of N bytes
/// (zend.max_allowed_stack_size - zend.reserved_stack_size) reached. Infinite recursion?`;
/// elephc has no per-call-site context in the fatal path (it is entered with almost no
/// stack left), so it reports the same condition without the byte count and location.
pub(crate) const STACK_OVERFLOW_MSG: &str =
    "Fatal error: Maximum call stack size reached. Infinite recursion?\n";
/// Fatal error message when an array allocation request cannot be sized safely,
/// i.e. `capacity * elem_size` does not fit in the machine word. PHP reports the
/// same class of failure as a `ValueError` naming the offending argument
/// (`array_fill(): Argument #2 ($count) is too large`); elephc's runtime has no
/// per-call-site context inside `__rt_array_new`, so it reports the shared cause.
pub(crate) const ARRAY_ALLOC_SIZE_MSG: &str =
    "Fatal error: requested array size exceeds the maximum allowed array size\n";
/// Fatal error message when `range()` cannot represent the requested interval,
/// because `end - start + 1` overflows a signed 64-bit element count. Matches
/// PHP's `ValueError: The supplied range exceeds the maximum array size`.
pub(crate) const RANGE_SIZE_MSG: &str =
    "Fatal error: The supplied range exceeds the maximum array size\n";
/// Fatal error message when `buffer_new<T>()` receives a negative length or a
/// length whose `len * stride` payload size does not fit in the machine word.
/// `buffer_new` is an elephc extension with no PHP equivalent, so the wording is
/// elephc's own rather than a PHP parity string.
pub(crate) const BUFFER_ALLOC_SIZE_MSG: &str =
    "Fatal error: buffer_new() length is negative or exceeds the maximum buffer size\n";
/// Fatal error message when a runtime string producer is asked for a result whose byte
/// count cannot be allocated: either the size computation itself wrapped (`str_repeat()`'s
/// `len * times`, an encoder's `2 * len` / `3 * len` expansion) or the requested size
/// exceeds the configured heap capacity. PHP reports the same class of failure as
/// `Fatal error: Possible integer overflow in memory allocation (...)`; elephc's runtime
/// has no per-call-site operand context, so it reports the shared cause.
pub(crate) const ALLOC_OVERFLOW_MSG: &str =
    "Fatal error: Possible integer overflow in memory allocation\n";
/// Fatal error message when `str_repeat()` receives a `$times` argument less than 0.
pub(crate) const STR_REPEAT_TIMES_MSG: &str =
    "Fatal error: str_repeat(): Argument #2 ($times) must be greater than or equal to 0\n";
/// Fatal error message when a `printf`-family conversion requests a field width outside
/// PHP's accepted range. PHP raises `ValueError: Width must be between 0 and 2147483647`;
/// elephc has no catchable-error path inside `__rt_sprintf`, so it reports the same text
/// as a controlled fatal instead of writing past the conversion buffer.
pub(crate) const SPRINTF_WIDTH_MSG: &str =
    "Fatal error: Uncaught ValueError: Width must be between 0 and 2147483647\n";
/// Fatal error message when a `printf`-family conversion would write past the shared
/// 64 KiB `_concat_buf` result arena. PHP grows its result buffer on the heap; elephc's
/// formatted results live in the fixed concat arena, so an oversized result is reported
/// instead of overrunning the arena.
pub(crate) const SPRINTF_OVERFLOW_MSG: &str =
    "Fatal error: sprintf(): formatted result exceeds the 65536-byte string buffer\n";
/// Fatal error message when a `printf`-family format string consumes more arguments than
/// were supplied. PHP raises `ArgumentCountError`; elephc reports the same class of error
/// as a controlled fatal because the alternative is reading past the pushed argument records.
pub(crate) const SPRINTF_ARGCOUNT_MSG: &str =
    "Fatal error: Uncaught ArgumentCountError: sprintf(): too few arguments\n";
/// Fatal error message when a `printf`-family format string uses a conversion character
/// PHP does not define. The runtime never forwards an unrecognized conversion to libc
/// `snprintf` (that would expose `%n` and friends), so it reports PHP's `ValueError` instead.
pub(crate) const SPRINTF_UNKNOWN_SPEC_MSG: &str =
    "Fatal error: Uncaught ValueError: Unknown format specifier\n";
/// Catchable `\ValueError` message when `hash()` receives an unknown algorithm name.
pub(crate) const HASH_UNKNOWN_ALGO_MSG: &str =
    "hash(): Argument #1 ($algo) must be a valid hashing algorithm";
/// Catchable `\ValueError` message when `hash_init()` receives an unknown algorithm name.
pub(crate) const HASH_INIT_UNKNOWN_ALGO_MSG: &str =
    "hash_init(): Argument #1 ($algo) must be a valid hashing algorithm";
/// Catchable `\ValueError` message when `hash_hmac()` receives an unknown algorithm
/// name or a non-cryptographic checksum (PHP rejects HMAC over crc32/adler/fnv/joaat).
pub(crate) const HASH_HMAC_UNKNOWN_ALGO_MSG: &str =
    "hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm";
/// Catchable `\TypeError` message when `hash_update()` is handed a HashContext that
/// `hash_final()` already consumed. Captured verbatim from PHP 8.5.6
/// (`php -d xdebug.mode=off`); PHP words all three as one sentence naming the callee.
pub(crate) const HASH_UPDATE_FINALIZED_CTX_MSG: &str =
    "hash_update(): Argument #1 ($context) must be a valid, non-finalized HashContext";
/// Catchable `\TypeError` message for a second `hash_final()` on the same context.
pub(crate) const HASH_FINAL_FINALIZED_CTX_MSG: &str =
    "hash_final(): Argument #1 ($context) must be a valid, non-finalized HashContext";
/// Catchable `\TypeError` message for `hash_copy()` of an already-finalized context.
pub(crate) const HASH_COPY_FINALIZED_CTX_MSG: &str =
    "hash_copy(): Argument #1 ($context) must be a valid, non-finalized HashContext";
/// Catchable `\ValueError` message when `mb_strlen()` receives an unknown encoding name.
pub(crate) const MB_STRLEN_UNKNOWN_ENCODING_MSG: &str =
    "mb_strlen(): Argument #2 ($encoding) must be a valid encoding";
