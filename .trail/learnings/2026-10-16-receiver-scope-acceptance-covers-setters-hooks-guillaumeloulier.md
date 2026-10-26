---
title: Receiver scope acceptance covers setters hooks and eval
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-nested-property-receiver-owners
touches:
  - docs/internals/the-codegen.md
  - src/ir_lower/stmt/instance_property_writes.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/nested-property-receiver-owners/build-final.jsonl"
  - ".artifacts/nested-property-receiver-owners/build-final.log"
  - ".artifacts/nested-property-receiver-owners/chain_assignment"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.compile"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.php"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.s"
  - ".artifacts/nested-property-receiver-owners/magic_receiver"
---

## Decided
The bounded property receiver scope is validated on ordinary, temporary and magic paths, including RHS throws, store TypeError and throwing magic setters. The final extended setter/hook/eval run is terminal at 46/46 and the independently pinned seven-fixture matrix is terminal at 21/21 native clean heaps plus five-target assembly.

## Tried and ruled out
The original hydration back-reference test exposed setup lifetime loss before serialization. Preserve it intact and integrate this receiver correction as a prerequisite. The scalar parser owner and overwritten-default defects remain separate hydration work.

## Watch out
Evidence is .artifacts/nested-property-receiver-owners/v2-setter-paths.log and target-matrix/results.json. Eval cold bridge preparation made the sequential extended run slow; its terminal result is green, not a timeout. Real iOS execution, source interface reference-return compatibility and full php-src acceptance remain open.
