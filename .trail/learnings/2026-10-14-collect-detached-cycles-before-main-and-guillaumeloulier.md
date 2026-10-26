---
title: Collect detached cycles before main and request frames return
author: Guillaumeloulier
date: 2026-10-14
branch: fix/main-shutdown-cycle-collection
touches:
  - src/codegen/frame.rs
  - tests/codegen/runtime_gc/cow_and_cycles.rs
  - tests/web_tests.rs
---

## Decided
A main-return epilogue must explicitly collect cycles after local, static-local, global and static-property owners retire, before resource cleanup and before leaving the main activation. A web handler must collect detached request-local cycles after local cleanup while persistent global roots remain registered. Use the shared target-aware ABI call to __rt_gc_collect_cycles_explicit so gc_disable() does not suppress shutdown collection.

## Tried and ruled out
Root retirement alone leaves detached cycles alive: on origin/main 62cfeb85ef, two focused regressions print only body instead of body followed by the cycle destructor. The SPL campaign independently reproduced leaked self-backed ArrayObject cycles, but its constructor regression depends on branch-specific SPL semantics and belongs in that feature branch, not this main fix.

## Watch out
Process exit and persistent-library lifecycle are different; do not require a process-exit collector in an iOS static-library entry point. Keep destructor callbacks inside the active instrumentation frame. The integration branch verified native macOS ARM64, Linux ARM64 and Linux x86_64 cycle/disabled-GC probes with balanced heap counts, and assembled both iOS targets; that is not native iOS SDK validation. The broad integration destructor filter has an independently reproduced pre-existing reference-property replacement failure before the main epilogue, so do not claim the whole suite or PHP shutdown semantics are fully compliant from this fix.
