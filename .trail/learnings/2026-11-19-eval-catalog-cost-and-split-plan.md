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

## Step 2 (part one): the catalog unit no longer names program data (2404b3cac3)
- Every string literal a moved span references is renamed after the FNV-1a hash of its own bytes
  (`_catalog_str_s<hex>`) and emitted inside the catalog unit. The program keeps its own copy for
  its own references. Measured on the opaque fixture: 8,603 catalog literals, **zero `_str_`**.
- Property-default thunks (`_class_propinit_*`, 107 of them here) left the candidate set: they are
  nameless and synthetic too, but there is one per user class.
- Two traps in literal capture, both real in the generator's output: a literal can contain a raw
  newline AND `//` (phar/URL strings), and the last literal of the file has no following line to
  close it.
- Remaining program dependence, measured by `--timings`: **12** symbols (`_class_ctor_<class>_4`),
  i.e. constructor thunks named after the *class index*, referenced from the catalog but defined
  in the program unit.

## Step 2 (part two): a content-addressed object cache (8abf0c82b1)
- `src/catalog_cache.rs` stores the assembled object under
  `~/.cache/elephc/catalog/catalog-<key>.o` with a `.len` witness, and returns a hit or miss.
- Key = build identity + target + PHP profile id + the emitted text, with **only `L`-prefixed
  labels** normalized. A global symbol ending in a number can encode a program decision
  (`_class_ctor_7_4` = class 7's constructor), so it stays in the key and different class
  numbering misses instead of reusing a wrong helper.
- On a hit the catalog assembler never runs: `<stem>.catalog.o` is not created next to the program.
- Two programs with the same builtin set produce units that differ **only** in local label numbers
  and in the `__FILE__` path baked into the eval error text — so the unit is program-independent
  except for the source path, which is a legitimate per-compilation input and stays in the key.

## The cache exposed a determinism bug (66c5a90b4a)
- The catalog unit was program-independent after data attribution, yet every build computed a
  different cache key. Diffing two units from two runs of the SAME source showed the same lines in
  a different ORDER (9,578 differing lines in a small 10-class fixture).
- Cause: `lower_dynamic_constructor_thunks` walked `module.class_infos`, a hash map, appending one
  `_class_ctor_<id>_<argc>` thunk per class. The per-process hasher seed decided the order, which
  reordered the emitted assembly of every opaque-eval build.
- Fix: sort the classes by `class_id` (then name) before lowering the thunks.
  `dynamic_new_thunks_emit_identical_assembly_across_processes` fails without it and passes with
  it; `tests/compiler_determinism_tests.rs` already had the harness and the doctrine, it just had
  no fixture that produced dynamic-`new` thunks.
- Lesson: a content-addressed cache is also a determinism test. It found in one run what the
  existing suites did not cover.

## Step 3: the catalog is four objects (daea809153)
- `split_catalog_units(asm, spans, 4)`: functions that share an assembler-local `L` label are
  grouped first (union-find over def/ref), then each group is placed whole on the currently
  smallest unit. Splitting a shared `L` label across objects cannot work: Mach-O drops those
  labels from the symbol table whatever their declared binding.
- Every non-local symbol a unit defines is promoted there, since any other unit may reach it by
  name; the visibility pass still hides all of them, so no exports and no dead-strip roots.
- Measured on the opaque fixture: four units of 24.2 / 10.9 / 10.6 / 10.4 MB, assembly 241.02 s
  (sequential two units) → **156.19 s** cold, **89.46 s** on a cache hit that runs only the
  program assembler. Totals on that host (1332 s / 1102 s) are contention, not the compiler.
- Numeric labels (`1:`) are assembler-local too and must never be promoted; a definition filter
  that only accepts names starting like a symbol is what keeps `.globl 1` out of the units.
- The program-dependent metric must count only real crossings: counting every promotion reported
  2,828 symbols where 12 actually cross (436e0c4dce).

## What a3sl still requires (not implemented)
- Skipping lowering needs the catalog list *before* the bodies are lowered, plus placeholder
  `Function`s with correct names and signatures so the program unit's by-name calls still resolve
  and codegen never emits them. The cached object then supplies the bodies.
- The cache key for that cannot be the emitted text (it is computed after emission). It must be a
  trusted signature over the lowering inputs, which is a correctness bet the text key does not
  make: a wrong key here miscompiles instead of missing.
- Safe scope for the next session: land the placeholders behind a flag, keep the text key as the
  source of truth, and only promote the signature key once a determinism soak (the existing
  `compiler_determinism_tests` harness) covers an opaque-eval fixture.

## Step 4 (a3sl, first increment): the emission itself is cached (8c30ee4a48)
- A rebuild now skips codegen for the catalog: `src/codegen/catalog_reuse.rs` stores the unit texts
  under a digest and the three emission loops skip the functions it covers.
- The digest is over the POST-optimization EIR of every catalog function plus target, PHP profile,
  runtime features, source path, and class ids. Post-optimization is not laziness: the optimizer's
  inliner copies callee bodies into callers, so a key taken before it would change the code of the
  functions that inline the catalog — skipping emission perturbs nothing else.
- `ELEPHC_CATALOG_VERIFY=1` emits anyway and compares with the entry, so a digest that misses an
  input becomes a warning instead of a silent miscompile.
- Still NOT cached: lowering and the EIR optimizer for catalog functions (~24.7 s and ~67.7 s of
  the 208 s baseline). Skipping the optimizer needs the inliner to stop inlining catalog bodies
  into program functions first, which is a design change, not a flag.

## The first a3sl reuse broke the opaque link (7f885cc00d)
- Symptom: on a reuse build the binary was never produced and `ld` reported `_static_*` symbols
  undefined, referenced from both units.
- Cause: the module-level eval helper block (`emit_eval_property_helpers`, static-property helpers,
  inventory, callable support, ...) is emitted in `finalize_user_asm` and lands INSIDE the catalog
  unit. Skipping the catalog functions therefore left the unit with only the helper block, and the
  helper block's own definitions moved, so symbols the program unit names went missing.
- Fix: the cache stores the function bodies only. The helper block is rebuilt on every build (it
  reads module state the key does not cover: slots, class usage), then joined with the cached
  bodies and split using the recorded per-function boundaries.
- Known limitation: after a reuse the split can differ from the one that filled the object cache
  (3 units on the miss, 4 on the reuse in the token fixture), so the catalog is assembled again on
  a reuse. The emission win stands; making the two splits agree needs the emission to record the
  exact span boundaries it would produce, which is the next refinement.
