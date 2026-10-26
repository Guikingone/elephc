---
title: Keep append owner retirement at its single-operation boundary
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-container-write-local-owners
touches:
  - src/codegen/lower_inst/arrays.rs
  - src/ir_lower/stmt/array_write_core.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/container_write_local_owners.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
---

## Decided
Scope the widened-source cleanup to the language ArrayPush lowerer. Its lower_array_push_value helper is also called repeatedly for variadic array_push, where a published box becomes the owner between steps; each additional step would require the existing retake lease protocol. Avoid applying single-operation cleanup unconditionally to that shared helper.

## Tried and ruled out
The exploratory PHP variadic fixtures did not reach codegen: the checker rejected a foreach-widened receiver and an array-to-object retype. Their compile logs are retained as non-execution evidence. The language append, object setter and native SPL controls remain executable and require the full final target replay after this placement change.

## Watch out
Trail check's two ownership-narrative matches are lexical: the receiver comment is followed by argument publication through the shared evaluator, and the test preamble describes coverage, not a restored leak. Keep the key-before-RHS-throw and fresh-RHS destructor assertions. Explain those exact matches in the commit and retain the original check log; do not edit old learnings or weaken tests to satisfy the checker.
