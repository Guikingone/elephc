//! Purpose:
//! Reproduces PHP 8.5's `openssl_get_cipher_methods()` catalog from the OpenSSL
//! 3.6.4 default provider: `EVP_CIPHER_do_all_provided`, lowercase names, sorted.
//!
//! Called from:
//! - `crate::abi` for the C method-inventory and IV-length entry points.
//!
//! Key details:
//! - The provider is loaded once and the packed trailing-NUL catalog is cached;
//!   the benchmark's repeated calls only copy an immutable buffer.
//! - No legacy provider is loaded, matching PHP's default 135-name list. Loading
//!   legacy would add BF/CAST/DES/IDEA/RC2/RC4/SEED names PHP does not report.

use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::OnceLock;

use crate::ffi::{
    EVP_CIPHER_do_all_provided, EVP_CIPHER_fetch, EVP_CIPHER_free, EVP_CIPHER_get0_name,
    EVP_CIPHER_get_iv_length, OSSL_PROVIDER_load,
};
use crate::CipherError;

/// The packed trailing-NUL catalog, built once on first use.
static CATALOG: OnceLock<Vec<u8>> = OnceLock::new();

/// The number of names in the packed catalog.
static CATALOG_COUNT: OnceLock<usize> = OnceLock::new();

/// Loads the default provider once for this process.
fn ensure_default_provider() {
    static PROVIDER: OnceLock<()> = OnceLock::new();
    PROVIDER.get_or_init(|| {
        let default = CString::new("default").expect("static provider name");
        // A loaded static provider is intentionally retained for the process lifetime.
        let _ = unsafe { OSSL_PROVIDER_load(std::ptr::null_mut(), default.as_ptr()) };
    });
}

/// Collects one provider cipher name during enumeration.
extern "C" fn collect_name(cipher: *mut crate::ffi::EVP_CIPHER, arg: *mut c_void) {
    if cipher.is_null() || arg.is_null() {
        return;
    }
    let name = unsafe { EVP_CIPHER_get0_name(cipher) };
    if name.is_null() {
        return;
    }
    let name = unsafe { CStr::from_ptr(name) };
    let Ok(name) = name.to_str() else {
        return;
    };
    let names = unsafe { &mut *(arg as *mut Vec<String>) };
    names.push(name.to_ascii_lowercase());
}

/// Builds the lowercase, sorted, deduplicated default-provider catalog.
fn build_catalog() -> Vec<u8> {
    ensure_default_provider();
    let mut names: Vec<String> = Vec::new();
    unsafe {
        EVP_CIPHER_do_all_provided(
            std::ptr::null_mut(),
            collect_name,
            &mut names as *mut Vec<String> as *mut c_void,
        );
    }
    names.sort();
    names.dedup();
    let mut packed: Vec<u8> = Vec::with_capacity(names.iter().map(|name| name.len() + 1).sum());
    for name in names {
        packed.extend_from_slice(name.as_bytes());
        packed.push(0);
    }
    packed
}

/// Returns the cached trailing-NUL packed catalog.
pub(crate) fn catalog_packed() -> &'static [u8] {
    CATALOG.get_or_init(build_catalog)
}

/// Returns the number of catalog names.
pub(crate) fn catalog_count() -> usize {
    *CATALOG_COUNT.get_or_init(|| {
        catalog_packed()
            .split(|byte| *byte == 0)
            .filter(|name| !name.is_empty())
            .count()
    })
}

/// Returns the default IV length for a fetched cipher name.
pub(crate) fn cipher_iv_length(name: &[u8]) -> Result<usize, CipherError> {
    let name = std::str::from_utf8(name).map_err(|_| CipherError::UnknownCipher)?;
    let name = CString::new(name).map_err(|_| CipherError::UnknownCipher)?;
    let cipher = unsafe {
        EVP_CIPHER_fetch(std::ptr::null_mut(), name.as_ptr(), std::ptr::null())
    };
    if cipher.is_null() {
        return Err(CipherError::UnknownCipher);
    }
    let length = unsafe { EVP_CIPHER_get_iv_length(cipher) };
    unsafe { EVP_CIPHER_free(cipher) };
    usize::try_from(length).map_err(|_| CipherError::UnknownCipher)
}
