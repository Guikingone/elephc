---
title: SPL hydration hooks are indirect edges and require borrowed argument owners
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - examples/spl-containers/main.php
  - src/codegen/lower_inst/method_intrinsics.rs
  - src/codegen_support/runtime/spl/doubly_linked_list.rs
  - src/ir_lower/program/spl_discovery.rs
  - tests/codegen/runtime_gc/spl_list_subclass_storage.rs
---

## Decided
Treat the typed Unserialize runtime operation as an indirect method edge. Wire data may name any declared class, so retain supported SPL __unserialize/__wakeup implementations even when no source call mentions them. Keep this EIR discovery change target-neutral and separate from native storage layout. Integrate main fix 6bcf307b82 for borrowed direct intrinsic arguments and serialized-state producer arrays.

## Tried and ruled out
The immutable b74 replay has 29/37 unchanged DLL tests passing. A direct __unserialize call restores an item, while the builtin path has a zero magic pointer and omits the body entirely. Retaining the hook restores the two element slots but initially yields null payloads: the direct base-class push bypassed the borrowing method adapter and stole the foreach variable's Mixed reference. Merely retaining the hook is insufficient. Both dispatch paths and snapshot ownership are now fixed; no PHPT source or expectation was edited.

## Watch out
Builtin round-trip and wire-only hydration controls pass with clean heaps. The full integrated list filter is 31/31 with the general destructor-timing case explicitly still registered and excluded; the existing SPL class surface is 34/34. The main ownership artifact is independently pinned for 81 native cases plus five-target assembly; it is not the integrated target result. The completed full corpus remains 639/1710 until a fresh full replay. Keep b74 source, compiler, nine bridge archives and its 37-test results immutable. Fresh integrated pinning/replay and native iOS remain open.
