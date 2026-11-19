---
title: The opaque-eval catalog costs 208s per program, and the way to split it
author: Guillaume
date: 2026-11-19
branch: perf/eval-catalog-cache
---

## Decided
- The opaque-eval catalog (a program whose `eval()` source is only known at run time) is emitted
  once per program even though it is identical for one (target, runtime features, PHP version).
  It will become its own assembly unit, then a content-addressed cache entry beside the runtime
  object (`src/runtime_cache.rs` already does exactly this for the runtime).
- Order: separate the unit first (that single refactor unlocks both the cache and parallel
  assembly), then cache the OPTIMIZED catalog, then split the assembly into N chunks.

## Measured
- 72-byte program with `eval($code)`: 95 MB / 2,940,569 lines of assembly, 166,065 labels,
  47.8 MB binary. Phases: `as` 80.9s (38.9%), EIR optimizer 67.7s (32.5%), codegen 31.8s
  (15.3%), lowering 24.7s (11.9%); total 208s, 285G instructions retired, 533 MB peak RSS.
- Each function already owns its `.text` section (2,562 section switches).
- The shared data block is one `.data` region of ~83k lines whose `_str_*` symbols are `.globl`
  (10,619 declarations), so a second object can reference them.
- Of 22,416 `__elephc_eval_*` wrappers, only 221 are `.globl` (the ones the Rust eval bridge
  declares); the program never references the local ones, so they can move.

## Tried and ruled out
- `--no-ir-opt` for the catalog: SLOWER, 237s vs 208s. The optimizer is what shrinks the code:
  lowering 24.7→60.0s, codegen 31.8→42.6s, assembly 80.9→132.0s. The cache must store the
  optimized form.
- Deduplicating catalog symbols: only 2,936 of the 22,416 wrappers are single-instruction
  thunks (13%) with 2,807 distinct targets; the rest are real per-class bodies.

## Watch out
- The split boundary must be recorded per function in the emission loop
  (`src/codegen/block_emit.rs`, the three loops over `module.functions`, `module.class_methods`,
  `module.closures`): capture `emitter.buf.len()` before and after each call and keep the range
  when `function.flags.is_synthetic && function.source_name.is_none()`.
- Prelude and `.data` stay in the program unit; the catalog unit references `_str_*` across the
  object boundary. Assemble both (`src/linker/mod.rs:207`) and link both
  (`src/linker/mod.rs:330`).
- First proof: the binary produced before and after the split must be identical, before any
  caching is added.

## Measured later the same day (span probes on the split branch)
- `emit_module` writes 31,762,160 bytes and `finalize_user_asm` 20,926,776 on the opaque fixture
  (52,688,936 total; the 95 MB figure came from the dev-O1 compiler in another worktree, so
  compare ratios within one build only).
- Recording EVERY module function raises coverage from 885 spans / 37,960,954 bytes to
  1,018 / 49,762,644, i.e. **94.4% of the unit**. The catalog is therefore the module-vector
  functions plus the contiguous eval helper block in `finalize_user_asm`.
- The 22,416 `__elephc_eval_*` labels are NOT 22k functions: ~1k functions emit several variant
  labels each (`_inner`, `_scope_ok`, `_eval_shadow`).
- The split criterion still needs widening: `is_synthetic && source_name.is_none()` captures 884
  of the 1,018 module functions (19.9 of 31.8 MB); the other 134 functions (~11.9 MB) need a name
  or flag rule before the second unit can be sliced safely on a program with user code.
