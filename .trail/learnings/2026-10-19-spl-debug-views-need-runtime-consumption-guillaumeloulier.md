---
title: "SPL debug views need runtime consumption, lazy retention and asymmetric storage checks"
author: Guillaumeloulier
date: 2026-10-19
branch: work/spl-opcache-runtime-integration
touches:
  - crates/elephc-magician/src/context.rs
  - crates/elephc-magician/src/context/classlike_objects.rs
  - crates/elephc-magician/src/context/closure_metadata.rs
  - crates/elephc-magician/src/context/closure_owners.rs
  - crates/elephc-magician/src/context/core.rs
  - crates/elephc-magician/src/context/reflection_registry.rs
  - crates/elephc-magician/src/ffi/dynamic_destructors.rs
  - crates/elephc-magician/src/interpreter/expressions.rs
  - crates/elephc-magician/src/interpreter/expressions/closures.rs
  - crates/elephc-magician/src/interpreter/expressions/evaluation.rs
---

## Decided
The PR 1806 integration worktree combines the SPL feature, the native debug consumer, handler shutdown fixes and Magician capture-owner deltas without overwriting their original worktrees. Native var_dump must invoke __debugInfo before printing the object header and retain the receiver through the hook. A Mixed wrapper lease must release the original boxed owner, not its raw object payload. Release the owned result wrapper after retaining the returned array so by-reference views do not stay rooted too long. Keep the native hook reachable through src/ir_lower/program/spl_metadata.rs even when PHP only calls var_dump.

## Tried and ruled out
Metadata generation and a passing direct __debugInfo call did not prove native rendering: an unchanged upstream SplObjectStorage_var_dump PHPT rendered an empty object until lazy method retention was fixed. With integration V3, that PHPT and gh18322 pass unchanged, and all fifteen renderer controls are PHP-exact and heap-clean. Five supported runtime assemblies also pass; this does not establish Linux execution or iOS runtime parity. The new ArrayObject/ArrayIterator getter controls expose a clean COW mismatch: PHP storage writes remain visible through an existing debug view, but writing that view itself detaches it. A PHP reference would incorrectly make both directions live.

## Watch out
The unchanged bug69264 now reaches incorrect existing SplHeap/SplPriorityQueue debug return representations; their body schemas and physical heap order also need correction. The initial SplFileInfo getter still needs root/trailing-slash and filesystem-subclass coverage. Do not reuse isolated Magician V6 Linux archives for the combined branch: the campaign bases differ in 79 Magician files. Filtered PHPT complete=false results never update the full 1710-test acceptance score. Check .plans/runtime-integration-1806.md and .artifacts/integration/v3 for exact compiler/archive hashes, native failures and the pending Cargo results. No publication has been performed.
