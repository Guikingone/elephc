---
title: "Runtime Context Register (spike)"
description: "Decision note for the ctx-register spike: reserving x28/r14 for per-context runtime state, measured cost, fiber trap, and the M0 migration path."
sidebar:
  order: 30
---

**Source:** `src/codegen_support/runtime/ctx.rs` — layout, register convention,
`__rt_ctx_init`, and ctx-relative access helpers.

## Why a context register

The runtime keeps mutable per-process state in global `.comm` symbols:
`_heap_off`, `_heap_free_list`, `_heap_small_bins`, `_concat_buf`,
`_concat_off`, plus exception, fiber, and stack-guard words. A single
`--rt-ctx` build routes that state through one reserved register — `x28` on
AArch64, `r14` on x86_64 — pointing at a `_rt_ctx` struct. This is the
foundational refactor for the sandbox-threads plan: a second execution context
(a spawned thread in M1) only needs a different register value, not a second
copy of the runtime.

The flag `--rt-ctx` (CLI → `RuntimeFeatures::ctx_register`, cache-key bit 12)
selects the mode per build; the legacy symbol-addressed runtime remains the
default and both modes coexist in the same compiler.

## The layout

`_rt_ctx` places the small scalars first and the 64 KiB concat scratch last:

| Offset | Field |
|---|---|
| 0 | `_concat_off` |
| 8 | `_heap_off` |
| 16 | `_heap_free_list` |
| 24 | `_heap_small_bins[4]` |
| 56 | `_concat_buf` (65536 bytes) |
| 65592 | (16-byte aligned total) |

Scalar offsets stay within the AArch64 unsigned-imm12 window so every access
is a single `ldr/str xN, [x28, #imm]` — no intermediate address
materialization, unlike the legacy `adrp` + `add` pair per symbol.

## Register choice

`x28`/`r14` were chosen over the alternatives:

- **x18** is reserved by Apple's AArch64 ABI and never usable.
- **TLS** requires hand-emitting TLV descriptors (Darwin) or `.tbss`+GOT
  indirection (ELF) in the runtime object, adds a load per access, and fights
  the runtime cache's plain-symbol identity. Rejected for the register's zero
  measured cost.
- Both registers are callee-saved, saved and restored whole by
  `__rt_fiber_switch`, and are now excluded from the linear-scan allocator
  pools (AArch64 pool drops to x21–x27; x86_64 already only allocated rbx).

On x86_64, reserving `r14` also required migrating every hand-written runtime
helper that scratched it (~120 uses across 12 files) to `rbx`, which the
runtime never used and preserves by contract everywhere. The fiber wrapper's
descriptor scratch moved to `r15` with a descriptor reload, and the x86_64
callback trampolines re-publish the ctx pointer before reaching compiled PHP
code (see Foreign entries below).

## Measured results (macos-aarch64, host build)

An allocation-heavy program (4M array/string allocations + a 200k-entry hash
build), 5 alternating runs of the same compiled binaries:

- legacy: 1.41–1.44 s
- `--rt-ctx`: 1.33–1.40 s

The ctx mode is equal-to-slightly-faster (the imm-offset load replaces an
`adrp+add` pair on the hot alloc/free paths). **Cost of the reserved register:
in the noise.** The pool reduction (8→7 callee-saved AArch64) did not measurably
hurt the bench.

## The fiber trap (found by the spike, fixed)

Generator and Fiber bodies run on freshly-mmap'd stacks whose fake initial
frame is deliberately zeroed, so the *first* `__rt_fiber_switch` into a
coroutine restores `x28 = 0` — and every allocation inside the body faults at
address `0x28`. The fix: `__rt_fiber_entry` re-publishes the `_rt_ctx` pointer
before any user code runs on the fiber stack. This is a permanent contract of
the ctx mode: **every entry point that adopts a fresh stack must re-publish
the ctx register.** The e2e test
`test_cli_rt_ctx_fibers_and_generators_re_publish_ctx` locks this behavior.

## Partial routing is incorrect routing (also found by the spike)

The first routing iteration only ctx-gated `__rt_heap_alloc`. The bench then
exhausted an 8 MB heap that the legacy build served fine: frees landed on the
*global* `_heap_free_list` (still read by nothing), and decref range checks
compared against the *global* `_heap_off` (permanently zero) so blocks were
never released. A ctx runtime where any heap-family helper still reads a
global is silently broken — recycling, refcount range checks, and the GC
walkers must all agree on the same state. The M0 migration must therefore be
complete-per-family, not incremental-per-helper.

This hazard is now mechanically enforced: ctx builds omit the legacy
`_heap_off`/`_heap_free_list`/`_heap_small_bins` symbols from the runtime data
section entirely, so any helper that still materializes them fails the link
with an undefined-symbol error instead of corrupting state at runtime.

## Foreign entries must re-publish (spike review, B2)

The fiber trap generalizes: the ctx register is callee-saved, so a host that
calls into compiled code preserves ITS value — pointing at host data, not at
zero and not at `_rt_ctx`. Every entry that reaches compiled PHP code from
foreign context must re-publish the pointer first (publish-only; never reset
allocator state mid-flight). Currently published at:

- the executable main prologue (full `__rt_ctx_init`),
- `__rt_fiber_entry` (zeroed fiber stacks),
- every cdylib/staticlib exported-function boundary wrapper,
- `elephc_init` (the library-mode lifecycle entry, which also zeroes the ctx
  fields — the library equivalent of the main prologue),
- extern FFI callback trampolines (called by foreign code like `qsort`).

## Scratch audit (spike review, B3)

A hand-written helper that starts using the ctx register as an ordinary
scratch register would silently corrupt the state pointer, and no other test
would catch it. `ctx_mode_runtime_never_scratches_the_ctx_register` scans the
ENTIRE ctx-mode runtime text on both architectures and fails on any ctx
register reference outside the sanctioned shapes (publish sequences,
ctx-relative accesses, fiber-switch save/restore pairs).

## Current state and what M0 must finish

Validated on macos-aarch64 AND linux-x86_64: the whole heap family routes
through the reserved register on both architectures (alloc, free,
heap_free_safe, incref/decref/heap-kind/GC range checks, the heap-debug
validator, descriptor release, object-handle and wrapper-cast checks, the web
arena reset); main installs the pointer via `__rt_ctx_init`; fiber entry and
every foreign-entry wrapper re-publish it. `--rt-ctx` binaries compile, link,
run, recycle their heap, and survive generators and fibers. The x86_64 `r14`
scratch uses are fully migrated to `rbx`, and the CLI accepts the flag on
every supported target.

Still on legacy addressing (the M0 work list):

- `_concat_buf`/`_concat_off` helpers (`__rt_concat_reserve`, `__rt_concat_publish`,
  `__rt_concat_grow`, the JSON/`sprintf`/diagnostic concat consumers) — the
  concat buffer lives in `_rt_ctx` but is still read through the globals.
- `_rt_ctx` as an emitter-shaped pool (array + free list) instead of a single
  instance, and `__rt_ctx_init`/`__rt_ctx_destroy` exported for the M1
  thread-pool bridge.

The full generated-runtime gate tests
(`ctx_feature_generates_ctx_addressed_runtime_end_to_end`,
`legacy_feature_keeps_symbol_addressed_runtime`), the scratch audit, and the
CLI e2e tests (`test_cli_rt_ctx_*`, including heap recycling and the
legacy-vs-ctx golden cross-check) pin both modes.