//! Purpose:
//! Exposes allocation-free mbstring catalog queries used by native count fast paths.
//!
//! Called from:
//! - Generated `count(mb_encoding_aliases(...))` and `count(mb_list_encodings())` helpers.
//!
//! Key details:
//! - Both functions return counts only; no array or string is materialized.
//! - An unknown encoding returns `-1` so the caller can run the shared bridge and keep PHP's
//!   ValueError diagnostics.
//! - The names are read as a borrowed byte slice; no callback or request-state borrow crosses
//!   the C boundary.

use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::encoding::Encoding;

/// Returns the number of public aliases for an encoding, or `-1` when it is unknown.
///
/// # Safety
/// `name` must be null or point to `len` readable bytes for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn elephc_mbstring_encoding_aliases_count_v1(
    name: *const u8,
    len: u64,
) -> i64 {
    catch_unwind(AssertUnwindSafe(|| {
        if len > isize::MAX as u64 || (len != 0 && name.is_null()) {
            return -1;
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(name, len as usize) }
        };
        Encoding::lookup_c_string(bytes).map_or(-1, |encoding| encoding.aliases().len() as i64)
    }))
    .unwrap_or(-1)
}

/// Returns the number of canonical encodings exposed by `mb_list_encodings()`.
#[no_mangle]
pub extern "C" fn elephc_mbstring_list_encodings_count_v1() -> i64 {
    catch_unwind(AssertUnwindSafe(|| Encoding::all().count() as i64)).unwrap_or(-1)
}
