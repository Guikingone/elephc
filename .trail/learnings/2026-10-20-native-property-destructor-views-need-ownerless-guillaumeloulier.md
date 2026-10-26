---
title: Native property destructor views need ownerless array graphs and owned getter results
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
The private NativePropertyReplace operation retires the old owner before publication, captures destructor errors, publishes the incoming value and returns Throwable/null. Native getters acquire a logical old-value view instead of retaining the Mixed box at refcount zero. Array structure is captured into a raw memo keyed by source identity, with ownerless value snapshots and only inert string stabilizers. Memo identity handles cycles/shared subgraphs. A zero-ref array observed during a child destructor is materialized into a fresh Mixed-element array; live arrays use the ordinary acquired read. Freed payloads invalidate memo snapshots before allocator reuse, with the allocator volatile registers preserved. M34 verifies the primitive: seven native view cases in both optimizer modes, 57 combined focused regressions, 42 actual macOS/Linux executions and 28 iOS assembly cases, all guarded and clean.

## Tried and ruled out
M32 passed object view, type bypass, pending error publication and resurrection, but the array view failed bad refcount by retaining its retiring source array. Returning only a placeholder array would hide the failure; the added full-content and nested-array controls now preserve actual data and object identity. Runtime assembly alone was insufficient. Test-only artifact export uses ELEPHC_TEST_TARGET in separate processes because target() is cached; ELEPHC_NATIVE_VIEW_ASM_DIR emits checked assembly without claiming execution, then the external verifier links/executes each native target. Production M33/M34 artifacts are identical, and the ordinary host assertion branch is rerun separately.

## Watch out
The full campaign is not complete: selected unchanged heap PHPT remains 27/36, with eight loading failures and one typed by-reference compare failure. Hash borrowed views, clone interception before a retiring-box retain, native name/readonly/dynamic-key loading, public __unserialize validation/insertion/exception chaining and native reference detachment remain required. A clean PHP hash view oracle expects old:text:4|new. __rt_hash_iter_next_value yields borrowed logical values and dereferences tag-11 entries; preserving writable reference identity will require the raw reference protocol rather than pretending value snapshots are reference cells. Keep PHP insertion-order destruction and error chaining intact. No commit or push was performed.
