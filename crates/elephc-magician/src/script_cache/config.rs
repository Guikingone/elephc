//! Purpose:
//! Owns the runtime script cache's configuration — the `opcache.*` directives that
//! actually govern caching behaviour, as opposed to the ones elephc merely reports.
//! Defaults come from the shared directive matrix; generated code overrides them with
//! the compile-time `--ini`-effective values before the first include.
//!
//! Called from:
//! - `crate::script_cache::store` for every lookup and fill decision.
//! - `crate::ffi::context::__elephc_eval_configure_opcache()` (generated code).
//!
//! Key details:
//! - `enabled` is the master gate. It mirrors `opcache_cache_enabled`, so a default
//!   CLI binary caches nothing and behaves exactly as it did before this module
//!   existed; `--web` and `--ini opcache.enable_cli=1` turn it on.
//! - Thread-local, mirroring `crate::eval_php_profile`: the configuration is a
//!   property of the whole compiled binary, and elephc programs execute the setter
//!   and every eval fragment on one thread, while parallel `cargo test` threads stay
//!   isolated from one another.
//! - Defaults here must stay reachable without generated code: every harness that
//!   links this archive directly observes the cache DISABLED, which is the
//!   pre-existing behaviour.

use std::cell::RefCell;

/// The `opcache.*` subset that governs what the runtime script cache actually does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptCacheConfig {
    /// The master gate: `opcache.enable && (web || opcache.enable_cli)`.
    pub(crate) enabled: bool,
    /// `opcache.validate_timestamps` — revalidate an entry against the file's mtime.
    pub(crate) validate_timestamps: bool,
    /// `opcache.revalidate_freq` — seconds between two revalidations of one entry.
    pub(crate) revalidate_freq: u64,
    /// `opcache.max_file_size` — refuse larger files; `0` means no limit.
    pub(crate) max_file_size: u64,
    /// `opcache.memory_consumption` — the cache's byte budget.
    pub(crate) memory_consumption: usize,
    /// `opcache.max_accelerated_files` — the cache's entry-count ceiling.
    pub(crate) max_accelerated_files: usize,
}

impl ScriptCacheConfig {
    /// Returns the configuration a binary without generated OPcache wiring observes.
    ///
    /// Disabled, so linking this archive without elephc's codegen — every test harness
    /// in this crate included — keeps the pre-cache behaviour of re-reading and
    /// re-parsing each include.
    pub(crate) const fn disabled() -> Self {
        Self {
            enabled: false,
            validate_timestamps: true,
            revalidate_freq: 2,
            max_file_size: 0,
            memory_consumption: 128 * 1024 * 1024,
            max_accelerated_files: 10_000,
        }
    }

    /// Returns whether a file of `size` bytes may be admitted under `max_file_size`.
    ///
    /// php-src treats `0` as "no limit" rather than "refuse everything", and compares
    /// with a strict `>`: a file of exactly `max_file_size` bytes is admitted.
    pub(crate) const fn admits_size(&self, size: u64) -> bool {
        self.max_file_size == 0 || size <= self.max_file_size
    }
}

thread_local! {
    /// The configuration the binary embedding this bridge was compiled with.
    static SCRIPT_CACHE_CONFIG: RefCell<ScriptCacheConfig> =
        RefCell::new(ScriptCacheConfig::disabled());
}

/// Installs the compile-time OPcache configuration for the current thread.
pub(crate) fn set_config(config: ScriptCacheConfig) {
    SCRIPT_CACHE_CONFIG.with(|cell| *cell.borrow_mut() = config);
}

/// Returns a copy of the configuration active on the current thread.
pub(crate) fn config() -> ScriptCacheConfig {
    SCRIPT_CACHE_CONFIG.with(|cell| cell.borrow().clone())
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Pins the configuration defaults and the `max_file_size` admission rule.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - The disabled default is load-bearing: it is what keeps a CLI binary and every
    //!   direct consumer of this archive on the pre-cache code path.

    use super::*;

    /// Verifies the no-codegen default leaves the cache off.
    #[test]
    fn default_configuration_disables_the_cache() {
        assert!(!ScriptCacheConfig::disabled().enabled);
    }

    /// Verifies `max_file_size = 0` admits any size, matching php-src's "no limit".
    #[test]
    fn zero_max_file_size_admits_every_file() {
        let config = ScriptCacheConfig::disabled();

        assert!(config.admits_size(0));
        assert!(config.admits_size(u64::MAX));
    }

    /// Verifies a non-zero `max_file_size` refuses only strictly larger files.
    #[test]
    fn max_file_size_admits_the_boundary_and_refuses_beyond_it() {
        let config = ScriptCacheConfig {
            max_file_size: 1024,
            ..ScriptCacheConfig::disabled()
        };

        assert!(config.admits_size(1023));
        assert!(config.admits_size(1024));
        assert!(!config.admits_size(1025));
    }

    /// Verifies the thread-local setter round-trips and stays isolated per thread.
    #[test]
    fn configuration_round_trips_on_the_installing_thread() {
        let installed = ScriptCacheConfig {
            enabled: true,
            revalidate_freq: 7,
            ..ScriptCacheConfig::disabled()
        };
        set_config(installed.clone());

        assert_eq!(config(), installed);
        assert!(std::thread::spawn(|| !config().enabled)
            .join()
            .expect("probe thread should not panic"));
    }
}
