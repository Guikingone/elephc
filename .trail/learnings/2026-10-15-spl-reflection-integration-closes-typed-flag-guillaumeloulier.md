---
title: SPL Reflection integration closes typed flag heap exceptions
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
Integrate shared Reflection owner fix 8d817c8bc0 while retaining all SPL/OPcache container, key and cycle regressions. Replace the older return coercion's associative-array-only Mixed release with the common unwind-aware owner protocol for every heap result; never retain the old release alongside the new retirement. Strengthen the SPL typed-flag Reflection control to require successful execution and a clean heap.

## Tried and ruled out
Resolve cherry-pick overlaps by preserving every existing runtime_gc test module and using the unified source-owner coercion. Matrix-v2 passed all 42 output and heap checks but failed end-of-run compiler provenance: adjacent codegen tests replaced the CLI. Preserve that invalid evidence and use independently copied compiler and rlib artifacts for matrix-v3.

## Watch out
Integrated validation passes eleven Reflection heap, fourteen projection, three mixed-method-result, eight container-write-owner and sixteen array-key-owner controls with PHP cross-check (52 total). Explicit CLI build is warning-free. Pinned matrix-v3 uses unchanged fourteen PHP fixtures from matrix-v1: all 42 native executions match PHP and have clean heaps, closing the former three flag_constant_types lifetime exceptions. CLI and rlib hashes stay fixed through the run. All five targets assemble; real iOS SDK/native acceptance remains open. The shared 430-test Reflection run is still in progress. Full campaign acceptance remains frozen at 639/1710 until a new pinned replay. This checkpoint does not authorize publication.
