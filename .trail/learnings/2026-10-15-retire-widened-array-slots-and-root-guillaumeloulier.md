---
title: Retire widened array slots and root object setter operands
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-container-write-local-owners
touches:
  - src/codegen/lower_inst/arrays.rs
  - src/ir_lower/stmt/array_write_core.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
  - ".artifacts/constructor-unwind-owners/fused.php.stderr"
  - ".artifacts/constructor-unwind-owners/fused.php.stdout"
  - ".artifacts/constructor-unwind-owners/green-property.log"
---

## Decided
Before a consuming indexed append, call the existing release_mutated_source_local_owner hook for its source local. A concrete array extracted from physical Mixed storage owns a lease already, so the old box must retire before the helper consumes that lease and publishes the replacement. This runs through the shared backend for all targets.

For object dimension setters, publish receiver, key and value temporary owners before later expressions can throw. Retire argument records in reverse evaluation order and then the receiver. Share the existing key/value evaluator: bare-variable keys are read after the RHS, while expression keys run before it. Other array/property callers keep their original evaluation and ownership policy.

## Tried and ruled out
On origin/main a1a8428a894bf4cb66650ab682834c328758eae0, late-boxed append retained two blocks / 112 bytes and indexed object writes retained six / 800 bytes. Fixing the receiver alone left a four-block fresh RHS tree and a one-block string key when RHS evaluation threw. A release after a consuming append would target a different ownership boundary. Neither resetting a reused local nor dropping heap assertions repairs those causes.

## Watch out
Eight GC controls match PHP with clean heaps, including key/RHS exceptions, destructor order, exact key evaluation order and native SPL setters. All eight also execute cleanly on macOS ARM64 and both Linux architectures; programs and runtime assemble for every supported target. iOS SDK/native acceptance remains separate. Builtin/property and dynamic boxed receiver write paths have their own contracts; this result does not close those families or Throwable trace-argument retention. The separate mixed-method return box fix remains on its own main branch.
