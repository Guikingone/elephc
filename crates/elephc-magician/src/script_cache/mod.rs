//! Purpose:
//! The runtime script cache for dynamically included PHP files — the only tier of an
//! elephc binary that is not already compiled at link time, and therefore the only
//! one where an opcode-cache-shaped saving still exists.
//!
//! Called from:
//! - `crate::interpreter::include_exec` for every runtime include/require.
//! - `crate::ffi::context` for the OPcache configuration and API bridge symbols.
//!
//! Key details:
//! - The included file, not the eval fragment, is the unit. `crate::parse_cache`
//!   remains the byte-keyed cache for `eval()` of an arbitrary string, where there is
//!   no path to key on; this module keys on the canonical path and so can also skip
//!   the file read and the `<?php`/`?>` scan that the byte-keyed cache cannot.
//! - Measured on the eval-hosted include benchmark, the cache addresses ~20.6 ns per
//!   source byte below the old 64 KiB fragment cap, and the whole re-parse above it —
//!   a 128 KiB include cost 10.98 ms uncached against a ~0.26 ms fixed floor.
//! - Everything here is inert unless the OPcache cache is enabled for this binary.
//! - Two submodules are about DIAGNOSTICS rather than caching, and are here because
//!   they share that same enabled gate: `accel_log` is php-src's `zend_accel_error`
//!   channel (timestamped, pid-tagged, gated by `opcache.log_verbosity_level` rather
//!   than by `error_reporting`), and `file_cache` reproduces its startup refusal of a
//!   bad `opcache.file_cache`. elephc has no on-disk opcode cache, but reference PHP
//!   REFUSES TO START on a bad setting, and that refusal is observable.

pub(crate) mod accel_log;
pub(crate) mod config;
pub(crate) mod file_cache;
pub(crate) mod segments;
pub(crate) mod store;

pub(crate) use accel_log::{set_config as set_accel_log_config, AccelLogConfig};
pub(crate) use config::{config, set_config, ScriptCacheConfig};
pub(crate) use file_cache::{validate_file_cache_directives, FileCacheConfig};
pub(crate) use segments::ScriptSegment;
pub(crate) use store::load_script;

#[allow(unused_imports)]
pub use store::{
    cached_scripts, compile_file, discard, is_cached, schedule_restart, stats, CachedScriptInfo,
    ScriptCacheStats,
};
