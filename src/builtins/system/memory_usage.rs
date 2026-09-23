//! Purpose:
//! Home of PHP's memory-reporting pair — `memory_get_usage()` and
//! `memory_get_peak_usage()` — and their semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - WHAT THESE HONESTLY REPORT. php's numbers come from the Zend memory manager:
//!   `$real_usage = false` is the bytes `emalloc` currently has handed out, and
//!   `$real_usage = true` is the bytes the manager took from the system in 2 MB
//!   chunks. elephc has its own allocator with exactly the same two quantities
//!   already maintained on every allocation and free — `_gc_live` (bytes in live
//!   blocks, header included) and `_heap_off` (the bump offset, i.e. every byte ever
//!   carved out of the arena) — so both modes answer with a MEASURED figure from the
//!   allocator that served the program, not an invented one and not a process-wide
//!   RSS reading that would also count the binary's own text and stacks.
//! - The counters are maintained unconditionally by `__rt_heap_alloc` /
//!   `__rt_heap_free`, NOT only under `--heap-debug`; `--heap-debug` only prints them.
//! - Where this DIVERGES from php, and it must be documented rather than hidden:
//!   * `memory_get_usage(true)` starts at 0 in a fresh process, where php reports one
//!     2 MB chunk immediately. elephc's arena is a BSS buffer that is demand-paged, so
//!     "taken from the system" means "carved out of the arena", which begins empty.
//!   * Bytes that are not elephc heap bytes are not counted: interpreter-side Rust
//!     allocations, libc `malloc` inside a linked C library (curl, iconv), and the
//!     `_heap_buf` reservation itself. php's figure has the mirror-image blind spots
//!     (it does not count `malloc` outside the Zend MM either).
//!   * `_heap_off` never decreases, because a freed block is parked on the free list
//!     rather than returned to the arena. php's real usage is likewise sticky.
//! - There is NO separate eval implementation of the number. The interpreter reaches
//!   the same two counters through the `__elephc_eval_memory_get_usage` /
//!   `__elephc_eval_memory_get_peak_usage` C-ABI entry points, for the reason
//!   `headers_sent` gives: the state lives in generated-runtime storage, so an
//!   eval-local answer would describe a different heap than the one the program uses.

builtin! {
    contract: "memory_get_usage",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::MemoryGetUsage,
    ),
}

builtin! {
    contract: "memory_get_peak_usage",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::MemoryGetPeakUsage,
    ),
}
