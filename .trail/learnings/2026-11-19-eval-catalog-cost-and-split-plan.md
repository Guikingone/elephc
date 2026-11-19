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

## Step 1 landed: the catalog is its own object (56df61c12d)
- `src/codegen_support/catalog_unit.rs` splits the emitted text; `src/pipeline/backend.rs` writes
  `<stem>.catalog.s`, assembles it, and `src/linker/mod.rs` links both objects.
- Candidate criterion is `FunctionFlags`-independent: `function.source_name.is_none()`. Measured on
  the opaque fixture: **1,018 functions / 49,762,644 bytes = 94.4% of the unit**.
- Measured on the same fixture: **1,005 of 1,018 move** (49,707,387 bytes). The 13 that stay reach
  an `L`-prefixed label across the boundary.
- Cross-object symbols: the generator leaves most shared helpers file-local, so the split promotes
  what crosses with `.globl`; the existing visibility pass then emits `.private_extern` for every
  one of them (3,053 promotions, 3,053 private-extern directives, zero new exports).
- **The catalog unit must open `.text`.** The generator emits the text section once and then emits
  every function into it, so a unit made only of function bodies inherits whatever section the
  previous text left (the data section, for a recombined file). Symptom: the function lands in
  `.data`, the process faults on the first instruction (`EXC_BAD_ACCESS code=2`).
- `GeneratedUserAsm::single_unit()` exists because two test harnesses assemble one file themselves.
- Binary equivalence: same size (2,188,480 bytes), same stdout, same global symbol set; `__text`
  bytes differ in order only, because sections are laid out per object.

## Why the cache still cannot hash that object
- The catalog unit references **9,163 distinct `_str_*`/`_arr_*` symbols** that live in the
  program's data section, and their numbering is program-dependent: `_str_0` is `""` in a trivial
  program and `"count(): Argument #1 ..."` in the opaque fixture.
- It also carries **107 `_class_propinit_*`** functions, which are program-specific (one per user
  class), because they too have no source declaration.
- So the next change is data attribution: record which data entries each catalog function emits,
  move them into the catalog unit, and give them catalog-stable names. Only then is the unit a
  function of (target, features, PHP version) alone and hashable.
- Assembler time does not drop on the cold path (79 s for two sequential `as` runs vs 81 s for
  one): the win there needs `elephc-ngg1` (parallel assembly), and the win overall needs the cache.
