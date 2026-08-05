# Oversized source-file refactor

## Task checklist

- [x] Record the authoritative oversized-file inventory and cohesion rules.
- [ ] Split AST-to-EIR expression and statement lowering into focused modules.
- [ ] Split EIR program/function/context responsibilities where files mix state,
      discovery, declaration, and lowering concerns.
- [ ] Reduce EIR codegen dispatcher and facade files to routing/orchestration.
- [ ] Split codegen builtin families for I/O, arrays, strings, objects,
      Reflection, and eval along semantic boundaries.
- [ ] Split checker-owned Reflection and date/time synthetic metadata by class or
      metadata family.
- [ ] Split eval, OPcache, and PHAR files that combine independent analysis,
      rendering, archive-format, or FFI responsibilities.
- [ ] Slim the remaining pipeline, optimizer, parser, runtime, Magician, and
      native-dependency orchestrators.
- [ ] Split oversized test files only when they mix independent feature areas;
      preserve cohesive API-surface and regression corpora.
- [ ] Run focused validation after every slice, then perform a final build,
      hygiene check, and residual oversized-file audit.
- [ ] Document why every production Rust file still above 500 physical lines is
      a cohesive leaf, generated/source data, or an intentionally indivisible
      target-specific emitter.

## Goal

Reduce oversized, mixed-responsibility source files into cohesive Rust modules
without changing PHP behavior, EIR contracts, emitted assembly, supported-target
coverage, runtime ownership, or public APIs. The repository's 500-line policy is
a warning sign rather than a mechanical hard limit, so the completion criterion
is not that every file becomes shorter than 500 lines. Completion requires every
remaining file above the threshold to have one clear responsibility for which a
further split would create artificial fragmentation.

The baseline is branch `refactor/split-oversized-files` at
`93d0726d6a7852c70133b6e9dd4de03d54bc05e8`. The initial tracked-file census
found 382 files above 500 physical lines: 251 production Rust files, 95 test
files, 22 documentation/plan files, 7 generated/data/asset files, 6 tooling/CI
files, and one showcase source file.

## Scope

### Structural dispatchers and orchestrators

These are mandatory refactor targets even when they are only slightly above the
threshold, because the repository policy explicitly requires coordinators to
stay slim:

- `src/ir_lower/expr/mod.rs`
- `src/ir_lower/stmt/mod.rs`
- `src/ir_lower/program.rs`
- `src/codegen/lower_inst.rs`
- `src/codegen/lower_inst/builtins.rs`
- `src/codegen/mod.rs`
- `src/types/checker/inference/expr/mod.rs`
- `src/codegen_support/runtime/emitters.rs`
- `src/pipeline.rs`
- `src/codegen_support/mod.rs`
- `src/optimize.rs`
- `src/types/checker/functions/resolution/mod.rs`
- `crates/elephc-magician/src/interpreter/statements/dispatch.rs`
- `src/types/checker/driver/mod.rs`
- `src/parser/stmt/mod.rs`
- `src/native_deps/orchestration.rs`

### Mixed feature-family containers

These files have clear internal seams and must be split without changing their
existing entry points:

- `src/codegen/lower_inst/builtins/io.rs`
- `src/codegen/lower_inst/objects/reflection.rs`
- `src/codegen/lower_inst/builtins/arrays.rs`
- `src/codegen/lower_inst/builtins/eval.rs`
- `src/codegen/lower_inst/objects.rs`
- `src/codegen_support/runtime/eval_bridge.rs`
- `src/types/checker/builtin_types/reflection.rs`
- `src/eval_aot.rs`
- `src/opcache_prelude.rs`
- `crates/elephc-phar/src/lib.rs`
- `src/types/checker/builtin_types/datetime.rs`
- `src/codegen/lower_inst/builtins/strings.rs`

### Conditional targets discovered during implementation

Other production or test files above 500 lines enter scope when inspection shows
that they mix dispatch, validation, state collection, emission, unrelated
features, or independent test surfaces. They remain out of scope when inspection
shows a single cohesive algorithm, runtime routine, target implementation,
registry, generated body, or API-compatibility corpus.

## Explicit cohesive exclusions

The initial audit identified these representative files as cohesive despite
their size. They are not to be mechanically split unless implementation work
uncovers a real responsibility boundary:

- `src/image_prelude.rs`: one embedded PHP image surface; treat extraction of
  source data separately from Rust modularity.
- `src/opcache/directives.rs`: the versioned OPcache directive registry.
- `src/ir/runtime_fn.rs`: the typed runtime-function inventory.
- Target-specific single-routine assembly emitters such as
  `src/codegen_support/runtime/system/date/arm64.rs` and
  `src/codegen_support/runtime/system/date/linux_x86_64.rs`.
- Single-algorithm emitters such as `serialize.rs`, `unserialize.rs`,
  `stream_filter.rs`, and `file_get_contents_url.rs`.
- Generated registries, data files, lockfiles, documentation, plans, binary
  assets, and intentionally exhaustive API-surface fixtures.

## Refactor rules

- Preserve public and crate-visible entry points unless a caller migration is
  part of the same slice.
- Move code before changing it; behavior changes are outside this plan.
- Every new Rust file starts with the required module-level Rustdoc preamble,
  and every moved or added function keeps a specific `///` docblock.
- Do not run `cargo fmt`; preserve assembly-comment alignment exactly.
- Keep every supported target in the same module split. A module boundary must
  not create target-specific semantic drift.
- Preserve ownership, cleanup, ABI, runtime symbol, EIR validation, and source
  evaluation-order contracts.
- Keep commits thematic and independently buildable.

## Validation

For each slice:

1. Run `cargo build` or the narrow package build covering the moved code.
2. Run the smallest focused test binary/filter for the affected surface.
3. Run `./scripts/check_asm_comments.py` for touched assembly emitters.
4. Run `git diff --check`.

At completion:

1. Recompute physical line counts from `git ls-files`.
2. Inspect every remaining production Rust file above 500 lines and record its
   cohesion rationale.
3. Run `cargo build` and the focused suites accumulated by the slices.
4. Run broader tests only where the cross-cutting module moves cannot be covered
   responsibly by focused tests; otherwise rely on CI for the full target matrix.
5. Verify the worktree contains no unintended generated or unrelated changes.
