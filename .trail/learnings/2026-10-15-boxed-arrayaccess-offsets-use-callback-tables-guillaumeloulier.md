---
title: Boxed ArrayAccess offsets use callback tables before key conversion
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-arrayaccess-mixed-object-keys
touches:
  - docs/internals/the-codegen.md
  - src/codegen/lower_inst.rs
  - src/codegen/lower_inst/mixed_array_runtime.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/mixed-arrayaccess-keys/build-final.jsonl"
  - ".artifacts/mixed-arrayaccess-keys/build-final.log"
  - ".artifacts/mixed-arrayaccess-keys/callback-stack-v5.log"
  - ".artifacts/mixed-arrayaccess-keys/callbacks-expanded-v4.log"
  - ".artifacts/mixed-arrayaccess-keys/callbacks-v3.log"
  - ".artifacts/mixed-arrayaccess-keys/example.compile.log"
---

## Decided
For boxed receiver reads, writes and fetch-for-write operations, inspect the existing class offset callback tables before PHP array-key conversion. Invoke their raw-object and boxed-argument ABI through the shared argument materializer. Declared PHP array receivers and missing native-only SPL callback entries keep the existing fallback. Preserve object identity and non-normalized boolean, float, null, array and numeric-string offsets for PHP ArrayAccess implementations.

## Tried and ruled out
An interface-dispatch branch passed initial object key controls but coupled the path to source interface return adapters. Use the existing boxed callback ABI instead. A saved callee is below the argument conversion cleanup records, not always at the top of the temporary stack: popping it jumped into a key cell for literal offsets. Load it at cleanup_bytes and discard its slot only after argument cleanup. The separate fetch-for-write runtime target also needed the callback branch; fixing the ordinary read path alone left nested offsets coerced.

## Watch out
Exact main 132b4da6f7 turns object offsets into integers in both read and write controls. Current focused validation passes nine controls after the final nested-fetch change; forty-four ArrayAccess controls passed before that last sibling-path edit. Seven PHP-derived fixtures pass all twenty-one native macOS ARM64/Linux ARM64/Linux x86_64 runs with clean heaps and five-target assembly in .artifacts/mixed-arrayaccess-keys/target-matrix. CLI 3e74e1f6d0b699921d80fdd1b0b4f2f87ae6818bd96353a526028951cedd073d and its exact linked rlib are independently pinned. A by-reference ArrayAccess implementation fails the same source-interface return diagnostic on unmodified main; its strict fixture is preserved on fix/main-arrayaccess-reference-return. This slice does not close that gate, the existing PHP compatibility notes, real iOS execution, or full php-src acceptance.
