---
title: Reflection metadata maps reserve capacity from their actual entry counts
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-reflection-metadata-owners
touches:
  - src/codegen/lower_inst/objects/reflection.rs
  - src/codegen/lower_inst/objects/reflection/collection_emit.rs
  - src/codegen/lower_inst/objects/reflection/default_emit.rs
  - tests/codegen/runtime_gc/reflection_metadata_owners.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
---

## Decided
Reflection associative metadata maps reserve a power-of-two capacity from twice the known entry count, with capacity one for an empty map. This keeps full ReflectionClass metadata for SplDoublyLinkedList within the unchanged default 8 MiB heap. Ordinary hash growth and copy-on-write stay authoritative for maps returned to PHP.

## Tried and ruled out
On the pinned owner-fixed compiler, the SPL Reflection probe exhausted the default heap. A 64 MiB diagnostic completed with 68,809 allocations/frees, no live blocks and a 15,340,288-byte peak: the graph is finite and retaining a larger heap is not the fix. Reserving sixteen buckets for every empty metadata slot wastes most of that footprint. Compact maps produce the right default-heap output; the first full heap-debug test hit its 60-second timeout during cleanup, after printing 0:0.

## Watch out
Thirteen focused PHP-cross-checked regressions pass, including the default-heap SPL graph with equal allocation/free counters and an empty returned map growing to twenty entries without modifying its copy or the metadata source. Small owner controls still use full heap debug. Full heap-debug validation of the large graph needs a separately timed artifact because validating the entire free list on each release is quadratic. Do not count the timed-out first run or the enlarged-heap diagnostic as final default-heap proof. All-target verification and the separate eval ReflectionParameter owner discrepancy remain open.
