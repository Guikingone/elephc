---
title: DLL integration preserves feature adapters and aligns property masks
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - examples/spl-containers/main.php
  - src/codegen/lower_inst/callable_descriptors.rs
  - src/codegen/lower_inst/objects/clone_and_spl.rs
  - src/codegen/lower_inst/objects/dynamic_factory.rs
  - src/codegen/lower_inst/objects/dynamic_mixed_candidates.rs
  - src/codegen/lower_inst/objects/fixed_new.rs
  - src/codegen/lower_inst/objects/reflection/class_members.rs
  - src/codegen/runtime_callable_invoker/string_return.rs
  - src/codegen/runtime_metadata/callables.rs
---

## Decided
Integrate the main DLL storage checkpoint while preserving the feature branch's existing SplFixedArray subclass allocation, FixedArray clone adapter, native dump adapters and owner regression modules. DLL clones now use the generic declared-property copy plus the shared finishing step; remove the old DLL adapter-table entry. Projection masks skip reserved NUL fields at the same point as serial-property rows so their positions stay aligned. Empty visible rows retain a zero mask entry.

## Tried and ruled out
The incoming main checkpoint is based on a newer main tree and its conflict context includes unrelated renderer changes. Keep the feature renderer's typed rows and existing lookup rules, adding only the reserved-field exclusion. Whole-side conflict selection would lose FixedArray support or reference renderer helpers absent from this branch. A first conflict script used a greedy end-marker suffix and truncated six file tails. Restore the conflict contents from Git's stages, then match end-marker text with a bounded single-line pattern. All full emitter siblings and file tails are preserved before compilation.

## Watch out
After resolution, 24 list controls pass with PHP cross-check, including the formerly default-heap Reflection failure; all 18 projection controls also pass. The new backing-object fixture verifies one public and two mangled/cast user properties while hiding native slots. Append integration is validated separately. Strict cycle-destructor timing remains an open registered main semantic; do not remove its expected PHP order. Root standalone native69 evidence predates this integration, so the integrated target matrix and full PHPT replay still need fresh pins. No publication has occurred.
