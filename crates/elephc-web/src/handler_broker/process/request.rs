//! Purpose:
//! Supervises request-isolation handlers, with one tracked child PID per dispatched request.
//!
//! Called from:
//! - `super::broker_loop()` when `BrokerMode::Request` is selected.
//!
//! Key details:
//! - Cancellations kill live children, close queued descriptors, or briefly record only IDs
//!   that are newer than every acknowledged dispatch.
//! - Every disposable child is synchronously reaped during normal supervision or shutdown.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::os::fd::RawFd;
use std::sync::atomic::Ordering;

use super::super::control::{self, Dispatch};
use super::super::BrokerConfig;
use super::handler::{run_request_child, write_failure_response};
use super::{
    kill_and_reap, reap_exact, receive_acknowledged_dispatch, BROKER_POLL_MILLIS,
    BROKER_SHUTDOWN_REQUESTED,
};

/// Supervises one disposable handler PID per accepted request.
pub(super) unsafe fn run(
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    worker_pid: libc::pid_t,
    handler: extern "C" fn(),
    config: BrokerConfig,
) {
    let mut active = HashMap::<u64, libc::pid_t>::new();
    let mut pending = VecDeque::<Dispatch>::new();
    let mut early_cancels = HashSet::<u64>::new();
    let mut greatest_dispatch = 0u64;
    loop {
        if BROKER_SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            break;
        }
        reap_request_children(&mut active);
        start_pending_requests(
            &mut pending,
            &mut active,
            handler,
            config.concurrency,
            dispatch_fd,
            cancel_fd,
        );
        if libc::getppid() != worker_pid {
            break;
        }
        let mut poll_fds = [
            libc::pollfd {
                fd: dispatch_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: cancel_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let ready = libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            BROKER_POLL_MILLIS,
        );
        if ready < 0 {
            if io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            break;
        }
        if poll_fds[1].revents & libc::POLLIN != 0 {
            match control::recv_id(cancel_fd) {
                Ok(Some(id)) => cancel_request(
                    id,
                    greatest_dispatch,
                    &mut active,
                    &mut pending,
                    &mut early_cancels,
                ),
                Ok(None) | Err(_) => break,
            }
        }
        if poll_fds[0].revents & libc::POLLIN != 0 {
            match receive_acknowledged_dispatch(dispatch_fd) {
                Ok(Some(dispatch)) => {
                    greatest_dispatch = greatest_dispatch.max(dispatch.id);
                    if early_cancels.remove(&dispatch.id) {
                        libc::close(dispatch.channel);
                    } else {
                        pending.push_back(dispatch);
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        if poll_fds
            .iter()
            .any(|fd| fd.revents & (libc::POLLERR | libc::POLLNVAL) != 0)
        {
            break;
        }
    }
    for dispatch in pending {
        libc::close(dispatch.channel);
    }
    terminate_request_children(&active);
}

/// Starts queued request children until the configured live-PID cap is reached.
unsafe fn start_pending_requests(
    pending: &mut VecDeque<Dispatch>,
    active: &mut HashMap<u64, libc::pid_t>,
    handler: extern "C" fn(),
    concurrency: usize,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
) {
    while active.len() < concurrency {
        let Some(dispatch) = pending.pop_front() else {
            break;
        };
        let inherited_channels = pending.iter().map(|queued| queued.channel).collect::<Vec<_>>();
        let pid = libc::fork();
        if pid == 0 {
            libc::close(dispatch_fd);
            libc::close(cancel_fd);
            for channel in inherited_channels {
                libc::close(channel);
            }
            run_request_child(handler, dispatch.channel, dispatch.id);
        }
        if pid < 0 {
            write_failure_response(dispatch.channel);
            libc::close(dispatch.channel);
            continue;
        }
        libc::close(dispatch.channel);
        active.insert(dispatch.id, pid);
    }
}

/// Applies a worker cancellation to a live or queued request ID.
unsafe fn cancel_request(
    id: u64,
    greatest_dispatch: u64,
    active: &mut HashMap<u64, libc::pid_t>,
    pending: &mut VecDeque<Dispatch>,
    early_cancels: &mut HashSet<u64>,
) {
    if let Some(pid) = active.remove(&id) {
        kill_and_reap(pid);
        return;
    }
    if let Some(index) = pending.iter().position(|dispatch| dispatch.id == id) {
        if let Some(dispatch) = pending.remove(index) {
            libc::close(dispatch.channel);
        }
        return;
    }
    if id > greatest_dispatch {
        early_cancels.insert(id);
    }
}

/// Reaps every exited disposable request child and removes its request mapping.
unsafe fn reap_request_children(active: &mut HashMap<u64, libc::pid_t>) {
    loop {
        let mut status = 0;
        let pid = libc::waitpid(-1, &mut status, libc::WNOHANG);
        if pid <= 0 {
            break;
        }
        if let Some(id) = active
            .iter()
            .find_map(|(id, child)| (*child == pid).then_some(*id))
        {
            active.remove(&id);
            if libc::WIFSIGNALED(status) {
                eprintln!(
                    "elephc-web: request handler {pid} terminated by signal {} while serving {id}",
                    libc::WTERMSIG(status)
                );
            } else if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) != 0 {
                eprintln!(
                    "elephc-web: request handler {pid} exited with status {} while serving {id}",
                    libc::WEXITSTATUS(status)
                );
            }
        }
    }
}

/// Terminates and synchronously reaps every live request child at broker shutdown.
unsafe fn terminate_request_children(active: &HashMap<u64, libc::pid_t>) {
    for pid in active.values() {
        libc::kill(*pid, libc::SIGKILL);
    }
    for pid in active.values() {
        reap_exact(*pid);
    }
}
