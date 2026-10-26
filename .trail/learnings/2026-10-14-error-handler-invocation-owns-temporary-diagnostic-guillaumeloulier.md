---
title: Error handler invocation owns temporary diagnostic arguments
author: Guillaumeloulier
date: 2026-10-14
branch: fix/main-error-handler-invocation-owners
touches:
  - src/codegen_support/runtime/error_handlers.rs
  - src/codegen/lower_inst/core_builtins/handlers.rs
  - src/codegen/lower_inst/callables.rs
  - tests/codegen/runtime_gc/handler_registration_owners.rs
---

## Decided
AOT diagnostic dispatch transfers its freshly boxed argument container to __rt_error_handler_invoke_owned_args. The runtime activation releases that owner on normal return and exception unwind. The existing borrowed entry remains for eval/runtime callers that already own their container. An ownership flag is cleared before release, and handler restoration preserves a callback-installed replacement.

## Tried and ruled out
On clean origin/main 62cfeb85ef, three throwing trigger_error handlers retained 24 blocks, and a callback installing a replacement before throwing retained eight blocks. Releasing arguments only after normal invocation bypassed cleanup on longjmp. The owned entry fixes both regressions; all six registration-owner tests pass. The obsolete backend-only release helper has no remaining callers and is removed.

## Watch out
Keep the boxed callback result alive while releasing arguments. Do not make the borrowed entry consume its caller's container. Keep the shared frame ownership slot distinct from the saved callback result. This is a main defect isolated on fix/main-error-handler-invocation-owners; the SPL engine-deprecation caller must use the same owned entry after integration. Native and target validation must be recorded separately from focused macOS tests.
