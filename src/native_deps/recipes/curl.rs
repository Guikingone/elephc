//! Purpose:
//! Builds a static libcurl 8.21.0 archive carrying every protocol this pin can reach without
//! an unpinnable dependency, against the already materialized `openssl`, `zlib`, `nghttp2`
//! and `libssh2` native packages.
//!
//! Called from:
//! - `crate::native_deps::recipe::CuratedRecipes` for curl recipe revision 3.
//!
//! Key details:
//! - `curl` is the first catalog package with non-empty `dependencies`; its recipe never probes
//!   the system for OpenSSL/zlib/nghttp2/libssh2 and only trusts the prefixes materialization
//!   already built and passed through `RecipeRequest::dependency_prefixes`.
//! - THE PROTOCOL SET IS DELIBERATE, AND MOSTLY MADE OF FLAGS THAT ARE *NOT* HERE. curl's
//!   own configure defaults everything except LDAP(S), SMB and NTLM to enabled, so revision 2
//!   drops the `--disable-{rtsp,dict,telnet,tftp,pop3,imap,smtp,gopher,mqtt}` wall revision 1
//!   carried and adds only the three opt-ins. MEASURED against the pinned `configure.ac`:
//!   `--enable-smb` and `--enable-ntlm` are opt-IN (`AC_ARG_ENABLE(smb, ..., AC_MSG_RESULT(no))`
//!   — the *default* branch answers no), which is why revision 1's `--disable-smb` was a no-op
//!   and why SMB needs NTLM's DES-based session key to exist at all
//!   (`lib/smb.c`: `#if defined(CURL_ENABLE_SMB) && defined(USE_CURL_NTLM_CORE)`).
//! - WHAT IS STILL OFF, AND WHY: LDAP(S) needs OpenLDAP, which is not in the catalog (see
//!   `.superpowers/sdd/curl-punchlist/r3-protocols-report.md` for the measured cost); HTTP/3
//!   needs ngtcp2 + nghttp3 (`docs/DEPRECATE.md:80`: "OpenSSL-QUIC (removed in 8.19.0)", so
//!   the only non-experimental QUIC path left in this pin is `--with-ngtcp2 --with-nghttp3`);
//!   brotli/zstd/libpsl/libidn2 are content/name features, not protocols. Each of those stays
//!   an explicit `--without-*`/`--disable-*` so a build machine that happens to have the
//!   library installed cannot quietly change what this artifact contains.
//! - RTMP IS NOT IN THAT LIST, AND NEEDS NO FLAG: `docs/DEPRECATE.md:82` records "RTMP
//!   (removed in 8.20.0)", and the pinned tree has no RTMP code left to enable — the sole
//!   surviving mention is a `NULL /* rtmp version */` slot in `lib/version.c`. Revision 1's
//!   `--without-librtmp` was already dead by this pin (configure answers `unrecognized
//!   options: --without-librtmp`), so revision 2 drops it rather than implying a defense it
//!   does not provide.
//! - iOS uses curl 8.21's OpenSSL + Apple SecTrust integration. Revision 3 passes
//!   `--with-apple-sectrust` and explicitly disables file/path defaults for both iOS targets,
//!   so a transfer with no application CA override verifies through Security.framework instead
//!   of looking for Unix PEM files that do not exist in the iOS sandbox. A user-provided
//!   `CURLOPT_CAINFO`, `CURLOPT_CAPATH`, or `$CURL_CA_BUNDLE` remains a custom trust decision and
//!   follows libcurl's normal rule of replacing SecTrust unless `CURLSSLOPT_NATIVE_CA` is also
//!   requested.
//! - Other targets deliberately receive no `--with-ca-bundle`/`--with-ca-path`. `configure`'s
//!   `CURL_CHECK_CA_BUNDLE` probes the BUILD machine and bakes whatever absolute path it finds in
//!   as `CURL_CA_BUNDLE` (and bakes nothing when `--host=` makes it cross-compile). Portability
//!   there is solved at run time in `crates/elephc-curl/src/ca.rs`, which recognizes a working
//!   baked path and otherwise probes fixed distribution root-store files. That bridge first
//!   recognizes the `AppleSecTrust` feature and skips PEM discovery, so it cannot accidentally
//!   turn native iOS trust back off by setting `CURLOPT_CAINFO` itself.

use std::fs;
use std::path::{Path, PathBuf};

use crate::codegen_support::platform::Target;

use super::super::error::{NativeError, NativeErrorKind};
use super::super::recipe::RecipeRequest;
use super::super::toolchain::run_checked;
use super::util::{copy_regular, require_regular};

/// curl configure flags that make iOS use Security.framework's native trust store.
fn ca_configuration_args(target: Target) -> &'static [&'static str] {
    if target.is_ios() {
        &[
            "--with-apple-sectrust",
            "--without-ca-bundle",
            "--without-ca-path",
        ]
    } else {
        &[]
    }
}

/// Builds libcurl into the catalog-declared staging prefix.
pub fn build(request: &RecipeRequest<'_>) -> Result<(), NativeError> {
    let build = request.staging_prefix.join("build");
    let include = request.staging_prefix.join("include/curl");
    let library = request.staging_prefix.join("lib");
    fs::create_dir_all(&build)
        .map_err(|error| NativeError::io("create curl build directory", &build, error))?;
    fs::create_dir_all(&include)
        .map_err(|error| NativeError::io("create curl include directory", &include, error))?;
    fs::create_dir_all(&library)
        .map_err(|error| NativeError::io("create curl library directory", &library, error))?;

    let openssl_prefix = dependency_prefix(request, "openssl")?;
    let zlib_prefix = dependency_prefix(request, "zlib")?;
    let nghttp2_prefix = dependency_prefix(request, "nghttp2")?;
    let libssh2_prefix = dependency_prefix(request, "libssh2")?;

    let configure = request.source.join("configure");
    require_regular("curl", &configure)?;
    let mut command = request.toolchain.command(Path::new("/bin/sh"));
    command.current_dir(&build).arg(&configure).args([
        "--disable-shared",
        "--enable-static",
    ]);
    command.arg(format!("--with-openssl={}", openssl_prefix.display()));
    command.arg(format!("--with-zlib={}", zlib_prefix.display()));
    // Both take a PREFIX, and both reach the same `-I<prefix>/include -L<prefix>/lib` branch
    // of curl's configure on every machine: the pkg-config branch each one tries first is
    // scoped to `<prefix>/lib/pkgconfig` (`CURL_EXPORT_PCDIR` exports `PKG_CONFIG_LIBDIR`,
    // which OVERRIDES the system search path rather than adding to it), and these recipes
    // retain no `.pc` files, so pkg-config always answers "not found" whether or not the
    // build machine has one installed. A `.pc` would be actively wrong here anyway: recipes
    // build into a staging directory that is renamed to its content-addressed home
    // afterwards, so any absolute prefix baked in at build time would name a path that no
    // longer exists.
    command.arg(format!("--with-nghttp2={}", nghttp2_prefix.display()));
    command.arg(format!("--with-libssh2={}", libssh2_prefix.display()));
    command.args(ca_configuration_args(request.target));
    command.args([
        "--enable-smb",
        "--enable-ntlm",
        "--disable-ldap",
        "--disable-ldaps",
        "--disable-manual",
        "--disable-docs",
        "--without-libpsl",
        "--without-brotli",
        "--without-zstd",
        "--without-libidn2",
        "--without-ngtcp2",
        "--without-nghttp3",
        "--without-quiche",
    ]);
    if request.target != Target::detect_host() {
        command.arg(format!("--host={}", request.toolchain.target_tuple));
    }
    run_checked(&mut command, "configure trusted curl recipe")?;

    let mut make = request.toolchain.command(Path::new("make"));
    make.current_dir(&build).args(["-C", "lib"]);
    run_checked(&mut make, "build trusted libcurl static library")?;

    copy_regular("curl", &build.join("lib/.libs/libcurl.a"), &library.join("libcurl.a"))?;
    for header in request.version.retained_headers {
        let Some(name) = header.strip_prefix("include/curl/") else {
            continue;
        };
        copy_regular("curl", &request.source.join("include/curl").join(name), &include.join(name))?;
    }

    let archive = library.join("libcurl.a");
    let mut inspect = request.toolchain.command(&request.toolchain.ar);
    inspect.arg("t").arg(&archive);
    run_checked(&mut inspect, "validate trusted libcurl static archive")?;
    fs::remove_dir_all(&build)
        .map_err(|error| NativeError::io("remove trusted curl build tree", &build, error))?;
    Ok(())
}

/// Resolves one already-materialized catalog dependency's final artifact prefix.
fn dependency_prefix<'a>(request: &'a RecipeRequest<'a>, name: &str) -> Result<&'a PathBuf, NativeError> {
    request.dependency_prefixes.get(name).ok_or_else(|| {
        NativeError::new(
            NativeErrorKind::Build,
            format!("curl recipe requires the '{name}' dependency to be materialized first"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::platform::{AppleVariant, Arch, Platform};

    /// Verifies both first-class iOS artifacts enable SecTrust and carry no PEM default.
    #[test]
    fn ios_targets_use_apple_sectrust_without_file_defaults() {
        for target in [
            Target::new_apple(Arch::AArch64, AppleVariant::IOS),
            Target::new_apple(Arch::AArch64, AppleVariant::IOSSimulator),
        ] {
            assert_eq!(
                ca_configuration_args(target),
                &[
                    "--with-apple-sectrust",
                    "--without-ca-bundle",
                    "--without-ca-path",
                ],
                "{} must verify through the Apple system trust store",
                target.as_str()
            );
        }
    }

    /// Verifies the existing runtime PEM discovery remains authoritative off iOS.
    #[test]
    fn non_ios_targets_do_not_enable_apple_sectrust() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            assert!(
                ca_configuration_args(target).is_empty(),
                "{} must retain file-based CA discovery",
                target.as_str()
            );
        }
    }
}
