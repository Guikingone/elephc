---
title: Defer non-scalar ArrayAccess offsets until receiver dispatch
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-arrayaccess-mixed-object-keys
touches:
  - examples/arrayaccess-object-labels/main.php
  - src/codegen/lower_inst.rs
  - src/codegen/lower_inst/hashes.rs
  - src/codegen/lower_inst/mixed_array_runtime.rs
  - src/codegen_support/mod.rs
  - src/codegen_support/runtime/data/fixed.rs
  - src/codegen_support/runtime/emitters/managed.rs
  - src/codegen_support/runtime/objects/mixed_array_fetch_for_write.rs
  - src/codegen_support/runtime/objects/mixed_array_get.rs
  - src/codegen_support/runtime/objects/mixed_array_set.rs
---

## Decided
A boxed receiver selects the offset rules. Preserve concrete object, indexed array, hash array and Closure offsets in a borrowed key pair until that choice. Integer high word -1 and nonnegative string lengths remain unchanged; high words below -1 encode -(runtime tag + 2). Object callbacks recover the original boxed offset, while actual array branches reject it with a type-named TypeError. The getter, setter and fetch-for-write paths share this rule on both CPU architectures.

## Tried and ruled out
Moving PHP callback dispatch ahead of coercion fixed boxed offsets but still failed to compile a concrete object key in the emitted fallback. Mapping unsupported Mixed tags to integer zero silently read the wrong array entry. Do not allocate an extra cleanup owner merely to carry the deferred key: borrow its existing payload and acquire an owner only when recovering a callback argument.

## Watch out
Array rejection occurs after receiver selection and RHS evaluation. A null write receiver becomes an array before rejection; a non-ArrayAccess object reports its receiver error first. Test nested fetches, key identity, full diagnostic bytes and normal or throwing RHS destructors, including previous-exception chaining. Keep the existing paired nonheap-result summary and native consumed-argument owner prerequisite: without it a local object offset could expose a freed error message despite a balanced heap. Twenty-one focused runtime-GC tests pass. Resource offsets, directly typed array-key validation and reference-return ArrayAccess remain separate open gates; this change makes no all-types or full PHP parity claim.
