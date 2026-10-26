---
title: Final cycle collection must precede eval context disposal
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-gc-cycle-scheduling
cost: { minutes: 793, tokens: 6876893 }
touches:
  - src/codegen/frame.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/gc-cycle-scheduling/control-callers.log"
  - ".artifacts/gc-cycle-scheduling/direct-controls/acyclic_release_stays_immediate.stderr"
  - ".artifacts/gc-cycle-scheduling/direct-controls/explicit_collection_while_disabled.stderr"
  - ".artifacts/gc-cycle-scheduling/direct-controls/results.json"
  - ".artifacts/gc-cycle-scheduling/direct-controls/shutdown_after_function_return.stderr"
  - ".artifacts/gc-cycle-scheduling/direct-controls/shutdown_after_overwrite.stderr"
  - ".artifacts/gc-cycle-scheduling/direct-controls/shutdown_with_automatic_gc_disabled.stderr"
  - ".artifacts/gc-cycle-scheduling/eval-declaration-lifetime/acyclic_unset"
---

## Decided
Normal CLI shutdown forces gc_collect_cycles_explicit after local, static and global roots retire and before resource inventory reset. Request teardown uses the same final collection. Dispose eval contexts afterwards so cyclic destructors can still invoke opaque eval. This final pass ignores gc_disable, which controls automatic collection only. Keep acyclic refcount destruction immediate.

## Tried and ruled out
Clean main d354273274 leaks self-cycles and misses destructors after last-owner overwrite, function return, or disabled automatic GC. Unset instead collects too early; that scheduling defect is separate. The initial direct-control run used the cloned historical de94 CLI, not rebuilt source, and is explicitly marked baseline evidence. Seven strict PHP/heap tests pass in the rebuilt implementation, including explicit main return and a main eval bootstrap plus an eval-using destructor. Native web compilation also includes final collection.

## Watch out
Do not replace PHP possible-root thresholds with live native heap-block counts: current gc_status threshold is zero and roots counts native graph nodes. Automatic scheduling and exit/die/fatal shutdown remain open. Opaque eval declarations also fail across function contexts on clean main, while repeated eval in the same frame works; preserve that separate baseline rather than weakening shutdown tests. Broader cleanup/exception tests and target-native evidence remain required at this checkpoint.
