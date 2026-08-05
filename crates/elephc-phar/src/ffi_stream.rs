//! Purpose:
//! C ABI wrappers for buffered PHAR write streams.
//!
//! Called from:
//! - Generated PHAR stream open, append, and finalize paths.
//!
//! Key details:
//! - Synthetic descriptors remain one-shot handles into the bounded stream table.

use super::*;

/// C ABI wrapper that opens a buffered write stream for a literal PHAR entry.
///
/// Returns a synthetic descriptor in the `0x50000000..0x50000020` range, or
/// `usize::MAX` when no stream slot is available or the target is invalid.
///
/// # Safety
/// Each pointer must be valid for its paired byte length unless that length is
/// zero. `entry_ptr` must not describe an empty entry name.
#[no_mangle]
pub unsafe extern "C" fn elephc_phar_stream_open_entry(
    archive_ptr: *const u8,
    archive_len: usize,
    entry_ptr: *const u8,
    entry_len: usize,
) -> usize {
    let result = std::panic::catch_unwind(|| {
        let entry = slice(entry_ptr, entry_len);
        if entry.is_empty() {
            return None;
        }
        allocate_write_stream(WriteStream {
            target: WriteStreamTarget::Entry {
                archive: slice(archive_ptr, archive_len).to_vec(),
                entry: entry.to_vec(),
            },
            payload: Vec::new(),
        })
    });
    match result {
        Ok(Some(fd)) => fd,
        _ => usize::MAX,
    }
}

/// C ABI wrapper that opens a buffered write stream for a runtime PHAR URL.
///
/// Returns a synthetic descriptor in the `0x50000000..0x50000020` range, or
/// `usize::MAX` when no stream slot is available or the URL is invalid.
///
/// # Safety
/// `url_ptr` must be valid for `url_len` bytes unless `url_len` is zero.
#[no_mangle]
pub unsafe extern "C" fn elephc_phar_stream_open_url(
    url_ptr: *const u8,
    url_len: usize,
) -> usize {
    let result = std::panic::catch_unwind(|| {
        let url = slice(url_ptr, url_len);
        if !url.starts_with(b"phar://") {
            return None;
        }
        allocate_write_stream(WriteStream {
            target: WriteStreamTarget::Url(url.to_vec()),
            payload: Vec::new(),
        })
    });
    match result {
        Ok(Some(fd)) => fd,
        _ => usize::MAX,
    }
}

/// C ABI wrapper that appends bytes to an open PHAR write stream.
///
/// Returns the number of consumed bytes on success and `usize::MAX` when `fd`
/// does not name an open PHAR write stream.
///
/// # Safety
/// `data_ptr` must be valid for `data_len` bytes unless `data_len` is zero.
#[no_mangle]
pub unsafe extern "C" fn elephc_phar_stream_append(
    fd: usize,
    data_ptr: *const u8,
    data_len: usize,
) -> usize {
    let result = std::panic::catch_unwind(|| {
        append_write_stream(fd, slice(data_ptr, data_len))
    });
    match result {
        Ok(Some(len)) => len,
        _ => usize::MAX,
    }
}

/// C ABI wrapper that finalizes and closes an open PHAR write stream.
///
/// Returns `1` on success and `0` on failure. The stream slot is released before
/// the archive write is attempted, matching fclose-style one-shot finalization.
#[no_mangle]
pub extern "C" fn elephc_phar_stream_finalize(fd: usize) -> usize {
    let result = std::panic::catch_unwind(|| finalize_write_stream(fd));
    match result {
        Ok(Some(())) => 1,
        _ => 0,
    }
}
