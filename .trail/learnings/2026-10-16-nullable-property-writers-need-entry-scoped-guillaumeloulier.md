---
title: Nullable property writers need entry-scoped branch labels
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-unserialize-property-owners
touches:
  - src/codegen/eval_property_helpers.rs
  - tests/codegen/runtime_gc/unserialize_property_owners.rs
  - ".artifacts/unserialize-property-owners/additional-controls-v1.json"
  - ".artifacts/unserialize-property-owners/asm-comments.log"
  - ".artifacts/unserialize-property-owners/build-final.jsonl"
  - ".artifacts/unserialize-property-owners/build-final.log"
  - ".artifacts/unserialize-property-owners/commit-message.txt"
  - ".artifacts/unserialize-property-owners/example.compile"
  - ".artifacts/unserialize-property-owners/example.expected"
  - ".artifacts/unserialize-property-owners/example.stderr"
---

## Decided
Shared nullable-property store emitters namespace their branch labels by the writer's entry-specific failure label as well as the physical slot identity. ARM64 and x86_64 use the same rule. Eval setters, ordinary unserialize hydration and explicit internal property restoration can all be present for one nullable property.

## Tried and ruled out
A slot-only set label stem emitted duplicate tagged_scalar_null/done symbols when more than one writer used the shared helper. The pinned ordinary-owner commit 1fc27c02f5 reproduces this with opaque eval plus an ordinary nullable-property unserialize, even without the feature's explicit restoration builtin. Deduplicating the property-hash helper alone does not prevent these local branch-label collisions. Changing validation or coercion would not fix an assembler symbol conflict.

## Watch out
Future writer entries must provide a distinct error-label namespace. The change does not alter the property layout, emitted runtime instructions, passing modes or cell ownership. Combined writer configurations need assembly coverage on every supported target, alongside native owner regressions. Full php-src acceptance remains a separate campaign gate.
