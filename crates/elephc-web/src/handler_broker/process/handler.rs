//! Purpose:
//! Executes one materialized web request inside a handler child and writes bounded failures.
//!
//! Called from:
//! - `super::request` for disposable request children.
//! - `super::pool` for each dispatch handled by a persistent child.
//!
//! Key details:
//! - Request-local session and response state are reset before invoking generated PHP.
//! - The configured execution alarm is armed only around handler execution and cancelled after it.

use std::fs::File;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};

use crate::handler_ipc;
use crate::request_state::{self, RequestMeta};

use super::super::MAX_EXEC_SECS;
use super::{restore_default_signal};

/// Reads and executes one request in a disposable child, then exits immediately.
pub(super) unsafe fn run_request_child(handler: extern "C" fn(), response_fd: RawFd, id: u64) -> ! {
    restore_default_signal(libc::SIGCHLD);
    restore_default_signal(libc::SIGTERM);
    let stream = File::from_raw_fd(response_fd);
    let complete = execute_handler_request(handler, stream, id);
    libc::_exit(if complete { 0 } else { 1 });
}

/// Materializes one request snapshot, resets request-local state, and invokes PHP.
pub(super) unsafe fn execute_handler_request(handler: extern "C" fn(), mut stream: File, id: u64) -> bool {
    let request = match handler_ipc::read_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("elephc-web: handler could not read request {id}: {error}");
            write_failure_response(stream.as_raw_fd());
            return false;
        }
    };
    crate::session::elephc_web_session_reset();
    request_state::set_request(
        request.method,
        request.uri,
        request.path,
        request.query,
        request.headers,
        request.body,
        RequestMeta {
            remote_addr: request.remote_addr,
            remote_port: request.remote_port,
            server_addr: request.server_addr,
            server_port: request.server_port,
            protocol: request.protocol,
        },
    );
    request_state::begin_response_stream(stream.as_raw_fd());
    request_state::set_capture(true);
    let seconds = MAX_EXEC_SECS.load(std::sync::atomic::Ordering::Relaxed);
    if seconds > 0 {
        libc::alarm(seconds);
    }
    handler();
    if seconds > 0 {
        libc::alarm(0);
    }
    let complete = request_state::finish_response_stream();
    if !complete {
        eprintln!("elephc-web: handler could not finish response stream");
    }
    complete
}

/// Emits the bounded 500 response used when a handler cannot be started.
pub(super) unsafe fn write_failure_response(fd: RawFd) {
    let headers = Vec::new();
    let body = b"Internal Server Error";
    let _ = handler_ipc::write_response_start(fd, 500, &headers)
        && handler_ipc::write_response_chunks(fd, body)
        && handler_ipc::write_response_end(fd);
}
