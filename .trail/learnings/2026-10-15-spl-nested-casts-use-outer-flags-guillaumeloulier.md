---
title: SPL nested casts use outer flags and raw backing storage
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - examples/spl-array-views/.gitignore
  - examples/spl-array-views/main.php
  - src/codegen/lower_inst/builtins/types.rs
  - src/codegen_support/runtime/arrays/mixed_cast_array.rs
  - src/codegen_support/runtime/data/mod.rs
  - src/codegen_support/runtime/data/object_projection.rs
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/emitters/managed.rs
  - src/codegen_support/runtime/objects/array_cast.rs
---

## Decided
The feature branch retains its raw, named and public iteration property views when integrating the shared array-cast handler. Its builtin flags slot is __elephc_flags; metadata must bind that slot instead of main's flags name. The existing nonvirtual getArrayCopy provider already returns the full raw backing table, including mangled properties, nested backing and self backing. Normalize that returned copy for a cast and keep own-property projection separate.

## Tried and ruled out
An earlier nested probe was incorrectly interpreted as recursive application of the inner wrapper's STD_PROP_LIST flag. Pinned PHP 8.5.10 and spl_array_set_array prove the distinction: a one-argument wrapper constructor copies the input wrapper's flags, while an explicit flags argument does not. With explicit outer flags=0, casts ignore inner flags and use the underlying data. A recursive inner array-cast implementation would introduce wrong semantics. The earlier learning's warning that a raw provider cannot serve feature casts was an unverified inference, superseded by these executed controls. Omitted-argument flag inheritance remains a separate constructor gate.

## Watch out
Fourteen focused integration tests now pass; thirteen assert clean heaps and the typed-constant Reflection control checks functional metadata only. The unchanged ArrayObject_proptable_canonicalization PHPT passes with its original deprecation and EXPECTF. The 108-case replay and target matrix are pending at note creation; consult their final artifacts before claiming wider acceptance. Preserve the stale-compiler first replay as invalid evidence; the fresh v2 replay is authoritative. No source fixture or PHPT expectation was changed to earn that pass. Do not conflate scoped diagnostic suppression in focused ownership fixtures with strict PHPT replay, which retains upstream diagnostics.
