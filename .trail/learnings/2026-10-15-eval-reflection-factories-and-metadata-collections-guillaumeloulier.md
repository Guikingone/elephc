---
title: Eval Reflection factories and metadata collections transfer producer owners
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-reflection-metadata-owners
touches:
  - crates/elephc-magician/src/interpreter/reflection/owner_materialization.rs
  - src/codegen/eval_reflection_owner_helpers.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/eval_operand_owners.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
---

## Decided
The eval Reflection factory transfers its fresh raw object reference into the returned Mixed cell through emit_box_current_owned_value_as_mixed on both architectures. Metadata arguments remain borrowed and their existing field retains stay intact. Reflection parameter, named-type, member and class-map collection producers use EvalArrayBuilder so a borrowed array_set does not strand fresh key/value owners; typed string-array construction retains its existing representation.

## Tried and ruled out
AOT metadata adoption alone left the eval-created parameter fixture at 21 retained blocks. The native eval factory still boxed a fresh object through the retaining conversion. Fixing that makes the direct parameter graph completely clean (86 allocations and frees); the old baseline-plus-seven assertion then failed because zero is stricter than eight. It is replaced by zero plus a clean-heap assertion, with the AOT control preserved.

A richer ReflectionMethod fixture then exposed 52 retained blocks/4,896 bytes even though an empty class was clean. Fresh parameter objects and keys were passed to borrowed collection stores without retiring their producer references. Reusing the existing builder closes the class-with-method and method-with-typed-parameter graphs. No hand-decrement or signature/shape change is required.

## Watch out
The strengthened parameter control and both new class/method heap controls pass with PHP cross-check. The earlier 429/430 broad result is historical; it has not been rerun as a whole after these fixes. The AOT control's separately observed retained block is still a diagnostic, and other reflection API collection producers outside owner_materialization remain to audit. Real all-target eval execution and full PHPT acceptance remain open. Preserve the strict assertions and the original failed diagnostics; do not restore the offset-based allowance.
