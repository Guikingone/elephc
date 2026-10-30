//! Purpose:
//! Exposes allocation-light progressive-search entry points for the shared mbregex session.
//!
//! Called from:
//! - Generated `mb_ereg_search*` calls whose arguments already have their PHP representations.
//!
//! Key details:
//! - The setter and getter operate on the same thread-local session as the generic bridge.
//! - `init` is restricted to the ASCII `[0-9]+` pattern by the codegen fast path; the step,
//!   position, and set-position entries are state-shape agnostic.
//! - Invalid subjects return zero, and a failed set-position returns zero so the caller can run
//!   the generic bridge and preserve PHP's exact ValueError diagnostics.

use std::panic::{catch_unwind, AssertUnwindSafe};
use super::*;
use crate::regex::Limits;

/// Initializes progressive search with the literal `[0-9]+` pattern.
///
/// # Safety
/// `subject` must be null or point to `len` readable bytes for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn elephc_mbstring_ereg_search_init_simple_v1(
    subject: *const u8,
    len: u64,
) -> i32 {
    catch_unwind(AssertUnwindSafe(|| unsafe {
        if !crate::regex::provider_available() || len > isize::MAX as u64 || (len != 0 && subject.is_null()) {
            return -1;
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            std::slice::from_raw_parts(subject, len as usize)
        };
        REGEX.with(|session| {
            match session.initialize_simple(bytes) {
                Ok(true) => 1,
                Ok(false) => 0,
                Err(_) => -1,
            }
        })
    }))
    .unwrap_or(-1)
}

/// Performs one progressive search using the session's retained pattern and subject.
#[no_mangle]
pub extern "C" fn elephc_mbstring_ereg_search_step_v1() -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        if !crate::regex::provider_available() {
            return -1;
        }
        REGEX.with(|session| {
            let limits = Limits { stack: None, retry: None };
            match session.search(None, None, "mb_ereg_search", limits, &mut |_| {}) {
                Ok(Some(_)) => 1,
                Ok(None) => 0,
                Err(_) => -1,
            }
        })
    }))
    .unwrap_or(-1)
}

/// Returns the session's progressive byte position.
#[no_mangle]
pub extern "C" fn elephc_mbstring_ereg_search_getpos_v1() -> i64 {
    catch_unwind(AssertUnwindSafe(|| {
        REGEX.with(|session| i64::try_from(session.position()).unwrap_or(-1))
    }))
    .unwrap_or(-1)
}

/// Sets the progressive byte position, returning one on success and zero on range failure.
#[no_mangle]
pub extern "C" fn elephc_mbstring_ereg_search_setpos_v1(position: i64) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        REGEX.with(|session| i32::from(session.set_position(position).is_ok()))
    }))
    .unwrap_or(0)
}
