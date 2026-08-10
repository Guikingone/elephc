//! Purpose:
//! Implements the Unix-domain control protocols used by isolated web handlers.
//! Transfers request stream descriptors and fixed-size lifecycle messages.
//!
//! Called from:
//! - `crate::handler_broker` on the async worker side.
//! - `crate::handler_broker::process` in the threadless broker and pool children.
//!
//! Key details:
//! - Request dispatch uses one atomic `SCM_RIGHTS` datagram containing a u64 ID.
//! - The receiver acknowledges that ID before the sender closes its descriptor.
//! - Cancellation and completion messages are fixed-size datagrams, so partial
//!   messages are rejected rather than silently changing lifecycle state.

use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

/// One descriptor-bearing request accepted from a control socket.
pub(super) struct Dispatch {
    pub(super) id: u64,
    pub(super) channel: RawFd,
}

/// Creates a close-on-exec Unix datagram socket pair with SIGPIPE protection.
pub(super) fn datagram_pair() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut descriptors = [-1; 2];
    loop {
        if unsafe {
            libc::socketpair(
                libc::AF_UNIX,
                libc::SOCK_DGRAM,
                0,
                descriptors.as_mut_ptr(),
            )
        } == 0
        {
            break;
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    }
    let first = unsafe { OwnedFd::from_raw_fd(descriptors[0]) };
    let second = unsafe { OwnedFd::from_raw_fd(descriptors[1]) };
    set_close_on_exec(first.as_raw_fd())?;
    set_close_on_exec(second.as_raw_fd())?;
    set_no_sigpipe(first.as_raw_fd())?;
    set_no_sigpipe(second.as_raw_fd())?;
    Ok((first, second))
}

/// Marks an internal descriptor close-on-exec so user subprocesses cannot retain it.
pub(super) fn set_close_on_exec(fd: RawFd) -> io::Result<()> {
    let flags = loop {
        let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if result >= 0 {
            break result;
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    };
    loop {
        if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    }
}

/// Marks a worker-side control descriptor nonblocking for Tokio `AsyncFd`.
pub(super) fn set_nonblocking(fd: RawFd) -> io::Result<()> {
    let flags = loop {
        let result = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if result >= 0 {
            break result;
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    };
    loop {
        if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } >= 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    }
}

/// Prevents a closed internal socket from terminating a macOS process via SIGPIPE.
#[cfg(target_os = "macos")]
fn set_no_sigpipe(fd: RawFd) -> io::Result<()> {
    let enabled: libc::c_int = 1;
    loop {
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
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    }
}

/// Leaves Linux sockets on their native `MSG_NOSIGNAL` send path.
#[cfg(target_os = "linux")]
fn set_no_sigpipe(_fd: RawFd) -> io::Result<()> {
    Ok(())
}

/// Transfers a request descriptor and dispatch ID as one atomic datagram.
pub(super) fn send_dispatch(
    control: RawFd,
    id: u64,
    channel: RawFd,
    nonblocking: bool,
) -> io::Result<()> {
    unsafe {
        let mut payload = id.to_be_bytes();
        let mut iov = libc::iovec {
            iov_base: payload.as_mut_ptr().cast(),
            iov_len: payload.len(),
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
        loop {
            let written = libc::sendmsg(control, &message, send_flags(nonblocking));
            if written == payload.len() as isize {
                return Ok(());
            }
            if written < 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(error);
            }
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "broker dispatch datagram was only partially sent",
            ));
        }
    }
}

/// Receives one descriptor-bearing dispatch, or `None` when the peer is gone.
pub(super) unsafe fn recv_dispatch(control: RawFd) -> io::Result<Option<Dispatch>> {
    let mut payload = MaybeUninit::<[u8; 8]>::uninit();
    let mut iov = libc::iovec {
        iov_base: payload.as_mut_ptr().cast(),
        iov_len: 8,
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
        if received != 8 || message.msg_flags & (libc::MSG_CTRUNC | libc::MSG_TRUNC) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "broker dispatch message was truncated",
            ));
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        let expected_len = libc::CMSG_LEN(std::mem::size_of::<RawFd>() as _)
            .try_into()
            .map_err(|_| io::Error::other("descriptor header length does not fit cmsghdr"))?;
        if header.is_null()
            || (*header).cmsg_level != libc::SOL_SOCKET
            || (*header).cmsg_type != libc::SCM_RIGHTS
            || (*header).cmsg_len != expected_len
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "broker dispatch omitted its request descriptor",
            ));
        }
        let bytes = payload.assume_init();
        return Ok(Some(Dispatch {
            id: u64::from_be_bytes(bytes),
            channel: std::ptr::read(libc::CMSG_DATA(header).cast::<RawFd>()),
        }));
    }
}

/// Sends one lifecycle ID through a blocking datagram socket.
pub(super) fn send_id(control: RawFd, id: u64) -> io::Result<()> {
    send_bytes(control, &id.to_be_bytes())
}

/// Receives one lifecycle ID, rejecting partial datagrams.
pub(super) unsafe fn recv_id(control: RawFd) -> io::Result<Option<u64>> {
    let Some(bytes) = recv_fixed::<8>(control)? else {
        return Ok(None);
    };
    Ok(Some(u64::from_be_bytes(bytes)))
}

/// Sends a pool-child completion or retirement notification.
pub(super) fn send_status(control: RawFd, id: u64, retiring: bool) -> io::Result<()> {
    let mut payload = [0u8; 9];
    payload[..8].copy_from_slice(&id.to_be_bytes());
    payload[8] = u8::from(retiring);
    send_bytes(control, &payload)
}

/// Receives a pool-child completion or retirement notification.
pub(super) unsafe fn recv_status(control: RawFd) -> io::Result<Option<(u64, bool)>> {
    let Some(payload) = recv_fixed::<9>(control)? else {
        return Ok(None);
    };
    let mut id = [0u8; 8];
    id.copy_from_slice(&payload[..8]);
    Ok(Some((u64::from_be_bytes(id), payload[8] != 0)))
}

/// Writes one complete fixed-size datagram, retrying interrupted sends.
fn send_bytes(control: RawFd, bytes: &[u8]) -> io::Result<()> {
    loop {
        let written = unsafe {
            libc::send(
                control,
                bytes.as_ptr().cast(),
                bytes.len(),
                send_flags(false),
            )
        };
        if written == bytes.len() as isize {
            return Ok(());
        }
        if written < 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(error);
        }
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "broker lifecycle datagram was only partially sent",
        ));
    }
}

/// Reads exactly one fixed-size datagram or reports a truncated lifecycle message.
unsafe fn recv_fixed<const N: usize>(control: RawFd) -> io::Result<Option<[u8; N]>> {
    let mut bytes = [0u8; N];
    loop {
        let received = libc::recv(control, bytes.as_mut_ptr().cast(), N, 0);
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
        if received != N as isize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "broker lifecycle message was truncated",
            ));
        }
        return Ok(Some(bytes));
    }
}

/// Selects no-SIGPIPE and optional nonblocking flags on Linux.
#[cfg(target_os = "linux")]
fn send_flags(nonblocking: bool) -> libc::c_int {
    libc::MSG_NOSIGNAL | if nonblocking { libc::MSG_DONTWAIT } else { 0 }
}

/// Selects the optional nonblocking flag on macOS, where `SO_NOSIGPIPE` is set.
#[cfg(target_os = "macos")]
fn send_flags(nonblocking: bool) -> libc::c_int {
    if nonblocking { libc::MSG_DONTWAIT } else { 0 }
}
