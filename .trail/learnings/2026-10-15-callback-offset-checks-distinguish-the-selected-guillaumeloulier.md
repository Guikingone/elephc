---
title: Callback offset checks distinguish the selected ABI from discarded exploration
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-arrayaccess-mixed-object-keys
touches:
  - docs/internals/the-codegen.md
  - examples/arrayaccess-object-labels/.gitignore
  - examples/arrayaccess-object-labels/main.php
  - src/codegen/lower_inst.rs
  - src/codegen/lower_inst/array_access_dynamic.rs
  - src/codegen/lower_inst/mixed_array_runtime.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/array_access_mixed_keys.rs
  - ".artifacts/mixed-arrayaccess-keys/build-final.jsonl"
  - ".artifacts/mixed-arrayaccess-keys/build-final.log"
---

## Decided
The boxed ArrayAccess offset change invokes the existing class offset callback tables. Shared argument materialization provides argument ownership and the source callback ABI remains authoritative. The source interface adapter rules stay unchanged.

## Tried and ruled out
The first trail check reported three lexical matches against the earlier note's historical exploration. Those messages do not identify a runtime or ABI disagreement. The earlier discarded implementation used interface dispatch; the selected implementation uses offset callback tables.

## Watch out
Review the emitted call target and the decision section when a lexical check quotes historical attempts. Preserve the earlier learning and record the interpretation in the commit. Current evidence is the final nine focused controls and the pinned twenty-one native runs with clean heaps; the exact-main reference-return diagnostic remains open on its dedicated branch.
