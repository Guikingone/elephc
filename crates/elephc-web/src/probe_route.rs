//! Purpose:
//! Bridge to the sampling probe and the exact profiler, shared by every web mode.
//!
//! Called from:
//! - `crate::worker`, for the default prefork model.
//! - `crate::handler_broker::process::handler`, for pool- and request-isolated
//!   children, where PHP runs in a handler child rather than in the worker.
//!
//! Key details:
//! - The compiled program's core runtime always defines the `.comm` pointer slots
//!   (zero by default); under `--probe` / `--instrument` the compiler stores the
//!   real entry points into them. Reading a slot and calling through it when
//!   non-null keeps monitoring pay-for-use, with no dlsym and no compile-time
//!   coupling to the probe crate.
//! - Every entry point here is a no-op when its slot is zero, so a binary built
//!   without monitoring pays one load per call site and nothing else.
//! - This lives in its own module because it used to be private to `worker`,
//!   which is precisely why `--web-isolation=pool|request` ran no monitoring at
//!   all: the isolated path never goes through `worker`.

extern "C" {
    /// Runtime `.comm` slot holding `elephc_probe_set_route` under `--probe`,
    /// else zero. Mangled per target like the other runtime externs.
    static elephc_probe_route_fn: usize;
    /// Runtime `.comm` slot holding `elephc_probe_rearm` under `--probe`, else
    /// zero — the worker re-arm the fork disarmed.
    static elephc_probe_rearm_fn: usize;
    /// Runtime `.comm` slot holding `elephc_instr_trace_begin` under
    /// `--instrument`, else zero — opens this request's W3C trace context.
    static elephc_instr_trace_fn: usize;
    /// Runtime `.comm` slot holding `elephc_instr_request` under
    /// `--instrument`, else zero — brackets one request's own profile.
    static elephc_instr_request_fn: usize;
    /// Runtime `.comm` slot holding `elephc_probe_verify_query`, else zero —
    /// checks that a profiling request is signed by the build key.
    static elephc_probe_verify_fn: usize;
}

type SetRouteFn = unsafe extern "C" fn(*const u8, usize);
type RearmFn = unsafe extern "C" fn();
type TraceFn = unsafe extern "C" fn(*const u8, usize, *const u8, usize);
type RequestFn = unsafe extern "C" fn(u32);
type VerifyFn = unsafe extern "C" fn(*const u8, usize) -> u32;

/// The route-tagging function, or `None` when the slot is empty because the
/// binary was built without the probe.
fn resolve() -> Option<SetRouteFn> {
    let addr = unsafe { std::ptr::addr_of!(elephc_probe_route_fn).read() };
    if addr == 0 {
        None
    } else {
        Some(unsafe { std::mem::transmute::<usize, SetRouteFn>(addr) })
    }
}

/// Re-arms the probe timer in this worker, if the probe is linked.
pub fn rearm() {
    let addr = unsafe { std::ptr::addr_of!(elephc_probe_rearm_fn).read() };
    if addr != 0 {
        let rearm = unsafe { std::mem::transmute::<usize, RearmFn>(addr) };
        unsafe { rearm() };
    }
}

/// Stamps `route` onto samples taken until `clear`.
pub fn set(route: &str) {
    if let Some(set_route) = resolve() {
        unsafe { set_route(route.as_ptr(), route.len()) };
    }
}

/// Clears the active route so idle-worker samples are untagged.
pub fn clear() {
    if let Some(set_route) = resolve() {
        unsafe { set_route(std::ptr::null(), 0) };
    }
}

/// Whether a profiling request carries a signature this binary accepts.
///
/// Profiling costs the request real time and reveals the shape of the code,
/// so asking for it is privileged: only a holder of the build key can. An
/// unsigned trigger would let anyone who can set a header profile production.
/// A binary without the tooling answers no, which is also the right answer.
pub fn query_is_authentic(value: &str) -> bool {
    let addr = unsafe { std::ptr::addr_of!(elephc_probe_verify_fn).read() };
    if addr == 0 {
        return false;
    }
    let verify = unsafe { std::mem::transmute::<usize, VerifyFn>(addr) };
    (unsafe { verify(value.as_ptr(), value.len()) }) != 0
}

/// Starts or ends this request's own profile, when the binary carries hooks.
///
/// A production binary can ship instrumented and dormant (`ELEPHC_INSTR_OFF=1`),
/// costing a load and a branch per call, and still answer the exact same
/// question a dev build answers — for the one request that asked. That is the
/// difference between "profiling is a dev thing" and "profiling is a thing you
/// can do where the problem actually is".
/// Brackets a request, saying WHY rather than only whether.
///
/// `1` this request was authorized by a signed header, `2` offer it — start
/// a slice only if something is waiting for one — and `0` end whatever
/// started. The bridge cannot answer "is anyone waiting": that state lives in
/// the instrumentation, so it reports what it knows and lets the other side
/// decide.
pub fn profile_request_kind(kind: u32) {
    let addr = unsafe { std::ptr::addr_of!(elephc_instr_request_fn).read() };
    if addr == 0 {
        return;
    }
    let bracket = unsafe { std::mem::transmute::<usize, RequestFn>(addr) };
    unsafe { bracket(kind) };
}

/// Opens the exact profiler's trace context for this request from the
/// inbound W3C `traceparent` (absent → a new trace is started). Pass the
/// header value, or `None`. No-op unless `--instrument` filled the slot.
/// `route` is passed alongside so an exact capture can be broken down per
/// endpoint, the way the sampler's route tagging already allows.
pub fn trace_begin(traceparent: Option<&str>, route: &str) {
    let addr = unsafe { std::ptr::addr_of!(elephc_instr_trace_fn).read() };
    if addr == 0 {
        return;
    }
    let begin = unsafe { std::mem::transmute::<usize, TraceFn>(addr) };
    let (tp, tp_len) = match traceparent {
        Some(value) => (value.as_ptr(), value.len()),
        None => (std::ptr::null(), 0),
    };
    unsafe { begin(tp, tp_len, route.as_ptr(), route.len()) };
}
