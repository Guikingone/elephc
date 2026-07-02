//! Purpose:
//! Defines PHP `ext/pcntl` signal integer constants exposed by elephc.
//! Keeps signal numbers and handler pseudo-constants in one source of truth for
//! type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - The registered signal numbers (`SIGHUP`, `SIGINT`, `SIGQUIT`, `SIGKILL`,
//!   `SIGALRM`, `SIGTERM`) share the same POSIX values across every supported
//!   target (macOS and Linux), so a single table is target-agnostic.
//! - `SIG_DFL`/`SIG_IGN` are the default/ignore handler pseudo-constants.

/// Tuple of `(name, value)` pairs for PHP `ext/pcntl` signal integer constants.
///
/// Covers the common terminal/alarm signals plus the `SIG_DFL`/`SIG_IGN`
/// handler pseudo-constants used with `pcntl_signal()`.
pub(crate) const PCNTL_INT_CONSTANTS: &[(&str, i64)] = &[
    ("SIGHUP", 1),
    ("SIGINT", 2),
    ("SIGQUIT", 3),
    ("SIGKILL", 9),
    ("SIGALRM", 14),
    ("SIGTERM", 15),
    ("SIG_DFL", 0),
    ("SIG_IGN", 1),
];

/// Tuple of `(name, macos_value, linux_value)` for `ext/pcntl` user signals
/// whose integer values DIFFER across supported targets (macOS vs Linux), so
/// they cannot live in the target-agnostic `PCNTL_INT_CONSTANTS`. macOS:
/// SIGUSR1=30, SIGUSR2=31. Linux (x86_64 and aarch64): SIGUSR1=10, SIGUSR2=12.
/// The value is selected by the codegen prescan from the compile target.
pub(crate) const PCNTL_PLATFORM_SIGNALS: &[(&str, i64, i64)] = &[
    ("SIGUSR1", 30, 10),
    ("SIGUSR2", 31, 12),
];

#[cfg(test)]
mod tests {
    use super::{PCNTL_INT_CONSTANTS, PCNTL_PLATFORM_SIGNALS};

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        PCNTL_INT_CONSTANTS
            .iter()
            .find(|(constant_name, _)| *constant_name == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Verifies signal numbers match the POSIX values shared across targets.
    #[test]
    fn test_signal_number_values() {
        assert_eq!(value_of("SIGHUP"), 1);
        assert_eq!(value_of("SIGINT"), 2);
        assert_eq!(value_of("SIGQUIT"), 3);
        assert_eq!(value_of("SIGKILL"), 9);
        assert_eq!(value_of("SIGALRM"), 14);
        assert_eq!(value_of("SIGTERM"), 15);
    }

    /// Verifies the handler pseudo-constants match PHP's ext/pcntl values.
    #[test]
    fn test_signal_handler_pseudo_constants() {
        assert_eq!(value_of("SIG_DFL"), 0);
        assert_eq!(value_of("SIG_IGN"), 1);
    }

    /// Asserts no duplicate names exist in `PCNTL_INT_CONSTANTS`.
    #[test]
    fn test_pcntl_constants_have_unique_names() {
        let mut names: Vec<&str> = PCNTL_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate pcntl constant name");
    }

    /// Verifies the platform-conditional SIGUSR1/SIGUSR2 values match the
    /// macOS and Linux signal numbers (macOS: 30/31, Linux: 10/12). Cross-checked
    /// with `php -r 'echo SIGUSR1;'` on macOS (prints `30`).
    #[test]
    fn test_platform_signal_values() {
        let sigusr1 = PCNTL_PLATFORM_SIGNALS
            .iter()
            .find(|(n, _, _)| *n == "SIGUSR1")
            .expect("SIGUSR1 present");
        assert_eq!(sigusr1.1, 30, "macOS SIGUSR1");
        assert_eq!(sigusr1.2, 10, "Linux SIGUSR1");
        let sigusr2 = PCNTL_PLATFORM_SIGNALS
            .iter()
            .find(|(n, _, _)| *n == "SIGUSR2")
            .expect("SIGUSR2 present");
        assert_eq!(sigusr2.1, 31, "macOS SIGUSR2");
        assert_eq!(sigusr2.2, 12, "Linux SIGUSR2");
    }

    /// Asserts no duplicate names exist in `PCNTL_PLATFORM_SIGNALS` and that
    /// none of its names collide with the target-agnostic `PCNTL_INT_CONSTANTS`.
    #[test]
    fn test_platform_signals_have_unique_names() {
        let mut names: Vec<&str> = PCNTL_PLATFORM_SIGNALS.iter().map(|(n, _, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate platform signal name");
        for (n, _, _) in PCNTL_PLATFORM_SIGNALS {
            assert!(
                !PCNTL_INT_CONSTANTS.iter().any(|(cn, _)| cn == n),
                "{n} collides with PCNTL_INT_CONSTANTS"
            );
        }
    }
}
