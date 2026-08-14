//! Purpose:
//! Implements PHP `extract()` against the materialized eval activation scope.
//! Array keys become runtime-named scope entries consumed by eval and dynamic includes.
//!
//! Called from:
//! - Generated EIR backend assembly through `__elephc_eval_extract`.
//!
//! Key details:
//! - Extracted cells are transferred to the scope as owned values.
//! - Collision and prefix modes follow PHP's `EXTR_*` constants.

use crate::abi::ElephcEvalScope;
use crate::errors::EvalStatus;
use crate::interpreter::RuntimeValueOps;
use crate::runtime_hooks::ElephcRuntimeOps;
use crate::scope::ScopeCellOwnership;
use crate::value::{RuntimeCell, RuntimeCellHandle};
use std::slice;

const EXTR_SKIP: i64 = 1;
const EXTR_PREFIX_SAME: i64 = 2;
const EXTR_PREFIX_ALL: i64 = 3;
const EXTR_PREFIX_INVALID: i64 = 4;
const EXTR_PREFIX_IF_EXISTS: i64 = 5;
const EXTR_IF_EXISTS: i64 = 6;
const EXTR_REFS: i64 = 256;
const EVAL_TAG_INT: u64 = 0;
const EVAL_TAG_STRING: u64 = 1;

/// Extracts array entries into a materialized caller scope and returns the count.
///
/// A negative result is an [`EvalStatus`] code negated for ABI transport. The generated
/// caller converts that status into the same fatal path used by runtime `eval()` failures.
///
/// # Safety
/// `scope` and `array` must be valid bridge/runtime handles. `prefix_ptr` must be readable
/// for `prefix_len` bytes when the length is non-zero.
#[no_mangle]
pub unsafe extern "C" fn __elephc_eval_extract(
    scope: *mut ElephcEvalScope,
    array: *mut RuntimeCell,
    flags: i64,
    prefix_ptr: *const u8,
    prefix_len: u64,
) -> i64 {
    std::panic::catch_unwind(|| unsafe {
        extract_inner(scope, array, flags, prefix_ptr, prefix_len)
    })
    .unwrap_or_else(|_| -i64::from(EvalStatus::RuntimeFatal.code()))
}

/// Runs the `extract()` operation after the exported panic boundary is installed.
///
/// # Safety
/// Mirrors [`__elephc_eval_extract`].
unsafe fn extract_inner(
    scope: *mut ElephcEvalScope,
    array: *mut RuntimeCell,
    flags: i64,
    prefix_ptr: *const u8,
    prefix_len: u64,
) -> i64 {
    let Some(scope) = scope.as_mut() else {
        return extract_error(EvalStatus::RuntimeFatal);
    };
    if array.is_null() || (prefix_len > 0 && prefix_ptr.is_null()) {
        return extract_error(EvalStatus::RuntimeFatal);
    }
    let Ok(prefix_len) = usize::try_from(prefix_len) else {
        return extract_error(EvalStatus::RuntimeFatal);
    };
    let prefix = if prefix_len == 0 {
        &[]
    } else {
        slice::from_raw_parts(prefix_ptr, prefix_len)
    };
    let Ok(prefix) = std::str::from_utf8(prefix) else {
        return extract_error(EvalStatus::RuntimeFatal);
    };
    let mode = flags & !EXTR_REFS;
    if !(0..=EXTR_IF_EXISTS).contains(&mode) || flags & EXTR_REFS != 0 {
        return extract_error(EvalStatus::UnsupportedConstruct);
    }

    let array = RuntimeCellHandle::from_raw(array);
    let mut values = ElephcRuntimeOps::new();
    let Ok(len) = values.array_len(array) else {
        return extract_error(EvalStatus::RuntimeFatal);
    };
    let mut extracted = 0_i64;
    for position in 0..len {
        let Ok(key) = values.array_iter_key(array, position) else {
            return extract_error(EvalStatus::RuntimeFatal);
        };
        let name = extract_key_name(&mut values, key);
        let Ok(Some(name)) = name else {
            let _ = values.release(key);
            if name.is_err() {
                return extract_error(EvalStatus::RuntimeFatal);
            }
            continue;
        };
        let Some(target) = extract_target_name(scope, &name, prefix, mode) else {
            let _ = values.release(key);
            continue;
        };
        let value = values.array_get(array, key);
        let _ = values.release(key);
        let Ok(value) = value else {
            return extract_error(EvalStatus::RuntimeFatal);
        };
        if let Some(replaced) = scope.set(target, value, ScopeCellOwnership::Owned) {
            let _ = values.release(replaced);
        }
        extracted += 1;
    }
    extracted
}

/// Converts one foreach-visible array key into its PHP extraction name.
fn extract_key_name(
    values: &mut ElephcRuntimeOps,
    key: RuntimeCellHandle,
) -> Result<Option<String>, EvalStatus> {
    match values.type_tag(key)? {
        EVAL_TAG_STRING => String::from_utf8(values.string_bytes(key)?)
            .map(Some)
            .map_err(|_| EvalStatus::RuntimeFatal),
        EVAL_TAG_INT => Ok(Some((values.raw_value_word(key)? as i64).to_string())),
        _ => Ok(None),
    }
}

/// Applies collision/prefix policy and returns the final variable name when extractable.
fn extract_target_name(
    scope: &ElephcEvalScope,
    name: &str,
    prefix: &str,
    mode: i64,
) -> Option<String> {
    let valid = is_valid_php_variable_name(name);
    let exists = scope.contains_visible(name);
    let prefix_name = || format!("{prefix}_{name}");
    let target = match mode {
        0 if valid => name.to_string(),
        EXTR_SKIP if valid && !exists => name.to_string(),
        EXTR_PREFIX_SAME if valid && exists => prefix_name(),
        EXTR_PREFIX_SAME if valid => name.to_string(),
        EXTR_PREFIX_ALL if valid || is_prefixable_invalid_name(name) => prefix_name(),
        EXTR_PREFIX_INVALID if valid => name.to_string(),
        EXTR_PREFIX_INVALID if is_prefixable_invalid_name(name) => prefix_name(),
        EXTR_PREFIX_IF_EXISTS if valid && exists => prefix_name(),
        EXTR_IF_EXISTS if valid && exists => name.to_string(),
        _ => return None,
    };
    is_valid_php_variable_name(&target).then_some(target)
}

/// Returns whether bytes encoded as UTF-8 form a PHP variable identifier.
fn is_valid_php_variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic() || !first.is_ascii())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric() || !ch.is_ascii())
}

/// Returns whether PHP can repair an invalid key by prefixing it.
fn is_prefixable_invalid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric() || !ch.is_ascii())
}

/// Encodes one eval status as a negative ABI return value.
fn extract_error(status: EvalStatus) -> i64 {
    -i64::from(status.code())
}
