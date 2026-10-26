---
title: Scalar method contracts retire arguments and native adapters own consumed inputs
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-nonheap-method-argument-owners
touches:
  - docs/internals/the-codegen.md
  - src/codegen/runtime_metadata/callables.rs
  - src/codegen/runtime_metadata/mod.rs
  - src/ir_lower/expr/method_metadata.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/nonheap-method-owners/build-final.jsonl"
  - ".artifacts/nonheap-method-owners/build-final.log"
  - ".artifacts/nonheap-method-owners/example.compile"
  - ".artifacts/nonheap-method-owners/example.expected"
  - ".artifacts/nonheap-method-owners/example.stderr"
---

## Decided
Infer ReturnArgAlias::None for non-reference methods whose declared return is an actual scalar, void or never value, even when an injected builtin has no AST alias summary. Keep pointer, resource, reference and heap-return contracts conservative. Generated native method-symbol adapters borrow PHP arguments and acquire independent owners at IntrinsicCall::consumed_mixed_parameters positions before tail-calling consuming helpers.

## Tried and ruled out
The alias-summary change alone made the ordinary void and throwing controls clean, but a native list then lost its nested payload despite balanced final heap counters. Port the method-wrapper owner acquisition from the dedicated main SPL storage root, independently of its storage layout. The first manual target matrix omitted the Phar archive required by the dynamic add candidate; preserve that failed link and use real pinned Phar archives for all three native platforms.

## Watch out
Exact main 132b4da6f7 leaks eight blocks when SPL metadata makes an otherwise void __unserialize call conservative. Eight strict regressions now pass, including heap-return aliases, normal and throwing calls, list push/unshift/add/offsetSet, fixed-array writes and bounds errors. Nine PHP-derived fixtures pass all twenty-seven native macOS ARM64/Linux ARM64/Linux x86_64 runs with clean heaps and five-target assembly in .artifacts/nonheap-method-owners/target-matrix-with-phar. The CLI, exact rlib, bridge archives and source snapshots are hash guarded. Legacy direct intrinsic and inherited-storage work remains on its own branch; ordinary unserialize property owners and interface reference-return diagnostics remain separate open gates, alongside full php-src acceptance and real iOS execution.
