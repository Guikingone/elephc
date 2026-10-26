---
title: Untyped physical property slots need Mixed GC and JSON descriptors
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-untyped-property-gc-descriptors
touches:
  - docs/internals/the-codegen.md
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/system/json_encode_mixed.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/untyped-property-gc-descriptors/callers.log"
  - ".artifacts/untyped-property-gc-descriptors/declared-slot-api.log"
  - ".artifacts/untyped-property-gc-descriptors/direct-null-crashes.json"
  - ".artifacts/untyped-property-gc-descriptors/eval-composition.log"
  - ".artifacts/untyped-property-gc-descriptors/existing-json-controls.log"
  - ".artifacts/untyped-property-gc-descriptors/existing-json-tests.log"
---

## Decided
Treat a physical property slot whose inferred type is Void and whose declaration has no type as boxed Mixed (descriptor tag 7) in both GC and JSON metadata. Use ClassInfo::property_slot_is_declared(index, name), which preserves the distinction for inherited physical slots. JSON Mixed dispatch must test both zero and the encoded null sentinel before dereferencing a cell; reuse emit_branch_if_null_container on ARM64 and x86_64.

## Tried and ruled out
Metadata alone fixed the restored string, array and cycle leaks but made untyped null JSON dereference its sentinel and crash. Direct runs confirmed SIGSEGV; the test helper's LLDB timeout was secondary. A guessed declared_property_types field does not exist; the physical-slot API is authoritative. Fixtures assigning a concrete value in PHP source refine the slot and hide this defect, so restoration fixtures have no source assignment.

## Watch out
The focused gate passed 19 strict hydration/GC controls, the opaque-eval composition control and 28 existing JSON controls; a unit checks descriptors on all five targets. Native matrix acceptance and full php-src acceptance are separate gates. This does not fix reference-return adapters, shutdown destructor timing, unsupported property-key/type diagnostics, or real iOS execution. Preserve failed probes and unchanged PHPT expectations.
