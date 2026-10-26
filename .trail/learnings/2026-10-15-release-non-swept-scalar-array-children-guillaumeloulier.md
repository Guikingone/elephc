---
title: Release non-swept scalar array children during cycle cleanup
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-gc-scalar-array-leaves
touches:
  - src/codegen_support/runtime/arrays/decref_any.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
  - ".artifacts/constructor-unwind-owners/green-property.log"
  - ".artifacts/constructor-unwind-owners/green.log"
---

## Decided
During final cycle sweep, decref_any suppresses child decrements only for representations the collector sweeps independently. Indexed arrays with scalar/string/other non-graph element tags must follow ordinary refcount release even when their owning object is being collected. Match the existing collector bounds (array payload tags 4 through 7), preserving kind and the persistent COW bit handling on both architectures.

## Tried and ruled out
A clean dbea935750 regression with an untyped integer-array property leaves one 72-byte block after explicit collection. A typed PHP array property already releases correctly because its boxed path reaches ordinary array cleanup. Changing the property type or omitting heap checks only hides the concrete-storage defect. Expanding the collector's graph set is unnecessary; the missing operation is the leaf owner's decrement.

## Watch out
Eight focused cases cover integers, floats, booleans, strings, empty arrays, external sharing, a typed-array guard and object-array destructor behavior. The 37-case GC/resurrection selection passes. Both existing decref emitter unit controls pass across their supported target loops, the warning-free build passes, and all 24 native executions match PHP with clean heaps on macOS ARM64 and both Linux architectures. All eight programs and runtime assemble for the five supported targets; iOS SDK/native acceptance remains separate. If collector eligibility changes later, keep sweep suppression consistent with it. Do not skip decrements of non-swept leaves, and do not turn cyclic array children back into recursive frees.
