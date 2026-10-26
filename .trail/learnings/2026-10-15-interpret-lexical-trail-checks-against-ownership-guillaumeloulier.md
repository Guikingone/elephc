---
title: Interpret lexical Trail checks against ownership regression narratives
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-mixed-method-result-owners
touches:
  - src/codegen/lower_inst/method_resolution.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/mixed_method_results.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
  - ".artifacts/constructor-unwind-owners/green-property.log"
---

## Decided
The 2026-10-15 mixed-method result change uses the existing owning conversion at method_resolution.rs and retains heap assertions in its regression tests. Trail check against HEAD reports two lexical conflicts, not semantic reversions; explain them in the commit alongside red/green evidence.

## Tried and ruled out
The checker matches the repair comment describing the redundant lease and the test preamble describing dispatch coverage against the learning's historical failure narrative. Neither match identifies a newly introduced ownership path. The old and corrected binaries were executed against the same fixtures before committing.

## Watch out
Keep the original diagnostic and this interpretation. Do not alter fixtures, remove assertions, or edit an old learning to obtain a clean automated result. This interpretation applies to those two exact matches; inspect future warnings independently.
