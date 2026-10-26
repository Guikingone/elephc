---
title: Release hash children in PHP insertion order
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-hash-destruction-order
cost: { minutes: 1150, tokens: 9140658 }
touches:
  - docs/internals/memory-model.md
  - src/codegen_support/runtime/arrays/hash_free_deep.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/hash-destruction-order/asm-comment-check.json"
  - ".artifacts/hash-destruction-order/baseline-source.json"
  - ".artifacts/hash-destruction-order/baseline-v2/escaped_reference"
  - ".artifacts/hash-destruction-order/baseline-v2/escaped_reference.php"
  - ".artifacts/hash-destruction-order/baseline-v2/escaped_reference.s"
  - ".artifacts/hash-destruction-order/baseline-v2/reference"
  - ".artifacts/hash-destruction-order/baseline-v2/reference.php"
---

## Decided
Deep hash release must follow the insertion-order head at header+24 and successor at entry+56, exactly like iteration, rather than scan physical buckets. Both ARM64 and x86_64 emitters save the successor in their own frame before invoking child cleanup. Keep the existing protected cleanup boundary, reference-cell release, GC suppression and exception chaining intact.

## Tried and ruled out
The SPL debug snapshot exposed reversed obj/inf destructors despite a clean final heap. A minimal ordinary hash reproduces it; the equivalent packed array does not. Do not reorder the snapshot pairs to compensate. Six of eight focused controls fail with the precursor emitter, including rehash/reinsert, replacement, reference cells, shared nested rows and throwing destructors. After the fix all eight pass, 38 existing cleanup/reference/resurrection controls pass, and nine fixtures pass natively on macOS ARM64 and both Linux architectures with clean heaps; both iOS assemblies pass too.

## Watch out
A root cargo test with features curl can overwrite the plain Magician staticlib with a curl-enabled archive. Two eval resurrection controls initially failed to link optional curl symbols, not at runtime. Rebuild elephc-magician with no default features from the SAME source, assert features[] and pin its archive before retrying the already built test executable. Do not borrow a different-source frozen bridge or count a linker failure as a PHP behavior regression. Main debug snapshot rendering remains separate: replacing a foldable debug body exposes an extra backing row unless runtime debug callbacks are implemented.
