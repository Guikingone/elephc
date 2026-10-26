---
title: Share physical builtin result types with unchecked lowering
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-spl-storage-debug-info
cost: { minutes: 1099, tokens: 8922631 }
touches:
  - src/builtins/system/get_mangled_object_vars.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/storage-debug-info/build-preflight-v2.json"
  - ".artifacts/storage-debug-info/build-preflight-v3.json"
  - ".artifacts/storage-debug-info/build-preflight-v4.json"
  - ".artifacts/storage-debug-info/checkout-verification.json"
  - ".artifacts/storage-debug-info/disk-full-recovery.json"
  - ".artifacts/storage-debug-info/disk-preflight-cancellation.json"
  - ".artifacts/storage-debug-info/focused-v1.log"
  - ".artifacts/storage-debug-info/focused-v2.log"
---

## Decided
get_mangled_object_vars returns a raw string-keyed Mixed-value hash through the typed Core operation. Its AOT home now declares that result with BuiltinResultType::Shared, so checker records, synthesized bodies and callable lowering agree on the physical value. The neutral PHP contract, Core selector, effects, ownership and target support are unchanged. The old checker hook only returned a constant type and supplied no extra validation.

## Tried and ruled out
Ordinary checked and first-class source controls already passed before this fix. They cannot establish that unchecked lowering is safe. The regression removes CheckResult.builtin_call_types, lowers an actual call and inspects its Core instruction on every supported target: the old pinned library labels the raw pointer Mixed, while the corrected descriptor produces AssocArray<Str,Mixed>. LLDB on a synthesized SPL debug body showed object_to_hash followed by mixed_unbox and hash_to_mixed receiving invalid address0x16. Changing the caller's local annotation or its PHP array return declaration did not fix that mismatch.

## Watch out
Keep the full storage/value assertions and heap checks alongside the missing-checker-record regression. Three focused tests and nine native executions on macOS ARM64/Linux ARM64/Linux x86_64 pass; assembly passes on both iOS targets too. The callable control needs the real crypto bridge during manual linking. This is a prerequisite, not complete SPL debug rendering: the separate snapshot body and runtime var_dump callback work remain unfinished. Regenerate and audit builtin registries when changing home semantics.
