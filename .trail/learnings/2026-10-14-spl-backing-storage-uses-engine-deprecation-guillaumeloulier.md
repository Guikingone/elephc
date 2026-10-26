---
title: SPL backing storage uses engine deprecation severity
author: Guillaumeloulier
date: 2026-10-14
branch: feat/spl-opcache-php-src-1806
touches:
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - crates/elephc-builtin-contract/src/support.rs
  - src/types/checker/builtin_spl_classes/storage.rs
  - src/builtins/system/core_support.rs
  - src/codegen/lower_inst/core_builtins/handlers.rs
  - src/ir_lower/context.rs
  - crates/elephc-magician/src/interpreter/builtins/core/runtime_introspection.rs
  - tests/codegen/spl/storage_subclasses.rs
---

## Decided
SPL object-backed storage diagnostics are engine deprecations (E_DEPRECATED 8192), not user-triggered E_USER_DEPRECATED. The compiler-internal __elephc_deprecated contract binds a typed CoreBuiltinOp::InternalDeprecated AOT path and a documented eval adapter. Public trigger_error validation remains limited to its four user levels. The shared contract explicitly allows this internal entry in eval, with direct and values dispatch.

## Tried and ruled out
The original severity/mask regressions were red. Three native semantic controls, the throwing-handler heap control, two eval controls, three contract-support checks and seven builtin parity checks now pass. The first display test assumed stdout; the integration harness's log is stderr. The CLI's default display is stdout, so the corrected native mask matrix compares against PHP with explicit display/log settings. An ini_set fixture was invalid in the narrow harness and is retained as invalid evidence. Native callback arguments and incomplete constructor receivers required independent main fixes; they are isolated and integrated with their own provenance.

## Watch out
A diagnostic can invoke arbitrary user code and throw; the typed operation must participate in opaque-call operand roots. Preserve synthetic caller lines and use the owned handler argument boundary. Eval's default mask must select 8192 without accepting that level in trigger_error. Whole focused SPL is 363/363, and all four diagnostic probes have clean heaps on the three executable targets; all five targets assemble. The fresh 108-case ArrayObject replay retains exactly the preceding statuses (35 pass, no gains or lost passes). Do not convert focused greens into a 100% claim; full corpus, iOS SDK and Throwable trace-argument retention gates remain open. Generated internal page ordering shifts after the new helper are expected registry output, not manual edits.
