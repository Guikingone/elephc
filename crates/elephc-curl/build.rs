//! Purpose:
//! Adds native link-search/lib directives for `elephc-curl`'s own test binary,
//! and ONLY for that binary, so `cargo test -p elephc-curl` can link real
//! libcurl/libssh2/libssl/libcrypto/libz/libnghttp2 static archives from an
//! installed `elephc native` package when a developer asks for it.
//!
//! Called from:
//! - Cargo, automatically, before compiling any target of this crate
//!   (including `cargo build -p elephc-curl` and `cargo test -p elephc-curl`).
//!
//! Key details:
//! - `cargo build -p elephc-curl` never needs this: the crate's `staticlib`/
//!   `rlib` outputs never invoke the system linker, so libcurl's unresolved
//!   `extern "C"` symbols stay unresolved in the archive regardless of
//!   whether these directives were emitted. The PHP-program linker supplies
//!   `libcurl.a` at final-binary link time instead.
//! - `cargo test -p elephc-curl` DOES produce a real executable, so it needs
//!   every symbol resolved. The crate's own gated unit tests
//!   (`src/tests.rs`) that call the real ABI live behind
//!   `#[cfg(elephc_curl_native)]`, which this script only emits when
//!   `ELEPHC_CURL_LIB_DIR` is set. When the cfg is absent, nothing in the
//!   test binary references libcurl's symbols (the crate's own always-linked
//!   `elephc_curl_*` entry points are simply never called by anything in that
//!   build, so the linker's dead-stripping drops them) and the test binary
//!   links cleanly, running a single test that prints a clear skip message.
//! - `ELEPHC_CURL_LIB_DIR` must point at curl's OWN `lib/` directory
//!   (containing `libcurl.a`). `ELEPHC_CURL_LIBSSH2_LIB_DIR` /
//!   `ELEPHC_CURL_NGHTTP2_LIB_DIR` / `ELEPHC_CURL_OPENSSL_LIB_DIR` /
//!   `ELEPHC_CURL_ZLIB_LIB_DIR` point at the sibling libssh2/nghttp2/OpenSSL/zlib
//!   `lib/` directories — these are separate `elephc native` packages with
//!   unrelated content-hashed paths, so no single directory covers all six
//!   archives. Link order mirrors libcurl's own dependency order: curl -> ssh2
//!   -> ssl -> crypto -> z -> nghttp2 (libssh2 needs OpenSSL and zlib, so it has
//!   to precede them; nghttp2 needs nothing, so it can trail), which is the same
//!   sequence `src/native_deps/catalog.rs`' `CURL_VERSIONS.dependencies` produces
//!   for the production link. Apple test targets also receive curl's upstream
//!   Security/CoreFoundation/CoreServices trio for SecTrust; macOS alone adds the
//!   empirically required
//!   SystemConfiguration resolver framework.
//! - `ELEPHC_CURL_LIB_DIR` NAMES THE SAME ENV VAR AS `src/linker/bridges.rs`'s
//!   `BRIDGES` table entry for `elephc_curl`, but for a DIFFERENT purpose there:
//!   the production compiler reads it as an override for the directory
//!   containing a prebuilt `libelephc_curl.a` BRIDGE archive (this crate's own
//!   staticlib output), not curl's real `libcurl.a`. The two readers never run
//!   in the same process — this build script only runs under `cargo
//!   build`/`cargo test -p elephc-curl`, never inside a compiled `elephc`
//!   invocation — so there is no runtime collision, but a human reading both
//!   names side by side should not conflate them.

use std::env;
use std::path::Path;

/// Emits the custom `cfg`s this build script may set, so rustc's
/// `unexpected_cfgs` lint stays quiet regardless of which branch below runs.
fn declare_check_cfg() {
    println!("cargo:rustc-check-cfg=cfg(elephc_curl_native)");
}

/// Adds a native search path for `dir` when it exists, so a missing/typo'd
/// override directory fails at link time with a clear "symbol not found"
/// instead of a silently-ignored search path.
///
/// Every `rustc-link-lib=static=…` below is emitted UNCONDITIONALLY, even when
/// its directory variable is unset — deliberately, and the asymmetry is the
/// point. Dropping the `-l` too would hand the linker a build with no libssh2
/// (say) at all, which surfaces as a wall of undefined `libssh2_*` symbols
/// referenced from `libcurl.a` and reads like a libcurl problem. Keeping the
/// `-l` makes the same mistake fail as `library not found for -lssh2`, which
/// names the thing that is actually missing. The one variable that IS required
/// is `ELEPHC_CURL_LIB_DIR`: without it `main` returns early and none of these
/// directives are emitted, so the gated tests are not compiled in either.
fn add_search_path(dir: &std::ffi::OsStr) {
    println!("cargo:rustc-link-search=native={}", Path::new(dir).display());
}

/// Configures native libcurl and dependency archives when a test artifact is supplied.
fn main() {
    declare_check_cfg();

    println!("cargo:rerun-if-env-changed=ELEPHC_CURL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ELEPHC_CURL_LIBSSH2_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ELEPHC_CURL_NGHTTP2_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ELEPHC_CURL_OPENSSL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ELEPHC_CURL_ZLIB_LIB_DIR");

    let Some(curl_lib_dir) = env::var_os("ELEPHC_CURL_LIB_DIR") else {
        // No native artifact configured. Emit nothing: `cargo build -p
        // elephc-curl` already does not need it, and `cargo test -p
        // elephc-curl` will link cleanly because the gated real tests
        // (behind `elephc_curl_native`) simply are not compiled in.
        return;
    };

    // Real native artifacts requested: wire up libcurl's own dependency link
    // order (curl -> ssh2 -> ssl -> crypto -> z -> nghttp2), plus the Apple
    // frameworks curl's OpenSSL/SecTrust backend needs there.
    add_search_path(&curl_lib_dir);
    println!("cargo:rustc-link-lib=static=curl");

    if let Some(ssh2_lib_dir) = env::var_os("ELEPHC_CURL_LIBSSH2_LIB_DIR") {
        add_search_path(&ssh2_lib_dir);
    }
    println!("cargo:rustc-link-lib=static=ssh2");

    if let Some(ssl_lib_dir) = env::var_os("ELEPHC_CURL_OPENSSL_LIB_DIR") {
        add_search_path(&ssl_lib_dir);
    }
    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=static=crypto");

    if let Some(z_lib_dir) = env::var_os("ELEPHC_CURL_ZLIB_LIB_DIR") {
        add_search_path(&z_lib_dir);
    }
    println!("cargo:rustc-link-lib=static=z");

    // Last on purpose: nothing in the chain above resolves nghttp2's symbols, and
    // nghttp2 itself pulls in nothing further.
    if let Some(nghttp2_lib_dir) = env::var_os("ELEPHC_CURL_NGHTTP2_LIB_DIR") {
        add_search_path(&nghttp2_lib_dir);
    }
    println!("cargo:rustc-link-lib=static=nghttp2");

    let target_os = env::var_os("CARGO_CFG_TARGET_OS");
    if matches!(target_os.as_deref(), Some(os) if os == "macos" || os == "ios") {
        // Mirror curl 8.21's `APPLE_SECTRUST_LDFLAGS` exactly: Security evaluates the
        // certificate chain, while the implementation uses CoreFoundation objects and
        // upstream retains the CoreServices umbrella in the supported Apple link set.
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreServices");
    }
    if target_os.as_deref() == Some(std::ffi::OsStr::new("macos")) {
        // macOS additionally compiles `Curl_macos_init` proxy/NAT64 setup, whose
        // `SCDynamicStoreCopyProxies` reference needs SystemConfiguration. iOS excludes
        // that source path through TargetConditionals.
        println!("cargo:rustc-link-lib=framework=SystemConfiguration");
    }

    // Compile the crate's real, libcurl-calling unit tests in.
    println!("cargo:rustc-cfg=elephc_curl_native");
}
