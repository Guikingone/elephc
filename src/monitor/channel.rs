//! Purpose:
//! Hands a program `monitor` launches its control channel, and finds the build
//! key that authorizes the request. Possession of the channel — an inherited
//! socket on fd 3 — is the credential for a launched program, so there is
//! nothing to copy, replay, or set from another shell.
//!
//! Called from:
//! - `local::run`, before spawning the target.
//!
//! Key details:
//! - The child end is inherited across the spawn; both ends close on drop.
//! - The key comes from the `<binary>.key` sidecar, or an explicit override.

use super::*;

/// Answers one HTTP request with the current bytes of `path`. Ignores the
/// request target: this server has exactly one resource.
pub(crate) fn serve_one_request(mut stream: std::net::TcpStream, path: &str) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader, Write};
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    {
        // Consume the request line and headers so the client can send the body
        // and read the response cleanly.
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        loop {
            let mut header = String::new();
            let n = reader.read_line(&mut header)?;
            if n == 0 || header == "\r\n" || header == "\n" {
                break;
            }
        }
    }
    let body = std::fs::read(path).unwrap_or_default();
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&body)?;
    stream.flush()
}

/// Compiles a `.php` target with the monitoring capability embedded.
///
/// One function for what used to be two, because the two mechanisms are no
/// longer two commands: whichever of them ends up reading the program, the build
/// that produces it is the same build.
///
/// Deliberately *without* `--debug-info`: the embedded sampler resolves frames
/// through the symbol table the capability carries, not through DWARF, so debug
/// info buys this path nothing and only makes the compile slower. (It also used
/// to break it outright on ELF, until the inline-thunk section restore in
/// `runtime_wrappers.rs` fixed the underlying layout bug.)
pub(crate) fn compile_php_monitored(source: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate elephc: {e}"))?;
    let status = process::Command::new(exe)
        .args(["--with-monitoring", source])
        .status()
        .map_err(|e| format!("cannot run elephc: {e}"))?;
    if !status.success() {
        return Err(format!("compiling {source} with --with-monitoring failed"));
    }
    Ok(spawnable_path(source.trim_end_matches(".php")))
}

/// Creates the socketpair that tells a spawned binary it is being monitored.
///
/// The credential is the channel itself. Only this process holds the other end,
/// so there is nothing for anyone else to copy, find in a log, or replay — unlike
/// an environment variable, which every process on the machine can read, and
/// which therefore has to be signed to be safe at all.
pub(crate) fn open_control_channel() -> Option<ControlChannel> {
    unsafe {
        let mut fds = [0i32; 2];
        if libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) != 0 {
            return None;
        }
        let channel = ControlChannel {
            parent: fds[0],
            child: fds[1],
        };
        // Written BEFORE the fork, so the marker is waiting in the buffer rather
        // than racing the child's init.
        let wrote = libc::send(
            channel.parent,
            CONTROL_MAGIC.as_ptr() as *const libc::c_void,
            CONTROL_MAGIC.len(),
            0,
        );
        if wrote != CONTROL_MAGIC.len() as isize {
            return None;
        }
        Some(channel)
    }
}

/// Arranges for `channel`'s child end to arrive as `CONTROL_FD` in the spawned
/// process.
///
/// `pre_exec` runs in the forked child between fork and exec, where only
/// async-signal-safe calls are permitted — `dup2` and `close` are. Nothing else
/// happens here for that reason.
pub(crate) fn attach_control_channel(command: &mut process::Command, channel: &ControlChannel) {
    use std::os::unix::process::CommandExt as _;
    let child_fd = channel.child;
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(child_fd, CONTROL_FD) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            // The duplicate is what the child keeps; clear CLOEXEC so it survives
            // the exec that follows.
            if libc::fcntl(CONTROL_FD, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

/// Whether `path` is a binary built with `--with-monitoring`.
///
/// Read from the FILE, not from the running process: the whole value of the
/// check is telling someone "this binary cannot answer that question" before
/// anything is launched. Running it and reporting an empty profile would read as
/// "your program is fast", which is the worst possible way to be wrong.
pub(crate) fn carries_monitoring(path: &std::path::Path) -> bool {
    // Regular files only. `fs::read` on a character device never returns —
    // `monitor /dev/zero` read until the machine gave out — and on a directory
    // it fails in a way that used to read as "no marker".
    if !std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false) {
        return false;
    }
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    bytes
        .windows(MONITORING_MARKER.len())
        .any(|window| window == MONITORING_MARKER)
}

/// Deliberately strict, with no reduced fallback. An external sampler could still
/// produce time shares for an unequipped binary, but shipping that as a silent
/// downgrade means two different things arrive under one command and the reader
/// has to notice which — the exact ambiguity this whole design removes. One
/// answer, or an error naming the fix.
pub(crate) fn require_monitoring(path: &std::path::Path) -> Result<(), String> {
    // Say what is actually wrong. Every read failure used to collapse into
    // "not built with --with-monitoring", so a typo'd path, a directory, or a
    // permission problem all sent the user off to rebuild a binary that was
    // never the issue — an error that confidently names the wrong cause is
    // worse than one that admits it does not know.
    match std::fs::metadata(path) {
        Ok(meta) if !meta.is_file() => {
            return Err(format!(
                "{} is not a file, so there is nothing to run.",
                path.display()
            ));
        }
        Err(error) => {
            return Err(format!("cannot read {}: {error}", path.display()));
        }
        Ok(_) => {}
    }
    if carries_monitoring(path) {
        return Ok(());
    }
    Err(format!(
        "{} was not built with --with-monitoring, so there is nothing to monitor.\n  \
         Rebuild it:  elephc --with-monitoring <source>.php\n  \
         Or point monitor at the source and let it build:  elephc monitor <source>.php",
        path.display()
    ))
}

/// Resolves the build key for `--probe-host`: `ELEPHC_PROBE_KEY` hex if set,
/// else the `<socket-without-.sock>.key` file, else a `.key`
/// next to the socket path.
pub(crate) fn resolve_probe_key(cmd: &MonitorCommand, socket: &str) -> Result<[u8; 32], String> {
    if let Some(path) = &cmd.probe_key {
        let hex = std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read probe key {path}: {error}"))?;
        return parse_hex_key(hex.trim())
            .ok_or_else(|| format!("probe key {path} is not 64 hex characters"));
    }
    if let Ok(hex) = std::env::var("ELEPHC_PROBE_KEY") {
        return parse_hex_key(hex.trim())
            .ok_or_else(|| "ELEPHC_PROBE_KEY is not 64 hex characters".to_string());
    }
    let candidates = [
        format!("{}.key", socket.trim_end_matches(".sock")),
        format!("{socket}.key"),
    ];
    for candidate in &candidates {
        if let Ok(hex) = std::fs::read_to_string(candidate) {
            return parse_hex_key(hex.trim()).ok_or_else(|| {
                format!("probe key sidecar {candidate} is not 64 hex characters")
            });
        }
    }
    Err(format!(
        "no build key: pass --key <file>, set ELEPHC_PROBE_KEY, or place a .key \
         file next to {socket}"
    ))
}
