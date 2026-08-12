//! Purpose:
//! Coordinates the threadless broker process and shared process/signal primitives.
//! Isolation-specific request and pool supervision live in focused child modules.
//!
//! Called from:
//! - `crate::handler_broker::PrestartedBroker::start()` immediately after fork.
//!
//! Key details:
//! - Request mode never ignores `SIGCHLD`; every disposable PID is reaped.
//! - Both descriptor-transfer hops are acknowledged before sender-side close.
//! - A no-op `SIGCHLD` handler interrupts broker polling as soon as a child exits.
//! - Pool mode replaces crashed, cancelled, timed-out, and quota-retired children.
//! - Parent loss or forwarded `SIGTERM` terminates and reaps every descendant
//!   before the broker exits.

use std::io;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};

use super::control::{self, Dispatch};
use super::{BrokerConfig, BrokerMode};

mod handler;
mod pool;
mod request;

/// Poll interval used to notice parent death and reap children without a signal handler.
const BROKER_POLL_MILLIS: libc::c_int = 100;
/// Set by the broker's `SIGTERM` handler so its supervision loop can clean up.
static BROKER_SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Runs the selected broker model until the worker parent disappears.
pub(super) unsafe fn broker_loop(
    dispatch: RawFd,
    cancel: RawFd,
    worker_pid: libc::pid_t,
    handler: extern "C" fn(),
    config: BrokerConfig,
) {
    BROKER_SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
    install_shutdown_handler();
    ignore_signal(libc::SIGPIPE);
    install_child_exit_handler();
    match config.mode {
        BrokerMode::Pool => pool::run(dispatch, cancel, worker_pid, handler, config),
        BrokerMode::Request => request::run(dispatch, cancel, worker_pid, handler, config),
    }
    libc::close(dispatch);
    libc::close(cancel);
}

/// Receives one dispatch and confirms descriptor ownership before the worker releases its copy.
unsafe fn receive_acknowledged_dispatch(dispatch_fd: RawFd) -> io::Result<Option<Dispatch>> {
    let Some(dispatch) = control::recv_dispatch(dispatch_fd)? else {
        return Ok(None);
    };
    if let Err(error) = control::send_id(dispatch_fd, dispatch.id) {
        libc::close(dispatch.channel);
        return Err(error);
    }
    Ok(Some(dispatch))
}

/// Kills one exact child and waits until its PID is reaped.
unsafe fn kill_and_reap(pid: libc::pid_t) {
    libc::kill(pid, libc::SIGKILL);
    reap_exact(pid);
}

/// Waits for one exact child, retrying interrupted waits.
unsafe fn reap_exact(pid: libc::pid_t) {
    loop {
        let mut status = 0;
        let waited = libc::waitpid(pid, &mut status, 0);
        if waited == pid {
            return;
        }
        if waited < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        return;
    }
}

/// Records a broker shutdown request so normal loop cleanup can reap all children.
extern "C" fn handle_shutdown(_signal: libc::c_int) {
    BROKER_SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

/// Installs an interrupting `SIGTERM` handler for cooperative broker teardown.
unsafe fn install_shutdown_handler() {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = handle_shutdown as extern "C" fn(libc::c_int) as libc::sighandler_t;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(libc::SIGTERM, &action, std::ptr::null_mut());
}

/// No-op signal hook used only to interrupt the broker's blocking `poll`.
extern "C" fn handle_child_exit(_signal: libc::c_int) {}

/// Makes child exit observable immediately without ignoring or asynchronously reaping it.
unsafe fn install_child_exit_handler() {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction =
        handle_child_exit as extern "C" fn(libc::c_int) as libc::sighandler_t;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(libc::SIGCHLD, &action, std::ptr::null_mut());
}

/// Installs an ignored signal disposition in a threadless broker or handler child.
unsafe fn ignore_signal(signal: libc::c_int) {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = libc::SIG_IGN;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(signal, &action, std::ptr::null_mut());
}

/// Restores default signal semantics in a handler-owning child.
unsafe fn restore_default_signal(signal: libc::c_int) {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = libc::SIG_DFL;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(signal, &action, std::ptr::null_mut());
}
