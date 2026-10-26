//! Purpose:
//! Shares the compiler-owned SPL heap property prefix with native mutation scopes.
//!
//! Called from:
//! - SPL metadata injection, compiler-call validation and native scope clone materialization.
//!
//! Key details:
//! - Inherited object layouts keep the base property prefix, with 16-byte property slots.
//! - Reserved NUL names cannot be declared by PHP source; scope calls validate the prefix.

pub(crate) const VALUES: &str = "\0__elephc_heap_values";
pub(crate) const COUNT: &str = "\0__elephc_heap_count";
pub(crate) const STATE: &str = "\0__elephc_heap_state";
pub(crate) const ERROR: &str = "\0__elephc_heap_error";
pub(crate) const PRIORITIES: &str = "\0__elephc_heap_priorities";
pub(crate) const EXTRACT_FLAGS: &str = "\0__elephc_heap_extract_flags";

pub(crate) const VALUES_OFFSET: usize = 8;
pub(crate) const COUNT_OFFSET: usize = 8 + 16;
pub(crate) const ERROR_OFFSET: usize = 8 + 3 * 16;
pub(crate) const PRIORITIES_OFFSET: usize = 8 + 4 * 16;

/// Confirms that raw native scope operations address the canonical unboxed property prefix.
pub(crate) fn has_native_prefix(info: &super::ClassInfo) -> bool {
    [VALUES, COUNT, STATE, ERROR].iter().enumerate().all(|(index, name)| {
        info.properties.get(index).is_some_and(|(actual, _)| actual == name
            && !info.property_slot_is_reference(index, actual))
    })
}
