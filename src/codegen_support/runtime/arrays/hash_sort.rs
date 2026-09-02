//! Purpose:
//! Coordinates target-specific hash-table link sort emitters used by PHP's stable
//! key- and value-preserving array sorts.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` through
//!   `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - Sorting relinks the insertion-order chain without moving hash buckets or payloads.
//! - Both targets use the same stable bottom-up merge-sort contract and `O(n log n)`
//!   comparison bound.

mod aarch64;
mod x86_64;

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Mode word selecting an ascending key sort (`ksort`).
pub(super) const MODE_KEY_ASCENDING: i64 = 0;

/// Mode word selecting a descending key sort (`krsort`).
pub(super) const MODE_KEY_DESCENDING: i64 = 1;

/// Mode word selecting an ascending value sort (`asort`).
pub(super) const MODE_VALUE_ASCENDING: i64 = 2;

/// Mode word selecting a descending value sort (`arsort`).
pub(super) const MODE_VALUE_DESCENDING: i64 = 3;

/// Emits every hash link-order sort helper for the active target.
pub fn emit_hash_sort(emitter: &mut Emitter) {
    super::hash_key_compare::emit_hash_key_compare(emitter);
    match emitter.target.arch {
        Arch::X86_64 => x86_64::emit(emitter),
        Arch::AArch64 => aarch64::emit(emitter),
    }
}

/// Returns the PHP-facing hash-sort entry points with their mode words and descriptions.
pub(super) fn entry_points() -> [(&'static str, i64, &'static str); 4] {
    [
        ("__rt_hash_ksort", MODE_KEY_ASCENDING, "sort a hash by key ascending"),
        ("__rt_hash_krsort", MODE_KEY_DESCENDING, "sort a hash by key descending"),
        ("__rt_hash_asort", MODE_VALUE_ASCENDING, "sort a hash by value ascending"),
        ("__rt_hash_arsort", MODE_VALUE_DESCENDING, "sort a hash by value descending"),
    ]
}
