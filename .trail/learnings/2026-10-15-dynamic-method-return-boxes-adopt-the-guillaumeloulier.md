---
title: Dynamic method return boxes adopt the callee reference
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-mixed-method-result-owners
touches:
  - src/codegen/lower_inst/method_resolution.rs
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
Concrete ordinary method returns carry a caller-owned reference. When store_call_result adapts that return to a Mixed/union slot, use emit_box_current_owned_value_as_mixed, which retires the original reference after the box acquires it. Preserve the separate by-reference-return path.

## Tried and ruled out
On origin/main a1a8428a894bf4cb66650ab682834c328758eae0, a dynamic receiver choosing an associative or indexed array method retained four blocks / 1240 bytes. A fresh object return missed its destructor before the next statement. Borrowed boxing adds a second unclaimed payload reference; changing caller cleanup or weakening the heap assertion would hide the producer defect. Typed array property metadata is needed for the shared-property count control; the original untyped fixture failed inference before execution.

## Watch out
Three new GC regressions and 14 related mixed-method tests pass with PHP cross-check. Nine native cases across macOS ARM64, Linux ARM64 and Linux x86_64 match PHP and have clean heaps; runtime/program assembly covers all five targets. iOS assembly is not an SDK/native acceptance result. The SPL mixed-wrapper regression also contains a separate reused-local write ownership failure; do not claim this result fixes that boundary. Trail's full caller graph includes interface, callable-array, static and intrinsic consumers of store_call_result; keep them on the shared owning-result contract.
