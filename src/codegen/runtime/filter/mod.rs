//! Purpose:
//! Collects the `ext/filter` (`filter_var()`) runtime emitters: dedicated
//! decimal-only int/float parsers and the boolean token matcher, plus the
//! shared PHP-filter whitespace trimmer they all use. The module owns
//! re-export wiring only — each leaf file implements exactly one helper.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` during the filter runtime section.
//!
//! Key details:
//! - These are DEDICATED parsers (never `strtol`/`intval`/`is_numeric` reuse):
//!   PHP's `FILTER_VALIDATE_INT`/`FILTER_VALIDATE_FLOAT` reject octal/hex forms
//!   that generic numeric-string parsing accepts, so validation must not be
//!   delegated to a parser with a different accepted grammar.

mod trim_ws;
mod validate_bool;
mod validate_float;
mod validate_int;

/// Trims PHP-filter whitespace (`{0x09, 0x0A, 0x0B, 0x0D, 0x20}`, NOT form feed)
/// from both ends of a PHP string.
pub use trim_ws::emit_filter_trim_ws;
/// Dedicated decimal-only integer parser backing `FILTER_VALIDATE_INT`.
pub use validate_int::emit_filter_validate_int;
/// Dedicated decimal-only float grammar scanner (value computed via `strtod`
/// only after acceptance) backing `FILTER_VALIDATE_FLOAT`.
pub use validate_float::emit_filter_validate_float;
/// Boolean token matcher backing `FILTER_VALIDATE_BOOL(EAN)`.
pub use validate_bool::emit_filter_validate_bool_str;
