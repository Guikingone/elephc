//! Purpose:
//! Linux process regressions for sampling across pending signal-delivery stops.
//!
//! Called from:
//! - `monitor::ptrace` through the Rust test harness.
//!
//! Key details:
//! - Uses a real CPU-bound C tracee; requires cc and permission to trace a child.
//! - Every fixture detaches, kills and reaps its child even after an assertion fails.

use super::*;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Tracee {
    child: Child,
    directory: std::path::PathBuf,
    seized: bool,
}

impl Tracee {
    /// Builds and starts a traceable process, waiting for its handlers to be ready.
    fn start() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!("elephc-signal-{}-{id}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("tracee.c");
        let binary = directory.join("tracee");
        std::fs::write(&source, include_str!("../../tests/fixtures/monitor/signals.c")).unwrap();
        let compiled = Command::new("cc").args(["-O0", "-fno-omit-frame-pointer"])
            .arg(&source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let child = Command::new(binary).stdout(Stdio::piped()).spawn().unwrap();
        let mut tracee = Self { child, directory, seized: false };
        let fd = tracee.child.stdout.as_ref().unwrap().as_raw_fd();
        // SAFETY: this only changes nonblocking mode on our live pipe reader.
        assert_ne!(unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) }, -1);
        assert_eq!(tracee.read_marker(), b'R');
        tracee
    }

    /// Waits at most two seconds for one async-signal-safe handler notification.
    fn read_marker(&mut self) -> u8 {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut marker = [0];
            match self.child.stdout.as_mut().unwrap().read(&mut marker) {
                Ok(1) => return marker[0],
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                other => panic!("tracee pipe: {other:?}"),
            }
            assert!(Instant::now() < deadline, "tracee did not deliver the signal");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Sends one handled signal and waits for its delivery stop to become pending.
    fn signal_stop(&self, signal: libc::c_int) {
        // SAFETY: the target is our live fixture with handlers installed.
        assert_eq!(unsafe { libc::kill(self.child.id() as libc::pid_t, signal) }, 0);
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.progress().0 != 't' {
            assert!(Instant::now() < deadline, "signal-delivery stop never arrived");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Reads task state and CPU ticks without stopping the process being measured.
    fn progress(&self) -> (char, u64) {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", self.child.id())).unwrap();
        let fields: Vec<_> = stat.rsplit_once(')').unwrap().1.split_whitespace().collect();
        let ticks = fields[11].parse::<u64>().unwrap() + fields[12].parse::<u64>().unwrap();
        (fields[0].chars().next().unwrap(), ticks)
    }
}

impl Drop for Tracee {
    /// Releases the trace relationship and reaps the fixture on every exit path.
    fn drop(&mut self) {
        if self.seized {
            detach(self.child.id());
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// Delivers a signal between ticks and verifies CPU progress before detachment.
fn sampling_preserves_signal_and_progress(signal: libc::c_int, marker: u8) {
    let mut tracee = Tracee::start();
    let pid = tracee.child.id();
    seize(pid).expect("kernel must allow tracing the child for this regression");
    tracee.seized = true;
    assert!(matches!(sample_thread(pid, pid), SampleOutcome::Stack(_)));
    tracee.signal_stop(signal);
    let before = tracee.progress().1;
    // Keep tracing throughout. A resume deferred until detach cannot pass this.
    for _ in 0..40 {
        assert!(matches!(sample_thread(pid, pid), SampleOutcome::Stack(_)));
        std::thread::sleep(Duration::from_millis(10));
    }
    let after = tracee.progress().1;
    assert!(after > before + 1, "target stopped making progress: {before} -> {after}");
    assert_eq!(tracee.read_marker(), marker);
    // Detach must consume a pending delivery stop without manufacturing a new one.
    tracee.signal_stop(signal);
    detach(pid);
    tracee.seized = false;
    assert_eq!(tracee.read_marker(), marker, "detach must deliver the pending signal");
    assert_eq!(unsafe { libc::kill(pid as libc::pid_t, signal) }, 0);
    assert_eq!(tracee.read_marker(), marker, "handlers must also work after detach");
}

/// A running seized thread that never stops must not park the sampler.
///
/// `PTRACE_SEIZE` leaves the tracee running. Waiting for a stop that will not
/// come is the D-state hang: the bound has to fire, and it has to fire as
/// `TimedOut` so `sample_thread` can skip the tid for later ticks this window.
#[test]
fn wait_for_stop_times_out_on_a_running_seized_tracee() {
    let mut tracee = Tracee::start();
    let pid = tracee.child.id();
    seize(pid).expect("kernel must allow tracing the child for this bound");
    tracee.seized = true;
    let started = Instant::now();
    let error = wait_for_stop(pid).expect_err("a running seized thread has no stop to report");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut, "{error}");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "the wait must fail closed, not block: {:?}",
        started.elapsed()
    );
}

/// A successful detach must actually release the relationship.
///
/// The next window seizes again. If detach issued `PTRACE_DETACH` against a
/// running tid and ignored the `ESRCH`, the second seize would fail and a
/// live view would report a refusal about a program it was still tracing.
#[test]
fn detach_releases_a_runnable_tracee_so_it_can_be_seized_again() {
    let mut tracee = Tracee::start();
    let pid = tracee.child.id();
    seize(pid).expect("kernel must allow tracing the child for this release");
    tracee.seized = true;
    assert!(detach(pid), "a runnable seized thread must stop and detach");
    tracee.seized = false;
    seize(pid).expect("a released thread must be seizable again");
    tracee.seized = true;
}

/// A pending handled SIGUSR1 cannot stall a process for the capture window.
#[test]
fn handled_signal_keeps_running_during_capture() {
    sampling_preserves_signal_and_progress(libc::SIGUSR1, b'U');
}

/// Genuine SIGTRAP delivery reaches its handler while sampler traps stay synthetic.
#[test]
fn genuine_sigtrap_reaches_handler_during_capture() {
    sampling_preserves_signal_and_progress(libc::SIGTRAP, b'T');
}
