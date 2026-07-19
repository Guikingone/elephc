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
mod reflect_member_registry;
mod user;

/// Emit the closed-world constant/enum registry data tables for runtime lookups.
pub(crate) use const_registry::emit_const_registry_data;
/// Emit the closed-world class/interface/trait registry data tables for runtime lookups.
pub(crate) use const_registry::emit_class_registry_data;
/// Emit the closed-world per-class relation payload registry backing non-literal
/// `class_implements()`/`class_parents()`/`class_uses()`.
pub(crate) use class_relation_registry::emit_class_relation_registry_data;
pub(crate) use fixed::emit_runtime_data_fixed;
/// Emit the closed-world flat method/property registry tables backing dynamic
/// `ReflectionMethod`/`ReflectionProperty` construction and `getMethod`/`getProperty`.
pub(crate) use reflect_member_registry::emit_reflect_member_registry_data;
/// Row-layout byte-offset/size constants — the shared ABI contract between the table emitter
/// here and the raw-assembly dynamic dispatcher in
/// `crate::codegen_ir::lower_inst::objects::reflection_members`.
pub(crate) use reflect_member_registry::{
    CLASS_ID_ROW_CLASS_ID_OFFSET, CLASS_ID_ROW_SIZE, INDEX_ROW_SIZE, METHOD_ROW_MODIFIERS_OFFSET,
    METHOD_ROW_REAL_NAME_LEN_OFFSET, METHOD_ROW_REAL_NAME_PTR_OFFSET, METHOD_ROW_SIZE,
    PROPERTY_ROW_MODIFIERS_OFFSET, PROPERTY_ROW_SIZE,
};
/// Computes `ReflectionClass::getMethods()`/`getProperties()`'s PHP declaration order (and
/// parent-private-exclusion filtering) from `ClassInfo` directly — shared between the flat
/// member registry's per-row `decl_order` field and the `ReflectionClass` construction-time
/// metadata bake (`crate::codegen_ir::lower_inst::objects::reflection`).
pub(crate) use reflect_member_registry::{method_decl_order_and_names, property_decl_order_and_names};
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
/// Fatal error message when `ob_start()` would push past the fixed
/// `OB_MAX_LEVELS` output-buffering nesting cap.
pub(crate) const OB_MAX_LEVELS_MSG: &str =
    "Fatal error: ob_start(): maximum output buffering nesting level exceeded\n";
/// Fatal error message when one output-buffer level's captured bytes would
/// exceed its fixed `OB_LEVEL_CAP` (1 MiB) scratch region.
pub(crate) const OB_OVERFLOW_MSG: &str =
    "Fatal error: output buffer exceeds the 1 MiB per-level limit\n";
/// `ob_get_status()`'s `name` field for elephc's one supported buffer kind — a
/// plain buffer with no callback (php -n verified: real PHP reports the SAME
/// literal for a callback-free `ob_start()`).
pub(crate) const OB_STATUS_NAME: &str = "default output handler";
/// Fatal error message when `printf()` is called while an output-buffering
/// level is active. `printf()`'s formatted-write result goes straight to a
/// raw `write(1, …)` syscall (`emit_printf_write_result`), bypassing the
/// `__rt_stdout_write` choke point `ob_start()` intercepts — real PHP DOES
/// buffer `printf()` output; this is a disclosed supported-subset divergence
/// (loud, never silently written outside the active buffer).
pub(crate) const OB_PRINTF_UNSUPPORTED_MSG: &str =
    "Fatal error: printf() inside an active output buffer is not supported\n";
/// `vprintf()` counterpart of `OB_PRINTF_UNSUPPORTED_MSG` (same raw-syscall bypass).
pub(crate) const OB_VPRINTF_UNSUPPORTED_MSG: &str =
    "Fatal error: vprintf() inside an active output buffer is not supported\n";
/// Fatal error message when `var_dump()` walks array/hash contents while an
/// output-buffering level is active. The per-element runtime walkers
/// (`__rt_var_dump_array_*`/`__rt_var_dump_hash`) perform raw `write(1, …)`
/// syscalls, bypassing the `__rt_stdout_write` choke point `ob_start()`
/// intercepts — real PHP DOES buffer `var_dump()` output; this is a
/// disclosed supported-subset divergence (loud, never silently written
/// outside the active buffer). Scalar `var_dump()` values are unaffected —
/// they already route through `__rt_stdout_write` directly.
pub(crate) const OB_VAR_DUMP_UNSUPPORTED_MSG: &str =
    "Fatal error: var_dump(): array/hash contents inside an active output buffer are not supported\n";
/// Fatal error message when `print_r()` walks array/hash/Mixed contents while
/// an output-buffering level is active (see `OB_VAR_DUMP_UNSUPPORTED_MSG`;
/// `__rt_print_r_indexed`/`__rt_print_r_hash`/`__rt_print_r_value` share the
/// same raw-syscall bypass — `__rt_print_r_value` also renders a Mixed-typed
/// scalar, since `print_r()` on a `Mixed`-typed value always redispatches
/// through it regardless of the held payload's own type). Scalar `print_r()`
/// values with a concrete (non-`Mixed`) static type are unaffected.
pub(crate) const OB_PRINT_R_UNSUPPORTED_MSG: &str =
    "Fatal error: print_r(): array/hash/Mixed value contents inside an active output buffer are not supported\n";
/// Stderr notice when `ob_end_clean()` is called with no active output buffer
/// (php -n verified message text; elephc's AOT model has no file:line to report).
pub(crate) const OB_END_CLEAN_EMPTY_MSG: &str =
    "Warning: ob_end_clean(): Failed to delete buffer. No buffer to delete\n";
/// Stderr notice when `ob_end_flush()` is called with no active output buffer
/// (php -n verified message text; elephc's AOT model has no file:line to report).
pub(crate) const OB_END_FLUSH_EMPTY_MSG: &str =
    "Warning: ob_end_flush(): Failed to delete and flush buffer. No buffer to delete or flush\n";
