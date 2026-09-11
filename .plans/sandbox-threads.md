# Sandbox Threads — tracking plan

Status: **M0 in progress** — the ctx-register foundation ships on
`spike/runtime-ctx-register` (PR #958, which replaced #954). Correct on
macos-aarch64 and on linux-x86_64 for everything the probes and the repaired
gates cover; the remaining CI shards are named under "Round 4".
Last updated: after round 4 (the first CI run this branch ever had).

## Goal

Make compiled PHP parallelisable through "sandbox" Rust threads: each thread owns
its runtime instance (`_rt_ctx`), no pointer crosses an arena boundary, values
transfer by deep copy through an `elephc-parallel` bridge. Threads are still a
plan; what is being built now is the foundation — per-context state reached
through a reserved register.

## Locked decisions

| Topic | Decision |
|---|---|
| Architecture | **(d)** sandbox threads with isolated heaps + explicit transfer (initial Kimi K3 consultation: d > c > b > a) |
| Ctx register | `x28` (AArch64) / `r14` (x86_64) — callee-saved, out of both regalloc pools (AArch64 pool 8→7), saved whole by `__rt_fiber_switch`. x18 excluded (Apple-reserved), TLS rejected (cost + complexity) |
| `_rt_ctx` layout | Scalars first (imm12 window: concat_off@0, heap_off@8, free_list@16, small_bins@24, heap_base@56, heap_max@64, 64 KiB concat buffer@72), 16-byte aligned |
| PHP API | See "API" below — arbitrated 2026-09-11, replaces the original `parallel_spawn()`/`parallel_join()` sketch |
| Concat migration | WHOLE family in one change (the contract and its ~47 consumers share one piece of state — never partial) |
| Tripwire | Ctx builds **omit** the legacy heap/concat symbols from the data section → any missed routing is a link error |
| rbx contract (x86_64) | Every runtime helper that touches rbx MUST preserve it (push/pop or a frame spill, one restore per return path) — mechanically audited |
| Foreign entries | Every entry reaching compiled PHP from foreign context re-publishes the ctx pointer (publish-only, never a mid-flight reset) and gives the host's value back on every return path |

## Milestones

### Done (branch `spike/runtime-ctx-register`)

- **Spike** `569146ac44`: `--rt-ctx` mode (CLI flag → feature bit 12 → cache key),
  `_rt_ctx` layout, `__rt_ctx_init` in the main prologue, AArch64 heap family
  routed through x28, x28 removed from the regalloc pool. A/B bench: ctx ≥ legacy
  (the reserved register's cost is in the noise).
- **Round-1 review remediation** `2cd87b87af`: x86_64 as a first-class target
  (r14 routing, ~120 r14 scratch uses moved to rbx), publish at foreign entries
  (cdylib exports, `elephc_init`, FFI trampolines), fail-closed x28/r14 scratch
  audit, link tripwire, `__rt_ctx_init` clobber contract + argv e2e, pre-existing
  fiber crash test (dedicated 32 MiB stack).
- **Concat helpers** `0eca9336cc`: `emit_concat_off_load/store`,
  `emit_concat_off_store_imm`, `emit_concat_buf_address` — contract: the legacy
  arms clobber no caller scratch (AArch64 store through the historical x6, loads
  through the destination register / RIP-relative).
- **Full concat migration** `800ae4d8a7`: ~350 sites across 77 files, both
  targets, concat tripwire armed. Traps found and fixed: 10+ "kept the address"
  orphans (itoa, ftoa, strtoupper, implode ×3, resource_to_string, json_decode,
  json_encode_float, …).
- **Round-2 review remediation** `dafbf9589f` + `cf97f98077`:
  - NB1: the rbx contract — 7 files protected (vsprintf/zval ×3 spills,
    json_encode_str, spl_dll ×4 push+rax pad, http.rs→r8); mechanical audit plus
    a negative control; the str_to_number collision repaired (integer-significand
    flag moved to a stack slot).
  - NB2: **a real ABI bug** — the AArch64 trampoline overwrote the C host's x28
    without restoring it → spill/restore added; ordering tests on both targets;
    the exception path documented (longjmp passes over the trampoline).
  - NB3: **a real orphan** — php_uname's epilogue loaded through a stale x6
    address; mechanical dangling-x6 audit over the whole legacy runtime.
  - Kitchen-sink ctx gate (all features, both targets, zero legacy symbols);
    `--rt-ctx --heap-debug` e2e.

### Round 3 (human review + repaired audits)

The B3 audit (`ctx_mode_runtime_never_scratches_the_ctx_register`) filtered lines
on the target's COMMENT PREFIX: it read comment text and never the instructions,
so it was **green forever, on both targets**. Repaired (two tests share one
scanner), it immediately reported 3 AArch64 offenders and 81 on x86_64. Defects
found and fixed in the same pass:

| Defect | Scope | Evidence |
|---|---|---|
| `__rt_wordwrap` kept its output cursor in **x28** and then called `__rt_concat_publish` | ctx AArch64 | PHP witness: the 2nd and 3rd `wordwrap()` of one expression printed the 1st one's text (`test_cli_rt_ctx_concat_backed_results_do_not_overwrite_each_other`) |
| The r14→rbx migration **collided** with an rbx already in use: `wordwrap` (source base vs lastspace), `grapheme_strrev` (source pointer vs destination start), `sprintf` (write cursor vs sequential argument index) | **legacy AND ctx**, x86_64 | reading; a pure regression against `main` |
| **Orphaned** `_concat_off` address: the ctx helper loads the VALUE, the address register is no longer written but still dereferenced — `chr()`, `spl_object_hash()`, `resource_to_string()`, `php_uname()`, `sprintf` (`[rbp-56]`) | **legacy AND ctx**, x86_64 | reading + a scripted audit; a wild store |
| `emit_concat_off_store_imm` emitted `str #65536, [x28]` for a non-zero value (no such instruction) | ctx AArch64 | every `--emit cdylib/staticlib --rt-ctx` build refused by the assembler |
| `elephc_init` published the ctx pointer **after** the concat reset, which is itself ctx-relative | ctx, both targets | SIGSEGV at address 0 in the first `elephc_init` of any ctx cdylib |
| cdylib/staticlib exports published x28/r14 **without saving the host's value** | ctx, both targets | C host with an assembly sentinel: `CLOBBERED` → `PRESERVED` (`test_rt_ctx_cdylib_export_preserves_the_hosts_ctx_register`) |

Lesson to keep: **an audit that cannot fail is not an audit**. The two mechanical
audits the round-1/2 reviews demanded each caught a real bug; this one caught
none, because it read nothing.

### Merging `origin/main` (361 commits) — what the merge cost

Two conflicts, but **four defects** only the repaired gates saw:

- `runtime/curl/warn_option.rs` (NEW from main) addresses `_concat_off` directly
  ⇒ in ctx mode the tripwire removes that symbol ⇒ **link failure** for any
  `--rt-ctx` build carrying curl. Routed through the ctx helpers.
- `__rt_pcntl_async_dispatch_preserving` (NEW) took **x28** as the base of its
  spill area and then called `__rt_pcntl_dispatch_pending`: PHP signal handlers
  therefore ran with a context pointer of `sp+512`. Base moved to x30 (already
  parked, and the `bl` clobbers it anyway).
- `__rt_getenv_all` x86_64 (NEW) kept the name length in **r14** across
  `__rt_str_persist` and `__rt_hash_set` ⇒ r14 → r15. The baseline had gone from
  81 to 84: **exactly what a shrink-only gate exists to catch**.
- `json_encode_array_int`: the conflict set main's SysV alignment fix
  (`sub rsp, 40`→`48`) against the branch's ctx line. Taking "ours" would have
  silently dropped main's fix.
- Main's new `every_x86_64_runtime_call_site_is_sysv_aligned` gate caught **two
  frames the branch's own NB1 remediation had misaligned** (`zval_pack_array_packed`
  104, `zval_unpack_array` 88: an 8-byte rbx slot added to an already-aligned
  frame). That gate did not exist on the branch's base.

Rule that falls out: **after every merge, replay the ctx gates** (x28 scratch,
r14 scratch, kitchen-sink, SysV alignment) before anything else.

### Round 4 — the first CI run this branch ever had

PR #954 sat 361 commits behind `main`, conflicting, so no job but `Classify` had
ever run on it. It was closed in favour of #958 from the fork, merged up to date.
CI then named three more x86_64 defects, all invisible on an AArch64 host:

- **`movzx rsid`** — the r14 migration pasted r8–r15's size suffix onto a legacy
  register (`r14d` → `rsid`; rsi's 32-bit view is `esi`). Three jobs red for two
  instructions. Every emitted-text test stayed green, because the text WAS what
  they assert. ⇒ new `runtime::assembles` gate: the whole generated runtime goes
  through a real assembler, every target × every feature × both modes, with a
  negative control feeding it this exact instruction.
- **`preg_replace`'s orphaned publish** — `mov QWORD PTR [r9], r10` through a
  register that used to hold the `_concat_off` address. Sixth instance of that
  class; it killed 5 preg tests plus `test_regex_iterator_modes` **in legacy
  mode**. The earlier orphan audit missed it because it asked whether the
  register had been WRITTEN (r9 had been, with data) instead of whether it had
  been established as an ADDRESS.
- **User codegen borrowed r14** — the audit scans the generated runtime and
  nothing else. `runtime_callable_invoker` used r14 as its type-tag register and
  then called the PHP callable through it, and `static_properties` used it for
  class-id comparisons. Five SIGSEGVs (`usort`, `array_filter`, `array_reduce`,
  `array_walk`, `array_any` with a closure). ⇒ new
  `test_cli_rt_ctx_user_codegen_never_scratches_the_ctx_register`: 23 offenders
  at first, 0 now, zero tolerance.

Method note worth keeping: localizing those five needed **one binary per helper**.
A buffered stdout loses whatever it still holds when the process dies, so the last
PASS line printed is NOT the last check that ran.

## Remaining work (M0 → M1)

### Done during the review sessions
- [x] **r14 migration finished on x86_64: 81 → 0.** The gate went from a
      shrinking baseline to zero tolerance. Order of preference: (1) a register
      outside the allocator pool (r15, r8, or a caller-saved one in a leaf);
      (2) rbx **with** the caller's value preserved; (3) a frame slot; (4) no
      register at all — six sites only compared once, so a memory-operand `cmp`
      replaced the pair.
      ⚠️ An 8-byte slot added to an already-aligned frame breaks SysV: exactly
      what `every_x86_64_runtime_call_site_is_sysv_aligned` catches.
- [x] **`_rt_ctx` as `.comm`**: a ctx binary went from 70 KB to 136 KB because of
      64 KB of zeroes written into `.data`. It is now **80 bytes SMALLER** than
      its legacy twin (it drops `_concat_off`/`_heap_off`).
- [x] **Per-context heap arena** — `heap_base` and `heap_max` in `_rt_ctx`, 59
      sites migrated. Without it `--rt-ctx` made the allocator's STATE
      per-context while the arena stayed one global `_heap_buf`: two contexts
      both starting at `heap_off = 0` hand out the same address. Tripwire: a ctx
      build materializes `_heap_buf` exactly once (in the init), a legacy build
      still names it everywhere.
- [x] **8→7 pool bench (required before any neutrality claim)**: two spill-heavy
      programs, each built twice from the same tree with x28 in and out of the
      pool, alternating runs → 1.325 s vs 1.315 s and 0.670 s vs 0.675 s. Inside
      the spread ⇒ **x28 stays out of the pool in BOTH modes** (one register
      discipline). Known limit: in the first program the allocator kept only 3
      values in callee-saved registers, hence the second, written to have eight
      in flight.
- [x] **linux-x86_64 execution** (Docker linux/amd64): probes covering every
      rewritten helper plus json and getenv, **ALL PASS in legacy AND `--rt-ctx`**.
      The first x86_64 execution this branch ever had.

### Remaining M0
- [ ] **Green linux-x86_64 CI.** Still red at the time of writing:
      `codegen::spl::classes::test_phase4_spl_doubly_linked_list_delete_iteration_modes`
      and the sprintf/vsprintf formatting family. `__rt_sprintf` and
      `__rt_vsprintf` are both in the alignment audit's
      `NOT_STATICALLY_ANALYZABLE` list, so nothing checked the frames the
      branch changed there — `__rt_vsprintf` went `sub rsp, 64` → `72`, which
      flips the call alignment (fixed to 80).
- [ ] **Drop the `--rt-ctx` CLI flag** and make the ctx runtime unconditional.
      Decided 2026-09-11: the flag gives the user nothing observable (same
      program, same output, cost in the noise, smaller binary) — unlike
      `--web-isolation`, which trades crash isolation against throughput and is a
      legitimate compile-time choice. Every defect found in rounds 3 and 4 came
      from maintaining two arms. Sequence it AFTER x86_64 CI is green: removing
      the fallback before the survivor is proven is the wrong order.
- [ ] **Multi-context pool**: `_rt_ctx` as an array + free list instead of a
      single instance; `__rt_ctx_init`/`__rt_ctx_destroy` exported for the M1
      bridge. (`--heap-size` per thread to document.)
- [ ] **State families still on globals** — inventory done:
      `emit_runtime_data_fixed` declares **174 `.comm` symbols**, of which only 9
      (heap ×3, concat ×2, heap_base/heap_max, plus the `_rt_ctx` block) are
      migrated. By family, most to least blocking for M1:
      1. **exceptions**: `_exc_handler_top`, `_exc_value`, `_exc_call_frame_top`
         — a thread that throws walks the MAIN thread's handler chain. Not "may
         corrupt": structurally wrong. **Blocks M1.**
      2. **fibers/stack**: `_fiber_current`, `_stack_limit`,
         `_fiber_main_saved_sp/_exc/_call_frame` — the stack guard compares
         against main's bounds, so a thread on its own mmap'd stack either never
         trips it or trips it at once. **Blocks M1.**
      3. **GC counters**: `_gc_allocs/_gc_frees/_gc_live/_gc_peak/_gc_collecting`
         — a SEMANTIC decision to take before coding: per-context (per-thread
         numbers) or atomic (process numbers). `_gc_collecting` is a lock, not a
         counter: per-context either way.
      4. **shared buffers**: `_cstr_buf`, `_cstr_buf2`, `_empty_str`, the
         ob/print_r buffers — `_empty_str` is read-only and can stay global, the
         rest are per-context scratch.
      5. **resource registries**: `_dir_handles`, `_glob_handles`,
         `_bzstream_handles`, `_buffer_registry_*` — per-context if a thread may
         open resources, otherwise an explicit boundary.
      6. **bridge slots** (`_elephc_tls_*_fn`, `_elephc_crypto_*_fn`, …): written
         once at startup, never rewritten ⇒ stay global.
      Same discipline as concat: **a whole family per change**, each with its
      tripwire.
- [ ] **Full linux-x86_64 suite**: `./scripts/test-linux-x86_64.sh` (Docker,
      emulated and slow) or a CI shard. Recipe for re-running the targeted probes
      without starting over: the persistent docker volume `elephc-x86-ctx` plus
      `scratchpad/x86run.sh` (incremental build).
- [ ] Error-path matrix for the concat consumers × 2 modes × 2 targets; ctx
      staticlib parity (the ctx cdylib is green since round 3, the staticlib is
      still unprobed).
- [ ] **The x6 borrow (AArch64 legacy)**: the legacy arm of
      `emit_concat_off_store` materializes the address in **x6**, an ARGUMENT
      register, across 122 call sites. `legacy_aarch64_runtime_has_no_dangling_x6_stores`
      covers the runtime text, **not user codegen**. A `debug_assert` now catches
      the direct misuse (x6 passed as the VALUE). To settle: extend the audit to
      user codegen, or pay 2 instructions (`str`/`ldr` around a scratch) on the
      legacy concat path. ⚠️ In ctx mode the question disappears (no legacy arm),
      so change nothing while legacy is the shipped mode.

## API — arbitrated 2026-09-11

Replaces the original `parallel_spawn()`/`parallel_join()` sketch. Prior study:
`Psl\Async` (Revolt fibers: `run(): Awaitable`, `TaskGroup`, `Channel\bounded()`),
Java 27 `StructuredTaskScope`, Swift 6 `Sendable`, Kotlin `coroutineScope`, Rust
`thread::scope`, ext/parallel.

**Three invariants every one of them shares**, which we adopt:
1. **A scope owns its children** — it does not return before they finish,
   failures aggregate, cancellation propagates down. Go is the only one without,
   and it is its most documented design regret.
2. **The scheduler is a parameter, not another API** (virtual vs platform threads
   in Java, `Dispatchers.*` in Kotlin).
3. **What may cross the boundary is THE question.** The level at which each
   language answers it is what separates them: the type system (Rust `Send`,
   Swift `Sendable`), the runtime (ext/parallel), or nothing at all (Java, Go —
   shared memory).

**What elephc has and no PHP runtime does**: an AOT checker. We refuse **at
compile time** what cannot cross, where ext/parallel throws at run time. That is
the Swift 6 bet, and it is the differentiator.

### Namespaces
`Elephc\Async` (fibers, one thread) and `Elephc\Parallel` (threads, separate
arenas). The `Elephc\` prefix is deliberate: no collision with PSL or any other
library, including one running ON elephc.

### Two surfaces, two handles (arbitrated: separate, less risk)
```
Elephc\Async      Awaitable<T>, TaskGroup, Cancellation
                  run(Closure): Awaitable, await(Awaitable): T
                  all() any() first() concurrently() series() sleep() later()

Elephc\Parallel   Future<T>, TaskGroup, Channel / Sender / Receiver
                  run(Closure, mixed ...$args): Future, join(Future): T
                  concurrently(iterable<Closure>): array
```
`Awaitable` ≠ `Future`, with no implicit conversion: **the type states the cost**.

### Transfer contract — REFUSED at compile time
| Crosses | Refused, with a named error |
|---|---|
| scalars (copied) | `use (&$x)` — would alias two arenas |
| arrays of transferable values (deep copy) | objects (identity, destructors, per-arena class descriptors) |
| `Cancellation` — **by reference, the ONLY exception** | resources (fds, streams, PDO handles) |
| the return value, copied back at `join` | a closure whose target is not statically resolvable, and anything `Borrowed` |

### Cancellation — by TOKEN (`Elephc\Async\Cancellation`)
Amp v3's short name, not `CancellationToken`.

The reason is technical, not aesthetic: **the cancellation flag has to live
outside both arenas** (a deep copy would never observe the parent's `cancel()`),
so in a shared atomic cell. That holds for either model. A token puts that sharing
in a **named type**, so the transfer rule stays "nothing by reference EXCEPT
`Cancellation`" — one auditable exception. Structural cancellation
(Swift/Kotlin) would make that exception ambient and unverifiable, and would need
another `_rt_ctx` field plus a lookup at every suspension point.

Second argument: structural cancellation acts at suspension points. A worker doing
`heavy($n)` with no `await` has none — the structural model would do nothing, or
would require signal-based preemption. The token stays honest.

`TaskGroup` cancels its children when one fails, **using** an internal token: we
keep Java's scope semantics without Swift's ambient state.

### Out of v1 — refused with a named error, never stubbed
`Parallel\Runtime` (persistent worker, arrives with the pool), `Async\Semaphore` /
`Sequence` / `WaitGroup`, ext/parallel's `parallel\*` compatibility.

PSL is a portability target **in the other direction**: no adapter layer, but a
test that compiles a `Psl\Async` fragment on elephc and checks it lands on our
primitives.

### M1 order, revised by the "separate handles" arbitration
Separating `Awaitable` from `Future` takes await-inside-a-thread out of v1, which
takes fibers off the critical path:
1. **Exceptions + stack-limit into `_rt_ctx`** — blocking: a worker that throws
   must unwind ITS chain, and its stack guard must know ITS stack.
2. **Pool `_rt_ctx[]`** + `__rt_ctx_init`/`__rt_ctx_destroy` exported.
3. **The transfer check in the checker** — writable and testable BEFORE any
   thread exists, on programs that never run. An independent deliverable, and the
   most differentiating part.
4. `__rt_value_clone_into(dst_ctx, src)` (cycle-safe deep copy through the GC/COW
   walkers) + staticlib bridge + an EIR `spawn` node → `Parallel\TaskGroup`.
5. Per-context fibers, `Parallel\Runtime`, MPSC channels — after v1.

### M2 (optional)
A `shared` bit in the `kind` word, atomic incref/decref gated on it, the rule
"shared ⇒ never mutated in place", opt-in structures through the bridge.

## Reviews

- Round 1 (Kimi K3, via Ollama): "request changes" — B1-B5 plus gaps D1-D10.
  Fully remediated (`2cd87b87af`).
- Round 2 (Kimi K3, via Ollama): "do not merge as is" — NB1-NB4. Fully
  remediated (`dafbf9589f`). Both mechanical audits it demanded caught a real bug
  (the host's x28, php_uname's orphan).
- Round 3 (human): the B3 audit was inert; see the table above.
- Round 4 (CI): the first run this branch ever had; three more x86_64 defects.

## References

- Decision note: `docs/internals/runtime-ctx-register.md`
- CLI: `--rt-ctx` in `docs/compiling/cli-reference.md` (to be removed, see M0)
- PR: illegalstudio/elephc#958 (replaces #954), head on `Guikingone:spike/runtime-ctx-register`
- Ctx module: `src/codegen_support/runtime/ctx.rs`
- Full reviews: transcripts in the conversation sessions (rounds 1 and 2 via
  `ollama run kimi-k3:cloud`).
