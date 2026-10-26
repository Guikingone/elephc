---
title: Diagnostic owner regressions do not revive rejected cleanup
author: Guillaumeloulier
date: 2026-10-14
branch: fix/main-error-handler-invocation-owners
touches:
  - src/codegen_support/runtime/error_handlers.rs
  - tests/codegen/runtime_gc/handler_registration_owners.rs
---

## Decided
Interpret Trail conflict snippets against the full learning and actual assertions. The owned diagnostic invocation entry is the chosen repair, and throwing-handler fixtures are required regressions. Neither brings back the bypassed normal-return-only cleanup.

## Tried and ruled out
The current check flagged the phrase about the owned entry inside the learning's successful outcome as if it had been rejected. It also matched the clean-main throwing fixture against the red baseline narrative. Both are lexical false positives: the new cleanup consumes the argument owner during unwind, and the tests require zero live heap blocks after the same input.

## Watch out
Record this rationale in the commit instead of deleting a regression or editing an immutable learning. Evidence for this change is six focused tests, a warning-free build, six matching native executions with clean heaps, and assembly for all five targets. The iOS archive/native SDK gate is separate and remains open. Do not call the Trail check automatically clean.
