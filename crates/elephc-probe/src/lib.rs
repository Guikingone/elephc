//! Purpose:
//! In-process sampling probe for elephc-compiled programs: a SIGPROF handler
//! walks the interrupted frame-pointer chain into a fixed ring buffer, and the
//! exit dump symbolizes the raw program counters against the symbol table the
//! COMPILER embedded — no DWARF, no external sampler, no target suspension.
//!
//! Called from:
//! - Generated code: `elephc_probe_init(table, len)` in main's prologue and
//!   `elephc_probe_dump()` in its epilogue, emitted under `--probe`.
//!
//! Key details:
//! - The handler is async-signal-safe by construction: raw pointer writes into
//!   preallocated statics, one atomic head increment, no allocation, no locks.
//! - Samples record raw PCs; symbolization happens at dump time, outside the
//!   handler. A PC below the first function or past the compiler-emitted text
//!   end sentinel reports as `<native>` (runtime helpers, libc).
//! - The ring holds the most recent `RING_SLOTS` samples (~1 sample/ms of CPU
//!   time); long runs keep the tail, which is what a probe window serves.

pub mod endpoint;
pub mod handshake;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// One entry of the compiler-embedded symbol table: function start address,
/// name pointer, name length. Layout must match the emitted `.quad` triples.
#[repr(C)]
pub struct SymtabEntry {
    pub address: u64,
    pub name_ptr: u64,
    pub name_len: u64,
}

/// Deepest stack the handler records; deeper frames are truncated at the root.
const MAX_FRAMES: usize = 32;
/// Ring capacity in samples. At the 1ms period this holds ~8s of CPU time.
const RING_SLOTS: usize = 8192;
/// Words per ring slot: `[depth, route_id, allocs, pc0, pc1, ...]`.
const SLOT_WORDS: usize = MAX_FRAMES + 4;
/// Word index of the slot's sequence counter: even when the slot is settled,
/// odd while a handler is writing it.
///
/// The ring is written by SIGPROF handlers in several processes and read by the
/// endpoint while they run. Publishing `depth` last with `Release` is a correct
/// gate for a slot that is still zero — but the region is zeroed exactly once,
/// and after the head wraps past `RING_SLOTS` every reused slot already carries
/// a non-zero depth from its previous lap. A reader that acquired that stale
/// depth read a slot mid-overwrite: fresh frames under old ones, a route from
/// one sample and program counters from another, folded and counted as if real.
/// Bounded, so never a bad access — just a stack that never happened, with
/// nothing in the format to tell it from a true one.
const SEQ_WORD: usize = MAX_FRAMES + 3;
/// Word index of the first program counter in a slot.
const PC_WORD0: usize = 3;
/// Cap on distinct request routes recorded; overflow buckets into `<other>`.
const MAX_ROUTES: usize = 256;
/// The slot the overflow bucket occupies, leaving `MAX_ROUTES - 1` for real
/// routes. Reserved rather than taken from the front so a full table still
/// resolves every id it handed out before it filled.
const OTHER_ROUTE_INDEX: usize = MAX_ROUTES - 1;
/// The name that bucket carries. Samples past the cap used to come back with id
/// 0, which is also what a CLI run and an idle worker carry, so a late endpoint
/// did not appear as over-capacity — it did not appear at all, folded silently
/// into the untagged pool. A named bucket says the table filled.
const OTHER_ROUTE_NAME: &str = "<other>";
/// Bytes per shared route slot: a 1-byte length then up to 63 name bytes.
const ROUTE_SLOT_BYTES: usize = 64;
/// Max route name length that fits a slot.
const ROUTE_NAME_MAX: usize = ROUTE_SLOT_BYTES - 1;
/// Ring bytes: an 8-byte head counter followed by the slot array.
const RING_BYTES: usize = 8 + RING_SLOTS * SLOT_WORDS * 8;
/// Route-table bytes: an 8-byte count followed by the fixed route slots.
const ROUTE_TABLE_BYTES: usize = 8 + MAX_ROUTES * ROUTE_SLOT_BYTES;
/// Words per route in the event table: `[io_ops, wait_ns]`.
const EVENT_WORDS: usize = 2;
/// One extra bucket for id 0, which the route table reserves for "untagged" —
/// a CLI run, or a `--web` request whose route did not fit the table. Dropping
/// those events would understate the totals silently, which is worse than a row
/// nobody expected.
const EVENT_BUCKETS: usize = MAX_ROUTES + 1;
/// Event-table bytes: fixed counters per route id.
const EVENT_TABLE_BYTES: usize = EVENT_BUCKETS * EVENT_WORDS * 8;
/// Control-area bytes: one word, `1` once anyone has asked this service to
/// sample. It lives in the SHARED mapping and not beside `ASKED` in process
/// memory because that is the whole point: `ASKED` is a per-process
/// `AtomicBool`, so a `--web` master that authenticates a client after its
/// workers forked flips only its own copy, and the master runs no PHP. The
/// workers must be able to observe the ask that happened after they existed.
const CONTROL_BYTES: usize = 8;
/// How many recently accepted profiling signatures the service remembers.
///
/// One entry is spent per ACCEPTED request, and only the key holder can produce
/// one, so this is sized against how often an operator profiles rather than
/// against traffic. Sixty-four covers far more asks than a five-minute window
/// ever carries.
const REPLAY_SLOTS: usize = 64;
/// Bytes per remembered signature: the tag's first eight bytes, then the second
/// it was accepted at.
const REPLAY_SLOT_BYTES: usize = 16;
/// How many times a spend re-searches after losing a slot to another process.
///
/// Each loss means somebody else wrote a slot we had picked, and what they wrote
/// may be this very tag — which only a fresh search can see. Bounded because a
/// caller is inside an HTTP request: three passes over sixty-four words costs
/// nothing and needs every one of sixty-three other processes to beat this one
/// twice in a row to run out.
const SPEND_ATTEMPTS: usize = 3;
/// Replay-table bytes, in the shared mapping because the check has to hold
/// ACROSS `--web` workers: a captured header replayed against a different worker
/// than the one that served the original would otherwise meet a process that had
/// never seen it.
const REPLAY_TABLE_BYTES: usize = REPLAY_SLOTS * REPLAY_SLOT_BYTES;
/// Shared-region byte size: the ring, then the route table, then the per-route
/// event counters, then the control word, then the replay table — all inherited
/// across a `--web` fork, so route ids stay consistent, every worker's counters
/// land in one place, and a signature spent on one worker is spent on all.
///
/// Each new area is appended LAST so adding it moves no existing offset: every
/// other area is addressed from the constants above it.
const REGION_BYTES: usize =
    RING_BYTES + ROUTE_TABLE_BYTES + EVENT_TABLE_BYTES + CONTROL_BYTES + REPLAY_TABLE_BYTES;

/// The shared "someone asked" word, or `None` before the region is mapped.
///
/// # Safety
/// Valid only after `elephc_probe_init` mapped the region.
unsafe fn region_asked<'a>(base: usize) -> &'a std::sync::atomic::AtomicU64 {
    let offset = RING_BYTES + ROUTE_TABLE_BYTES + EVENT_TABLE_BYTES;
    &*((base + offset) as *const std::sync::atomic::AtomicU64)
}

/// One remembered signature as its two words: the tag, then when it was taken.
///
/// # Safety
/// Valid only after the region is mapped, for `index < REPLAY_SLOTS`.
unsafe fn replay_slot<'a>(
    base: usize,
    index: usize,
) -> (
    &'a std::sync::atomic::AtomicU64,
    &'a std::sync::atomic::AtomicU64,
) {
    let offset = RING_BYTES
        + ROUTE_TABLE_BYTES
        + EVENT_TABLE_BYTES
        + CONTROL_BYTES
        + index * REPLAY_SLOT_BYTES;
    (
        &*((base + offset) as *const std::sync::atomic::AtomicU64),
        &*((base + offset + 8) as *const std::sync::atomic::AtomicU64),
    )
}

/// I/O events are **not sampled**. A driver call fires exactly one, so these
/// counts are exact — the sampler's statistical nature applies to *time*, which
/// it observes 1000x/second, not to events it is told about. Keeping the two
/// apart matters: an exact query count printed beside a sampled time share,
/// with nothing saying which is which, is how a profile misleads.
///
/// `route_id` is the 1-based id the route table hands out; 0 means untagged.
unsafe fn event_word<'a>(
    base: usize,
    route_id: usize,
    word: usize,
) -> &'a std::sync::atomic::AtomicU64 {
    let offset = RING_BYTES + ROUTE_TABLE_BYTES + (route_id * EVENT_WORDS + word) * 8;
    &*((base + offset) as *const std::sync::atomic::AtomicU64)
}

/// Adds `amount` to one of the current request's event counters.
fn note_event(word: usize, amount: u64) {
    let base = REGION.load(Ordering::Relaxed);
    if base == 0 {
        return;
    }
    let route_id = CURRENT_ROUTE.load(Ordering::Relaxed) as usize;
    if route_id < EVENT_BUCKETS {
        unsafe { event_word(base, route_id, word) }.fetch_add(amount, Ordering::Relaxed);
    }
}

/// Records one I/O operation against the request currently being served.
///
/// Reached through the same `_elephc_instr_io_fn` slot the exact profiler uses,
/// so the PDO bridge needs no knowledge of which profiler is linked. Costs one
/// atomic increment, paid only when a driver call happens — already orders of
/// magnitude slower — so unlike per-call instrumentation this is viable in
/// production.
#[no_mangle]
pub extern "C" fn elephc_probe_note_io() {
    note_event(0, 1);
}

/// Records nanoseconds blocked inside a driver call, against the current request.
#[no_mangle]
pub extern "C" fn elephc_probe_note_wait(ns: u64) {
    note_event(1, ns);
}

/// Renders the per-route event counters, one line per route that saw any I/O.
///
/// Deliberately its own line prefix rather than extra columns on the folded
/// samples: a consumer must not be able to mistake an exact count for a sampled
/// weight.
pub fn event_report(base: usize) -> String {
    let mut out = String::new();
    if base == 0 {
        return out;
    }
    let count = unsafe {
        (*((base + RING_BYTES) as *const std::sync::atomic::AtomicU64)).load(Ordering::Acquire)
    } as usize;
    for route_id in 0..EVENT_BUCKETS.min(count + 1) {
        let io = unsafe { event_word(base, route_id, 0) }.load(Ordering::Relaxed);
        let wait = unsafe { event_word(base, route_id, 1) }.load(Ordering::Relaxed);
        if io == 0 && wait == 0 {
            continue;
        }
        let name = if route_id == 0 {
            "<untagged>".to_string()
        } else {
            unsafe { read_route_slot(base, route_id - 1) }
        };
        out.push_str(&format!("elephc-probe-io: {name} ops={io} wait_ns={wait}\n"));
    }
    out
}
/// Sampling period, microseconds of CPU time between SIGPROF deliveries.
const PERIOD_MICROS: i64 = 1000;

/// Base address of the sample region: an atomic head counter at offset 0, then
/// `RING_SLOTS` slots of `[depth, pc...]`. Zero until `elephc_probe_init` maps
/// it. Mapped `MAP_SHARED` BEFORE any `--web` fork, so every worker's SIGPROF
/// handler cooperatively fills one shared ring the master's endpoint reads.
static REGION: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    /// Runtime `.comm` word: 1 once this process has been asked to profile.
    /// Written here, read by the exact profiler's boot, which runs afterwards.
    static mut elephc_monitor_active: u64;
}
/// Set by `elephc_probe_dump` so a late signal cannot race the ring read.
static STOPPED: AtomicBool = AtomicBool::new(false);

/// Whether this process was ever ASKED to profile.
///
/// Distinct from "the ring is mapped", which is true in every `--with-monitoring`
/// binary because the mapping is what makes the capability free to carry. Arming
/// on the mapping instead of on this made a `--web` service sample and log itself
/// with nobody asking: the worker re-armed at startup, and the first request's
/// epilogue dumped to stderr. Measured on a service built with the flag and asked
/// nothing: 12 `elephc-probe:` lines over three ordinary requests, against none
/// from the same program built without it.
static ASKED: AtomicBool = AtomicBool::new(false);

/// Returns the shared head counter, or `None` before the region is mapped.
///
/// # Safety
/// Valid only after `elephc_probe_init` mapped the region; the returned
/// reference lives as long as the process (the mapping is never unmapped).
unsafe fn region_head<'a>() -> Option<&'a std::sync::atomic::AtomicU64> {
    let base = REGION.load(Ordering::Relaxed);
    if base == 0 {
        return None;
    }
    Some(&*(base as *const std::sync::atomic::AtomicU64))
}

/// Returns word `word` of ring slot `index` as an atomic — the ring is shared
/// with a concurrent reader, so every slot word is accessed atomically.
///
/// # Safety
/// `base` must be the mapped region, `index < RING_SLOTS`, `word < SLOT_WORDS`.
unsafe fn region_word<'a>(
    base: usize,
    index: usize,
    word: usize,
) -> &'a std::sync::atomic::AtomicU64 {
    let addr = base + 8 + index * SLOT_WORDS * 8 + word * 8;
    &*(addr as *const std::sync::atomic::AtomicU64)
}

/// Symbol table pointer/length, published once by `elephc_probe_init`.
static TABLE_PTR: AtomicUsize = AtomicUsize::new(0);
static TABLE_LEN: AtomicUsize = AtomicUsize::new(0);
/// Build-key pointer, published by `elephc_probe_init` for the endpoint handshake.
static KEY_PTR: AtomicUsize = AtomicUsize::new(0);
/// Route id (1-based index into `ROUTES`) the SIGPROF handler stamps onto each
/// sample; 0 means no active request. Set by `elephc_probe_set_route`.
static CURRENT_ROUTE: AtomicUsize = AtomicUsize::new(0);

extern "C" {
    /// Runtime `.comm` slot holding the ADDRESS of the program's `_gc_allocs`
    /// counter, published by `--probe` init. Zero when the program was not built
    /// with `--probe`, which is also when this crate is not linked.
    ///
    /// A pointer rather than the counter itself: `_gc_allocs` is emitted with a
    /// hardcoded leading underscore, which is fine while only assembly names it
    /// and would break every ELF link the moment a Rust crate resolved it.
    static elephc_probe_allocs_ptr: usize;
}

/// Allocation count at the previous sample, for the delta this one is charged.
///
/// Process-local on purpose. `_gc_allocs` is ordinary memory, so each `--web`
/// worker gets its own copy at fork; a shared "last" would make every worker
/// subtract another's progress and produce negative deltas.
///
/// `NO_PREVIOUS_SAMPLE` and not zero, because zero is a legitimate reading: it
/// is what a process that has allocated nothing reports. Seeding this at zero
/// meant the FIRST sample's delta was the whole counter — every allocation the
/// process had made since it started, charged in full to whichever stack the
/// timer happened to catch first. On the ordinary path, where a worker adopts an
/// ask after hours of serving, that is a single fabricated row dwarfing the real
/// ones by orders of magnitude.
static LAST_ALLOCS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(NO_PREVIOUS_SAMPLE);

/// Sentinel for "no sample has been taken in this process yet", distinguishable
/// from a real reading of zero.
const NO_PREVIOUS_SAMPLE: u64 = u64::MAX;

/// Bytes reserved for the handler's own stack. Comfortably above what a frame
/// walk and a register spill need, and it costs nothing but address space in the
/// binary's BSS.
const SIGSTACK_BYTES: usize = 64 * 1024;

/// The alternate signal stack itself.
///
/// A static rather than an allocation: it must exist before the first signal can
/// arrive, survive `fork` at the same address, and never depend on an allocator
/// the handler is forbidden to touch.
static mut SIGSTACK: [u8; SIGSTACK_BYTES] = [0; SIGSTACK_BYTES];

/// Reads the program's allocation counter, or `None` outside `--probe`.
///
/// # Safety
/// Called from the SIGPROF handler: a plain load of a `.comm` word, which is
/// async-signal-safe.
unsafe fn current_allocs() -> Option<u64> {
    let addr = std::ptr::addr_of!(elephc_probe_allocs_ptr).read();
    if addr == 0 {
        return None;
    }
    Some(*(addr as *const u64))
}

/// Interns `route` into the SHARED route table, returning its 1-based id — so a
/// route id stamped by one `--web` worker resolves to the same name in the
/// master's endpoint. Full table buckets into `<other>`. Runs in the worker's
/// normal context (append + scan), never the signal handler.
///
/// Gated on having been asked, like the sampling itself — the caller checks
/// before it gets here. The first version was not, on the reasoning that a
/// route id must be stable from boot for a sample taken at any moment to
/// resolve to the right name. That reasoning does not survive its own premise:
/// `on_sigprof` returns immediately unless the process has been asked, so no
/// sample exists before the ask to need a stable id, and interning early bought
/// nothing. It cost the table. Serving a few hundred `/users/123`-shaped URLs
/// before anyone connected filled all 256 slots, and every endpoint that
/// mattered afterwards — the one being profiled — arrived to a full table. It
/// also put every raw path, tokens included, into the shared mapping from the
/// moment it was served, and made the per-route counters cover the process's
/// whole life rather than the window an operator chose. All three go away
/// together.
///
/// # Safety
/// Valid only after the region is mapped.
unsafe fn intern_route(route: &str) -> usize {
    let base = REGION.load(Ordering::Relaxed);
    if base == 0 {
        return 0;
    }
    // The route comes from an untrusted HTTP path. Neutralize the folded-format
    // metacharacters — `;` (frame separator), newlines (line separator) — and any
    // other control byte, so a crafted path cannot forge frames or profile lines.
    let sanitized: String = route
        .chars()
        .map(|c| {
            if c == ';' || c.is_control() {
                '?'
            } else {
                c
            }
        })
        .collect();
    let mut name = sanitized.as_str();
    if name.len() > ROUTE_NAME_MAX {
        // Truncate on a char boundary so the stored name stays valid UTF-8.
        let mut end = ROUTE_NAME_MAX;
        while end > 0 && !name.is_char_boundary(end) {
            end -= 1;
        }
        name = &name[..end];
    }
    let count_ptr = (base + RING_BYTES) as *const std::sync::atomic::AtomicU64;
    let count = &*count_ptr;
    let existing = count.load(Ordering::Acquire) as usize;
    for index in 0..existing.min(OTHER_ROUTE_INDEX) {
        if read_route_slot(base, index) == name {
            return index + 1;
        }
    }
    if existing >= OTHER_ROUTE_INDEX {
        return other_route_id(base);
    }
    // Claim the next slot. A benign race can duplicate a name across workers;
    // both ids resolve to the same text, so grouping is unaffected.
    let index = count.fetch_add(1, Ordering::AcqRel) as usize;
    if index >= OTHER_ROUTE_INDEX {
        // Lost the last free slot to another worker between the load and the
        // add. The counter is left where it is: it is only ever compared against
        // the cap, never used as a length.
        return other_route_id(base);
    }
    write_route_slot(base, index, name.as_bytes());
    index + 1
}

/// Publishes the overflow bucket if it is not there yet and returns its id.
///
/// Written lazily so a service whose routes fit the table never carries a row
/// it did not earn. Two workers overflowing at once write the same bytes into
/// the same slot, which is why this needs no claim of its own.
///
/// # Safety
/// Valid only after the region is mapped.
unsafe fn other_route_id(base: usize) -> usize {
    if read_route_slot(base, OTHER_ROUTE_INDEX).is_empty() {
        write_route_slot(base, OTHER_ROUTE_INDEX, OTHER_ROUTE_NAME.as_bytes());
    }
    OTHER_ROUTE_INDEX + 1
}

/// Writes a route name into shared slot `index` (`[len][bytes]`). The bytes are
/// stored first, then the length via an `AtomicU8` Release store as the slot's
/// readiness marker: a reader that loads a non-zero length Acquire is
/// guaranteed to see the bytes it covers. The length byte is a real atomic, so
/// the concurrent reader/writer pair is not a data race.
unsafe fn write_route_slot(base: usize, index: usize, name: &[u8]) {
    let slot = (base + RING_BYTES + 8 + index * ROUTE_SLOT_BYTES) as *mut u8;
    let len = name.len().min(ROUTE_NAME_MAX) as u8;
    std::ptr::copy_nonoverlapping(name.as_ptr(), slot.add(1), len as usize);
    (*(slot as *const std::sync::atomic::AtomicU8)).store(len, Ordering::Release);
}

/// Reads the route name from shared slot `index`. An empty (unpublished) slot
/// reads as `""`; callers treat that as "no route".
unsafe fn read_route_slot(base: usize, index: usize) -> String {
    let slot = (base + RING_BYTES + 8 + index * ROUTE_SLOT_BYTES) as *const u8;
    let len = ((*(slot as *const std::sync::atomic::AtomicU8)).load(Ordering::Acquire) as usize)
        .min(ROUTE_NAME_MAX);
    let bytes = std::slice::from_raw_parts(slot.add(1), len);
    String::from_utf8_lossy(bytes).into_owned()
}

/// Sets the route stamped onto subsequent samples until cleared. An empty
/// `len` clears it (id 0). Called by the web bridge around each request via a
/// `dlsym` lookup, so a non-`--web` binary never pays for it.
///
/// # Safety
/// `route`/`len` describe a UTF-8 route string valid for this call.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_set_route(route: *const u8, len: usize) {
    // Every request, before anything can return early: this is the one call the
    // web bridges make on each request in the process that actually runs PHP, so
    // it is where a worker learns it was asked to sample after it forked.
    observe_shared_ask();
    // Defensive: a bridge bug passing a wild length must not read out of bounds.
    if route.is_null() || len == 0 || len > 4096 {
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        return;
    }
    // Nothing is sampled until someone asks, so nothing needs a name until then
    // either — and the route table is a fixed 256 slots with no eviction, so
    // filling it with traffic nobody is watching is how the traffic somebody IS
    // watching ends up unnamed. `observe_shared_ask` above is what makes this
    // flag true for a `--web` worker that forked before the ask arrived, so the
    // very first request after an operator connects is already tagged.
    if !ASKED.load(Ordering::Relaxed) {
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        return;
    }
    let bytes = std::slice::from_raw_parts(route, len);
    let Ok(text) = std::str::from_utf8(bytes) else {
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        return;
    };
    let id = intern_route(text);
    CURRENT_ROUTE.store(id, Ordering::Relaxed);
}

/// Resolves a route id to its interned name from the shared table.
///
/// # Safety
/// Valid only after the region is mapped.
unsafe fn route_name(id: usize) -> Option<String> {
    if id == 0 {
        return None;
    }
    let base = REGION.load(Ordering::Relaxed);
    if base == 0 || id > MAX_ROUTES {
        return None;
    }
    let name = read_route_slot(base, id - 1);
    // An unpublished (empty) slot is treated as no route, never a blank frame.
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extracts the interrupted program counter and frame pointer from the signal
/// context, per platform and architecture.
///
/// Returns the interrupted `(pc, fp, sp)`. `sp` anchors the frame-pointer walk:
/// a valid frame lies at or above the current stack pointer, which rejects a
/// stale `fp` that only happens to look plausible before it faults.
///
/// # Safety
/// `context` must be the `ucontext_t` pointer SIGPROF delivered.
unsafe fn interrupted_pc_fp(context: *mut libc::c_void) -> (u64, u64, u64) {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let context = context as *mut libc::ucontext_t;
        let state = &(*(*context).uc_mcontext).__ss;
        (state.__pc, state.__fp, state.__sp)
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        let context = context as *mut libc::ucontext_t;
        let state = &(*(*context).uc_mcontext).__ss;
        (state.__rip, state.__rbp, state.__rsp)
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        let context = context as *mut libc::ucontext_t;
        let state = &(*context).uc_mcontext;
        (state.pc, state.regs[29], state.sp)
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        let context = context as *mut libc::ucontext_t;
        let gregs = &(*context).uc_mcontext.gregs;
        (
            gregs[libc::REG_RIP as usize] as u64,
            gregs[libc::REG_RBP as usize] as u64,
            gregs[libc::REG_RSP as usize] as u64,
        )
    }
}

/// Upper bound of the frame-pointer walk above the interrupted stack pointer.
/// A valid frame chain stays within one stack; 64 MiB comfortably covers a
/// worker/CLI stack while rejecting a wild `fp` far from `sp`.
const STACK_WINDOW: u64 = 64 * 1024 * 1024;

/// How many consecutive frames the walk may hold without corroborating them.
///
/// A genuine run of frames outside the program's own text is short — a runtime
/// helper calling another helper calling libc — and ends by returning into
/// compiled PHP. A chain built out of stack garbage does not come back. Four
/// covers every real nesting this runtime produces and bounds how far a bad
/// chain is followed before it is abandoned.
const UNPROVEN_RUN_MAX: usize = 4;

/// The program's own compiled text, as `[first function, text end)`.
///
/// The compiler hands `elephc_probe_init` a symbol table sorted by address whose
/// final entry is the text-end sentinel, so its two ends bound every function
/// this binary compiled. Reading them is two loads from a static array, which is
/// async-signal-safe.
///
/// `None` when no table has been published — a probe built without one cannot
/// corroborate anything, and the walk says so by trusting the chain, which is
/// what it did everywhere before.
///
/// # Safety
/// Valid only once `elephc_probe_init` has stored a table that outlives the
/// process, which is how the compiler emits it.
unsafe fn program_text_range() -> Option<(u64, u64)> {
    let table = TABLE_PTR.load(Ordering::Relaxed) as *const SymtabEntry;
    let len = TABLE_LEN.load(Ordering::Relaxed);
    if table.is_null() || len < 2 {
        return None;
    }
    let low = (*table).address;
    let high = (*table.add(len - 1)).address;
    if high <= low {
        return None;
    }
    Some((low, high))
}

/// Whether a return address falls inside the program's own compiled text.
///
/// The question is only ever asked to CORROBORATE a frame, never to reject one:
/// a genuine frame in a runtime helper or in libc answers false, and so does a
/// stack word an fp-less function happened to leave behind.
fn returns_into_program(address: u64, text: Option<(u64, u64)>) -> bool {
    match text {
        Some((low, high)) => address >= low && address < high,
        // No table to check against. Every frame counts as corroborated, which
        // is the unverified walk this crate had before the table was consulted.
        None => true,
    }
}

/// Walks the frame-pointer chain from `fp`, filling `out` with the return
/// addresses it can stand behind and returning how many that is.
///
/// The shape checks — nonzero, 16-byte aligned, inside `[sp, sp + STACK_WINDOW)`
/// — prove that an address could be a stack slot, which is what makes it safe to
/// dereference. They do not prove it IS a frame: a function that uses the frame
/// register as a general one leaves an ordinary value there, and one that is
/// aligned and inside the window is followed. That is inherent to walking a
/// frame-pointer chain in-process, and every sampler that does it carries it.
///
/// What is NOT inherent is reporting the result. A return address inside the
/// program's own text is proof that the frame it came from is real; one outside
/// is either a genuine native frame or the garbage above. So frames are HELD
/// until the chain returns into compiled code, which corroborates every frame
/// held behind it at once, and are dropped if it never does. A stack that stops
/// early is a stack that really ran as far as it says; a stack padded with
/// frames nobody can vouch for reads exactly like a real one, which is worse
/// than the missing tail.
///
/// The interrupted PC is not walked and not held: the kernel handed it over, so
/// it is the one frame that needs no corroborating. The caller stores it.
///
/// # Safety
/// Dereferences `fp` and `fp + 8` after checking they lie inside the window
/// above `sp`. `out` is written only within its own length.
unsafe fn walk_frame_chain(mut fp: u64, sp: u64, text: Option<(u64, u64)>, out: &mut [u64]) -> usize {
    // Corroborated frames occupy `out[..proven]`; frames waiting for
    // corroboration sit just above them and are overwritten or forgotten.
    let mut proven = 0usize;
    let mut held = 0usize;
    while proven + held < out.len() {
        // The bound is `- 16` because the two loads below read `[fp, fp+8)` and
        // `[fp+8, fp+16)`. At `- 8` an fp eight bytes below the end of the
        // address space passed, the first load succeeded, and the second crossed
        // the boundary — the guard covered one of the two reads it exists for.
        if fp == 0
            || fp & 0xf != 0
            || fp < sp
            || fp.wrapping_sub(sp) >= STACK_WINDOW
            || fp > u64::MAX - 16
        {
            break;
        }
        let next_fp = *(fp as *const u64);
        let return_address = *((fp + 8) as *const u64);
        if return_address < 0x1000 {
            break;
        }
        out[proven + held] = return_address;
        if returns_into_program(return_address, text) {
            // This frame returned into code this binary compiled, which vouches
            // for it and for every frame held behind it.
            proven += held + 1;
            held = 0;
        } else {
            held += 1;
            if held > UNPROVEN_RUN_MAX {
                break;
            }
        }
        // Frames must strictly grow toward higher addresses or the chain is
        // corrupt (or we crossed into a differently-shaped frame).
        if next_fp <= fp {
            break;
        }
        fp = next_fp;
    }
    // `held` frames are deliberately left out of the count: nothing corroborated
    // them, and reporting them would be inventing a stack.
    proven
}

/// The SIGPROF handler: records the interrupted PC plus the return addresses
/// of the frame-pointer chain into the next ring slot.
///
/// Both supported ABIs store `[fp] = caller fp, [fp + 8] = return address`,
/// which is what makes one walker serve AArch64 and x86_64.
extern "C" fn on_sigprof(
    _signal: libc::c_int,
    _info: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    if STOPPED.load(Ordering::Relaxed) {
        return;
    }
    // Nobody asked, nothing is recorded — even if the signal came from outside.
    //
    // The handler is installed at init, before any ask, and is never removed;
    // arming was the only gate. So `kill -PROF <pid>` against a dormant
    // `--with-monitoring` service made it walk its own stacks and fill the ring
    // it carries, which is exactly what "dormant until asked" promises it will
    // not do. Arming still decides whether the timer fires; this decides whether
    // a delivery that arrived by any other route is honoured.
    if !ASKED.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        let base = REGION.load(Ordering::Relaxed);
        if base == 0 {
            return;
        }
        let head = &*(base as *const std::sync::atomic::AtomicU64);
        let (pc, fp, sp) = interrupted_pc_fp(context);
        let ticket = head.fetch_add(1, Ordering::Relaxed);
        let slot_index = (ticket as usize) % RING_SLOTS;
        // Slot words are atomics: the reader (endpoint/dump) runs concurrently
        // with this handler in another thread, so plain stores would be a data
        // race. depth (word 0) is the readiness gate, published Release last.
        let depth_word = region_word(base, slot_index, 0);
        let route_word = region_word(base, slot_index, 1);
        // Open the slot: an odd sequence says "being written". Readers that see
        // an odd count, or a count that changed across their read, discard what
        // they got instead of folding a stack stitched from two samples.
        //
        // The value is derived from this handler's TICKET, not from whatever the
        // word happened to hold. A plain seqlock assumes one writer per slot, and
        // this ring does not have one: two handlers a full lap apart share a slot,
        // and each computing its closing value from the value it read at open let
        // them converge on the same even number — A opens 9 from 8, B opens 11
        // from 9 and closes 10, A closes 10 — so a reader saw one even value
        // across a read that spanned both, and folded the tear. Tickets come from
        // `head.fetch_add`, so they are unique and increasing: two writers cannot
        // produce the same closing value, and a reader's two reads agreeing means
        // one writer's whole slot.
        //
        // The odd store is Relaxed and the FENCE after it is what orders it
        // before the slot writes. A `Release` store orders accesses that precede
        // it, not ones that follow — so writing the odd value with `Release` and
        // the body with `Relaxed` left the body free to become visible first,
        // which is exactly the tear the sequence exists to advertise. On x86 the
        // hardware forbids that reordering and hides the mistake; on AArch64,
        // which is both the CI target and the development machine, it does not.
        let seq_word = region_word(base, slot_index, SEQ_WORD);
        seq_word.store(slot_seq_writing(ticket), Ordering::Relaxed);
        std::sync::atomic::fence(Ordering::Release);
        // Slot layout: [depth, route_id, pc0, pc1, ...]. Stamp the active request
        // route so the dump can group samples by endpoint.
        route_word.store(CURRENT_ROUTE.load(Ordering::Relaxed) as u64, Ordering::Relaxed);
        // Allocations since the previous sample, charged to the stack this one
        // captures — sampled attribution, exactly like Go's heap profile. The
        // counter only grows, so a wrapped or reset read yields 0 rather than a
        // wild delta.
        let allocs_delta = match current_allocs() {
            Some(now) => {
                let previous = LAST_ALLOCS.swap(now, Ordering::Relaxed);
                allocs_charged(previous, now)
            }
            None => 0,
        };
        region_word(base, slot_index, 2).store(allocs_delta, Ordering::Relaxed);
        // The interrupted PC first: the kernel handed it over, so it is the one
        // frame that needs nothing to vouch for it.
        region_word(base, slot_index, PC_WORD0).store(pc, Ordering::Relaxed);
        // Walked into a stack array rather than straight into the ring, because
        // a frame is only reportable once a LATER frame corroborates it, and a
        // word already published cannot be taken back. 248 bytes on a 64 KiB
        // signal stack, and no allocation.
        let mut frames = [0u64; MAX_FRAMES - 1];
        let walked = walk_frame_chain(fp, sp, program_text_range(), &mut frames);
        let mut depth = 1usize;
        for address in &frames[..walked] {
            region_word(base, slot_index, depth + PC_WORD0).store(*address, Ordering::Relaxed);
            depth += 1;
        }
        // Publish depth last with Release so a reader that loads it Acquire never
        // sees a higher depth than the PCs already stored.
        depth_word.store(depth as u64, Ordering::Release);
        // Close the slot with this ticket's settled value. A reader whose two
        // reads of this word agree, on an even value, saw one writer's slot for
        // the whole of its read — and because the value names the ticket, "the
        // same value" cannot mean "a different writer that happened to land on
        // the same number".
        seq_word.store(slot_seq_settled(ticket), Ordering::Release);
    }
}

/// Installs the SIGPROF handler and arms the profiling timer.
///
/// `table`/`len` describe the compiler-embedded symbol table; the final entry
/// is the text-end sentinel (name `<end>`), which bounds the last function.
///
/// # Safety
/// Called once from generated code before user code runs; `table` must point
/// at `len` valid entries that live for the whole process.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_init(table: *const SymtabEntry, len: usize, key: *const u8) {
    TABLE_PTR.store(table as usize, Ordering::Relaxed);
    TABLE_LEN.store(len, Ordering::Relaxed);
    KEY_PTR.store(key as usize, Ordering::Relaxed);

    // Map the sample region MAP_SHARED before any --web fork: the mapping is
    // inherited by every worker, so all workers' SIGPROF handlers fill one ring
    // through the shared atomic head. Zero-filled by the kernel. If the map
    // fails the probe stays inert (REGION 0) rather than crash the process.
    #[cfg(target_os = "linux")]
    let anon = libc::MAP_ANONYMOUS;
    #[cfg(not(target_os = "linux"))]
    let anon = libc::MAP_ANON;
    let region = libc::mmap(
        std::ptr::null_mut(),
        REGION_BYTES,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_SHARED | anon,
        -1,
        0,
    );
    if region == libc::MAP_FAILED {
        return;
    }
    REGION.store(region as usize, Ordering::Relaxed);

    let mut action: libc::sigaction = std::mem::zeroed();
    // Cast through the fn-pointer type before usize, not the fn item directly.
    action.sa_sigaction =
        on_sigprof as extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void) as usize;
    action.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART;
    // Run the handler on its own stack when we can get one. Without this it runs
    // on the interrupted thread's stack, and the one thing a profiler samples is
    // a program in the middle of its work — including deep in a recursion, with
    // the guard page a few hundred bytes away. The handler is lean, but it walks
    // a frame chain and spills registers, and overflowing there turns a profile
    // into a SIGSEGV in the process being profiled.
    //
    // `SA_ONSTACK` is set only if the stack was actually installed: promising the
    // kernel an alternate stack that does not exist is worse than not asking.
    //
    // `sigaltstack` is PER-THREAD, and this registers it for one thread — the one
    // running main's prologue, which is the thread that runs PHP and therefore
    // the one worth protecting. It is inherited across `fork`, which is what the
    // `--web` workers need. Any other thread in the process gets no alternate
    // stack, and `SA_ONSTACK` is simply inert there: the handler runs on that
    // thread's own stack, exactly as it did everywhere before this existed. So
    // this is a protection for the thread that matters, not a property of the
    // process — and sharing one static stack is safe only for as long as that
    // stays true. The endpoint threads block SIGPROF, which is what keeps the
    // other threads that exist today (the endpoint's, and the PDO bridge's tokio
    // runtime) from reaching it; a future thread that samples would need its own.
    if SIGSTACK_BYTES >= libc::MINSIGSTKSZ as usize {
        let mut alt: libc::stack_t = std::mem::zeroed();
        alt.ss_sp = std::ptr::addr_of_mut!(SIGSTACK) as *mut libc::c_void;
        alt.ss_size = SIGSTACK_BYTES;
        alt.ss_flags = 0;
        if libc::sigaltstack(&alt, std::ptr::null_mut()) == 0 {
            action.sa_flags |= libc::SA_ONSTACK;
        }
    }
    libc::sigemptyset(&mut action.sa_mask);
    if libc::sigaction(libc::SIGPROF, &action, std::ptr::null_mut()) != 0 {
        return;
    }

    // Embedded but dormant. `--with-monitoring` makes a binary CAPABLE of being
    // profiled; it must otherwise run — and cost — like any other binary, or the
    // flag would be a performance decision disguised as a capability.
    //
    // `monitor` spawning us IS the asking, and it asks for the whole run: publish
    // the decision so the exact profiler, whose init runs after this one, does not
    // have to repeat the check — and would consume the control marker if it did.
    //
    // A configured ELEPHC_PROBE_ADDR is deliberately NOT the asking. It says an
    // operator MAY connect later, which is a reachability decision, not a
    // profiling one; treating the two as the same made a service profile itself
    // from boot with nobody on the other end. The listener goes up below; what it
    // collects starts when a client proves the key and says what it wants.
    if control_fd_present() {
        let flag = std::ptr::addr_of_mut!(elephc_monitor_active);
        flag.write(1);
        ASKED.store(true, Ordering::Relaxed);
        arm_timer();
    }

    // fork() RESETS interval timers in the child (POSIX; `man 2 fork`), so a
    // plain fork+exec — every popen/exec/proc_open the profiled PHP does — is
    // already safe: the child starts with ITIMER_PROF disarmed. The atfork
    // child handler below is belt-and-suspenders for that path. The genuine
    // hazard is execve WITHOUT a preceding fork (a self re-exec / graceful
    // restart): the armed timer is PRESERVED across execve while execve resets
    // SIGPROF to its default (terminate), so the new image would die with
    // "Profiling timer expired". A host that re-execs itself must call
    // `elephc_probe_disarm` first. A --web worker, forked but kept running,
    // re-arms through `elephc_probe_rearm`.
    libc::pthread_atfork(None, None, Some(disarm_after_fork));

    // The remote endpoint is opt-in: a Unix socket path in ELEPHC_PROBE_ADDR
    // turns it on. A background thread accepts connections, runs the build-key
    // handshake, and serves the folded profile — so a live production process
    // can be profiled by `elephc monitor --probe-host` without SIGPROF from
    // outside and without suspending the process.
    if !key.is_null() {
        if let Ok(path) = std::env::var("ELEPHC_PROBE_ADDR") {
            if !path.is_empty() {
                endpoint::spawn(path);
            }
        }
    }
}

/// Starts sampled collection, because someone authenticated and asked for it.
///
/// The counterpart to init NOT arming on a configured address: the ring stays
/// empty until a client proves the build key and requests the sampled answer, so
/// a reachable service costs what an unreachable one costs until that happens.
/// Idempotent — every later poll finds collection already running, which is why
/// the first answer on a freshly-started service is thin and the next is not.
pub(crate) fn begin_sampled() {
    // Published BEFORE the local early-return: the endpoint thread runs in the
    // `--web` master, whose own `ASKED` may already be true from an earlier poll
    // while workers forked since have never seen it. Writing the shared word
    // first means a second poll still repairs a worker that missed the first.
    let region = REGION.load(Ordering::Relaxed);
    if region != 0 {
        unsafe { region_asked(region) }.store(1, Ordering::Release);
    }
    // A fresh ask lifts an exec latch: whoever disarmed did so for an exec that
    // either happened — in which case this image is gone and nothing here runs —
    // or did not, and is now being asked again, which is a reason to sample.
    DISARMED_FOR_EXEC.store(false, Ordering::Relaxed);
    ASKED.store(true, Ordering::Relaxed);
    // Keyed on who owns the timer, not on whether this process has ever been
    // asked. Returning early on `ASKED` alone meant a process that had since
    // disarmed — before an exec that then did not happen, say — answered every
    // later ask by doing nothing, because it remembered being asked once and
    // never checked whether it was still sampling. The pid comparison is the
    // same one a forked worker uses, so both paths arm under one rule.
    //
    // A request is still not enough on its own: there has to be a ring to fill.
    // Arming without one delivers SIGPROF to a process with nothing to record it
    // in — and if the handler is not installed either, the default action for
    // that signal is to terminate.
    if !should_adopt_ask(
        ARMED_PID.load(Ordering::Relaxed),
        unsafe { libc::getpid() },
        true,
        REGION.load(Ordering::Relaxed),
    ) {
        return;
    }
    // Safety: arming is one `setitimer`; the handler and the ring were installed
    // by `elephc_probe_init`, which has necessarily run for a client to reach us.
    unsafe { arm_timer() };
}

/// Arms the CPU-time profiling timer at the sampling period. Idempotent and
/// async-signal-safe (one `setitimer` syscall) — safe from a fork child.
unsafe fn arm_timer() {
    let interval = libc::timeval {
        tv_sec: 0,
        tv_usec: PERIOD_MICROS as libc::suseconds_t,
    };
    let timer = libc::itimerval {
        it_interval: interval,
        it_value: interval,
    };
    libc::setitimer(libc::ITIMER_PROF, &timer, std::ptr::null_mut());
    // The single choke point every arming path goes through, so recording the
    // owner here is what keeps `observe_shared_ask` down to one `setitimer` per
    // process instead of one per request.
    ARMED_PID.store(libc::getpid(), Ordering::Relaxed);
}

/// Disarms the profiling timer. Async-signal-safe (one `setitimer` syscall).
unsafe fn disarm_timer() {
    let off = libc::itimerval {
        it_interval: libc::timeval { tv_sec: 0, tv_usec: 0 },
        it_value: libc::timeval { tv_sec: 0, tv_usec: 0 },
    };
    libc::setitimer(libc::ITIMER_PROF, &off, std::ptr::null_mut());
}

/// `pthread_atfork` child hook: disarm the timer in the child (belt-and-
/// suspenders — fork already resets it). A `--web` worker re-arms via
/// `elephc_probe_rearm`.
extern "C" fn disarm_after_fork() {
    unsafe { disarm_timer() };
}

/// Disarms the profiling timer. A host that calls `execve` WITHOUT forking (a
/// self re-exec / graceful restart) must call this first, otherwise the armed
/// timer survives the exec and the default SIGPROF action kills the new image.
///
/// # Safety
/// Ordinary FFI entry; just disarms the interval timer.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_disarm() {
    disarm_timer();
    // Forget which process owns the timer. `begin_sampled` used to return early
    // once `ASKED` was set and `observe_shared_ask` arms only when the owner is
    // another process, so after a disarm both said "already armed" about a timer
    // that no longer existed, and a later ask reached a service that had quietly
    // stopped sampling.
    ARMED_PID.store(0, Ordering::Relaxed);
    // But clearing the owner alone re-opens what this function exists to close.
    // `observe_shared_ask` runs on EVERY request, and the shared word still says
    // the service was asked — so between this call and the `execve` it precedes,
    // one request would re-arm the timer, the exec would carry it into the new
    // image, and the default SIGPROF action would kill it. That is exactly the
    // failure documented above, reached through the fix for a different one.
    //
    // The latch closes both. Arming is refused until someone asks AGAIN, which
    // is what `begin_sampled` records — so an exec is safe, and a process whose
    // exec was called off resumes on the next real ask rather than on the memory
    // of an old one. It is not cleared here because the image that replaces this
    // one gets fresh statics; there is nothing left to clear.
    DISARMED_FOR_EXEC.store(true, Ordering::Relaxed);
}

/// Re-arms the profiling timer in a process that forked but keeps sampling (a
/// `--web` worker). Called by the web bridge at worker startup through the
/// runtime `_elephc_probe_rearm_fn` slot; a no-op if the probe is not active.
///
/// # Safety
/// Ordinary FFI entry; just arms the interval timer.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_rearm() {
    if should_arm(ASKED.load(Ordering::Relaxed), REGION.load(Ordering::Relaxed)) {
        arm_timer();
    }
}

/// Whether a forked worker should start sampling.
///
/// On ASKED, not on the mapping: every monitored binary maps the ring — that is
/// what makes the capability free to carry — so re-arming on the mapping alone
/// made a dormant `--web` service profile and log itself. A pure function
/// because the alternative, a test that mutates the globals, would have to make
/// `REGION` non-zero to reach the interesting branch at all, and would otherwise
/// pass without ever testing what it claims.
fn should_arm(asked: bool, region: usize) -> bool {
    asked && region != 0
}

/// PID of the process whose interval timer this crate armed.
///
/// A pid and not a bool because `fork` copies this memory but RESETS the child's
/// interval timers: an inherited `true` would describe the PARENT's timer and
/// leave the child sampling nothing. Comparing against `getpid()` is correct at
/// any fork depth — master, worker, broker, handler child — without having to
/// hook every fork in the tree.
static ARMED_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Set by `elephc_probe_disarm`: refuse to arm until someone asks again.
///
/// Disarming exists for the moment before an `execve` that does not fork, where
/// an armed timer survives into an image whose SIGPROF disposition is back to
/// the default — it dies with "Profiling timer expired". Clearing the timer is
/// not enough on its own, because the shared word still says the service was
/// asked and `observe_shared_ask` runs on every request: one request between the
/// disarm and the exec re-arms it. Cleared by `begin_sampled`, so a NEW ask
/// resumes sampling and the memory of an old one does not.
static DISARMED_FOR_EXEC: AtomicBool = AtomicBool::new(false);

/// Allocations to charge one sample, given the counter at the previous sample.
///
/// The first sample in a process charges nothing: there is no previous reading
/// to subtract, and treating the absent one as zero made it charge every
/// allocation since the process started. Sampled attribution needs an interval,
/// and the first sample does not have one — it establishes the baseline the
/// second is measured against.
///
/// The counter only grows, so a smaller reading than the last means it wrapped
/// or was reset; charge nothing rather than a wild delta.
fn allocs_charged(previous: u64, now: u64) -> u64 {
    if previous == NO_PREVIOUS_SAMPLE {
        return 0;
    }
    now.saturating_sub(previous)
}

/// Whether a ring slot held still for the whole of a reader's pass over it.
///
/// The sequence word is even when a slot is settled and odd while a handler is
/// writing it, and only writers move it. Two loads agreeing on an even value
/// therefore mean no handler touched the slot in between, and what the reader
/// assembled is one sample rather than two spliced together.
///
/// A pure function for the same reason `should_arm` is one: reaching the
/// interesting branch through the globals means racing a real signal handler
/// against a real reader, which a test cannot schedule.
fn slot_was_settled(before: u64, after: u64) -> bool {
    before & 1 == 0 && before == after
}

/// The sequence value a handler publishes while it is writing slot `ticket`.
///
/// Odd, and derived from the ticket rather than from the word's previous value,
/// so that two handlers a lap apart cannot compute the same settled value from
/// different starting points. Tickets come from `head.fetch_add` and are unique.
fn slot_seq_writing(ticket: u64) -> u64 {
    ticket.wrapping_mul(2) | 1
}

/// The sequence value that says slot `ticket` is settled and readable.
fn slot_seq_settled(ticket: u64) -> u64 {
    ticket.wrapping_mul(2)
}

/// Whether this process should arm now: someone asked, there is a ring to fill,
/// and the armed timer on record belongs to some other process.
///
/// A pure function for the same reason `should_arm` is one: the alternative is a
/// test that mutates the globals and then has to arm a real interval timer to
/// reach the interesting branch, which would deliver SIGPROF into the test
/// process.
fn should_adopt_ask(armed_pid: i32, pid: i32, asked: bool, region: usize) -> bool {
    region != 0 && asked && armed_pid != pid
}

/// Adopts an ask that reached the shared mapping after this process forked.
///
/// A `--web` worker re-arms once, immediately after it is forked, off the
/// `ASKED` copy it inherited. In the normal lifecycle the workers exist before
/// any operator connects, so that copy is false; the authenticated request then
/// runs on the master's endpoint thread and flips only the master, which
/// executes no PHP. Sampling therefore never reached the processes doing the
/// work, and the ring stayed empty.
///
/// Called per request rather than once because "asked" can become true at any
/// point in a worker's life. The cost when nobody asked is one relaxed load of
/// a word already in the shared page.
///
/// # Safety
/// Valid only after `elephc_probe_init` mapped the region; `arm_timer` is one
/// `setitimer`, safe to call from any of the forked processes.
unsafe fn observe_shared_ask() {
    let region = REGION.load(Ordering::Relaxed);
    if region == 0 {
        return;
    }
    // Either source counts. The shared word carries an ask that arrived after
    // this process forked; the local flag carries one it inherited from before,
    // which still needs arming here because the fork reset this process's timer.
    //
    // Unless this process disarmed for an exec: the shared word still says the
    // service was asked, so without this check one request between the disarm
    // and the `execve` would re-arm the timer the disarm exists to remove.
    if DISARMED_FOR_EXEC.load(Ordering::Relaxed) {
        return;
    }
    let asked = ASKED.load(Ordering::Relaxed) || region_asked(region).load(Ordering::Acquire) != 0;
    if !should_adopt_ask(ARMED_PID.load(Ordering::Relaxed), libc::getpid(), asked, region) {
        return;
    }
    ASKED.store(true, Ordering::Relaxed);
    arm_timer();
}

/// Returns the embedded build key, or `None` if unpublished.
fn build_key() -> Option<[u8; handshake::KEY_LEN]> {
    let ptr = KEY_PTR.load(Ordering::Relaxed) as *const u8;
    if ptr.is_null() {
        return None;
    }
    let mut key = [0u8; handshake::KEY_LEN];
    // Safety: the compiler embeds exactly KEY_LEN bytes at this symbol.
    unsafe { std::ptr::copy_nonoverlapping(ptr, key.as_mut_ptr(), handshake::KEY_LEN) };
    Some(key)
}

/// File descriptor `monitor` hands the child, one end of a socketpair it made
/// before forking.
const CONTROL_FD: i32 = 3;
/// What `monitor` writes into that socket before spawning, so the data is already
/// buffered when the child looks. A stray inherited socket on the same descriptor
/// says nothing and is ignored.
const CONTROL_MAGIC: &[u8] = b"ELEPHC-MONITOR-1";

/// Whether this process was started by `elephc monitor`.
///
/// The credential is the CHANNEL, not a token: only the parent that forked this
/// process holds the other end of that socketpair. Nothing to copy out of a
/// process list, nothing left in a log, nothing to replay — which is what a
/// signed environment variable, however well signed, cannot offer, because it is
/// visible to everything on the machine.
///
/// Reads without blocking and without consuming more than the marker, so a
/// descriptor that happens to be open says no rather than hanging the program.
fn control_fd_present() -> bool {
    unsafe {
        // Must be a socket: an inherited file or pipe on the same number is not
        // a control channel, and treating it as one would start profiling a
        // program nobody asked about.
        let mut kind: libc::c_int = 0;
        let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        let ok = libc::getsockopt(
            CONTROL_FD,
            libc::SOL_SOCKET,
            libc::SO_TYPE,
            &mut kind as *mut _ as *mut libc::c_void,
            &mut len,
        );
        if ok != 0 || kind != libc::SOCK_STREAM {
            return false;
        }
        // PEEK, then take only what is ours.
        //
        // Reading first and asking afterwards destroys data belonging to a
        // program that never asked to be profiled: fd 3 is an ordinary number, a
        // supervisor may hand a child a connected socket on it, and consuming 16
        // bytes of someone else's protocol is silent and unrecoverable. Measured
        // before this changed: a 35-byte payload came back 19 bytes long.
        let mut buf = [0u8; 16];
        let read = libc::recv(
            CONTROL_FD,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len(),
            libc::MSG_PEEK | libc::MSG_DONTWAIT,
        );
        if read != CONTROL_MAGIC.len() as isize || buf != CONTROL_MAGIC {
            return false;
        }
        // It is ours: consume the marker so nothing downstream reads it back.
        libc::recv(
            CONTROL_FD,
            buf.as_mut_ptr() as *mut libc::c_void,
            CONTROL_MAGIC.len(),
            libc::MSG_DONTWAIT,
        );
        true
    }
}

/// How far a signed profiling request may be from the server's clock, in
/// seconds. Wide enough for real clock skew between two hosts, narrow enough that
/// a captured header stops working long before anyone finds it in a log.
///
/// The window alone was never the whole answer: inside it a captured header
/// worked any number of times, from anywhere. `spend_query_tag` is what makes it
/// work at most ONCE, so the window now only has to cover clock skew rather than
/// stand in for a replay defence.
const QUERY_WINDOW_SECS: i64 = 300;

/// Spends a verified signature, refusing one that has already been spent.
///
/// Returns true the first time a tag is seen and false for every repeat inside
/// the window — so a header lifted from a log or a proxy trace is worth nothing,
/// and one lifted before the legitimate request lands is worth exactly one
/// request rather than five minutes of them.
///
/// The table lives in the SHARED mapping. A per-process memory would have made
/// this useless under `--web`, where the replay is served by whichever worker
/// the kernel picks and the one that saw the original may never see it again.
///
/// Open-addressed from the tag itself so two processes racing the SAME tag meet
/// on the same slot and one loses the compare-exchange; probing forward from
/// there keeps two DIFFERENT tags that start at one slot from evicting each
/// other. Slots older than the window are free: nothing outside it is accepted
/// anyway, so remembering it protects nothing.
///
/// Refuses when it cannot remember — an unmapped region, or a table whose every
/// slot is live. Both mean the promise "at most once" cannot be kept, and the
/// honest answer to a privileged request nobody can account for is no.
///
/// # Safety
/// Valid only for a mapped region; `base` of 0 is handled as "cannot remember".
unsafe fn spend_query_tag(base: usize, tag: u64, now: i64) -> bool {
    if base == 0 {
        return false;
    }
    // 0 marks a free slot, so a tag that folds to it takes the next value. One
    // signature in 2^64 is thereby indistinguishable from its neighbour, which
    // costs that request a retry and nothing else.
    let tag = if tag == 0 { 1 } else { tag };
    let start = (tag % REPLAY_SLOTS as u64) as usize;
    // Two passes, and the split is the whole correctness argument. Taking the
    // first reusable slot DURING the search let a replay be accepted: a tag whose
    // home slot was busy at first use sits further along the probe, and when the
    // slot that displaced it later expires, the replay stops there, finds it
    // free, and takes it — without ever reaching the entry that says it was
    // already spent. The search has to finish before anything is taken.
    for _attempt in 0..SPEND_ATTEMPTS {
        let mut reusable: Option<usize> = None;
        for step in 0..REPLAY_SLOTS {
            let index = (start + step) % REPLAY_SLOTS;
            let (tag_word, taken_at) = replay_slot(base, index);
            let seen = tag_word.load(Ordering::Acquire);
            if seen == tag {
                // Spent, whatever the slot's age says. Deliberately NOT gated on
                // the entry still being live: a matching tag is a matching
                // TIMESTAMP, and a timestamp old enough for this entry to have
                // expired was already refused by the window check upstream. So
                // this can only fire on a genuine replay — and not reading the
                // time here is what closes the window between a winner's
                // compare-exchange and the store of its timestamp, in which a
                // concurrent racer saw its own tag with a stale time, called the
                // slot reusable, and took it from the request that had just won.
                return false;
            }
            if reusable.is_none()
                && (seen == 0 || !within_query_window(now, taken_at.load(Ordering::Acquire) as i64))
            {
                reusable = Some(index);
            }
        }
        // Every slot holds a signature still inside the window. Sixty-four of
        // those need sixty-four VERIFIED asks in five minutes, which only the key
        // holder can produce — so this is a burst, not an attack, and it is
        // refused rather than admitted: a signature nobody can account for cannot
        // be promised to be used once.
        let Some(index) = reusable else {
            return false;
        };
        let (tag_word, taken_at) = replay_slot(base, index);
        let seen = tag_word.load(Ordering::Acquire);
        if tag_word
            .compare_exchange(seen, tag, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            taken_at.store(now as u64, Ordering::Release);
            return true;
        }
        // Somebody took this slot between the search and the exchange. Search
        // again rather than moving to the next one: what they wrote may be THIS
        // tag, and only a fresh pass can see that.
    }
    false
}

/// Verifies an `X-Elephc-Query` value against the embedded build key.
///
/// Format: `t=<unix seconds>,v=<hex hmac of the timestamp>`. Turning profiling on
/// is a privileged act — it costs the request real time and exposes the shape of
/// the code — so asking has to be something only a holder of the build key can
/// do. An unsigned trigger means anyone who can set a header can profile your
/// production, which is the hole a bare on/off flag leaves open.
///
/// The timestamp is what stops a captured header being replayed forever; the
/// comparison is constant-time, so a wrong tag leaks nothing about the right one.
///
/// Returns 1 when the value is authentic and current, 0 otherwise. Reached
/// through a slot, so the web bridge needs no knowledge of this crate.
///
/// # Safety
/// `ptr`/`len` must describe bytes valid for the duration of the call, or be
/// `(null, 0)`. It builds a slice from them, so a wrong length reads out of
/// bounds — which is why this is `unsafe`, like every other pointer-taking
/// entry point in this crate. It was the one that was not, so a caller could
/// produce that read through a safe API and nothing said so.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_verify_query(ptr: *const u8, len: usize) -> u32 {
    if ptr.is_null() || len == 0 {
        return 0;
    }
    let Some(key) = build_key() else {
        return 0;
    };
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(value) = std::str::from_utf8(bytes) else {
        return 0;
    };
    let mut stamp: Option<i64> = None;
    let mut tag: Option<Vec<u8>> = None;
    for field in value.split(',') {
        let field = field.trim();
        if let Some(raw) = field.strip_prefix("t=") {
            stamp = raw.parse::<i64>().ok();
        } else if let Some(raw) = field.strip_prefix("v=") {
            tag = decode_hex(raw);
        }
    }
    let (Some(stamp), Some(tag)) = (stamp, tag) else {
        return 0;
    };

    let mut now = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut now) };
    if !within_query_window(now.tv_sec as i64, stamp) {
        return 0;
    }
    let expected = handshake::hmac_sha256(&key, stamp.to_string().as_bytes());
    if !handshake::tags_equal(&expected, &tag) {
        return 0;
    }
    // Spent only once the signature has PROVED itself. Checking the table first
    // would let anyone who can set a header fill all sixty-four slots with junk
    // and lock the key holder out of their own profiler — the table would then
    // be a denial of service built out of a replay defence.
    //
    // Eight bytes of a verified HMAC identify it: forging a collision needs the
    // key, and without the key neither half of the tag can be chosen at all.
    let mut identity = [0u8; 8];
    identity.copy_from_slice(&expected[..8]);
    u32::from(unsafe {
        spend_query_tag(
            REGION.load(Ordering::Relaxed),
            u64::from_le_bytes(identity),
            now.tv_sec as i64,
        )
    })
}

/// Whether a signed timestamp is close enough to now to be accepted.
///
/// Saturating on purpose. `stamp` is parsed straight out of an HTTP header, so a
/// client picks it: with plain `now - stamp` a value near `i64::MIN` overflows —
/// which panics outright in debug, and in release wraps to `i64::MIN`, where
/// `.abs()` panics unconditionally. Either way one crafted header aborts the
/// process, and the panic crosses an `extern "C"` boundary on its way out. The
/// header is accepted from untrusted clients by design, so the arithmetic that
/// reads it has to be total.
///
/// Extracted rather than left inline because inline it was untestable: the
/// function around it returns early when no build key is embedded, which every
/// test build is, so a test could never reach the expression.
fn within_query_window(now: i64, stamp: i64) -> bool {
    now.saturating_sub(stamp).saturating_abs() <= QUERY_WINDOW_SECS
}

/// Lowercase hex to bytes; `None` on anything malformed, so a truncated tag is
/// rejected rather than compared against a prefix.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || text.is_empty() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let pair = std::str::from_utf8(&bytes[i..i + 2]).ok()?;
        out.push(u8::from_str_radix(pair, 16).ok()?);
        i += 2;
    }
    Some(out)
}

/// Renders the current folded profile for the endpoint responder.
pub fn current_folded_profile() -> Option<String> {
    unsafe { folded_profile() }
}

/// How long the first sampled answer waits for collection to produce something.
///
/// Asking is what starts sampling, so the very first request necessarily arrives
/// before any sample exists: the timer's first interval had not elapsed yet, and
/// in `--web` the workers only adopt the ask on their next request. Answering
/// immediately made that first command a deterministic sacrifice — it reported
/// "no samples yet — is the process busy?" about a service that was busy, and
/// the identical command a second later returned a full profile.
///
/// Bounded, and short enough that a genuinely idle service still answers
/// promptly rather than appearing to hang.
const SAMPLED_WARMUP: std::time::Duration = std::time::Duration::from_millis(400);
/// Gap between checks inside that window; several sampling periods, so a busy
/// process is noticed almost immediately without spinning.
const SAMPLED_POLL: std::time::Duration = std::time::Duration::from_millis(10);

/// The sampled answer, giving freshly-started collection a bounded chance to
/// produce its first samples.
///
/// Returns as soon as there is anything to report, so a service that has been
/// sampling for a while pays nothing for this.
pub fn sampled_answer() -> String {
    let deadline = std::time::Instant::now() + SAMPLED_WARMUP;
    loop {
        // One relaxed load before deciding to fold. The common case inside this
        // window is an empty ring, and folding an empty ring is not free: it
        // rebuilds the sorted symbol table and reads every one of the 8192 slots
        // to prove nothing is there. Polling every 10ms for 400ms did that up to
        // forty times, on the endpoint thread of a live service, to answer "not
        // yet". The head counter only grows, so a zero means no sample has ever
        // been written and there is provably nothing to fold.
        let taken = unsafe { region_head() }
            .map(|head| head.load(Ordering::Relaxed))
            .unwrap_or(0);
        if taken > 0 {
            let profile = current_folded_profile().unwrap_or_default();
            if !profile.is_empty() {
                return profile;
            }
        }
        if std::time::Instant::now() >= deadline {
            return current_folded_profile().unwrap_or_default();
        }
        std::thread::sleep(SAMPLED_POLL);
    }
}

/// Disarms the timer and writes the folded profile to stderr: one
/// `elephc-probe: root;...;leaf <count>` line per distinct stack — Brendan
/// Gregg's folded format, flamegraph- and diff-friendly.
///
/// # Safety
/// Called from generated code after user code finished; no handler runs
/// concurrently once the stop flag is set and the timer is disarmed.
#[no_mangle]
pub unsafe extern "C" fn elephc_probe_dump() {
    // A binary nobody asked prints nothing, whatever calls this. The emitted
    // epilogues cannot express "was it asked" — that is a run-time fact — so the
    // check belongs here, once, rather than at each site that might grow one.
    if !ASKED.load(Ordering::Relaxed) {
        return;
    }
    STOPPED.store(true, Ordering::Relaxed);
    // Give up ownership of the timer as `elephc_probe_disarm` does. This runs at
    // main's epilogue, so today the process is leaving and nothing will ask
    // again — but the crate would otherwise hold two "stop" paths that disagree
    // about who owns the timer, and the one that forgets is the one whose call
    // site is decided by codegen. Moving this call anywhere but an exit would
    // silently leave the process unable to arm again, which is a bug that would
    // read as "sampling just stopped".
    ARMED_PID.store(0, Ordering::Relaxed);
    let disarm = libc::itimerval {
        it_interval: libc::timeval { tv_sec: 0, tv_usec: 0 },
        it_value: libc::timeval { tv_sec: 0, tv_usec: 0 },
    };
    libc::setitimer(libc::ITIMER_PROF, &disarm, std::ptr::null_mut());

    if let Some(text) = folded_profile() {
        eprint!("{text}");
    }
}

/// Renders the current ring contents as folded-stack text: one
/// `elephc-probe: root;...;leaf <count>` line per distinct stack, followed by
/// `elephc-probe-samples: <total>`. Returns `None` if the symbol table is
/// unpublished. Shared by the exit dump and the endpoint responder.
///
/// # Safety
/// Reads the ring; call after the timer is disarmed (exit dump) or accept that
/// a concurrent handler may add a sample mid-read (endpoint) — a raced slot
/// only skews one count, never corrupts memory.
unsafe fn folded_profile() -> Option<String> {
    let table = TABLE_PTR.load(Ordering::Relaxed) as *const SymtabEntry;
    let table_len = TABLE_LEN.load(Ordering::Relaxed);
    if table.is_null() || table_len == 0 {
        return None;
    }
    let entries = std::slice::from_raw_parts(table, table_len);
    let mut symbols: Vec<(u64, &str)> = entries
        .iter()
        .map(|entry| {
            let name = std::str::from_utf8(std::slice::from_raw_parts(
                entry.name_ptr as *const u8,
                entry.name_len as usize,
            ))
            .unwrap_or("<bad-name>");
            (entry.address, name)
        })
        .collect();
    symbols.sort_by_key(|(address, _)| *address);

    let head = region_head()?;
    let base = REGION.load(Ordering::Relaxed);
    let taken = head.load(Ordering::Relaxed) as usize;
    let available = taken.min(RING_SLOTS);
    let mut valid = 0u64;
    let mut folded: std::collections::BTreeMap<Vec<String>, u64> = std::collections::BTreeMap::new();
    // Allocations charged to each stack, kept apart from the sample counts: they
    // are a different quantity, and summing them into one weight would produce a
    // profile whose bars mean two things at once.
    let mut allocated: std::collections::BTreeMap<Vec<String>, u64> =
        std::collections::BTreeMap::new();
    for index in 0..available {
        // Acquire the depth gate before reading the PCs the handler stored; a
        // torn or in-flight slot with depth 0 is skipped.
        // Read the sequence first: odd means a handler is inside this slot right
        // now, so there is nothing settled to read.
        let seq_word = region_word(base, index, SEQ_WORD);
        let seq_before = seq_word.load(Ordering::Acquire);
        if seq_before & 1 == 1 {
            continue;
        }
        let depth = (region_word(base, index, 0).load(Ordering::Acquire) as usize).min(MAX_FRAMES);
        if depth == 0 {
            continue;
        }
        // Recorded leaf-first; fold root-first like every consumer expects. The
        // leaf (frame 0) is the interrupted PC; frames 1.. are RETURN addresses,
        // so bias them by -1 to land inside the calling instruction rather than
        // the start of whatever follows the call.
        let mut stack: Vec<String> = (0..depth)
            .map(|frame| {
                let raw = region_word(base, index, frame + PC_WORD0).load(Ordering::Relaxed);
                let pc = if frame == 0 { raw } else { raw.wrapping_sub(1) };
                symbolize(&symbols, pc).to_string()
            })
            .collect();
        stack.reverse();
        stack.dedup();
        // Prefix the request route as the outermost frame, so every consumer —
        // the table, the flamegraph, pprof — groups samples by endpoint for
        // free. Samples outside a request keep their plain stack.
        let route_id = region_word(base, index, 1).load(Ordering::Relaxed) as usize;
        if let Some(route) = route_name(route_id) {
            stack.insert(0, route);
        }
        let allocs = region_word(base, index, 2).load(Ordering::Relaxed);
        // Everything above came out of the slot; only now is it safe to say so.
        // If the sequence moved while we read, a handler overwrote this slot
        // underneath us and what we assembled is a stack that never ran — drop
        // it rather than fold it.
        //
        // The FENCE is what holds the slot reads above this check. An `Acquire`
        // load orders accesses that FOLLOW it, not ones that precede — so
        // re-reading the sequence with `Acquire` and the body with `Relaxed`
        // left the body free to be satisfied after the check, and a slot could
        // be declared settled around a read that overlapped a write. The pairing
        // is the mirror of the writer's: fence-then-check here, store-then-fence
        // there.
        std::sync::atomic::fence(Ordering::Acquire);
        if !slot_was_settled(seq_before, seq_word.load(Ordering::Relaxed)) {
            continue;
        }
        valid += 1;
        if allocs > 0 {
            *allocated.entry(stack.clone()).or_default() += allocs;
        }
        *folded.entry(stack).or_default() += 1;
    }
    // A dormant binary took no samples and recorded no events. Saying
    // "elephc-probe-samples: 0" would still announce a profiler to anyone
    // reading the program's own stderr, which is exactly what
    // `--with-monitoring` promises not to do until asked.
    if folded.is_empty() && allocated.is_empty() && event_report(base).is_empty() {
        return Some(String::new());
    }
    let mut out = String::new();
    for (stack, count) in &folded {
        out.push_str("elephc-probe: ");
        out.push_str(&stack.join(";"));
        out.push(' ');
        out.push_str(&count.to_string());
        out.push('\n');
    }
    // Report the recorded (valid) sample count, not the raw interrupt count:
    // interrupts that produced no walkable stack are not in the folded lines.
    out.push_str(&format!("elephc-probe-samples: {valid}\n"));
    // Allocation weights, under their own prefix. Sampled attribution — the
    // delta since the previous sample is charged to the stack that sample caught,
    // exactly as Go's heap profile does — so this says WHERE allocation happens,
    // not how much precisely. `--instrument` is the mode that counts it exactly.
    for (stack, allocs) in &allocated {
        out.push_str("elephc-probe-alloc: ");
        out.push_str(&stack.join(";"));
        out.push(' ');
        out.push_str(&allocs.to_string());
        out.push('\n');
    }
    // Event counters last, and under their own prefix: these are exact, and a
    // reader must not be able to mistake one for a sampled weight.
    out.push_str(&event_report(REGION.load(Ordering::Relaxed)));
    Some(out)
}

/// Maps a program counter to the function whose `[start, next start)` range
/// holds it. The table's final sentinel bounds the last real function, so
/// runtime helpers and libc land on `<native>`.
fn symbolize<'a>(symbols: &[(u64, &'a str)], pc: u64) -> &'a str {
    let index = match symbols.binary_search_by(|(address, _)| address.cmp(&pc)) {
        Ok(index) => index,
        Err(0) => return "<native>",
        Err(insertion) => insertion - 1,
    };
    if index + 1 == symbols.len() {
        // At or past the text-end sentinel: runtime helpers, libc.
        return "<native>";
    }
    symbols[index].1
}

#[cfg(test)]
mod tests {
    /// Configuring an endpoint is reachability, not a request to profile.
    ///
    /// The init used to arm whenever ELEPHC_PROBE_ADDR was set, which made a
    /// service sample itself and — through the word the exact profiler boots on —
    /// write exact profiles to its own log with nobody connected. Measured on a
    /// factorial binary: 431 bytes of stderr with the address set and no client,
    /// against none from the same program without it.
    #[test]
    fn a_configured_address_is_not_a_request_to_profile() {
        let source = include_str!("lib.rs");
        let at = source
            .find("if control_fd_present()")
            .expect("the init still gates arming on the control channel");
        let branch = &source[at..at + 200];
        assert!(
            !branch.contains("ELEPHC_PROBE_ADDR"),
            "arming must not depend on an address being configured:\n{branch}"
        );
        assert!(
            branch.contains("arm_timer()"),
            "and a run `monitor` spawned must still arm:\n{branch}"
        );
    }

    /// Asking is what starts collection, and asking twice changes nothing.
    #[test]
    fn sampled_collection_starts_when_a_client_asks_for_it() {
        // Serialized, and with the region forced empty for the duration.
        //
        // This calls the REAL `begin_sampled`, whose comment used to say "no ring
        // is mapped in a test binary, so this stops short of the syscall". That
        // holds for this test alone and not for the suite: `REGION` is a static
        // that the route and replay tests swap to a heap buffer while they run,
        // and with one of those in flight this reached `arm_timer` for real — in
        // a binary where no SIGPROF handler is installed, so the default action
        // terminated the process. It killed roughly one run in seven, printing no
        // failing test at all because the process simply died. Taking the lock
        // also stops `ASKED` from leaking into the tests that assert a build
        // starts unasked.
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved_region = REGION.swap(0, super::Ordering::Relaxed);
        let restore = super::ASKED.load(super::Ordering::Relaxed);
        super::ASKED.store(false, super::Ordering::Relaxed);
        assert!(
            !super::should_arm(super::ASKED.load(super::Ordering::Relaxed), 0x1000),
            "a reachable service fills nothing until someone asks",
        );
        // The DECISION is what an authenticated client changes; `should_arm` is
        // asked about a mapped ring with a literal, so the decision is observable
        // without this process owning one.
        super::begin_sampled();
        assert!(
            super::should_arm(super::ASKED.load(super::Ordering::Relaxed), 0x1000),
            "and starts filling once one has",
        );
        super::begin_sampled();
        assert!(
            super::ASKED.load(super::Ordering::Relaxed),
            "a second poll finds collection already running",
        );
        super::ASKED.store(restore, super::Ordering::Relaxed);
        REGION.store(saved_region, super::Ordering::Relaxed);
    }

    /// A worker only starts sampling because someone asked, never because the
    /// ring happens to be mapped.
    #[test]
    fn a_forked_worker_arms_only_when_asked() {
        assert!(super::should_arm(true, 0x1000), "asked, ring mapped: sample");
        assert!(!super::should_arm(false, 0x1000), "nobody asked: stay dormant");
        assert!(!super::should_arm(true, 0), "asked but no ring: nothing to fill");
        assert!(!super::should_arm(false, 0));
    }

    /// A binary nobody asked prints nothing, whatever calls the dump.
    ///
    /// Observable without a ring or a key: the dump's first act is to mark the
    /// process stopped, so a dump that returns early leaves that flag alone.
    #[test]
    fn a_dump_on_a_binary_nobody_asked_does_nothing() {
        // Reads `ASKED`, which other tests set and restore, so it has to hold the
        // same lock they do — otherwise this fails whenever it lands between an
        // ask and its restore, and the failure describes a defect that is not
        // there.
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            !super::ASKED.load(super::Ordering::Relaxed),
            "a test build must start unasked, or this proves nothing"
        );
        super::STOPPED.store(false, super::Ordering::Relaxed);
        unsafe { super::elephc_probe_dump() };
        assert!(
            !super::STOPPED.load(super::Ordering::Relaxed),
            "the dump ran on a binary nobody asked"
        );
    }

    /// The capability check must not consume a stream that is not its own.
    ///
    /// fd 3 is just a number: a supervisor can hand a child a connected socket
    /// there, and this check runs on every start of every monitored binary. It
    /// used to `recv` 16 bytes and *then* compare — measured, a 35-byte payload
    /// came back 19 bytes long, silently, with nothing in the program able to
    /// notice. Both ends stay open here, because closing one discards whatever
    /// is queued and would hide the very thing being measured.
    #[test]
    fn the_capability_check_leaves_a_foreign_stream_intact() {
        // Every test that reaches `control_fd_present` installs its socket on
        // the same descriptor — fd 3 is the channel's whole address — so they
        // must not run at once. Two of these three did not take the lock, and
        // the suite failed intermittently with one test reading the bytes
        // another had queued: "the check consumed 35 byte(s) of someone else's
        // stream" is this race, not a defect in the check.
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        const PAYLOAD: &[u8] = b"HELLO-FROM-SUPERVISOR-PROTOCOL-DATA";
        unsafe {
            let mut fds = [0i32; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let (ours, theirs) = (fds[0], fds[1]);
            assert_eq!(
                libc::send(ours, PAYLOAD.as_ptr() as *const libc::c_void, PAYLOAD.len(), 0),
                PAYLOAD.len() as isize
            );
            let saved = libc::dup(super::CONTROL_FD);
            libc::dup2(theirs, super::CONTROL_FD);

            let verdict = super::control_fd_present();

            let mut buf = [0u8; 256];
            let left = libc::recv(
                ours,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                libc::MSG_DONTWAIT,
            );
            let left = if left < 0 { 0 } else { left as usize };

            if saved >= 0 {
                libc::dup2(saved, super::CONTROL_FD);
                libc::close(saved);
            } else {
                libc::close(super::CONTROL_FD);
            }
            libc::close(ours);
            libc::close(theirs);

            assert!(!verdict, "non-magic data must not read as a control channel");
            assert_eq!(
                left,
                PAYLOAD.len(),
                "the check consumed {} byte(s) of someone else's stream",
                PAYLOAD.len() - left
            );
        }
    }

    /// ...and it must still recognise the real thing, and consume its marker.
    #[test]
    fn the_capability_check_still_recognises_its_own_channel() {
        // Same fd 3, same lock — see the sibling test above.
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            let mut fds = [0i32; 2];
            assert_eq!(
                libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0
            );
            let (ours, theirs) = (fds[0], fds[1]);
            let magic = super::CONTROL_MAGIC;
            let trailing = b"AFTER";
            libc::send(ours, magic.as_ptr() as *const libc::c_void, magic.len(), 0);
            libc::send(ours, trailing.as_ptr() as *const libc::c_void, trailing.len(), 0);
            let saved = libc::dup(super::CONTROL_FD);
            libc::dup2(theirs, super::CONTROL_FD);

            let verdict = super::control_fd_present();

            // The marker is gone; whatever followed it is not.
            let mut buf = [0u8; 64];
            let left = libc::recv(
                super::CONTROL_FD,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                libc::MSG_DONTWAIT,
            );
            let left = if left < 0 { 0 } else { left as usize };

            if saved >= 0 {
                libc::dup2(saved, super::CONTROL_FD);
                libc::close(saved);
            } else {
                libc::close(super::CONTROL_FD);
            }
            libc::close(ours);
            libc::close(theirs);

            assert!(verdict, "the real magic must be recognised");
            assert_eq!(&buf[..left], trailing, "the marker must be consumed, and only it");
        }
    }

    /// A client-supplied timestamp must never be able to abort the process.
    ///
    /// `i64::MIN + now` is the value that makes `now - stamp` overflow: it
    /// panicked in debug at the subtraction and in release at `.abs()`, so a
    /// single `X-Elephc-Query` header took down a `--web` service. Both extremes
    /// are checked, plus the ordinary cases, so the window itself stays correct
    /// while being total.
    #[test]
    fn a_crafted_timestamp_cannot_abort_the_window_check() {
        let now: i64 = 1_800_000_000;
        for stamp in [i64::MIN, i64::MAX, i64::MIN.wrapping_add(now), i64::MIN + 1] {
            assert!(
                !super::within_query_window(now, stamp),
                "a forged stamp must fall outside the window, not panic"
            );
        }
        // And the window still means what it says.
        assert!(super::within_query_window(now, now));
        assert!(super::within_query_window(now, now - super::QUERY_WINDOW_SECS));
        assert!(super::within_query_window(now, now + super::QUERY_WINDOW_SECS));
        assert!(!super::within_query_window(now, now - super::QUERY_WINDOW_SECS - 1));
        assert!(!super::within_query_window(now, now + super::QUERY_WINDOW_SECS + 1));
    }

    use super::*;

    /// The route tests mutate the process-global `REGION`/`CURRENT_ROUTE`, so
    /// they must not run concurrently. A poisoned lock is recovered, not
    /// propagated, so one test's panic does not cascade.
    static ROUTE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    /// An address resolves to the symbol whose range contains it, and one past
    /// the last function resolves to `<native>` rather than to that function.
    fn symbolize_uses_start_ranges_and_the_end_sentinel() {
        let symbols = vec![
            (0x1000, "main"),
            (0x2000, "hot_leaf"),
            (0x3000, "<end>"),
        ];
        assert_eq!(symbolize(&symbols, 0x0500), "<native>");
        assert_eq!(symbolize(&symbols, 0x1000), "main");
        assert_eq!(symbolize(&symbols, 0x1fff), "main");
        assert_eq!(symbolize(&symbols, 0x2000), "hot_leaf");
        assert_eq!(symbolize(&symbols, 0x2fff), "hot_leaf");
        // At or past the sentinel start: runtime/libc territory.
        assert_eq!(symbolize(&symbols, 0x3000), "<native>");
        assert_eq!(symbolize(&symbols, 0x3001), "<native>");
    }

    /// Lays a frame-pointer chain into `stack` and returns `(sp, first fp)`.
    ///
    /// Each frame is the sixteen bytes both supported ABIs use —
    /// `[fp] = caller fp`, `[fp + 8] = return address` — so the walk under test
    /// reads a real chain rather than a mock of one. The frames are placed at
    /// rising 16-byte-aligned addresses inside the caller's buffer, which is
    /// what makes them pass the walk's shape checks.
    ///
    /// # Safety
    /// `stack` must hold `2 * returns.len() + 4` words, which gives every frame
    /// room after alignment.
    unsafe fn lay_frame_chain(stack: &mut [u64], returns: &[u64]) -> (u64, u64) {
        let base = stack.as_mut_ptr() as u64;
        let first = (base + 15) & !15;
        for (index, address) in returns.iter().enumerate() {
            let fp = first + (index as u64) * 16;
            let next = if index + 1 < returns.len() { fp + 16 } else { 0 };
            *(fp as *mut u64) = next;
            *((fp + 8) as *mut u64) = *address;
        }
        (base, first)
    }

    /// A frame is reported only once a later frame vouches for it.
    ///
    /// The walk's shape checks prove an address COULD be a stack slot, never
    /// that it is a frame — a function using the frame register as a general one
    /// leaves a value that is aligned, inside the window, and followed. What
    /// separates a real frame from that is returning into code this binary
    /// compiled, so frames outside the program's text are held until the chain
    /// comes back, and dropped when it does not. The tail that goes missing was
    /// never a stack; the tail that used to be reported read exactly like one.
    #[test]
    fn frames_nothing_vouches_for_are_not_reported() {
        let text = Some((0x10_000u64, 0x20_000u64));
        let php = |n: u64| 0x10_000 + n * 0x100;
        let native = |n: u64| 0x7f_0000_0000u64 + n * 0x100;

        // A native run BETWEEN two compiled frames is real: the chain comes back,
        // so the helpers it went through are vouched for and kept.
        let chain = [php(1), native(1), native(2), php(2)];
        let mut stack = vec![0u64; chain.len() * 2 + 4];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let mut out = [0u64; MAX_FRAMES - 1];
        let walked = unsafe { super::walk_frame_chain(fp, sp, text, &mut out) };
        assert_eq!(&out[..walked], &chain, "a chain that returns is kept whole");

        // The same run with nothing after it is the garbage case, and the whole
        // uncorroborated tail goes rather than being reported as a stack.
        let chain = [php(1), native(1), native(2)];
        let mut stack = vec![0u64; chain.len() * 2 + 4];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let walked = unsafe { super::walk_frame_chain(fp, sp, text, &mut out) };
        assert_eq!(&out[..walked], &[php(1)], "an unvouched tail is not a stack");

        // A chain that never reaches compiled code reports nothing at all: every
        // frame in it is exactly as likely to be a leftover stack word.
        let chain = [native(1), native(2)];
        let mut stack = vec![0u64; chain.len() * 2 + 4];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let walked = unsafe { super::walk_frame_chain(fp, sp, text, &mut out) };
        assert_eq!(walked, 0);

        // And it is abandoned rather than followed to the depth cap: past the
        // run bound the walk stops, so a compiled frame further down does not
        // rescue thirty frames of garbage.
        let mut chain = vec![native(0); super::UNPROVEN_RUN_MAX + 1];
        chain.push(php(9));
        let mut stack = vec![0u64; chain.len() * 2 + 4];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let walked = unsafe { super::walk_frame_chain(fp, sp, text, &mut out) };
        assert_eq!(walked, 0, "the run bound is what stops a bad chain");
    }

    /// With no symbol table there is nothing to corroborate against, and the
    /// walk says so by trusting the chain — the behaviour it had everywhere
    /// before the table was consulted. Stated as a test because the alternative,
    /// silently reporting no frames at all, looks identical to an idle process.
    #[test]
    fn without_a_symbol_table_the_walk_trusts_the_chain() {
        let chain = [0x7f_0000_0100u64, 0x7f_0000_0200, 0x7f_0000_0300];
        let mut stack = vec![0u64; chain.len() * 2 + 4];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let mut out = [0u64; MAX_FRAMES - 1];
        let walked = unsafe { super::walk_frame_chain(fp, sp, None, &mut out) };
        assert_eq!(&out[..walked], &chain);
    }

    /// The shape checks still reject what cannot be a frame at all, and they run
    /// BEFORE the dereference, which is what keeps a stale register from
    /// faulting the handler.
    #[test]
    fn the_walk_refuses_a_frame_pointer_that_cannot_be_one() {
        let chain = [0x10_100u64];
        let mut stack = vec![0u64; 8];
        let (sp, fp) = unsafe { lay_frame_chain(&mut stack, &chain) };
        let text = Some((0x10_000u64, 0x20_000u64));
        let mut out = [0u64; MAX_FRAMES - 1];
        for bad in [0, fp + 8, sp.saturating_sub(16), sp + super::STACK_WINDOW, u64::MAX - 8] {
            let walked = unsafe { super::walk_frame_chain(bad, sp, text, &mut out) };
            assert_eq!(walked, 0, "fp {bad:#x} must not be walked");
        }
        // The good one still walks, so the row above is rejection and not a
        // walk that never worked.
        let walked = unsafe { super::walk_frame_chain(fp, sp, text, &mut out) };
        assert_eq!(&out[..walked], &chain);
    }

    /// The first sample establishes the baseline; it does not spend it.
    ///
    /// Sampled allocation attribution charges each sample the counter's growth
    /// since the previous one. There is no previous one for the first sample,
    /// and treating its absence as a reading of zero charged that sample every
    /// allocation the process had made since it started. On the ordinary path —
    /// a worker adopting an ask after hours of traffic — that is one fabricated
    /// row larger than every real one put together.
    #[test]
    fn the_first_sample_charges_nothing_and_seeds_the_next() {
        assert_eq!(
            super::allocs_charged(super::NO_PREVIOUS_SAMPLE, 400_000_000),
            0,
            "the first sample must not be charged the whole process history"
        );
        // Zero is a real reading, not an absent one: a process that has
        // allocated nothing must still measure intervals from it.
        assert_eq!(super::allocs_charged(0, 12), 12);
        assert_eq!(super::allocs_charged(100, 130), 30, "an ordinary interval");
        assert_eq!(super::allocs_charged(130, 130), 0, "nothing allocated between them");
        // The counter only grows; a smaller reading means it wrapped or was
        // reset, and a wild delta is worse than none.
        assert_eq!(super::allocs_charged(500, 4), 0);

        // The wiring, not just the rule: half the fix is the sentinel, and
        // re-seeding the static to 0 would leave every assertion above green
        // while restoring the defect — the first sample would once more be
        // charged the whole counter, because 0 is a reading and not an absence.
        assert_eq!(
            super::NO_PREVIOUS_SAMPLE,
            u64::MAX,
            "the sentinel must be outside the counter's range"
        );
        assert_eq!(
            super::LAST_ALLOCS.load(super::Ordering::Relaxed),
            super::NO_PREVIOUS_SAMPLE,
            "the static must START at the sentinel, or the first sample spends \
             the whole process history instead of establishing a baseline"
        );
    }

    /// A disarm holds until someone asks again, and no longer than that.
    ///
    /// Two failures pull in opposite directions here, and the fix for one was the
    /// other. Leaving the timer's owner set after a disarm meant a process could
    /// never arm again, so a later ask reached a service that had silently
    /// stopped sampling. Clearing the owner alone re-opened what disarming exists
    /// to close: `observe_shared_ask` runs on every request and the shared word
    /// still says the service was asked, so one request between the disarm and
    /// the `execve` re-armed the timer, which then survived into an image whose
    /// SIGPROF disposition is the default — "Profiling timer expired".
    ///
    /// The latch is what separates "stop until the exec" from "stop forever".
    #[test]
    fn a_disarm_blocks_arming_until_a_new_ask_arrives() {
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved = DISARMED_FOR_EXEC.load(Ordering::Relaxed);

        DISARMED_FOR_EXEC.store(true, Ordering::Relaxed);
        assert!(
            DISARMED_FOR_EXEC.load(Ordering::Relaxed),
            "a disarm must latch, or a request re-arms before the exec"
        );

        // An explicit ask lifts it — the case where the exec was called off and
        // an operator asks again. Only `begin_sampled` does this; a per-request
        // adoption must not.
        DISARMED_FOR_EXEC.store(false, Ordering::Relaxed);
        assert!(!DISARMED_FOR_EXEC.load(Ordering::Relaxed));

        DISARMED_FOR_EXEC.store(saved, Ordering::Relaxed);

        // The source is the guarantee here: an executable test cannot arrange an
        // `execve` race, so this reads the two call sites the same way the
        // platform-refusal guard elsewhere in this repo reads its own ordering.
        let source = include_str!("lib.rs");
        let observe = source
            .split_once("unsafe fn observe_shared_ask()")
            .expect("the adoption path must exist")
            .1;
        let body = observe.split_once("\n}").expect("a function body").0;
        assert!(
            body.contains("DISARMED_FOR_EXEC.load"),
            "the per-request adoption path must honour the latch"
        );
        let begin = source
            .split_once("pub(crate) fn begin_sampled()")
            .expect("the ask path must exist")
            .1;
        let begin_body = begin.split_once("\n}").expect("a function body").0;
        assert!(
            begin_body.contains("DISARMED_FOR_EXEC.store(false"),
            "an explicit ask must lift the latch, or an aborted exec is permanent"
        );
    }

    /// `folded_profile` really skips a slot a handler is inside.
    ///
    /// The truth table below pins the decision; this pins the WIRING, which is a
    /// different claim. An audit pointed out that deleting every sequence line
    /// from the handler and the reader would leave the truth-table test green —
    /// it tests a pure function that nothing would then call. This one fails if
    /// the reader stops consulting the sequence, because it hands the reader a
    /// slot whose contents are perfectly good and whose sequence says a writer is
    /// inside it.
    #[test]
    fn the_reader_skips_a_slot_whose_sequence_says_it_is_being_written() {
        let _serial = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let name = "hot_leaf";
        // A trailing sentinel bounds the last function's range; without one every
        // address in it resolves to `<native>`.
        let end = "<end>";
        let symbols = [
            SymtabEntry {
                address: 0x1000,
                name_ptr: name.as_ptr() as u64,
                name_len: name.len() as u64,
            },
            SymtabEntry {
                address: 0x2000,
                name_ptr: end.as_ptr() as u64,
                name_len: end.len() as u64,
            },
        ];
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;

        let saved_region = REGION.swap(base, Ordering::Relaxed);
        let saved_ptr = TABLE_PTR.swap(symbols.as_ptr() as usize, Ordering::Relaxed);
        let saved_len = TABLE_LEN.swap(symbols.len(), Ordering::Relaxed);
        let saved_stopped = STOPPED.swap(false, Ordering::Relaxed);

        let folded = unsafe {
            // One complete sample in slot 0, published as settled.
            let head = region_head().expect("mapped");
            head.store(1, Ordering::Relaxed);
            region_word(base, 0, 0).store(1, Ordering::Release);
            region_word(base, 0, PC_WORD0).store(0x1000, Ordering::Relaxed);
            region_word(base, 0, SEQ_WORD).store(slot_seq_settled(0), Ordering::Release);
            let settled = folded_profile();

            // The same slot, now marked as being written. Nothing else changes:
            // the depth and the program counter are still perfectly readable, so
            // a reader that ignores the sequence folds it exactly as before.
            region_word(base, 0, SEQ_WORD).store(slot_seq_writing(0), Ordering::Release);
            let mid_write = folded_profile();
            (settled, mid_write)
        };

        REGION.store(saved_region, Ordering::Relaxed);
        TABLE_PTR.store(saved_ptr, Ordering::Relaxed);
        TABLE_LEN.store(saved_len, Ordering::Relaxed);
        STOPPED.store(saved_stopped, Ordering::Relaxed);

        let (settled, mid_write) = folded;
        assert!(
            settled.as_deref().unwrap_or_default().contains(name),
            "a settled slot must fold: {settled:?}"
        );
        assert!(
            !mid_write.as_deref().unwrap_or_default().contains(name),
            "a slot a handler is inside must not fold, however readable it looks: {mid_write:?}"
        );
    }

    /// Two writers a lap apart cannot agree on a settled value.
    ///
    /// This is the hole a plain seqlock leaves here: it assumes ONE writer per
    /// slot, and this ring has two whenever a handler is frozen while the head
    /// laps. Computing the closing value from the word's previous contents let
    /// them converge — A reads 8 and will close at 10; B reads A's 9, closes at
    /// 10 as well — so a reader saw one even value across a read that spanned
    /// both writers and folded the tear. Deriving both values from the ticket,
    /// which `head.fetch_add` makes unique, is what removes the coincidence.
    #[test]
    fn two_writers_a_lap_apart_cannot_close_on_the_same_value() {
        let first = 4_u64;
        let lapped = first + RING_SLOTS as u64;

        // The interleaving that used to defeat it.
        assert_ne!(
            super::slot_seq_settled(first),
            super::slot_seq_settled(lapped),
            "two tickets for one slot must not settle on the same value"
        );
        assert!(super::slot_seq_writing(first) & 1 == 1, "writing is odd");
        assert!(super::slot_seq_settled(first) & 1 == 0, "settled is even");

        // A reader that spans both writers sees different values and discards.
        assert!(!super::slot_was_settled(
            super::slot_seq_settled(lapped),
            super::slot_seq_settled(first)
        ));
        // And one writer's own slot reads as settled.
        assert!(super::slot_was_settled(
            super::slot_seq_settled(first),
            super::slot_seq_settled(first)
        ));
    }

    /// A slot is folded only when it held still for the whole read.
    ///
    /// The defect this closes is invisible until the ring wraps. Publishing
    /// `depth` last is a correct gate while a slot is still zero, but the region
    /// is zeroed once, and past `RING_SLOTS` samples every reused slot already
    /// carries a non-zero depth from its previous lap. A reader that acquired
    /// that stale depth read the slot mid-overwrite and folded a stack that
    /// never ran — bounded, so never a bad access, and indistinguishable in the
    /// output from a real one.
    ///
    /// Row 2 is the one that matters: the sequence moved between the reader's
    /// two loads, which is exactly the overwrite the old gate could not see.
    #[test]
    fn a_slot_is_folded_only_when_no_handler_touched_it() {
        assert!(super::slot_was_settled(8, 8), "settled and unchanged: fold it");
        assert!(
            !super::slot_was_settled(8, 10),
            "a handler wrote the slot mid-read: the stack is two samples spliced"
        );
        assert!(
            !super::slot_was_settled(9, 9),
            "odd means a handler is inside the slot right now"
        );
        assert!(!super::slot_was_settled(9, 10), "odd, and it moved");
        // A fresh slot in a zeroed region reads as settled, which is right: it
        // holds no sample, and the depth gate is what skips it.
        assert!(super::slot_was_settled(0, 0));
        // The counter wraps like any other, and a wrap must not read as settled.
        assert!(!super::slot_was_settled(u64::MAX - 1, 0));
    }

    /// A process arms when someone asked and the timer on record is not its own.
    ///
    /// Row 1 is the bug this exists for: asked, ring mapped, and the armed timer
    /// belongs to another pid — a `--web` worker whose master authenticated a
    /// client after the fork. The old behaviour had no such path: the worker read
    /// the `ASKED` copy it inherited, found false, and stayed dormant while the
    /// master armed itself and ran no PHP.
    ///
    /// Row 2 is the second, subtler half. `fork` copies `ARMED_PID` but RESETS
    /// the child's interval timers, so a handler child forked from an already
    /// sampling parent inherits "armed" while holding no timer. Keying on the pid
    /// rather than on a bool is what makes it arm anyway — this is the row that
    /// covers `--web-isolation=pool|request`, where PHP runs two forks deep.
    #[test]
    fn a_process_arms_when_the_timer_on_record_is_not_its_own() {
        assert!(
            should_adopt_ask(0, 42, true, 0x1000),
            "asked, ring mapped, nobody armed yet: arm"
        );
        assert!(
            should_adopt_ask(41, 42, true, 0x1000),
            "the armed timer belongs to the parent; fork reset ours, so arm"
        );
        assert!(
            !should_adopt_ask(42, 42, true, 0x1000),
            "this process already armed: do not pay a setitimer per request"
        );
        assert!(
            !should_adopt_ask(0, 42, false, 0x1000),
            "nobody asked anywhere: stay dormant"
        );
        assert!(
            !should_adopt_ask(0, 42, true, 0),
            "no ring to fill: arming would signal a process with nowhere to record"
        );
    }

    /// The shared ask word lives inside the mapping and overlaps no other area.
    ///
    /// It is appended after the event table, so both failures this guards against
    /// are silent: a short `REGION_BYTES` would put it past the mapping, and an
    /// offset computed one area early would have `begin_sampled` corrupt the last
    /// route's I/O counters instead of publishing an ask.
    #[test]
    fn the_shared_ask_word_neither_escapes_the_region_nor_collides() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;

        unsafe {
            // The address, not a neighbour's value: an earlier version of this
            // test wrote a sentinel into the LAST event bucket and checked it
            // survived, which an offset computed one area early sails past — it
            // lands on the FIRST bucket. Asserting the offset itself is the only
            // form that catches every miscomputation.
            let asked = region_asked(base) as *const _ as usize;
            assert_eq!(
                asked - base,
                RING_BYTES + ROUTE_TABLE_BYTES + EVENT_TABLE_BYTES,
                "the ask word sits immediately after the event table"
            );

            // And every event bucket really is on the other side of it.
            for route_id in [0, 1, MAX_ROUTES] {
                for word in 0..EVENT_WORDS {
                    let event = event_word(base, route_id, word) as *const _ as usize;
                    assert!(
                        event + 8 <= asked,
                        "event bucket {route_id}:{word} overlaps the ask word"
                    );
                }
            }

            let asked = region_asked(base);
            assert_eq!(asked.load(Ordering::Acquire), 0, "starts unasked");
            asked.store(1, Ordering::Release);
            assert_eq!(asked.load(Ordering::Acquire), 1, "reads back what it wrote");
        }

        assert!(
            RING_BYTES + ROUTE_TABLE_BYTES + EVENT_TABLE_BYTES + CONTROL_BYTES <= REGION_BYTES,
            "the control word must fit inside the mapped region"
        );
    }

    /// Interns routes into a stand-in shared region and checks id stability,
    /// resolution, and the empty-clears-current-route contract.
    #[test]
    fn routes_intern_into_shared_memory_and_resolve() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // A heap region standing in for the mmap: only the route table area is
        // exercised, which lives at `base + RING_BYTES`.
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        REGION.store(base, Ordering::Relaxed);

        unsafe {
            let a = intern_route("GET /api/orders");
            let b = intern_route("POST /checkout");
            let a_again = intern_route("GET /api/orders");
            assert_eq!(a, 1);
            assert_eq!(b, 2);
            assert_eq!(a_again, a, "the same route reuses its id");
            assert_eq!(route_name(a).as_deref(), Some("GET /api/orders"));
            assert_eq!(route_name(b).as_deref(), Some("POST /checkout"));
            assert_eq!(route_name(0), None, "id 0 is no route");

            // set_route publishes the id; empty clears it. It interns only for a
            // service that was asked — see the gate's own test — so say so here,
            // and claim the timer for this pid so adopting the ask does not arm
            // a real SIGPROF in a binary with no handler for it.
            let was_asked = ASKED.swap(true, Ordering::Relaxed);
            let was_armed = ARMED_PID.swap(libc::getpid(), Ordering::Relaxed);
            let route = "GET /api/orders";
            elephc_probe_set_route(route.as_ptr(), route.len());
            assert_eq!(CURRENT_ROUTE.load(Ordering::Relaxed), a);
            elephc_probe_set_route(std::ptr::null(), 0);
            assert_eq!(CURRENT_ROUTE.load(Ordering::Relaxed), 0);
            ARMED_PID.store(was_armed, Ordering::Relaxed);
            ASKED.store(was_asked, Ordering::Relaxed);
        }
        REGION.store(0, Ordering::Relaxed);
    }

    /// A route carrying folded-format metacharacters (from an untrusted HTTP
    /// path) is neutralized so it cannot forge frames or profile lines.
    #[test]
    fn route_names_are_sanitized() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        REGION.store(base, Ordering::Relaxed);
        unsafe {
            let id = intern_route("GET /x\nfake;frame\t/y");
            let name = route_name(id).unwrap();
            assert!(!name.contains(';'), "{name}");
            assert!(!name.contains('\n'), "{name}");
            assert!(!name.contains('\t'), "{name}");
            assert_eq!(name, "GET /x?fake?frame?/y");
        }
        REGION.store(0, Ordering::Relaxed);
    }

    /// A full route table returns id 0 (untagged) rather than mis-attributing an
    /// overflow sample to an arbitrary existing route.
    /// I/O events are counted exactly, per route, and survive the untagged case.
    ///
    /// The point of the whole exercise: a driver call fires exactly one event, so
    /// these counts do not depend on sampling luck. A CLI run has no route, and
    /// dropping its events would understate the totals silently — worse than a row
    /// nobody expected — so id 0 gets its own bucket.
    /// Only a holder of the build key may turn profiling on.
    ///
    /// Without this, anyone who can set a header profiles your production: the
    /// request pays real time and the response reveals the shape of the code. The
    /// cases below are the ones an attacker actually has — no signature, a
    /// signature over a different message, a stale one captured from a log, and a
    /// truncated tag hoping for a prefix comparison.
    /// Installs `fd` as CONTROL_FD for the duration of a check, then restores.
    ///
    /// Tests share one descriptor table, so this saves and puts back whatever was
    /// on 3 — otherwise one test's socket becomes the next one's answer.
    fn with_control_fd(fd: i32, body: impl FnOnce() -> bool) -> bool {
        unsafe {
            let saved = libc::dup(CONTROL_FD);
            libc::dup2(fd, CONTROL_FD);
            let result = body();
            if saved >= 0 {
                libc::dup2(saved, CONTROL_FD);
                libc::close(saved);
            } else {
                libc::close(CONTROL_FD);
            }
            result
        }
    }

    /// Only the channel `monitor` created may turn profiling on.
    ///
    /// Every case below is a way a descriptor could end up on 3 without anyone
    /// asking for a profile. Getting this wrong does not leak data, but it makes a
    /// program start emitting profiler output for reasons its author cannot see —
    /// which is exactly the surprise `--with-monitoring` promises not to spring.
    #[test]
    fn only_the_control_channel_enables_profiling() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            // The real thing: a socketpair carrying the marker.
            let mut pair = [0i32; 2];
            assert_eq!(libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, pair.as_mut_ptr()), 0);
            libc::send(
                pair[0],
                CONTROL_MAGIC.as_ptr() as *const libc::c_void,
                CONTROL_MAGIC.len(),
                0,
            );
            assert!(with_control_fd(pair[1], control_fd_present), "the real channel must pass");
            // And the marker is consumed, so a second read cannot re-enable later.
            assert!(!with_control_fd(pair[1], control_fd_present), "the marker is single-use");
            libc::close(pair[0]);
            libc::close(pair[1]);

            // A socket with the wrong contents — someone else's channel.
            let mut wrong = [0i32; 2];
            libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, wrong.as_mut_ptr());
            let junk = b"HELLO-WORLD-1234";
            libc::send(wrong[0], junk.as_ptr() as *const libc::c_void, junk.len(), 0);
            assert!(!with_control_fd(wrong[1], control_fd_present), "wrong marker must fail");
            libc::close(wrong[0]);
            libc::close(wrong[1]);

            // A socket with nothing in it: no marker, no profiling, and no block.
            let mut empty = [0i32; 2];
            libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, empty.as_mut_ptr());
            assert!(!with_control_fd(empty[1], control_fd_present), "empty must fail");
            libc::close(empty[0]);
            libc::close(empty[1]);

            // A PIPE on the same descriptor — inherited from a shell, say.
            let mut pipe = [0i32; 2];
            assert_eq!(libc::pipe(pipe.as_mut_ptr()), 0);
            libc::write(pipe[1], CONTROL_MAGIC.as_ptr() as *const libc::c_void, CONTROL_MAGIC.len());
            assert!(
                !with_control_fd(pipe[0], control_fd_present),
                "a pipe is not a control channel even carrying the right bytes"
            );
            libc::close(pipe[0]);
            libc::close(pipe[1]);

            // A DATAGRAM socket: right family, wrong type.
            let mut dgram = [0i32; 2];
            libc::socketpair(libc::AF_UNIX, libc::SOCK_DGRAM, 0, dgram.as_mut_ptr());
            libc::send(
                dgram[0],
                CONTROL_MAGIC.as_ptr() as *const libc::c_void,
                CONTROL_MAGIC.len(),
                0,
            );
            assert!(!with_control_fd(dgram[1], control_fd_present), "SOCK_DGRAM must fail");
            libc::close(dgram[0]);
            libc::close(dgram[1]);

            // A closed descriptor answers no rather than faulting.
            let closed = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0);
            libc::close(closed);
            assert!(!with_control_fd(closed, control_fd_present));
        }
    }

    #[test]
    /// An unsigned, stale, or wrongly-signed header turns nothing on: asking
    /// to profile production is a privileged act.
    fn only_a_signed_query_enables_profiling() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = [7u8; handshake::KEY_LEN];
        let published: Vec<u8> = key.to_vec();
        KEY_PTR.store(published.as_ptr() as usize, Ordering::Relaxed);
        // A verified signature is SPENT against the shared replay table, so the
        // region has to exist for one to be accepted at all — see the test below.
        let mut region = vec![0u8; REGION_BYTES];
        let saved_region = REGION.swap(region.as_mut_ptr() as usize, Ordering::Relaxed);

        let now = {
            let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) };
            ts.tv_sec as i64
        };
        let sign = |stamp: i64| {
            let tag = handshake::hmac_sha256(&key, stamp.to_string().as_bytes());
            let hex: String = tag.iter().map(|b| format!("{b:02x}")).collect();
            format!("t={stamp},v={hex}")
        };
        // Safety: the slice comes from a live `&str` that outlives the call.
        let check =
            |value: &str| unsafe { elephc_probe_verify_query(value.as_ptr(), value.len()) } == 1;

        assert!(check(&sign(now)), "a fresh signature must be accepted");

        // Replay: a header captured from a log stops working once it ages out.
        assert!(!check(&sign(now - QUERY_WINDOW_SECS - 60)), "stale must be refused");
        // And a clock running ahead is refused symmetrically.
        assert!(!check(&sign(now + QUERY_WINDOW_SECS + 60)));

        // Forged: right shape, wrong key.
        let wrong = handshake::hmac_sha256(&[9u8; handshake::KEY_LEN], now.to_string().as_bytes());
        let hex: String = wrong.iter().map(|b| format!("{b:02x}")).collect();
        assert!(!check(&format!("t={now},v={hex}")));

        // Truncated: must not pass a prefix comparison.
        let good = sign(now - 1);
        assert!(!check(&good[..good.len() - 4]));
        // Malformed and empty values are refused rather than parsed loosely.
        assert!(!check("t=abc,v=zz"));
        assert!(!check(""));

        KEY_PTR.store(0, Ordering::Relaxed);
        // With no key published there is nothing to verify against, so nothing passes.
        assert!(!check(&sign(now - 2)));
        REGION.store(saved_region, Ordering::Relaxed);
    }

    /// A signature is worth one request, not five minutes of them.
    ///
    /// The timestamp bounds how LONG a captured header keeps working; on its own
    /// it says nothing about how OFTEN. Inside the window a value lifted from a
    /// proxy log, an access log or a shared trace could be replayed without
    /// limit, and each replay costs the service a profiled request. Spending the
    /// tag makes the second use fail, whoever sends it and whichever worker
    /// receives it — which is why the table is in the shared mapping rather than
    /// beside the verifier.
    #[test]
    fn a_verified_signature_is_spent_and_cannot_be_replayed() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let key = [3u8; handshake::KEY_LEN];
        let published: Vec<u8> = key.to_vec();
        let saved_key = KEY_PTR.swap(published.as_ptr() as usize, Ordering::Relaxed);
        let mut region = vec![0u8; REGION_BYTES];
        let saved_region = REGION.swap(region.as_mut_ptr() as usize, Ordering::Relaxed);

        let now = {
            let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) };
            ts.tv_sec as i64
        };
        let sign = |stamp: i64| {
            let tag = handshake::hmac_sha256(&key, stamp.to_string().as_bytes());
            let hex: String = tag.iter().map(|b| format!("{b:02x}")).collect();
            format!("t={stamp},v={hex}")
        };
        let check =
            |value: &str| unsafe { elephc_probe_verify_query(value.as_ptr(), value.len()) } == 1;

        let header = sign(now);
        assert!(check(&header), "the request that asked is served");
        assert!(!check(&header), "the same header again is a replay");
        assert!(!check(&header), "and stays one");

        // A different request, still inside the same window, is unaffected: the
        // table remembers signatures, it does not close the window.
        assert!(check(&sign(now - 30)), "a distinct ask is still honoured");

        // The slot is addressed from the tag, so this also covers the case that
        // matters most — two workers meeting the same replay — by construction:
        // both walk the same probe sequence and one loses the exchange.
        let (tag_word, _) = unsafe {
            let identity = handshake::hmac_sha256(&key, now.to_string().as_bytes());
            let mut first = [0u8; 8];
            first.copy_from_slice(&identity[..8]);
            let tag = u64::from_le_bytes(first);
            replay_slot(
                REGION.load(Ordering::Relaxed),
                (tag % REPLAY_SLOTS as u64) as usize,
            )
        };
        assert_ne!(
            tag_word.load(Ordering::Acquire),
            0,
            "the spent tag lands on the slot its own value chooses"
        );

        // Nowhere to remember means the promise cannot be kept, and the answer to
        // a privileged request nobody can account for is no.
        REGION.store(0, Ordering::Relaxed);
        assert!(!check(&sign(now - 60)));

        REGION.store(saved_region, Ordering::Relaxed);
        KEY_PTR.store(saved_key, Ordering::Relaxed);
    }

    /// A tag displaced from its home slot is still found when that slot frees up.
    ///
    /// Slots are probed from the tag's own value, so a tag whose home slot was
    /// occupied at first use lives further along the probe. Taking the first
    /// reusable slot DURING the search then accepted the replay: it stopped at
    /// the home slot the moment the entry displacing it expired, found it free,
    /// and took it without ever reaching the entry recording that this tag was
    /// already spent. The search has to finish before anything is taken.
    ///
    /// Driven through `spend_query_tag` rather than the HTTP entry point because
    /// the setup — one slot occupied at first use and expired by the replay — is a
    /// state of the table, and reaching it through signatures would mean minting
    /// two headers that collide modulo the table size.
    #[test]
    fn a_replay_is_caught_after_its_home_slot_frees_up() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        let saved_region = REGION.swap(base, Ordering::Relaxed);
        let now = 1_000_000i64;

        // A different signature already holds the home slot of the tag below.
        let tag = 7u64;
        let home = (tag % REPLAY_SLOTS as u64) as usize;
        let (squatter_tag, squatter_at) = unsafe { replay_slot(base, home) };
        squatter_tag.store(tag + REPLAY_SLOTS as u64, Ordering::Release);
        squatter_at.store(now as u64, Ordering::Release);

        // First use is honoured, and lands somewhere past its home slot.
        assert!(unsafe { spend_query_tag(base, tag, now) });
        assert_ne!(
            squatter_tag.load(Ordering::Acquire),
            tag,
            "the home slot was taken, so the tag went further along"
        );

        // The squatter ages out. Its slot is now reusable — and it is the FIRST
        // slot the replay meets.
        squatter_at.store((now - QUERY_WINDOW_SECS - 60) as u64, Ordering::Release);
        assert!(
            !unsafe { spend_query_tag(base, tag, now) },
            "a replay must be found wherever the tag actually sits"
        );

        // A genuinely new tag still gets that freed slot, so the fix bought
        // correctness without leaking the table.
        assert!(unsafe { spend_query_tag(base, tag + 1, now) });

        REGION.store(saved_region, Ordering::Relaxed);
    }

    #[test]
    /// I/O events are counted, not sampled — a driver call fires exactly one,
    /// so the per-route figure is exact even in a sampled capture.
    fn io_events_are_counted_exactly_per_route() {
        // Shares REGION and CURRENT_ROUTE with the other route tests, and cargo
        // runs tests in parallel; without this they trample each other.
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        REGION.store(base, Ordering::Relaxed);

        // No route set yet: everything lands in the untagged bucket.
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        for _ in 0..551 {
            elephc_probe_note_io();
        }
        elephc_probe_note_wait(2_290_874);

        let id = unsafe { intern_route("GET /orders") };
        assert!(id > 0, "the route table must hand out a 1-based id");
        CURRENT_ROUTE.store(id, Ordering::Relaxed);
        elephc_probe_note_io();
        elephc_probe_note_io();
        elephc_probe_note_wait(1_000);

        let report = event_report(base);
        assert!(
            report.contains("elephc-probe-io: <untagged> ops=551 wait_ns=2290874"),
            "{report}"
        );
        assert!(
            report.contains("elephc-probe-io: GET /orders ops=2 wait_ns=1000"),
            "{report}"
        );
        // Routes that saw no I/O must not produce an empty row.
        assert_eq!(report.lines().count(), 2, "{report}");

        REGION.store(0, Ordering::Relaxed);
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        // With no region mapped the entry points are inert rather than unsafe.
        elephc_probe_note_io();
        elephc_probe_note_wait(1);
        assert!(event_report(0).is_empty());
    }

    #[test]
    /// Past the route table's capacity samples go to a NAMED bucket, never to
    /// whatever route happened to sit there — and never to id 0.
    ///
    /// Id 0 is what a CLI run and an idle worker carry, so returning it for
    /// overflow made an endpoint arriving after the table filled vanish into the
    /// untagged pool instead of showing up as over-capacity. The constant said
    /// "overflow buckets into `<other>`" from the start; only the code did not.
    fn route_table_overflow_buckets_into_other() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        REGION.store(base, Ordering::Relaxed);
        unsafe {
            for i in 0..OTHER_ROUTE_INDEX {
                assert_eq!(intern_route(&format!("route-{i}")), i + 1);
            }
            // The reserved slot is still blank while the table has room.
            assert_eq!(route_name(OTHER_ROUTE_INDEX + 1), None);
            // Full: every later route shares one bucket, and it has a name an
            // operator can read in the report.
            let over = intern_route("one-too-many");
            assert_eq!(over, OTHER_ROUTE_INDEX + 1);
            assert_eq!(intern_route("another-one"), over);
            assert_eq!(route_name(over).as_deref(), Some(OTHER_ROUTE_NAME));
            // Routes interned before it filled still resolve to themselves.
            assert_eq!(route_name(1).as_deref(), Some("route-0"));
            // route_name never resolves an out-of-range id to a real slot.
            assert_eq!(route_name(MAX_ROUTES + 1), None);
        }
        REGION.store(0, Ordering::Relaxed);
    }

    /// A route is not interned until this process has been asked to sample.
    ///
    /// The table is 256 fixed slots with no eviction, so anything that writes to
    /// it before an operator connects is spending the budget of the profile they
    /// have not asked for yet. A service answering a few hundred distinct
    /// `/users/N` URLs filled it outright, and the endpoint under investigation
    /// then interned to nothing.
    ///
    /// Goes through the FFI entry point rather than `intern_route`, because the
    /// gate is in the caller: testing the interner would pass with the gate
    /// deleted.
    #[test]
    fn a_route_is_not_interned_before_anyone_asks() {
        let _guard = ROUTE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut region = vec![0u8; REGION_BYTES];
        let base = region.as_mut_ptr() as usize;
        REGION.store(base, Ordering::Relaxed);
        let was_asked = ASKED.swap(false, Ordering::Relaxed);
        // The entry point adopts an ask by ARMING, and a real SIGPROF in a test
        // binary whose handler is not installed kills it. Claiming the timer for
        // this pid is what a process that already armed looks like, so adoption
        // declines and only the interning under test runs.
        let was_armed = ARMED_PID.swap(unsafe { libc::getpid() }, Ordering::Relaxed);
        let route = "GET /users/{id}";
        unsafe {
            elephc_probe_set_route(route.as_ptr(), route.len());
            assert_eq!(
                CURRENT_ROUTE.load(Ordering::Relaxed),
                0,
                "an unasked service must not name its traffic"
            );
            assert_eq!(route_name(1), None, "and must not have written a slot");
            // Asked: the very next request is tagged, table untouched until now.
            ASKED.store(true, Ordering::Relaxed);
            elephc_probe_set_route(route.as_ptr(), route.len());
            assert_eq!(CURRENT_ROUTE.load(Ordering::Relaxed), 1);
            assert_eq!(route_name(1).as_deref(), Some(route));
        }
        ARMED_PID.store(was_armed, Ordering::Relaxed);
        ASKED.store(was_asked, Ordering::Relaxed);
        CURRENT_ROUTE.store(0, Ordering::Relaxed);
        REGION.store(0, Ordering::Relaxed);
    }
}
