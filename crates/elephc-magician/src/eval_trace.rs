//! Purpose:
//! Answers "is the eval trace on?" from a cached value instead of the environment, so a DISABLED
//! trace costs one relaxed atomic load rather than a `getenv`.
//!
//! Called from:
//! - every `[elephc-eval-trace]` emission site in the interpreter, the parser and the FFI layer.
//! - `crate::interpreter::builtins::network_env::putenv` to invalidate the cache.
//!
//! Key details:
//! - `ELEPHC_EVAL_TRACE` is read at 100+ sites, several of them per interpreted expression. `getenv`
//!   is not free: on macOS it takes `os_unfair_lock` and linearly scans `environ`, and a Symfony
//!   `--web` worker spent more samples there than anywhere else except the idle wait — for a
//!   facility that was switched OFF.
//! - The cache is INVALIDATED rather than permanent, because PHP's `putenv()` can set the variable
//!   mid-run and a `OnceLock` would then report the old answer forever. Invalidation keeps the
//!   observable behaviour exactly what reading the environment every time gave.
//! - Two questions are cached in one slot because they are answered by one lookup: whether the
//!   variable is set at all, and whether its value selects the FFI-entry trace.

use std::sync::atomic::{AtomicU8, Ordering};

/// The cached verdict has not been computed since the last invalidation.
const TRACE_UNKNOWN: u8 = 0;
/// `ELEPHC_EVAL_TRACE` is unset: every trace site is off.
const TRACE_OFF: u8 = 1;
/// `ELEPHC_EVAL_TRACE` is set to a value that does not select the FFI-entry trace.
const TRACE_ON: u8 = 2;
/// `ELEPHC_EVAL_TRACE` is set to `ffi` or `all`, which also selects the FFI-entry trace.
const TRACE_ON_FFI: u8 = 3;

static TRACE_STATE: AtomicU8 = AtomicU8::new(TRACE_UNKNOWN);

/// Returns whether any eval trace output is enabled.
pub(crate) fn enabled() -> bool {
    state() >= TRACE_ON
}

/// Returns whether the per-FFI-entry trace is enabled, which needs `ffi` or `all` specifically.
pub(crate) fn ffi_enabled() -> bool {
    state() == TRACE_ON_FFI
}

/// Forgets the cached verdict, so the next question re-reads the environment.
///
/// `putenv()` is the one way a running PHP program can change the answer.
pub(crate) fn invalidate() {
    TRACE_STATE.store(TRACE_UNKNOWN, Ordering::Relaxed);
}

/// Returns the cached verdict, reading the environment once when it is not known.
///
/// A race between two threads computing it is harmless: both read the same environment and store
/// the same value, so the slot converges without needing a lock.
fn state() -> u8 {
    let cached = TRACE_STATE.load(Ordering::Relaxed);
    if cached != TRACE_UNKNOWN {
        return cached;
    }
    let computed = match std::env::var_os("ELEPHC_EVAL_TRACE") {
        None => TRACE_OFF,
        Some(level) => {
            let level = level.to_string_lossy();
            if level == "ffi" || level == "all" {
                TRACE_ON_FFI
            } else {
                TRACE_ON
            }
        }
    };
    TRACE_STATE.store(computed, Ordering::Relaxed);
    computed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the verdict is recomputed after an invalidation rather than frozen.
    #[test]
    fn recomputes_after_invalidation() {
        TRACE_STATE.store(TRACE_ON_FFI, Ordering::Relaxed);
        assert!(enabled());
        assert!(ffi_enabled());
        invalidate();
        assert_eq!(TRACE_STATE.load(Ordering::Relaxed), TRACE_UNKNOWN);
    }

    /// Verifies an enabled trace that does not name `ffi` leaves the FFI-entry trace off.
    #[test]
    fn a_plain_level_does_not_enable_the_ffi_entry_trace() {
        TRACE_STATE.store(TRACE_ON, Ordering::Relaxed);
        assert!(enabled());
        assert!(!ffi_enabled());
        invalidate();
    }
}
