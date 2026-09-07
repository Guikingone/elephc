//! Purpose:
//! Identifies a Linux process across pid reuse by the kernel's starttime.
//!
//! Called from:
//! - `crate::monitor::attach::image_for()`, once, when the image is built.
//! - `crate::monitor::ptrace::attach_window()`, each window, before sampling.
//!
//! Key details:
//! - `/proc/<pid>/stat` field 22 (`starttime`) is unique for a pid until reboot.
//! - The parser is host-independent so the reuse check is tested without ptrace.
//! - A changed starttime is not "the target exited": it is a different process
//!   wearing the same pid, and sampling it against the original image is a
//!   confidently wrong profile.

/// The kernel's starttime for the process an attach started with.
///
/// Stored on the image rather than re-read from a pid alone, because a pid is
/// not an identity: after the original process dies the kernel can hand the
/// same number to someone else, and `/proc/<pid>` would then describe that
/// new program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProcessIdentity {
    pub(crate) pid: u32,
    pub(crate) starttime: u64,
}

/// Whether a pid is still the process we attached to, gone, or someone else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Identity {
    /// Same pid, same starttime.
    Same,
    /// `/proc` has nothing for this pid — the process ended.
    Gone,
    /// Same pid, different starttime: the kernel reused the number.
    Replaced,
}

/// Field 22 of `/proc/<pid>/stat` (1-based), which is starttime in clock ticks.
///
/// The comm field is in parentheses and may contain spaces or `)`, so the
/// reliable split is "everything after the last `)`", then the 20th token
/// (0-based 19) counting from `state`.
pub(crate) fn starttime_from_stat(stat: &str) -> Option<u64> {
    let tail = stat.rsplit_once(')')?.1;
    tail.split_whitespace().nth(19)?.parse().ok()
}

/// Reads the starttime of a live pid, or `None` when `/proc` has no such task.
#[cfg(target_os = "linux")]
pub(crate) fn process_starttime(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    starttime_from_stat(&stat)
}

/// Builds the identity of a live pid, or `None` when it cannot be read.
#[cfg(target_os = "linux")]
pub(crate) fn identity_of(pid: u32) -> Option<ProcessIdentity> {
    Some(ProcessIdentity { pid, starttime: process_starttime(pid)? })
}

/// Compares a remembered identity against what `/proc` says now.
#[cfg(target_os = "linux")]
pub(crate) fn identity_of_pid(expected: ProcessIdentity) -> Identity {
    match process_starttime(expected.pid) {
        None => Identity::Gone,
        Some(starttime) if starttime == expected.starttime => Identity::Same,
        Some(_) => Identity::Replaced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A realistic `/proc/stat` line: comm can contain spaces, and starttime
    /// sits at a fixed field after the closing paren, not at a fixed byte.
    fn stat_with(comm: &str, starttime: u64) -> String {
        // state ppid pgrp session tty tpgid flags minflt cminflt majflt cmajflt
        // utime stime cutime cstime priority nice num_threads itrealvalue starttime
        format!(
            "4242 ({comm}) R 1 1 1 0 -1 4194304 0 0 0 0 10 20 0 0 20 0 1 0 {starttime} 123456 0"
        )
    }

    /// The parser has to survive a comm that itself contains `)` and spaces,
    /// because that is legal and the naive split-on-first-paren reads the
    /// starttime from the wrong token.
    #[test]
    fn starttime_is_read_after_the_last_closing_paren() {
        assert_eq!(starttime_from_stat(&stat_with("elephc", 98765)), Some(98765));
        assert_eq!(
            starttime_from_stat(&stat_with("a ) weird name", 42)),
            Some(42)
        );
        assert_eq!(starttime_from_stat("no-paren 1 2 3"), None);
        assert_eq!(starttime_from_stat("1 (short) R 1"), None);
    }
}
