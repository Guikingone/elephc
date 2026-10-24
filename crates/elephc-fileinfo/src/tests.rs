//! Purpose:
//! Tests native fileinfo lifetime, flag restoration and thread-local result storage.
//!
//! Called from:
//! - Bridge unit tests with ELEPHC_FILEINFO_LIBMAGIC_LIB_DIR set.
//!
//! Key details:
//! - Every test uses the managed embedded database, never the host database.

use super::*;

/// Creates a MIME handle backed by the embedded database.
unsafe fn open() -> i64 {
    let handle = elephc_fileinfo_open(16, c"".as_ptr());
    assert_ne!(handle, 0, "{}", CStr::from_ptr(elephc_fileinfo_error()).to_string_lossy());
    handle
}

/// A binary buffer is length-delimited and a per-call encoding override is temporary.
#[test]
fn fileinfo_native_binary_and_flags() {
    unsafe {
        let h = open();
        let data = b"hello\0binary\0tail";
        assert_eq!(CStr::from_ptr(elephc_fileinfo_buffer(h, data.as_ptr().cast(), data.len() as i64, 1024)).to_bytes(), b"binary");
        assert_eq!(CStr::from_ptr(elephc_fileinfo_buffer(h, c"hello".as_ptr(), 5, 0)).to_bytes(), b"text/plain");
        assert_eq!(elephc_fileinfo_free(h), 1);
        assert_eq!(elephc_fileinfo_free(h), 1);
    }
}

/// A failed file query restores the persistent flags before a subsequent buffer query.
#[test]
fn fileinfo_native_failure_restores_flags() {
    unsafe {
        let h = open();
        elephc_fileinfo_file(h, c"/elephc-fileinfo-missing/file".as_ptr(), 1024);
        assert_eq!(elephc_fileinfo_ok(), 0);
        assert_eq!(CStr::from_ptr(elephc_fileinfo_buffer(h, c"hello".as_ptr(), 5, 0)).to_bytes(), b"text/plain");
        elephc_fileinfo_free(h);
    }
}

/// Another thread cannot invalidate a returned pointer while the first thread reads it.
#[test]
fn fileinfo_native_results_are_thread_local() {
    unsafe {
        let h = open();
        let result = elephc_fileinfo_buffer(h, c"hello".as_ptr(), 5, 0);
        std::thread::spawn(|| {
            let other = open();
            elephc_fileinfo_buffer(other, c"other".as_ptr(), 5, 1024);
            elephc_fileinfo_free(other);
        }).join().unwrap();
        assert_eq!(CStr::from_ptr(result).to_bytes(), b"text/plain");
        elephc_fileinfo_free(h);
    }
}
