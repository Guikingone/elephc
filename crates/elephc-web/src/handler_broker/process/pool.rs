//! Purpose:
//! Supervises a fixed pool of persistent handler children and replaces failed or retired slots.
//!
//! Called from:
//! - `super::broker_loop()` when `BrokerMode::Pool` is selected.
//!
//! Key details:
//! - Descriptor transfer is acknowledged before the broker closes its copy.
//! - Cancellation replaces a busy child, while quota retirement and crashes are reaped and
//!   replaced before more queued work is assigned.

use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::atomic::Ordering;

use super::super::control::{self, Dispatch};
use super::super::BrokerConfig;
use super::handler::execute_handler_request;
use super::{
    ignore_signal, kill_and_reap, reap_exact, receive_acknowledged_dispatch,
    restore_default_signal, BROKER_POLL_MILLIS, BROKER_SHUTDOWN_REQUESTED,
};

/// One persistent handler process supervised by a pool broker.
struct PoolSlot {
    pid: libc::pid_t,
    control: OwnedFd,
    busy: Option<u64>,
    retiring: bool,
}

/// Supervises a fixed set of persistent handler children.
pub(super) unsafe fn run(
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    worker_pid: libc::pid_t,
    handler: extern "C" fn(),
    config: BrokerConfig,
) {
    let (status_recv, status_send) = match control::datagram_pair() {
        Ok(pair) => pair,
        Err(error) => {
            eprintln!("elephc-web: pool broker could not create status channel: {error}");
            return;
        }
    };
    let mut slots = Vec::<PoolSlot>::new();
    let mut pending = VecDeque::<Dispatch>::new();
    for _ in 0..config.concurrency {
        match spawn_pool_child(
            &slots,
            &pending,
            dispatch_fd,
            cancel_fd,
            status_recv.as_raw_fd(),
            status_send.as_raw_fd(),
            handler,
            config.max_handler_requests,
        ) {
            Ok(slot) => slots.push(slot),
            Err(error) => {
                eprintln!("elephc-web: pool broker could not start handler: {error}");
                terminate_pool(&slots);
                return;
            }
        }
    }
    let mut early_cancels = HashSet::<u64>::new();
    let mut greatest_dispatch = 0u64;
    loop {
        if BROKER_SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            break;
        }
        if let Err(error) = reap_and_replace_pool(
            &mut slots,
            &pending,
            dispatch_fd,
            cancel_fd,
            status_recv.as_raw_fd(),
            status_send.as_raw_fd(),
            handler,
            config.max_handler_requests,
        ) {
            eprintln!("elephc-web: pool broker could not reap/replace handler: {error}");
            break;
        }
        if let Err(error) = assign_pending_pool(
            &mut slots,
            &mut pending,
            dispatch_fd,
            cancel_fd,
            status_recv.as_raw_fd(),
            status_send.as_raw_fd(),
            handler,
            config.max_handler_requests,
        ) {
            eprintln!("elephc-web: pool broker could not dispatch/replace handler: {error}");
            break;
        }
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
            libc::pollfd {
                fd: status_recv.as_raw_fd(),
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
        if poll_fds[2].revents & libc::POLLIN != 0 {
            match control::recv_status(status_recv.as_raw_fd()) {
                Ok(Some((id, retiring))) => {
                    if let Some(slot) = slots.iter_mut().find(|slot| slot.busy == Some(id)) {
                        slot.busy = None;
                        slot.retiring = retiring;
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        if poll_fds[1].revents & libc::POLLIN != 0 {
            match control::recv_id(cancel_fd) {
                Ok(Some(id)) => {
                    if let Err(error) = cancel_pool_request(
                        id,
                        greatest_dispatch,
                        &mut slots,
                        &mut pending,
                        &mut early_cancels,
                        dispatch_fd,
                        cancel_fd,
                        status_recv.as_raw_fd(),
                        status_send.as_raw_fd(),
                        handler,
                        config.max_handler_requests,
                    ) {
                        eprintln!("elephc-web: pool broker could not cancel/replace handler: {error}");
                        break;
                    }
                }
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
    terminate_pool(&slots);
}

/// Forks one persistent pool child and returns its broker-side control slot.
#[allow(clippy::too_many_arguments)]
unsafe fn spawn_pool_child(
    slots: &[PoolSlot],
    pending: &VecDeque<Dispatch>,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    status_recv: RawFd,
    status_send: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> io::Result<PoolSlot> {
    let (parent_control, child_control) = control::datagram_pair()?;
    let existing_controls = slots
        .iter()
        .map(|slot| slot.control.as_raw_fd())
        .collect::<Vec<_>>();
    let pending_channels = pending
        .iter()
        .map(|dispatch| dispatch.channel)
        .collect::<Vec<_>>();
    let pid = libc::fork();
    if pid == 0 {
        drop(parent_control);
        libc::close(dispatch_fd);
        libc::close(cancel_fd);
        libc::close(status_recv);
        for fd in existing_controls {
            libc::close(fd);
        }
        for fd in pending_channels {
            libc::close(fd);
        }
        run_pool_child(
            child_control.as_raw_fd(),
            status_send,
            handler,
            max_handler_requests,
        );
    }
    if pid < 0 {
        return Err(io::Error::last_os_error());
    }
    drop(child_control);
    Ok(PoolSlot {
        pid,
        control: parent_control,
        busy: None,
        retiring: false,
    })
}

/// Assigns queued requests to idle pool slots and replaces a slot on send failure.
#[allow(clippy::too_many_arguments)]
unsafe fn assign_pending_pool(
    slots: &mut Vec<PoolSlot>,
    pending: &mut VecDeque<Dispatch>,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    status_recv: RawFd,
    status_send: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> io::Result<()> {
    loop {
        let Some(index) = slots
            .iter()
            .position(|slot| slot.busy.is_none() && !slot.retiring)
        else {
            return Ok(());
        };
        let Some(dispatch) = pending.pop_front() else {
            return Ok(());
        };
        let sent = send_pool_dispatch(
            slots[index].control.as_raw_fd(),
            dispatch.id,
            dispatch.channel,
        );
        libc::close(dispatch.channel);
        if sent.is_ok() {
            slots[index].busy = Some(dispatch.id);
            continue;
        }
        if let Err(error) = replace_pool_slot(
            index,
            slots,
            pending,
            dispatch_fd,
            cancel_fd,
            status_recv,
            status_send,
            handler,
            max_handler_requests,
        ) {
            return Err(error);
        }
    }
}

/// Transfers one request channel and waits until its pool child owns the descriptor.
unsafe fn send_pool_dispatch(control_fd: RawFd, id: u64, channel: RawFd) -> io::Result<()> {
    control::send_dispatch(control_fd, id, channel, false)?;
    match control::recv_id(control_fd)? {
        Some(acknowledged) if acknowledged == id => Ok(()),
        Some(acknowledged) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("pool handler acknowledged request {acknowledged}, expected {id}"),
        )),
        None => Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "pool handler closed before acknowledging its request",
        )),
    }
}

/// Applies cancellation by killing a busy pool child or dropping a queued request.
#[allow(clippy::too_many_arguments)]
unsafe fn cancel_pool_request(
    id: u64,
    greatest_dispatch: u64,
    slots: &mut Vec<PoolSlot>,
    pending: &mut VecDeque<Dispatch>,
    early_cancels: &mut HashSet<u64>,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    status_recv: RawFd,
    status_send: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> io::Result<()> {
    if let Some(index) = slots.iter().position(|slot| slot.busy == Some(id)) {
        return replace_pool_slot(
            index,
            slots,
            pending,
            dispatch_fd,
            cancel_fd,
            status_recv,
            status_send,
            handler,
            max_handler_requests,
        );
    }
    if let Some(index) = pending.iter().position(|dispatch| dispatch.id == id) {
        if let Some(dispatch) = pending.remove(index) {
            libc::close(dispatch.channel);
        }
        return Ok(());
    }
    if id > greatest_dispatch {
        early_cancels.insert(id);
    }
    Ok(())
}

/// Reaps exited pool children and respawns their slots.
#[allow(clippy::too_many_arguments)]
unsafe fn reap_and_replace_pool(
    slots: &mut Vec<PoolSlot>,
    pending: &VecDeque<Dispatch>,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    status_recv: RawFd,
    status_send: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> io::Result<()> {
    loop {
        let mut status = 0;
        let pid = libc::waitpid(-1, &mut status, libc::WNOHANG);
        if pid == 0 {
            return Ok(());
        }
        if pid < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ECHILD) && slots.is_empty() {
                return Ok(());
            }
            return Err(error);
        }
        let Some(index) = slots.iter().position(|slot| slot.pid == pid) else {
            continue;
        };
        let old = slots.remove(index);
        if !old.retiring {
            if libc::WIFSIGNALED(status) {
                eprintln!(
                    "elephc-web: pool handler {pid} terminated by signal {} while serving {:?}",
                    libc::WTERMSIG(status),
                    old.busy
                );
            } else if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) != 0 {
                eprintln!(
                    "elephc-web: pool handler {pid} exited with status {} while serving {:?}",
                    libc::WEXITSTATUS(status),
                    old.busy
                );
            }
        }
        drop(old);
        let replacement = spawn_pool_child(
            slots,
            pending,
            dispatch_fd,
            cancel_fd,
            status_recv,
            status_send,
            handler,
            max_handler_requests,
        )?;
        slots.insert(index, replacement);
    }
}

/// Kills one pool child synchronously and inserts a fresh replacement slot.
#[allow(clippy::too_many_arguments)]
unsafe fn replace_pool_slot(
    index: usize,
    slots: &mut Vec<PoolSlot>,
    pending: &VecDeque<Dispatch>,
    dispatch_fd: RawFd,
    cancel_fd: RawFd,
    status_recv: RawFd,
    status_send: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> io::Result<()> {
    let old = slots.remove(index);
    kill_and_reap(old.pid);
    drop(old);
    let replacement = spawn_pool_child(
        slots,
        pending,
        dispatch_fd,
        cancel_fd,
        status_recv,
        status_send,
        handler,
        max_handler_requests,
    )?;
    slots.insert(index, replacement);
    Ok(())
}

/// Runs a persistent child loop, serving requests until its quota or a failure exits it.
unsafe fn run_pool_child(
    control_fd: RawFd,
    status_fd: RawFd,
    handler: extern "C" fn(),
    max_handler_requests: usize,
) -> ! {
    restore_default_signal(libc::SIGCHLD);
    restore_default_signal(libc::SIGTERM);
    ignore_signal(libc::SIGPIPE);
    let mut served = 0usize;
    loop {
        let dispatch = match control::recv_dispatch(control_fd) {
            Ok(Some(dispatch)) => dispatch,
            Ok(None) | Err(_) => libc::_exit(0),
        };
        if control::send_id(control_fd, dispatch.id).is_err() {
            libc::close(dispatch.channel);
            libc::_exit(1);
        }
        if control::set_close_on_exec(dispatch.channel).is_err() {
            libc::close(dispatch.channel);
            libc::_exit(1);
        }
        let stream = File::from_raw_fd(dispatch.channel);
        let complete = execute_handler_request(handler, stream, dispatch.id);
        served = served.saturating_add(1);
        if !complete {
            libc::_exit(3);
        }
        let retiring = max_handler_requests > 0 && served >= max_handler_requests;
        if let Err(error) = control::send_status(status_fd, dispatch.id, retiring) {
            eprintln!("elephc-web: pool handler could not report completion: {error}");
            libc::_exit(2);
        }
        if retiring {
            libc::_exit(0);
        }
    }
}

/// Terminates and reaps every persistent pool child.
unsafe fn terminate_pool(slots: &[PoolSlot]) {
    for slot in slots {
        libc::kill(slot.pid, libc::SIGKILL);
    }
    for slot in slots {
        reap_exact(slot.pid);
    }
}

