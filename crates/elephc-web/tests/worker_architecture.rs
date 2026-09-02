//! Purpose:
//! Architectural regressions for safe PHP handler isolation in broker-backed workers.
//!
//! Called from:
//! - `cargo test -p elephc-web --test worker_architecture` through Rust's test harness.
//!
//! Key details:
//! - Source invariants prevent forking or consuming Tokio's blocking pool from a request task.

/// Verifies request-time isolation uses a prestarted executor boundary instead
/// of calling `fork()` after the Tokio runtime and connection tasks exist.
#[test]
fn request_handler_path_does_not_fork_from_tokio() {
    let source = include_str!("../src/isolated_worker.rs");
    let isolated = source
        .split("async fn run_handler_isolated")
        .nth(1)
        .expect("isolated handler function remains present")
        .split("\n///")
        .next()
        .expect("isolated handler function has a bounded source section");

    assert!(
        !isolated.contains("libc::fork()"),
        "request tasks must dispatch to a prestarted handler executor, never fork a Tokio process"
    );
    assert!(
        !isolated.contains("spawn_blocking"),
        "handler IPC waits must not consume Tokio's shared blocking pool per request"
    );
}
