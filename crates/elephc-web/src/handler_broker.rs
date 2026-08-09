//! Purpose:
//! Prestarts a single-threaded handler broker before Tokio exists, then routes
//! request snapshots to disposable forked PHP handler processes.
//!
//! Called from:
//! - `crate::worker::serve`, before it constructs the worker's Tokio runtime.
//! - `crate::worker::run_handler_isolated`, for async request dispatch.
//!
//! Key details:
//! - Only the broker calls `fork()`, and the broker never creates threads.
//! - The worker starts a small post-fork monitor that reaps a dead broker and
//!   exits the worker so the master can rebuild the pair safely.
//! - A dedicated Unix stream per request carries bounded protocol frames; a
//!   fixed Unix datagram control socket transfers descriptors atomically.

use std::fs::File;
use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tokio::io::unix::AsyncFd;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::handler_ipc::{self, HandlerRequest, ResponseFrame, ResponseStart};
use crate::request_state::{self, RequestMeta};

/// Bounds disposable handler processes and their per-response IPC buffers.
const MAX_CONCURRENT_HANDLERS: usize = 8;
/// Worker exit status reserved for a reaped handler-broker failure.
const BROKER_FAILURE_EXIT_CODE: libc::c_int = 87;
/// Small stack sufficient for the broker monitor's blocking `waitpid` loop.
const BROKER_MONITOR_STACK_BYTES: usize = 64 * 1024;
/// Per-request execution-time ceiling inherited by every disposable child.
static MAX_EXEC_SECS: AtomicU32 = AtomicU32::new(0);

/// Pre-runtime broker endpoint that becomes async only after Tokio is built.
pub(crate) struct PrestartedBroker {
    control: OwnedFd,
}

/// Cloneable async dispatch handle shared by the worker's connection tasks.
#[derive(Clone)]
pub(crate) struct HandlerBroker {
    control: Arc<Mutex<AsyncFd<OwnedFd>>>,
    permits: Arc<Semaphore>,
}

/// One accepted handler response stream and its concurrency permit.
pub(crate) struct HandlerResponse {
    pub(crate) start: ResponseStart,
    reader: tokio::net::unix::OwnedReadHalf,
    _permit: OwnedSemaphorePermit,
}

/// Terminates an over-time handler child without affecting its broker.
extern "C" fn handle_exec_timeout(_signal: libc::c_int) {
    const MESSAGE: &[u8] =
        b"elephc-web: handler exceeded --max-execution-time; terminating request\n";
    unsafe {
        libc::write(2, MESSAGE.as_ptr().cast(), MESSAGE.len());
        libc::_exit(1);
    }
}

/// Installs the alarm handler before the broker process is created.
fn configure_execution_timeout(seconds: u32) {
    MAX_EXEC_SECS.store(seconds, Ordering::Relaxed);
    if seconds == 0 {
        return;
    }
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction =
            handle_exec_timeout as extern "C" fn(libc::c_int) as libc::sighandler_t;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;
        libc::sigaction(libc::SIGALRM, &action, std::ptr::null_mut());
    }
}

impl PrestartedBroker {
    /// Forks the dedicated single-threaded broker and keeps its control endpoint.
    pub(crate) fn start(handler: extern "C" fn(), max_exec_secs: u32) -> io::Result<Self> {
        configure_execution_timeout(max_exec_secs);
        let mut controls = [-1; 2];
        if unsafe {
            libc::socketpair(
                libc::AF_UNIX,
                libc::SOCK_DGRAM,
                0,
                controls.as_mut_ptr(),
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        let parent_control = unsafe { OwnedFd::from_raw_fd(controls[0]) };
        let broker_control = unsafe { OwnedFd::from_raw_fd(controls[1]) };
        set_close_on_exec(parent_control.as_raw_fd())?;
        set_close_on_exec(broker_control.as_raw_fd())?;
        set_no_sigpipe(parent_control.as_raw_fd())?;
        set_no_sigpipe(broker_control.as_raw_fd())?;
        let worker_pid = unsafe { libc::getpid() };
        let pid = unsafe { libc::fork() };
        if pid == 0 {
            drop(parent_control);
            unsafe {
                broker_loop(broker_control.as_raw_fd(), worker_pid, handler);
                libc::_exit(0);
            }
        }
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        drop(broker_control);
        if let Err(error) = set_nonblocking(parent_control.as_raw_fd()) {
            unsafe { terminate_and_reap_broker(pid) };
            return Err(error);
        }
        if let Err(error) = start_broker_monitor(pid) {
            unsafe { terminate_and_reap_broker(pid) };
            return Err(error);
        }
        Ok(Self {
            control: parent_control,
        })
    }

    /// Registers the already-running broker endpoint with Tokio's readiness driver.
    pub(crate) fn into_async(self) -> io::Result<HandlerBroker> {
        Ok(HandlerBroker {
            control: Arc::new(Mutex::new(AsyncFd::new(self.control)?)),
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_HANDLERS)),
        })
    }
}

/// Starts the post-fork worker thread that owns reaping the dedicated broker.
fn start_broker_monitor(pid: libc::pid_t) -> io::Result<()> {
    std::thread::Builder::new()
        .name("elephc-web-broker-monitor".to_string())
        .stack_size(BROKER_MONITOR_STACK_BYTES)
        .spawn(move || unsafe {
            reap_broker(pid);
            libc::_exit(BROKER_FAILURE_EXIT_CODE);
        })
        .map(|_| ())
}

/// Terminates a broker whose worker-side setup failed, then reaps it synchronously.
unsafe fn terminate_and_reap_broker(pid: libc::pid_t) {
    libc::kill(pid, libc::SIGTERM);
    reap_broker(pid);
}

/// Waits for the exact broker child, retrying interrupted waits until it is reaped.
unsafe fn reap_broker(pid: libc::pid_t) {
    loop {
        let mut status = 0;
        let waited = libc::waitpid(pid, &mut status, 0);
        if waited == pid {
            return;
        }
        if waited < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        if waited < 0 {
            return;
        }
    }
}

impl HandlerBroker {
    /// Dispatches one request and waits only until its status/headers are committed.
    pub(crate) async fn dispatch(&self, request: HandlerRequest) -> io::Result<HandlerResponse> {
        let permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "handler broker closed"))?;
        let (worker_stream, broker_stream) = UnixStream::pair()?;
        worker_stream.set_nonblocking(true)?;
        set_close_on_exec(worker_stream.as_raw_fd())?;
        set_close_on_exec(broker_stream.as_raw_fd())?;
        set_no_sigpipe(worker_stream.as_raw_fd())?;
        set_no_sigpipe(broker_stream.as_raw_fd())?;
        self.send_channel(broker_stream.as_raw_fd()).await?;
        drop(broker_stream);

        let stream = tokio::net::UnixStream::from_std(worker_stream)?;
        let (mut reader, mut writer) = stream.into_split();
        handler_ipc::write_request_async(&mut writer, &request).await?;
        drop(request);
        drop(writer);
        match handler_ipc::read_response_frame(&mut reader).await? {
            ResponseFrame::Start(start) => Ok(HandlerResponse {
                start,
                reader,
                _permit: permit,
            }),
            ResponseFrame::Chunk(_) | ResponseFrame::End => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "handler response did not begin with status and headers",
            )),
        }
    }

    /// Transfers one request-channel descriptor over the bounded control socket.
    async fn send_channel(&self, channel: RawFd) -> io::Result<()> {
        let control = self.control.lock().await;
        loop {
            let mut writable = control.writable().await?;
            match writable.try_io(|fd| send_fd(fd.get_ref().as_raw_fd(), channel)) {
                Ok(result) => return result,
                Err(_) => continue,
            }
        }
    }
}

impl HandlerResponse {
    /// Reads the next chunk or successful end frame from the handler child.
    pub(crate) async fn next_frame(&mut self) -> io::Result<ResponseFrame> {
        handler_ipc::read_response_frame(&mut self.reader).await
    }
}

/// Runs the broker's blocking descriptor-accept loop in its threadless process.
unsafe fn broker_loop(control: RawFd, worker_pid: libc::pid_t, handler: extern "C" fn()) {
    ignore_signal(libc::SIGPIPE);
    ignore_signal(libc::SIGCHLD);
    while wait_for_control(control, worker_pid) {
        let channel = match recv_fd(control) {
            Ok(Some(channel)) => channel,
            Ok(None) | Err(_) => break,
        };
        if set_close_on_exec(channel).is_err() {
            libc::close(channel);
            continue;
        }
        let mut stream = File::from_raw_fd(channel);
        let request = match handler_ipc::read_request(&mut stream) {
            Ok(request) => request,
            Err(_) => {
                write_failure_response(stream.as_raw_fd());
                continue;
            }
        };
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
        let pid = libc::fork();
        if pid == 0 {
            libc::close(control);
            run_handler_child(handler, stream.as_raw_fd());
        } else if pid < 0 {
            write_failure_response(stream.as_raw_fd());
        }
        drop(stream);
    }
    libc::close(control);
}

/// Waits for one control datagram while also detecting an exited worker parent.
unsafe fn wait_for_control(control: RawFd, worker_pid: libc::pid_t) -> bool {
    loop {
        let mut poll_fd = libc::pollfd {
            fd: control,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = libc::poll(&mut poll_fd, 1, 1000);
        if ready > 0 {
            return poll_fd.revents & libc::POLLIN != 0;
        }
        if ready < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
            return false;
        }
        if libc::getppid() != worker_pid {
            return false;
        }
    }
}

/// Executes PHP in a disposable child and exits without unwinding broker state.
unsafe fn run_handler_child(handler: extern "C" fn(), response_fd: RawFd) -> ! {
    restore_default_signal(libc::SIGCHLD);
    crate::session::elephc_web_session_reset();
    request_state::begin_response_stream(response_fd);
    request_state::set_capture(true);
    let seconds = MAX_EXEC_SECS.load(Ordering::Relaxed);
    if seconds > 0 {
        libc::alarm(seconds);
    }
    handler();
    if seconds > 0 {
        libc::alarm(0);
    }
    let complete = request_state::finish_response_stream();
    libc::close(response_fd);
    libc::_exit(if complete { 0 } else { 1 });
}

/// Emits the bounded 500 response used when the broker cannot start a handler.
unsafe fn write_failure_response(fd: RawFd) {
    let headers = Vec::new();
    let body = b"Internal Server Error";
    let _ = handler_ipc::write_response_start(fd, 500, &headers)
        && handler_ipc::write_response_chunks(fd, body)
        && handler_ipc::write_response_end(fd);
}

/// Installs an ignored signal disposition in the single-threaded broker.
unsafe fn ignore_signal(signal: libc::c_int) {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = libc::SIG_IGN;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(signal, &action, std::ptr::null_mut());
}

/// Restores child-reaping semantics inside a disposable PHP handler process.
unsafe fn restore_default_signal(signal: libc::c_int) {
    let mut action: libc::sigaction = std::mem::zeroed();
    action.sa_sigaction = libc::SIG_DFL;
    libc::sigemptyset(&mut action.sa_mask);
    action.sa_flags = 0;
    libc::sigaction(signal, &action, std::ptr::null_mut());
}

/// Marks an internal descriptor close-on-exec so user subprocesses cannot retain it.
fn set_close_on_exec(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Marks the worker control descriptor nonblocking for `AsyncFd` readiness.
fn set_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Prevents a closed internal socket from terminating a macOS worker via SIGPIPE.
#[cfg(target_os = "macos")]
fn set_no_sigpipe(fd: RawFd) -> io::Result<()> {
    let enabled: libc::c_int = 1;
    let result = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_NOSIGPIPE,
            (&enabled as *const libc::c_int).cast(),
            std::mem::size_of_val(&enabled) as libc::socklen_t,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// Leaves Linux sockets on their native MSG_NOSIGNAL send path.
#[cfg(target_os = "linux")]
fn set_no_sigpipe(_fd: RawFd) -> io::Result<()> {
    Ok(())
}

/// Sends one descriptor as an atomic `SCM_RIGHTS` control message.
fn send_fd(control: RawFd, channel: RawFd) -> io::Result<()> {
    unsafe {
        let mut byte = 0u8;
        let mut iov = libc::iovec {
            iov_base: (&mut byte as *mut u8).cast(),
            iov_len: 1,
        };
        let control_len = usize::try_from(libc::CMSG_SPACE(
            std::mem::size_of::<RawFd>() as _,
        ))
        .map_err(|_| io::Error::other("descriptor control message is too large"))?;
        let word_count = control_len.div_ceil(std::mem::size_of::<usize>());
        let mut ancillary = vec![0usize; word_count];
        let mut message: libc::msghdr = std::mem::zeroed();
        message.msg_iov = &mut iov;
        message.msg_iovlen = 1;
        message.msg_control = ancillary.as_mut_ptr().cast();
        message.msg_controllen = control_len
            .try_into()
            .map_err(|_| io::Error::other("descriptor control length does not fit msghdr"))?;
        let header = libc::CMSG_FIRSTHDR(&message);
        if header.is_null() {
            return Err(io::Error::other("failed to construct descriptor message"));
        }
        (*header).cmsg_level = libc::SOL_SOCKET;
        (*header).cmsg_type = libc::SCM_RIGHTS;
        (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<RawFd>() as _)
            .try_into()
            .map_err(|_| io::Error::other("descriptor header length does not fit cmsghdr"))?;
        std::ptr::write(libc::CMSG_DATA(header).cast::<RawFd>(), channel);
        let flags = send_message_flags();
        let written = libc::sendmsg(control, &message, flags);
        if written == 1 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

/// Receives one descriptor from the broker control socket, or `None` on EOF.
unsafe fn recv_fd(control: RawFd) -> io::Result<Option<RawFd>> {
    let mut byte = MaybeUninit::<u8>::uninit();
    let mut iov = libc::iovec {
        iov_base: byte.as_mut_ptr().cast(),
        iov_len: 1,
    };
    let control_len = usize::try_from(libc::CMSG_SPACE(
        std::mem::size_of::<RawFd>() as _,
    ))
    .map_err(|_| io::Error::other("descriptor control message is too large"))?;
    let word_count = control_len.div_ceil(std::mem::size_of::<usize>());
    let mut ancillary = vec![0usize; word_count];
    let mut message: libc::msghdr = std::mem::zeroed();
    message.msg_iov = &mut iov;
    message.msg_iovlen = 1;
    message.msg_control = ancillary.as_mut_ptr().cast();
    message.msg_controllen = control_len
        .try_into()
        .map_err(|_| io::Error::other("descriptor control length does not fit msghdr"))?;
    loop {
        let received = libc::recvmsg(control, &mut message, 0);
        if received == 0 {
            return Ok(None);
        }
        if received < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(error);
        }
        if received != 1 || message.msg_flags & (libc::MSG_CTRUNC | libc::MSG_TRUNC) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "broker control message was truncated",
            ));
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        let minimum_header_len = libc::CMSG_LEN(std::mem::size_of::<RawFd>() as _)
            .try_into()
            .map_err(|_| io::Error::other("descriptor header length does not fit cmsghdr"))?;
        if header.is_null()
            || (*header).cmsg_level != libc::SOL_SOCKET
            || (*header).cmsg_type != libc::SCM_RIGHTS
            || (*header).cmsg_len != minimum_header_len
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "broker control message omitted its request descriptor",
            ));
        }
        return Ok(Some(std::ptr::read(
            libc::CMSG_DATA(header).cast::<RawFd>(),
        )));
    }
}

/// Selects the nonblocking, no-SIGPIPE send flags available on this Unix target.
#[cfg(target_os = "linux")]
fn send_message_flags() -> libc::c_int {
    libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL
}

/// Selects nonblocking descriptor transfer flags on macOS.
#[cfg(target_os = "macos")]
fn send_message_flags() -> libc::c_int {
    libc::MSG_DONTWAIT
}
