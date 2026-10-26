---
title: Owned method receivers need unwind pins through argument and callee throws
author: Guillaumeloulier
date: 2026-10-20
branch: fix/main-method-receiver-unwind
touches:
  - src/ir_lower/expr/method_calls.rs
  - src/ir_lower/expr/nullable_method_calls.rs
  - tests/codegen/runtime_gc.rs
  - ".plans/method-receiver-unwind.md"
  - tests/codegen/runtime_gc/method_receiver_unwind.rs
---

## Decided
Ordinary and already-guarded method dispatch must pin an independently owned receiver after result staging, before arguments run, and through argument/intermediate cleanup. Retire the pin on normal completion, then keep the existing receiver release. Reuse pin_in_flight_owners: it retains nothing, excludes borrowed/local loads and preserves existing alias/result ownership. A result root must stay below the receiver and argument cleanup records.

## Tried and ruled out
Exact clean main df57a9800f4fb61bd4e876f0ddf309b9d08109f2 and current main 4fdabfb8b1cd0b3bc58e789c846cf5970e86490e leak receiver objects when an ordinary method or a later argument throws. EIR acquired a property receiver and released it only after method_call, leaving no unwind-visible owner. Correct stdout did not establish ownership. The dedicated current-main fix is two common lowering paths, not a SPL exception workaround or a change to borrowed receiver reference counts.

## Watch out
receiver-fix-1 under the SPL integration worktree pins CLI 808ab7f1236443d6ca3e3626b6c5a320b788ba11935134dcf48ac2cfcb5f0a41. All eighteen native on/off controls are PHP-exact and heap-clean; nine Rust regressions pass, and all five target checks pass including complete core-runtime Linux ARM64/x86_64 execution. The branch owns a separate target directory to avoid cross-worktree Cargo metadata reuse. Additional raw __get and specialized Reflection invocation paths still have ownership gaps and must not be counted as closed by this method-only fix. SPL next destructor counter/order is also a separate requirement. No commit or publication is authorized.
