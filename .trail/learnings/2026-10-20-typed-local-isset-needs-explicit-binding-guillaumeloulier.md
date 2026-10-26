---
title: Typed local isset needs explicit binding presence across interrupted stores
author: Guillaumeloulier
date: 2026-10-20
branch: work/isset-assignment-main-baseline
touches:
  - src/ir_lower/context.rs
  - src/ir_lower/expr/lazy_isset.rs
  - src/ir_lower/function.rs
  - src/types/checker/builtins/language_constructs.rs
  - src/types/checker/driver/init.rs
  - src/types/checker/driver/top_level.rs
  - src/types/checker/mod.rs
  - src/types/checker/null_probe.rs
  - tests/codegen/runtime_gc.rs
  - tests/error_tests/type_system.rs
---

## Decided
Track PHP local binding presence independently from its ordinary scalar payload. Lazy bare-local isset checks a hidden EIR integer flag before loading the value and testing nullness. Seed flags in function entry before a throwing RHS, using parameter slot identity rather than its source name. Instrument final EIR binding writes and clears so a late-discovered probe covers earlier stores and loop back edges; include flag state in speculative lowering snapshots and repair ValueDef positions after entry insertion. Newly declared Void probe loads after an abandoned binding need final physical slot typing for later loop iterations; ordinary reads retain their source-order type. Null raw ref-cell promotion also needs Mixed payload storage to distinguish null from integer zero without reboxing an already-bound external reference.

## Tried and ruled out
Zero, false and empty string are valid PHP values, so none can encode missing storage. The original typed string isset assumed every Str load existed after a throwing assignment on clean main df57a9800f4fb61bd4e876f0ddf309b9d08109f2. Optimized and unoptimized IR both retained the incorrect load. Seeding by parameter name misidentified a fresh binding after unset. Loosening all null-probe checker rules would also admit empty/coalesce/access-chain shapes without a corresponding safe storage path; accept only the bare-local isset sites covered by the new EIR lowering.

## Watch out
A6 evidence is in the SPL/OPcache main integration worktree under .artifacts/integration/isset-fix-a6. Native PHP/heap controls and 35 supported-target assembly/link controls pass, including 14 complete core-runtime Linux ARM64/x86_64 executions with exact PHP output and clean heaps. Rust focused suites are still pending at note creation. Undefined reference source/out-param initialization and array push after unset still fail compilation and are retained as open baseline gates. This bounded fix does not establish the 1710 PHPT campaign score, Fiber ownership, fatal diagnostics, clone behavior or SPL heap debug view parity. No commit or publication is authorized.
