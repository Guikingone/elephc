//! Purpose:
//! Dispatches native object comparison and SplObjectStorage entry equality.
//!
//! Called from:
//! - `super::emit_obj_loose_eq()` before standard PHP property comparison.
//!
//! Key details:
//! - Returns minus one for standard properties, otherwise the native boolean result.
//! - Exact storage instances compare object keys and loosely equal info values.
//! - Distinct storage subclasses are uncomparable, matching pinned php-src.

use crate::codegen_support::{emit::Emitter, platform::Arch};

mod aarch64;
mod x86_64;

/// Emits the target-specific native comparison policy and entry walker.
pub(super) fn emit_native_object_comparison(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => aarch64::emit(emitter),
        Arch::X86_64 => x86_64::emit(emitter),
    }
}
