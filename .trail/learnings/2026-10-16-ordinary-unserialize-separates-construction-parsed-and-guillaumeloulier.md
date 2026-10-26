---
title: "Ordinary unserialize separates construction, parsed and stored owners"
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-unserialize-property-owners
touches:
  - docs/internals/the-codegen.md
  - src/codegen/eval_property_helpers.rs
  - src/codegen/mod.rs
  - src/codegen_support/runtime/system/unserialize/decoder_aarch64.rs
  - src/codegen_support/runtime/system/unserialize/decoder_x86_64.rs
  - src/codegen_support/runtime/system/unserialize/storage_aarch64.rs
  - src/codegen_support/runtime/system/unserialize/storage_x86_64.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/serialize.rs
  - ".artifacts/unserialize-property-owners/additional-controls-v1.json"
---

## Decided
Ordinary unserialize properties use a user-assembly writer with the module's concrete slot metadata. The parser context owns parsed producer cells and the construction box before user-controlled property decoding. Typed slot replacement borrows the producer, retains the stored value and releases the old default; a successful object result acquires a separate lease. Mangled keys identify private ancestor slots. Nullable integer wire values accept actual int/null and reject other kinds with a property TypeError.

## Tried and ruled out
The old cacheable writer stored boxed values without retiring scalar producers and overwrote string defaults without releasing them. Releasing parsed cells immediately loses wire back-references. Keeping the construction owner only on the parser stack leaks partially hydrated objects after exceptions. Closing the context inside the property error helper as well as at the parser exception boundary corrupts outer state when __wakeup catches a nested unserialize failure; the unchanged outer back-reference fixture detected a bad refcount.

## Watch out
The parser boundary alone closes contexts after property TypeError. Internal explicit property-restoration builtins have different caller lifetimes and must not close an active parse context. Untyped properties without defaults need physical Mixed storage when restoring. Self-cycle collection, complete declared-type/readonly validation, malformed property keys, exact upstream PHPT acceptance and real iOS execution remain separate gates. Focused positive owner tests do not close those gates.
