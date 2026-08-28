//! Purpose:
//! Reads program counters out of a process this tool did not launch: attach,
//! stop, read the registers and walk the frame chain, detach. Linux only.
//!
//! Called from:
//! - `monitor::attach`, for `--attach <pid>`, which is handed a pid already
//!   running under someone else's control and has no channel to ask over.
//!
//! Key details:
//! - This is the one file that cannot be exercised without `ptrace`, which is
//!   why it holds the syscalls AND the loop that drives them, and nothing else.
//!   Everything that turns what it read into a profile lives in `elf` and
//!   `attach`, takes numbers and returns names, and is tested on any host.
//! - Attaching is INTRUSIVE in a way asking is not: the target is stopped for
//!   the duration of each sample. Every path out of a stop resumes it, because
//!   a target left stopped by a profiler that walked away is worse than no
//!   profile at all.
//! - A frame walk trusts the frame pointer. Code built without one cannot be
//!   walked this way, and the walk stops rather than inventing frames.

use std::io;

/// How deep a single frame walk may go.
///
/// A chain longer than this is a corrupt frame pointer, not a deep program: a
/// cycle in the chain would otherwise walk until some unrelated read failed.
/// Stopping at a fixed depth bounds the time the target is held stopped, which
/// is the cost every sample charges to the program being profiled.
const MAX_DEPTH: usize = 256;

/// A thread stopped and read.
pub(crate) struct Registers {
    /// Where the thread was interrupted.
    pub(crate) pc: u64,
    /// The frame pointer, which is where the chain walk starts.
    pub(crate) fp: u64,
}

/// Every thread of a process, as the kernel lists them.
///
/// A process is sampled thread by thread: `ptrace` acts on threads, not
/// processes, and a program whose work happens off the main thread would
/// otherwise profile as idle. The list is re-read every window because threads
/// come and go.
pub(crate) fn thread_ids(pid: u32) -> Vec<u32> {
    let Ok(entries) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return Vec::new();
    };
    let mut tids: Vec<u32> = entries
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse().ok())
        .collect();
    // Ascending, so the main thread leads and two windows line up rather than
    // reshuffling on directory order.
    tids.sort_unstable();
    tids
}

/// The path of the executable behind a pid, as the kernel resolved it.
///
/// Read through `/proc`, not from a command line: a program started through a
/// relative path, a symlink or a `PATH` lookup would otherwise be looked for
/// where it is not, and a deleted-but-running binary is still readable here.
pub(crate) fn executable_path(pid: u32) -> io::Result<std::path::PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/exe"))
}

/// The kernel's own memory map for a pid, which is what says where the
/// executable actually landed.
pub(crate) fn memory_maps(pid: u32) -> io::Result<String> {
    std::fs::read_to_string(format!("/proc/{pid}/maps"))
}

/// Why the kernel is likely to refuse an attach, when it is going to.
///
/// `ptrace_scope` refuses non-descendants on most distributions, and the
/// refusal a caller gets is a bare `EPERM` that reads like a bug in this tool.
/// Naming the knob is the difference between an operator who can fix it in one
/// line and one who files an issue.
pub(crate) fn attach_refusal_hint() -> Option<String> {
    let scope = std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope").ok()?;
    Some(explain_ptrace_scope(scope.trim())?.to_string())
}

/// What one `ptrace_scope` setting means for attaching, in the operator's
/// terms. Split out from reading the file so the wording is testable on a host
/// that does not have one — which includes every macOS developer machine.
pub(crate) fn explain_ptrace_scope(scope: &str) -> Option<&'static str> {
    match scope {
        "0" => None,
        "1" => Some(
            "the kernel's yama/ptrace_scope is 1, which allows attaching to descendants only; \
             run as root, grant CAP_SYS_PTRACE, or set it to 0",
        ),
        "2" => Some(
            "the kernel's yama/ptrace_scope is 2, which allows attaching to CAP_SYS_PTRACE \
             holders only",
        ),
        _ => Some("the kernel's yama/ptrace_scope forbids attaching entirely"),
    }
}

/// Attaches to one thread, leaving it running.
///
/// `SEIZE` rather than `ATTACH`: seizing does not stop the thread as a side
/// effect, so each later stop is one this code asked for and can undo exactly.
/// `ATTACH` conflates the two and leaves a thread stopped by a signal that a
/// resume then has to guess at.
pub(crate) fn seize(tid: u32) -> io::Result<()> {
    // SAFETY: a ptrace request with no memory operands. An invalid tid is the
    // kernel's to reject, which it does with ESRCH.
    let result = unsafe { libc::ptrace(libc::PTRACE_SEIZE, tid as libc::pid_t, 0, 0) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Asks a seized thread to stop. Does not wait for it.
///
/// Split from the wait deliberately. Once this SUCCEEDS the thread is stopped or
/// about to be, and from that moment every exit owes it a resume — so the split
/// is what lets the caller put the resume where nothing can step around it.
pub(crate) fn interrupt(tid: u32) -> io::Result<()> {
    // SAFETY: a ptrace request with no memory operands.
    let result = unsafe { libc::ptrace(libc::PTRACE_INTERRUPT, tid as libc::pid_t, 0, 0) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Waits until an interrupted thread is actually stopped.
///
/// The wait is not optional. `INTERRUPT` only *requests* the stop; reading
/// registers before the kernel delivered it returns whatever was there — a
/// plausible-looking address from an arbitrary moment, which is a wrong profile
/// rather than a missing one.
///
/// `EINTR` is retried rather than reported. A profiler runs with signals about
/// (its own timers, a Ctrl-C on the way) and an interrupted wait says nothing
/// about the thread; treating it as a failure would abandon a thread that is
/// stopped and waiting to be read.
pub(crate) fn wait_for_stop(tid: u32) -> io::Result<()> {
    loop {
        let mut status: libc::c_int = 0;
        // SAFETY: `status` is a live local for the duration of the call. __WALL
        // is required for threads, which are not children in the waitpid sense.
        let waited = unsafe { libc::waitpid(tid as libc::pid_t, &mut status, libc::__WALL) };
        if waited != -1 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

/// Lets a stopped thread run again.
///
/// Called on every path out of a stop, including the failing ones: a thread
/// left stopped is a program that has silently hung, and an operator who
/// attached a profiler to a running service has no reason to suspect the
/// profiler of it.
pub(crate) fn resume(tid: u32) {
    // SAFETY: no memory operands; a failure here means the thread already
    // exited, which needs no resuming.
    unsafe {
        libc::ptrace(libc::PTRACE_CONT, tid as libc::pid_t, 0, 0);
    }
}

/// Stops tracing a thread, leaving it running as it was found.
pub(crate) fn detach(tid: u32) {
    // SAFETY: no memory operands; ESRCH for a thread that has gone is fine.
    unsafe {
        libc::ptrace(libc::PTRACE_DETACH, tid as libc::pid_t, 0, 0);
    }
}

/// How many 64-bit words the kernel's register file is on this architecture,
/// and where the two registers a walk needs sit inside it.
///
/// Naming the indices rather than the struct keeps this to one shape and one
/// `iovec`. The layout is ABI — `user_regs_struct` on x86_64, `user_pt_regs` on
/// aarch64 — not a detail that drifts.
#[cfg(target_arch = "x86_64")]
const REGISTER_WORDS: usize = 27;
/// rbp, at index 4 of `user_regs_struct`.
#[cfg(target_arch = "x86_64")]
const FRAME_POINTER_INDEX: usize = 4;
/// rip, at index 16.
#[cfg(target_arch = "x86_64")]
const PROGRAM_COUNTER_INDEX: usize = 16;

/// x0..x30, then sp, pc, pstate.
#[cfg(target_arch = "aarch64")]
const REGISTER_WORDS: usize = 34;
/// x29 is the frame pointer by the AAPCS.
#[cfg(target_arch = "aarch64")]
const FRAME_POINTER_INDEX: usize = 29;
/// pc follows the 31 general registers and sp.
#[cfg(target_arch = "aarch64")]
const PROGRAM_COUNTER_INDEX: usize = 32;

/// Reads the two registers a frame walk starts from.
///
/// `GETREGSET` rather than the older `GETREGS`, because `GETREGS` does not
/// exist on aarch64 — the register file is architecture-shaped and only the
/// regset interface is common to both.
pub(crate) fn registers(tid: u32) -> io::Result<Registers> {
    let mut regs = [0u64; REGISTER_WORDS];
    let mut iov = libc::iovec {
        iov_base: regs.as_mut_ptr().cast::<libc::c_void>(),
        iov_len: std::mem::size_of_val(&regs),
    };
    // SAFETY: `iov` describes `regs`, which outlives the call; NT_PRSTATUS is
    // the regset that layout belongs to, and the kernel writes at most
    // `iov_len` bytes.
    let result = unsafe {
        libc::ptrace(
            libc::PTRACE_GETREGSET,
            tid as libc::pid_t,
            libc::NT_PRSTATUS,
            &mut iov as *mut libc::iovec,
        )
    };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(Registers { pc: regs[PROGRAM_COUNTER_INDEX], fp: regs[FRAME_POINTER_INDEX] })
}

/// Reads a run of bytes out of the target's address space.
///
/// `process_vm_readv` rather than word-at-a-time `PEEKDATA`: a frame is two
/// words, and a `PEEKDATA` pair is two syscalls for the same bytes — which is
/// the target held stopped twice as long, every frame of every sample.
pub(crate) fn read_memory(pid: u32, address: u64, into: &mut [u8]) -> io::Result<()> {
    let local = libc::iovec {
        iov_base: into.as_mut_ptr().cast::<libc::c_void>(),
        iov_len: into.len(),
    };
    let remote = libc::iovec {
        iov_base: address as usize as *mut libc::c_void,
        iov_len: into.len(),
    };
    // SAFETY: `local` describes `into`, which outlives the call. `remote` is an
    // address in ANOTHER process and is never dereferenced here; an unmapped one
    // comes back as EFAULT rather than faulting this process.
    let read = unsafe { libc::process_vm_readv(pid as libc::pid_t, &local, 1, &remote, 1, 0) };
    if read == -1 {
        return Err(io::Error::last_os_error());
    }
    if read as usize != into.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "short read from the target"));
    }
    Ok(())
}

/// Reads one frame off the chain: the caller's frame pointer, and the address
/// to return to.
///
/// Split from the walk so the walk's stopping rules can be read — and tested —
/// without a process to read from.
pub(crate) fn decode_frame(bytes: [u8; 16]) -> (u64, u64) {
    let next = u64::from_le_bytes(bytes[0..8].try_into().unwrap_or([0; 8]));
    let return_address = u64::from_le_bytes(bytes[8..16].try_into().unwrap_or([0; 8]));
    (next, return_address)
}

/// Whether a frame pointer can be followed at all.
///
/// Three ways it cannot: null, unaligned — no ABI here produces one, so it is a
/// register holding something that is not a pointer — or not climbing, which is
/// a cycle. Following any of them invents frames, and an invented frame does not
/// announce itself: it just names the wrong function as the expensive one. A
/// short true stack beats a long false one.
pub(crate) fn can_follow(fp: u64, previous: u64) -> bool {
    fp != 0 && fp % 8 == 0 && fp > previous
}

/// Walks the frame chain from an interrupted thread, innermost first.
///
/// One frame is two words: at `[fp]` the caller's frame pointer and at `[fp+8]`
/// the address to return to. The walk stops at the first read that fails, at a
/// pointer `can_follow` rejects, and at `MAX_DEPTH`.
pub(crate) fn walk(pid: u32, regs: &Registers) -> Vec<u64> {
    let mut chain = vec![regs.pc];
    let mut fp = regs.fp;
    for _ in 0..MAX_DEPTH {
        if !can_follow(fp, 0) {
            break;
        }
        let mut frame = [0u8; 16];
        if read_memory(pid, fp, &mut frame).is_err() {
            break;
        }
        let (next, return_address) = decode_frame(frame);
        if return_address == 0 {
            break;
        }
        chain.push(return_address);
        if !can_follow(next, fp) {
            break;
        }
        fp = next;
    }
    chain
}

/// Stops one thread, reads its stack, and lets it go again.
///
/// The resume is unconditional past the interrupt, which is the point of the
/// wrapper: every failure between the stop and the read still has to give the
/// thread back. It was not always so: the interrupt and the wait were one call,
/// so a wait that failed returned before any resume and left the thread stopped
/// until the window ended and the detach released it. A single `EINTR` was
/// enough, and the symptom would have been a program that stalls for as long as
/// it is being profiled — which reads as the profiler being slow, not as the
/// profiler having stopped it.
pub(crate) fn sample_thread(pid: u32, tid: u32) -> Option<Vec<u64>> {
    // Below this line the thread is stopped, or on its way to being; above it,
    // nothing has happened to it.
    if interrupt(tid).is_err() {
        return None;
    }
    let read = wait_for_stop(tid).and_then(|()| registers(tid));
    resume(tid);
    Some(walk(pid, &read.ok()?))
}

/// How often a thread is stopped and read, per second.
///
/// A prime, and deliberately not a round one: a program doing something on a
/// 100 Hz timer, sampled at 100 Hz, is caught in the same place every time and
/// reports one function as the whole profile. 99 keeps the sampler and the
/// program from marching in step.
///
/// It is also a budget. Every sample stops every thread, so the rate is what
/// the profiled program pays; this is the rate `py-spy` and `perf` default to
/// for the same reason.
const SAMPLE_HZ: u64 = 99;

/// Attaches to every thread of every process given, samples them for a window,
/// and detaches — returning the folded display stacks the rest of `monitor`
/// renders.
///
/// A tree, not one process, because `--attach` is documented to measure a
/// prefork server across all its workers and the macOS path does. The threads
/// of every process are read on every tick so each contributes samples in
/// proportion to the time it actually spent running.
///
/// The seizes happen once for the window rather than per sample: seizing is the
/// expensive half and the threads keep running between samples either way.
/// Threads and workers that appear mid-window are missed until the next one,
/// which is the price of not re-reading `/proc` at 99 Hz.
///
/// Failure is silence, not an error. A thread that exits mid-window, a stack
/// that cannot be walked, a program that ends early — none of them are worth
/// refusing a window over, because the samples that DID land are still true.
/// The caller learns the target is gone the same way it always has: an empty
/// window.
pub(crate) fn attach_window(
    pids: &[u32],
    duration_secs: u32,
    image: &super::attach::Image,
) -> Vec<(Vec<(String, super::Kind)>, u64)> {
    // Each process brings its own bias and its own threads; they share the
    // symbol table, because a prefork server's workers are forks of one image.
    // A process whose bias cannot be read is dropped rather than resolved
    // against a neighbour's, which would not fail — it would name the wrong
    // functions, and a table that is confidently wrong is worse than a short one.
    let mut targets: Vec<(u32, u64, Vec<u32>)> = Vec::new();
    for pid in pids {
        let Some(bias) = super::attach::bias_of(image, *pid) else { continue };
        let seized: Vec<u32> = thread_ids(*pid).into_iter().filter(|tid| seize(*tid).is_ok()).collect();
        if !seized.is_empty() {
            targets.push((*pid, bias, seized));
        }
    }
    if targets.is_empty() {
        return Vec::new();
    }
    let interval = std::time::Duration::from_nanos(1_000_000_000 / SAMPLE_HZ);
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(u64::from(duration_secs.max(1)));
    let mut stacks = Vec::new();
    while std::time::Instant::now() < deadline {
        let started = std::time::Instant::now();
        // Every process every tick, rather than one process for the whole
        // window each: a worker sampled for a third of the window contributes a
        // third of the samples, and its share of the table would be a third of
        // the truth.
        for (pid, bias, seized) in &targets {
            for tid in seized {
                let Some(chain) = sample_thread(*pid, *tid) else { continue };
                let stack = super::attach::display_stack(&chain, &image.symbols, *bias);
                if !stack.is_empty() {
                    stacks.push(stack);
                }
            }
        }
        // Sleep the remainder of the interval, not the whole of it: the samples
        // themselves take time, and sleeping a full interval on top of them
        // would drift the rate below the one documented above.
        if let Some(rest) = interval.checked_sub(started.elapsed()) {
            std::thread::sleep(rest);
        }
    }
    for (_, _, seized) in &targets {
        for tid in seized {
            detach(*tid);
        }
    }
    super::attach::fold(stacks)
}
