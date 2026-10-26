---
title: Compact Reflection capacity and regression coverage explain seven lexical checks
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
Keep the compact metadata implementation and the full existing Reflection owner regressions. Interpret the seven findings in compact-empty-metadata-trail-check.log against the actual arithmetic and the assertions, and explain them in the commit message. No old learning or regression fixture is edited to silence these findings.

## Tried and ruled out
Trail matches the new helper comment saying it avoids reserving sixteen buckets as if the code reintroduced that reservation. Its capacity is actually (entry_count * 2).next_power_of_two(): an empty map reserves one bucket. Trail also matches the historical default-heap SPL failure, the successful ReflectionMethod-without-getPrototype control, and typed-property diagnostic narratives against regression names and count assertions. Those are historical observations, not prohibitions on these supported Reflection operations.

## Watch out
All thirteen focused PHP-cross-checked tests pass, including getPrototype, the default-heap SPL graph with equal allocation/free counts, and growth/copy isolation of an empty returned metadata map. Existing small controls retain heap-debug clean assertions. The large full heap-debug artifact is separate from that passing test gate. Trail check remains non-green with seven lexical findings; do not represent it as green or remove tests and comments to make it green. No PHP type variant, property return representation, or metadata ownership policy changes in this capacity-only patch.
