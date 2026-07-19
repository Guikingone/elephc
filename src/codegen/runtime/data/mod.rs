//! Purpose:
//! Collects runtime data-section emitters and shared diagnostic string constants.
//! The module separates cacheable fixed data from user-program metadata emitted during compilation.
//!
//! Called from:
//! - `crate::codegen::runtime::emit_runtime_data_fixed()` and `crate::codegen::runtime::emit_runtime_data_user()`.
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

/// Fatal error message when `php_uname()` receives a `$mode` argument whose length is not exactly 1.
pub(crate) const PHP_UNAME_MODE_LEN_MSG: &str =
    "Fatal error: php_uname(): Argument #1 ($mode) must be a single character\n";
/// Fatal error message when `php_uname()` receives a `$mode` argument that is not one of the supported single-character values.
pub(crate) const PHP_UNAME_MODE_VALUE_MSG: &str =
    "Fatal error: php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\"\n";
/// Fatal error message when `dirname()` receives a `$levels` argument less than 1.
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
/// Fatal error message when `print_r($v, true)`'s captured output would exceed the
/// fixed 1 MiB `_pr_cap_buf` capture buffer.
pub(crate) const PRINT_R_CAPTURE_OVERFLOW_MSG: &str =
    "Fatal error: print_r(): return-string capture exceeds the 1 MiB limit\n";
/// Fatal error message when `print_r($v, true)`'s nesting depth guard trips,
/// which bounds self-referential/cyclic array structures instead of looping forever.
pub(crate) const PRINT_R_CAPTURE_RECURSION_MSG: &str =
    "Fatal error: print_r(): possible infinite recursion (nesting too deep)\n";
