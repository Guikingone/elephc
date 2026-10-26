---
title: Object equality must use physical properties independently of debug projections
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-object-comparison-properties
touches:
  - src/codegen_support/runtime/compare/obj_loose_eq.rs
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/objects/export_props.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/object-comparison/baseline-build.jsonl"
  - ".artifacts/object-comparison/baseline-build.log"
  - ".artifacts/object-comparison/baseline-compile.stderr"
  - ".artifacts/object-comparison/baseline-compile.stdout"
  - ".artifacts/object-comparison/baseline-native.stderr"
  - ".artifacts/object-comparison/baseline-native.stdout"
---

## Decided
Emit a distinct physical comparison descriptor table and have object loose equality use comparison count/value helpers. Parameterize the existing owned property reader by its emitted symbol and descriptor table, preserving one boxing/null/retention implementation for rendering and comparison on ARM64 and x86_64. Resolve shadowed private physical slots through the nearest ancestor whose visible slot index matches. Logical untyped Void slots are boxed Mixed.

## Tried and ruled out
Clean origin/main d354273274 with a pinned fresh CLI compares unequal user objects as equal when __debugInfo returns an empty array. This is a generic compiler defect, not only an SPL regression. On the feature snapshot 1ad5, hiding NUL-prefixed SplObjectStorage fields made observer_007 print four true values; exposing those fields in renderers would undo the intended PHP property surface. Do not fix comparison by removing display filtering or by calling user debug/serialization hooks.

## Watch out
Five strict PHP/heap regressions pass for empty/partial debug projections, array ownership, inherited private shadowing and avoiding debug-hook calls. Existing controls and the native target matrix are separate gates at this checkpoint. The change fixes renderer coupling; native builtin comparison handlers, cursor/key semantics, dynamic-property ordering/count and uninitialized-property distinctions still need their own evidence. Integrate the dedicated main prerequisite without changing frozen replay roots or PHPT expectations, then rerun observer_007 and the full corpus.
