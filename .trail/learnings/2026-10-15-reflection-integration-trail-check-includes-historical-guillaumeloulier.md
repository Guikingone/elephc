---
title: Reflection integration Trail check includes historical unstaged plan
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - src/codegen/lower_inst/objects/reflection/member_properties_emit.rs
  - src/codegen/lower_inst/objects/reflection/owner_emission.rs
  - src/codegen/lower_inst/objects/reflection/parameter_property_emit.rs
  - src/codegen/lower_inst/objects/reflection/string_attrs_emit.rs
  - src/ir_lower/stmt/return_coercions.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/reflection_metadata_owners.rs
  - tests/codegen/spl/object_projection.rs
---

## Decided
Retain the dirty campaign plan as historical evidence without adding it to the Reflection integration commit. Trail check includes unstaged files, so its current twenty plan matches refer to earlier observations and resume checkpoints outside the staged production patch. Keep the plan and tested regression coverage intact.

## Tried and ruled out
Lexical word overlap in historical outcomes is not evidence that a rejected implementation was reintroduced. The changed plan explicitly preserves failed probes, fixed-source provenance and incomplete acceptance gates; none of the twenty reported plan matches identifies an executable code conflict.

## Watch out
The integration commit explains this check result. Review its staged source and tests separately from the unstaged plan, including unified return-owner retirement and the strengthened typed-flag heap assertion. Pinned matrix-v3 and the 52 focused tests provide the executed evidence. Do not report Trail check as green or silently discard historical failures to suppress its matches.
