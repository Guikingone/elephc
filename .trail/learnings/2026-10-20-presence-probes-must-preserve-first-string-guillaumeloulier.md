---
title: Presence probes must preserve first string storage and captured reference ABI
author: Guillaumeloulier
date: 2026-10-20
branch: work/isset-assignment-main-baseline
touches:
  - src/ir_lower/context.rs
  - src/ir_lower/context/local_presence.rs
  - src/ir_lower/expr/lazy_isset.rs
  - src/ir_lower/function.rs
  - src/types/checker/builtins/language_constructs.rs
  - src/types/checker/driver/init.rs
  - src/types/checker/driver/top_level.rs
  - src/types/checker/mod.rs
  - src/types/checker/null_probe.rs
  - tests/codegen/runtime_gc.rs
---

## Decided
A bare isset probe may need to allocate an uninitialized placeholder slot, but it must not force a subsequent fresh string binding into Mixed storage. Specialize only an unstored probe placeholder to its first concrete declaration, retaining ordinary widening after a real null/value store. Defer dedicated Void probe-load metadata to final physical storage for loop back edges, and use Ownership::php_type_needs_lifetime_tracking for Borrowed metadata: Str, Callable and Buffer are tracked even though PhpType::is_refcounted excludes them. Keep ordinary PHP reads and existing external reference ABIs unchanged.

## Tried and ruled out
A6 passed all 70 isset tests and four checker tests but regressed test_core_reference_capture_unset_rebind_preserves_old_and_new_strings: the old closure survived, while the new closure lost new1/new2. Exact clean main retained both strings. The provisional Void slot had widened the new string capture to a Mixed payload. A7 then exposed OwnershipTypeMismatch when the deferred load became Str; fixing the lifetime predicate, rather than broadening is_refcounted or boxing an already-bound external reference, resolves it. Preserve the negative artifacts and never treat the earlier focused green group as acceptance.

## Watch out
A9 is pinned under the SPL/OPcache integration worktree .artifacts/integration/isset-fix-a9: CLI 843bedece6a66951900570dba14b13dfb7602ef0b1f129bb0252d5aea5f20f7e. Nineteen presence, five capture-owner and four checker Rust tests pass. Forty supported-target checks pass, including sixteen Linux ARM64/x86_64 executions, with exact PHP output and clean heaps. Integration M17 rebuild and print_r destructor replay remain pending at note creation. Undefined references, array push after unset, nullable-branch capture and Fiber ownership retain baseline failures; full 1710-PHPT compliance is not established. No commit or publication is authorized.
