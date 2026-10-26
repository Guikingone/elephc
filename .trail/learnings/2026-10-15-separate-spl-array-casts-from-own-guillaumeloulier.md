---
title: Separate SPL array casts from own-property enumeration
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-object-projection
touches:
  - src/codegen/lower_inst/builtins/types.rs
  - src/codegen_support/runtime/arrays/mixed_cast_array.rs
  - src/codegen_support/runtime/data/mod.rs
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/emitters/managed.rs
  - src/codegen_support/runtime/objects/mod.rs
  - src/codegen_support/runtime/objects/object_vars.rs
  - src/ir_lower/program/spl_metadata.rs
  - src/types/checker/builtin_spl_classes/storage.rs
  - tests/codegen/spl.rs
---

## Decided
ArrayObject and ArrayIterator casts select normalized backing data unless STD_PROP_LIST selects the wrapper's own PHP declarations. get_object_vars and get_mangled_object_vars always enumerate own declarations. Emit a separate per-slot projection mask so compiler storage stays hidden without changing serialization or GC layouts. Bind a nonvirtual internal copy provider, including compiler-owned live-view descendants; user getArrayCopy overrides do not affect casts. Both direct-object and boxed-Mixed casts use the shared runtime entry. Declare SPL flag constants as typed integers.

## Tried and ruled out
An ARM class id held in x9 was overwritten by symbol materialization; keep it in x12. A duplicate AppendIterator helper match arm was unreachable; extend the existing arm. getArrayCopy alone is not an adequate implementation on the feature branch, whose Mixed object storage must honor nested cast flags, self backing and numeric key normalization.

## Watch out
On this main-derived branch, 18 focused controls and 283 SPL tests pass. All ten fixtures match PHP on macOS ARM64 and both Linux architectures; nine fixtures are heap-clean on each. The typed-constant Reflection scenario still retains its metadata graph and is explicitly an open lifetime gate, not waived parity. All fixture and runtime assembly compiles for five targets; iOS SDK and native acceptance remain separate. The example matches PHP with a clean heap. Documentation audits pass; Python 3.9 needs the already installed tomli exposed as tomllib for the comparison generator. Docker links run in container-local temporary storage after a prior shared-mount linker bus error. Trail checker lexical matches against failure narratives do not replace executed regressions.
