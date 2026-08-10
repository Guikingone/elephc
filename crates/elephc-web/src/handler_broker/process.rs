//! Purpose:
//! Owns the threadless broker process for pool and request web isolation.
//! Tracks every handler PID, cancellation, completion, restart, and shutdown.
//!
//! Called from:
//! - `crate::handler_broker::PrestartedBroker::start()` immediately after fork.
//!
//! Key details:
//! - Request mode never ignores `SIGCHLD`; every disposable PID is reaped.
//! - Both descriptor-transfer hops are acknowledged before sender-side close.
//! - A no-op `SIGCHLD` handler interrupts broker polling as soon as a child exits.
//! - Pool mode replaces crashed, cancelled, timed-out, and quota-retired children.
//! - Parent loss terminates and reaps every descendant before the broker exits.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

use crate::handler_ipc;
use crate::request_state::{self, RequestMeta};

use super::control::{self, Dispatch};
use super::{BrokerConfig, BrokerMode, MAX_EXEC_SECS};

/// Poll interval used to notice parent death and reap children without a signal handler.
const BROKER_POLL_MILLIS: libc::c_int = 100;

/// One persistent handler process supervised by a pool broker.
struct PoolSlot {
    pid: libc::pid_t,
    control: OwnedFd,
    busy: Option<u64>,
    retiring: bool,
}

/// Runs the selected broker model until the worker parent disappears.
pub(super) unsafe fn broker_loop(
    dispatch: RawFd,
    cancel: RawFd,
    worker_pid: libc::pid_t,
    handler: extern "C" fn(),
    config: BrokerConfig,
) {
    ignore_signal(libc::SIGPIPE);
    install_child_exit_handler();
    match config.mode {
        BrokerMode::Pool => pool_broker_loop(dispatch, cancel, worker_pid, handler, config),
        BrokerMode::Request => request_broker_loop(dispatch, cancel, worker_pid, handler, config),
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

/// Supervises one disposable handler PID per accepted request.
unsafe fn request_broker_loop(
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

/// Supervises a fixed set of persistent handler children.
unsafe fn pool_broker_loop(
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

/// Reads and executes one request in a disposable child, then exits immediately.
unsafe fn run_request_child(handler: extern "C" fn(), response_fd: RawFd, id: u64) -> ! {
    restore_default_signal(libc::SIGCHLD);
    let stream = File::from_raw_fd(response_fd);
    let complete = execute_handler_request(handler, stream, id);
    libc::_exit(if complete { 0 } else { 1 });
}

/// Materializes one request snapshot, resets request-local state, and invokes PHP.
unsafe fn execute_handler_request(handler: extern "C" fn(), mut stream: File, id: u64) -> bool {
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
unsafe fn write_failure_response(fd: RawFd) {
    let headers = Vec::new();
    let body = b"Internal Server Error";
    let _ = handler_ipc::write_response_start(fd, 500, &headers)
        && handler_ipc::write_response_chunks(fd, body)
        && handler_ipc::write_response_end(fd);
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
