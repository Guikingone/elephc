//! `elephc monitor`: reading a service through its endpoint
//!
//! Moved out of a 5,379-line `monitor.rs` without a line of it changing, so
//! the split can be read as a move rather than a rewrite.

use super::*;

/// A `monitor` target that is a running service rather than a file.
///
/// `host:port`, or the same behind an `http://` scheme — the two spellings people
/// actually type. A filesystem path stays a path, so a socket at `/tmp/p.sock` is
/// still a socket and never mistaken for a host.
pub(crate) fn remote_target(spec: &str) -> Option<RemoteTarget> {
    let (tls, rest, default_port) = if let Some(rest) = spec.strip_prefix("https://") {
        (true, rest, 443)
    } else if let Some(rest) = spec.strip_prefix("http://") {
        (false, rest, 80)
    } else {
        (false, spec, 0)
    };
    // Anything after the authority is a path, not part of the address.
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority.starts_with('/') || authority.starts_with('.') || authority.is_empty() {
        return None;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            if host.is_empty() || port.parse::<u16>().is_err() {
                return None;
            }
            (host.to_string(), port.parse::<u16>().ok()?)
        }
        // A scheme implies its port; a bare name without one is a file, not a host.
        None if default_port != 0 => (authority.to_string(), default_port),
        None => return None,
    };
    Some(RemoteTarget { host, port, tls })
}

/// A running service `monitor` can read, and how to reach it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RemoteTarget {
    pub host: String,
    pub port: u16,
    /// Whether the connection is wrapped in TLS, with the certificate verified
    /// against the platform root store before a single protocol byte is sent.
    pub tls: bool,
}

impl RemoteTarget {
    fn authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Opens the connection, verifying the certificate first when the target is
    /// https.
    ///
    /// Verification is the entire difference between https and a plaintext port:
    /// without it, an attacker in the path answers the handshake and receives a
    /// profile — the shape of the code and the URLs it serves. Refusing an
    /// unverifiable certificate is therefore a hard failure, never a prompt.
    fn connect(&self) -> Result<Box<dyn ReadWrite>, String> {
        let timeout = std::time::Duration::from_secs(10);
        let tcp = std::net::TcpStream::connect(self.authority())
            .map_err(|error| format!("cannot reach {}: {error}", self.authority()))?;
        let _ = tcp.set_read_timeout(Some(timeout));
        let _ = tcp.set_write_timeout(Some(timeout));
        if !self.tls {
            return Ok(Box::new(tcp));
        }

        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server = rustls::pki_types::ServerName::try_from(self.host.clone())
            .map_err(|_| format!("{} is not a valid server name", self.host))?;
        let connection = rustls::ClientConnection::new(std::sync::Arc::new(config), server)
            .map_err(|error| format!("cannot start TLS to {}: {error}", self.host))?;
        Ok(Box::new(rustls::StreamOwned::new(connection, tcp)))
    }
}

/// Read + Write in one object, so the remote path is written once for both
/// transports rather than duplicated per socket type.
pub(crate) trait ReadWrite: std::io::Read + std::io::Write {}
impl<T: std::io::Read + std::io::Write> ReadWrite for T {}

/// Profiles a running `--probe` binary through its endpoint socket: reads the
/// build key from its `.key` file (or `ELEPHC_PROBE_KEY`), runs the
/// mutual HMAC handshake, receives the folded profile, and renders the same
/// table (plus optional Speedscope/pprof). Needs no macOS sampler.
pub(crate) fn run_probe_host(cmd: &MonitorCommand, socket: &str) -> i32 {
    use std::os::unix::net::UnixStream;
    let key = match resolve_probe_key(cmd, socket) {
        Ok(key) => key,
        Err(error) => {
            eprintln!("elephc monitor: {error}");
            return 1;
        }
    };
    // A path is a local socket; `host:port` is a service somewhere else. The
    // handshake is identical, so the transport is the only thing that differs —
    // which is what lets one command read a binary on this machine and a service
    // on another.
    let mut stream: Box<dyn ReadWrite> = match remote_target(socket) {
        Some(target) => match target.connect() {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("elephc monitor: {error}");
                return 1;
            }
        },
        None => match UnixStream::connect(socket) {
            Ok(stream) => Box::new(stream),
            Err(error) => {
                eprintln!("elephc monitor: cannot connect to probe at {socket}: {error}");
                return 1;
            }
        },
    };
    let nonce_c = probe_nonce();
    let want = if cmd.exact {
        elephc_probe::endpoint::WANT_EXACT
    } else {
        elephc_probe::endpoint::WANT_SAMPLED
    };
    let folded = match elephc_probe::endpoint::wire::client_handshake_and_fetch(
        &mut stream,
        &key,
        &nonce_c,
        want,
    ) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("elephc monitor: {error}");
            return 1;
        }
    };
    // Announced only once the peer has PROVEN it holds the same build key.
    // Printing it on connect claimed a relationship that had not been
    // established — and still printed when the TLS certificate was then refused,
    // which reads as "connected, then something odd happened" rather than
    // "never connected to anything trustworthy".
    println!(
        "connected to probe build {}",
        elephc_probe::handshake::fingerprint(&key)
    );
    if cmd.exact {
        // An exact slice is `elephc-instr:` rows, not folded stacks: handing it
        // to the sampled parser produced an empty display and a message saying
        // nothing had arrived, while the server had in fact just sent 290 bytes
        // of profile. Same rendering as a locally captured exact run, because it
        // is the same data.
        let graph = parse_instrument_dump(&folded);
        if graph.nodes.is_empty() {
            eprintln!(
                "elephc monitor: no exact slice arrived within the wait. A slice is \
                 rendered when a profiled request completes, so a service with no \
                 traffic has none to give yet; drop `--exact` for the sampled view, \
                 which does not need a request to finish."
            );
            return 1;
        }
        print!("{}", instrument_table(&graph));
        return 0;
    }
    let display = folded_text_to_display(&folded);
    if display.is_empty() {
        eprintln!("elephc monitor: the probe returned no samples yet — is the process busy?");
        return 1;
    }
    if let Some(out_path) = &cmd.out {
        if let Err(error) = write_speedscope(&display, out_path) {
            eprintln!("elephc monitor: {error}");
            return 1;
        }
        println!("wrote {out_path}");
    }
    if let Some(pprof_path) = &cmd.pprof_out {
        let stacks = php_folded_stacks(&display);
        if let Err(error) = std::fs::write(pprof_path, crate::pprof_encode::encode_folded_profile(&stacks)) {
            eprintln!("elephc monitor: cannot write {pprof_path}: {error}");
            return 1;
        }
        println!("wrote {pprof_path}");
    }
    // A remote probe capture has no local dSYM/source, so no line attribution.
    if let Err(error) = write_graph_exports(cmd, &display, "elephc probe (remote)", None) {
        eprintln!("elephc monitor: {error}");
        return 1;
    }
    print!("{}", why_table(&display, 1));
    print!("{}", probe_io_summary(&folded));
    0
}

/// A per-connection client nonce from the OS RNG (time-seeded fallback).
pub(crate) fn probe_nonce() -> [u8; 32] {
    use std::io::Read as _;
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let mut nonce = [0u8; 32];
        if file.read_exact(&mut nonce).is_ok() {
            return nonce;
        }
    }
    let mut state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    let mut nonce = [0u8; 32];
    for byte in nonce.iter_mut() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = (state >> 24) as u8;
    }
    nonce
}

