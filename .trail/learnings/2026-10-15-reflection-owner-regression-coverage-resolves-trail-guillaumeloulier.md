---
title: Reflection owner regression coverage resolves Trail lexical conflicts
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-reflection-metadata-owners
touches:
  - src/codegen/lower_inst/objects/reflection/member_properties_emit.rs
  - src/codegen/lower_inst/objects/reflection/owner_emission.rs
  - src/codegen/lower_inst/objects/reflection/parameter_property_emit.rs
  - src/codegen/lower_inst/objects/reflection/string_attrs_emit.rs
  - src/ir_lower/stmt/return_coercions.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/reflection_metadata_owners.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
---

## Decided
The Reflection producer-owner regressions intentionally retain getPrototype and typed-property metadata coverage. Trail check flags their words as reviving rejected approaches, but both are positive regression controls: the implementation now includes unwind-aware retirement of Mixed source cells after heap-return conversion, alongside fresh metadata boxing transfers.

## Tried and ruled out
The earlier observation ruled out changing Reflection boxing alone as a complete getPrototype fix. It did not rule out getPrototype as a supported PHP API or as a regression test. The typed-array-property note concerned a separate inference fixture and did not prohibit typed ReflectionProperty metadata.

## Watch out
Trail check has lexical false positives for these tests. Eleven regressions with PHP cross-check and 33 native target executions have matching output and clean heaps. Keep these regression assertions; do not delete them merely to silence the check. The 430-test broader run and full campaign acceptance remain separate gates.
