//! Purpose:
//! Verifies live-loop exit codes and diagnostics at the caller boundary.
//!
//! Called from:
//! - `monitor::local` through the Rust test harness.
//!
//! Key details:
//! - Subprocesses capture diagnostics without redirecting the harness's stderr.
//! - Linux self-attach always refuses, independently of yama or capabilities.

use super::*;

/// Runs the live caller in an isolated harness process and captures its diagnostics.
fn outcome(mode: &str) -> process::Output {
    process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "monitor::local::review_tests::live_outcome_child", "--nocapture"])
        .env("ELEPHC_TEST_LIVE_OUTCOME", mode)
        .output().unwrap()
}

/// A kernel refusal must reach the caller as failure without reporting target death.
#[cfg(target_os = "linux")]
#[test]
fn live_attach_refusal_is_failure_without_claiming_target_finished() {
    let output = outcome("refused");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("cannot attach to the target"), "{stderr}");
    assert!(!stderr.contains("program finished"), "{stderr}");
}

/// An all-stuck window must redraw and reach the next attach window on a live pid.
#[cfg(target_os = "linux")]
#[test]
fn live_attach_survives_an_all_stuck_window() {
    let output = outcome("stuck");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(!stderr.contains("no samples captured"), "{stderr}");
    assert!(stdout.matches("live ·").count() >= 2, "{stdout}");
}

/// A one-shot empty attach fails clearly instead of exporting a successful empty profile.
#[cfg(target_os = "linux")]
#[test]
fn one_shot_attach_reports_a_window_with_only_stuck_threads() {
    let output = outcome("stuck-once");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("target threads did not stop"), "{stderr}");
    assert!(!stderr.contains("may have exited"), "{stderr}");
}

/// A dead control socket does not imply the launched process has terminated.
#[test]
fn live_lost_channel_is_failure_and_leaves_running_target_alone() {
    let output = outcome("lost");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("lost the channel"), "{stderr}");
    assert!(!stderr.contains("program finished"), "{stderr}");
}

/// A confirmed early child exit retains the helpful short-lived-program diagnostic.
#[test]
fn live_finished_child_reports_short_window_without_failure() {
    let output = outcome("finished");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("program finished before the first window"), "{stderr}");
}

/// Subprocess entry point; exits with the actual live outcome after fixture cleanup.
#[test]
fn live_outcome_child() {
    let Ok(mode) = std::env::var("ELEPHC_TEST_LIVE_OUTCOME") else { return };
    let args = ["--attach".to_string(), std::process::id().to_string(), "--live".to_string()];
    let mut cmd = parse_monitor_args(&args).unwrap();
    cmd.duration_secs = 0;
    #[cfg(target_os = "linux")]
    if mode == "refused" {
        let pid = std::process::id();
        let mut image = super::super::attach::image_for(pid)
            .unwrap_or_else(|_| panic!("unstripped test executable must have an image"));
        let result = run_live(&cmd, pid, None, None, Some(&mut image));
        assert!(!result.leave_target_running);
        std::process::exit(result.code);
    }
    #[cfg(target_os = "linux")]
    if mode == "stuck" || mode == "stuck-once" {
        let mut tracee = super::super::ptrace::process_tests::Tracee::start();
        tracee.seized = true;
        let pid = tracee.child.id();
        let mut image = super::super::attach::image_for(pid)
            .unwrap_or_else(|error| panic!("{}", error.reason));
        let delayed = super::super::ptrace::test_faults::DelayedStops::new(pid, 2);
        cmd.duration_secs = 1;
        if mode == "stuck-once" {
            let code = run_once(&cmd, pid, None, None, Some(&mut image));
            assert_eq!(image.held, vec![pid]);
            drop(delayed);
            drop(tracee);
            std::process::exit(code);
        }
        // End after the held relationship is released, with a watchdog for regressions.
        let ending = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let mut was_traced = false;
            while std::time::Instant::now() < deadline {
                let status = std::fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
                let tracer = status.lines().find_map(|line| {
                    line.strip_prefix("TracerPid:")?.trim().parse::<u32>().ok()
                }).unwrap();
                if was_traced && tracer == 0 {
                    break;
                }
                was_traced |= tracer != 0;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
        });
        let result = run_live(&cmd, pid, None, None, Some(&mut image));
        ending.join().unwrap();
        let _ = tracee.child.wait();
        tracee.seized = false;
        assert!(delayed.detach_attempts() >= 1, "live must recover the held attach");
        drop(delayed);
        drop(tracee);
        std::process::exit(result.code);
    }
    let mut child = process::Command::new("sh")
        .args(["-c", if mode == "finished" { "exit 0" } else { "exec sleep 30" }])
        .spawn().unwrap();
    let mut channel = open_polled_control_channel().unwrap();
    // SAFETY: close the owned peer before asking, then prevent a double close.
    unsafe { libc::close(channel.child) };
    channel.forget_child();
    if mode == "finished" {
        child.wait().unwrap();
    }
    let result = run_live(&cmd, child.id(), Some(&mut child), Some(&channel), None);
    let still_running = child.try_wait().unwrap().is_none();
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(still_running, mode == "lost");
    assert_eq!(result.leave_target_running, mode == "lost");
    std::process::exit(result.code);
}
