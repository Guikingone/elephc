---
title: Plain eval verification needs a default Magician bridge archive
author: Guillaumeloulier
date: 2026-10-16
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - examples/object-property-comparison/.gitignore
  - examples/object-property-comparison/main.php
  - src/codegen_support/runtime/compare/obj_loose_eq.rs
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/data/user/comparison_props.rs
  - src/codegen_support/runtime/objects/export_props.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/object_comparison_properties.rs
  - tests/codegen/runtime_gc/spl_object_storage_comparison.rs
---

## Decided
Build and pin default runtime bridge archives for SPL/OPcache replay unless the generated program explicitly enables a capability needing another bridge. Record compiler and bridge feature sets separately. Require a real opaque-eval compile/link/heap smoke with the pinned artifacts before a full replay.

## Tried and ruled out
An explicit multi-package build with elephc-magician/curl overwrote the default archive. Plain eval then linked that archive without elephc_curl and failed on undefined elephc_curl_* symbols. Rebuilding elephc-magician without the optional feature makes the unchanged eval control pass. Compiler --features curl does not justify enabling every standalone runtime archive's Curl capability. Keep the failed log and the immutable older snapshot rather than replacing its archives in place.

## Watch out
The comparison integration has 46 passing strict controls plus the unchanged eval control passing after the archive-profile correction; no source expectations changed. The older 1ad5 frozen snapshot and incomplete full replay remain historical evidence with a recorded bridge configuration limitation. Prepare a fresh snapshot with default bridges, verify its eval smoke, and rerun unchanged PHPTs. Optional Curl-capability programs need their own coherent bridge/link configuration gate.
