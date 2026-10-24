//! Purpose:
//! Owns fileinfo's libmagic instances behind a panic-free, binary-safe C ABI.
//!
//! Called from:
//! - The compiler's fileinfo prelude through extern functions.
//!
//! Key details:
//! - Handles are monotonic IDs, never exposed pointers; destruction is idempotent.
//! - A mutex serializes access to each opaque instance, including flag restoration.
//! - Returned strings belong to the calling thread until its next bridge call.
//! - Native symbols resolve only when linking the managed libmagic package.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr, CString, OsStr};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

extern "C" {
    fn magic_open(flags: c_int) -> *mut c_void;
    fn magic_close(cookie: *mut c_void);
    fn magic_load(cookie: *mut c_void, path: *const c_char) -> c_int;
    fn elephc_magic_load_default(cookie: *mut c_void) -> c_int;
    fn magic_setflags(cookie: *mut c_void, flags: c_int) -> c_int;
    fn magic_getflags(cookie: *mut c_void) -> c_int;
    fn magic_buffer(cookie: *mut c_void, data: *const c_void, length: usize) -> *const c_char;
    fn magic_descriptor(cookie: *mut c_void, fd: c_int) -> *const c_char;
    fn magic_error(cookie: *mut c_void) -> *const c_char;
}

/// One owned opaque pointer; registry locking excludes all simultaneous access.
struct Magic(usize);
impl Drop for Magic {
    /// Releases exactly the native instance transferred to this owner.
    fn drop(&mut self) { unsafe { magic_close(self.0 as *mut c_void) }; }
}

#[derive(Default)]
struct Registry { next: i64, instances: HashMap<i64, Magic> }

/// Returns the process registry without exposing any interior pointer.
fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

thread_local! {
    static RESULT: RefCell<CString> = RefCell::new(CString::default());
    static ERROR: RefCell<CString> = RefCell::new(CString::default());
    static SUCCESS: RefCell<bool> = const { RefCell::new(false) };
}

/// Copies a native string before libmagic can invalidate its backing storage.
unsafe fn copy_string(value: *const c_char) -> CString {
    if value.is_null() { CString::default() } else { CStr::from_ptr(value).to_owned() }
}

/// Stores a call's outcome without returning storage shared with other threads.
fn outcome(result: Option<CString>, error: CString) {
    SUCCESS.with(|slot| *slot.borrow_mut() = result.is_some());
    RESULT.with(|slot| *slot.borrow_mut() = result.unwrap_or_default());
    ERROR.with(|slot| *slot.borrow_mut() = error);
}

/// Prevents a Rust panic from crossing the generated program's C ABI boundary.
fn guard<T: Default>(call: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(call)).unwrap_or_default()
}

/// Creates a native instance, loading a custom database or the embedded default.
/// The path must be a valid NUL-terminated string for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn elephc_fileinfo_open(flags: i64, path: *const c_char) -> i64 {
    guard(|| {
        let cookie = magic_open(flags as c_int);
        if cookie.is_null() {
            outcome(None, CString::new("Invalid mode").unwrap());
            return 0;
        }
        let instance = Magic(cookie as usize);
        let loaded = if path.is_null() || CStr::from_ptr(path).is_empty() {
            elephc_magic_load_default(cookie)
        } else { magic_load(cookie, path) };
        if loaded != 0 {
            outcome(None, copy_string(magic_error(cookie)));
            return 0;
        }
        let mut state = registry().lock().unwrap_or_else(|e| e.into_inner());
        let Some(id) = state.next.checked_add(1) else { return 0; };
        state.next = id;
        state.instances.insert(id, instance);
        outcome(Some(CString::default()), CString::default());
        id
    })
}

/// Releases an owned handle; an already released handle is harmless.
#[no_mangle]
pub extern "C" fn elephc_fileinfo_free(handle: i64) -> i64 {
    guard(|| {
        registry().lock().unwrap_or_else(|e| e.into_inner()).instances.remove(&handle);
        1
    })
}

/// Changes the instance's persistent libmagic flags.
#[no_mangle]
pub extern "C" fn elephc_fileinfo_set_flags(handle: i64, flags: i64) -> i64 {
    guard(|| {
        let state = registry().lock().unwrap_or_else(|e| e.into_inner());
        let Some(instance) = state.instances.get(&handle) else { return 0; };
        i64::from(unsafe { magic_setflags(instance.0 as *mut c_void, flags as c_int) } == 0)
    })
}

/// Queries bytes or a local filename, restoring per-call flags after success or failure.
unsafe fn query(handle: i64, data: *const c_char, length: i64, flags: i64, file: bool) {
    let state = registry().lock().unwrap_or_else(|e| e.into_inner());
    let Some(instance) = state.instances.get(&handle) else {
        outcome(None, CString::new("Invalid finfo object").unwrap());
        return;
    };
    let cookie = instance.0 as *mut c_void;
    let old = magic_getflags(cookie);
    let selected = if flags == 0 { old } else { flags as c_int };
    // PHP opens the stream first: paths follow symlinks and an unreadable file fails.
    magic_setflags(cookie, selected);
    let input = if file {
        match std::fs::File::open(std::path::Path::new(OsStr::from_bytes(CStr::from_ptr(data).to_bytes()))) {
            Ok(input) => Some(input),
            Err(error) => {
                magic_setflags(cookie, old);
                outcome(None, CString::new(error.to_string()).unwrap_or_default());
                return;
            }
        }
    } else { None };
    let result = if let Some(input) = &input { magic_descriptor(cookie, input.as_raw_fd()) } else {
        magic_buffer(cookie, data.cast(), length.max(0) as usize)
    };
    let copied = (!result.is_null()).then(|| copy_string(result));
    let error = copy_string(magic_error(cookie));
    magic_setflags(cookie, old);
    outcome(copied, error);
}

/// Identifies exactly `length` bytes, including embedded NULs; data must span that length.
#[no_mangle]
pub unsafe extern "C" fn elephc_fileinfo_buffer(handle: i64, data: *const c_char, length: i64, flags: i64) -> *const c_char {
    guard(|| {
        query(handle, data, length, flags, false);
        RESULT.with(|slot| slot.borrow().as_ptr())
    })
}

/// Identifies a local NUL-terminated filename without depending on a system database.
#[no_mangle]
pub unsafe extern "C" fn elephc_fileinfo_file(handle: i64, path: *const c_char, flags: i64) -> *const c_char {
    guard(|| {
        query(handle, path, 0, flags, true);
        RESULT.with(|slot| slot.borrow().as_ptr())
    })
}

/// Reports whether the calling thread's last query succeeded.
#[no_mangle]
pub extern "C" fn elephc_fileinfo_ok() -> i64 { SUCCESS.with(|slot| i64::from(*slot.borrow())) }

/// Returns the calling thread's last copied native error message.
#[no_mangle]
pub extern "C" fn elephc_fileinfo_error() -> *const c_char { ERROR.with(|slot| slot.borrow().as_ptr()) }

#[cfg(all(test, elephc_fileinfo_native))]
mod tests;
