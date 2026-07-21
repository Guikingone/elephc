//! Purpose:
//! Collects runtime data-section emitters and shared diagnostic string constants.
//! The module separates cacheable fixed data from user-program metadata emitted during compilation.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emit_runtime_data_fixed()` and `crate::codegen_support::runtime::emit_runtime_data_user()`.
//!
//! Key details:
//! - Symbol names and table layouts are link-time ABI shared with generated code and runtime helper labels.

mod class_relation_registry;
mod const_registry;
mod fixed;
mod instanceof;
mod user;

/// Emit the closed-world constant/enum registry data tables for runtime lookups.
pub(crate) use const_registry::emit_const_registry_data;
/// Emit the closed-world class/interface/trait registry data tables for runtime lookups.
pub(crate) use const_registry::emit_class_registry_data;
/// Emit the closed-world per-class relation payload registry backing non-literal
/// `class_implements()`/`class_parents()`/`class_uses()`.
pub(crate) use class_relation_registry::emit_class_relation_registry_data;
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
/// Fatal error message when `str_repeat()` receives a `$times` argument less than 0.
pub(crate) const STR_REPEAT_TIMES_MSG: &str =
    "Fatal error: str_repeat(): Argument #2 ($times) must be greater than or equal to 0\n";
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
/// Catchable `\ValueError` message when `mb_strlen()` receives an unknown encoding name.
pub(crate) const MB_STRLEN_UNKNOWN_ENCODING_MSG: &str =
    "mb_strlen(): Argument #2 ($encoding) must be a valid encoding";

/// Fatal error message when `print_r($v, true)`'s captured output would exceed the
/// fixed capture buffer (legacy capture-path diagnostic symbol, still emitted by `fixed`).
pub(crate) const PRINT_R_CAPTURE_OVERFLOW_MSG: &str =
    "Fatal error: print_r(): return-string capture exceeds the 1 MiB limit\n";
/// Fatal error message when `print_r($v, true)`'s nesting depth guard trips,
/// which bounds self-referential/cyclic array structures instead of looping forever.
pub(crate) const PRINT_R_CAPTURE_RECURSION_MSG: &str =
    "Fatal error: print_r(): possible infinite recursion (nesting too deep)\n";
/// PHP's own `E_WARNING` text (php-verified against PHP 8.5) when `Closure::bind()`/
/// `Closure::bindTo()` binds a non-null `$newThis` onto a `static` closure. `__rt_closure_bind`
/// emits this instead of rebinding and returns a null descriptor, matching PHP's `?Closure`
/// return type instead of silently ignoring the rejected bind or fataling.
pub(crate) const CLOSURE_BIND_STATIC_THIS_WARNING_MSG: &str =
    "Warning: Cannot bind an instance to a static closure, this will be an error in PHP 9\n";
/// Legacy raw-syscall printf output-buffer guard diagnostic (superseded by the shared
/// `__rt_stdout_write` choke point; kept while `fixed` still emits its data symbol).
pub(crate) const OB_PRINTF_UNSUPPORTED_MSG: &str =
    "Fatal error: printf() inside an active output buffer is not supported\n";
/// `vprintf()` counterpart of `OB_PRINTF_UNSUPPORTED_MSG` (same legacy guard symbol).
pub(crate) const OB_VPRINTF_UNSUPPORTED_MSG: &str =
    "Fatal error: vprintf() inside an active output buffer is not supported\n";
/// Legacy var_dump array-walk output-buffer guard diagnostic (guard is now a no-op).
pub(crate) const OB_VAR_DUMP_UNSUPPORTED_MSG: &str =
    "Fatal error: var_dump(): array/hash contents inside an active output buffer are not supported\n";
/// Legacy print_r array-walk output-buffer guard diagnostic (guard is now a no-op).
pub(crate) const OB_PRINT_R_UNSUPPORTED_MSG: &str =
    "Fatal error: print_r(): array/hash/Mixed value contents inside an active output buffer are not supported\n";
