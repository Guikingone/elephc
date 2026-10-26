---
title: Strict eval factory controls explain seven historical lexical findings
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-reflection-metadata-owners
touches:
  - crates/elephc-magician/src/interpreter/reflection/owner_materialization.rs
  - src/codegen/eval_reflection_owner_helpers.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/eval_operand_owners.rs
  - tests/codegen/runtime_gc/eval_reflection_factory_owners.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
---

## Decided
Keep the eval factory owning conversion, the borrowed-argument retains, the shared collection builder and all three clean-heap controls. Interpret the seven findings in eval-owner-factory-trail-check.log against those actual lifetime operations. Explain the categories in the commit message and keep the original diagnostics.

## Tried and ruled out
The findings associate repaired producer-lifetime comments and regression names with earlier failure descriptions. They also associate the rich method fixture with the old prototype and default-heap observations. Those observations describe previous program states. The code now adopts a newly allocated object and retires collection temporaries through the existing guarded builder. The tests require zero retained blocks rather than the old offset-based floor.

## Watch out
Seven initial findings are lexical. The strict parameter fixture and the class/method fixtures all pass with PHP output checks and clean heaps. Metadata arguments remain borrowed, typed string-array storage is unchanged, and the current method fixture includes a typed parameter and nested declaring metadata. These local controls do not close cross-target eval, the separately observed AOT control diagnostic, or the full corpus. Do not remove assertions or old notes to make this check appear green.
