---
title: Native print_r fields and assertions explain historical lexical findings
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-list-subclass-storage
touches:
  - examples/spl-containers/main.php
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/objects/print_r_object.rs
  - src/codegen_support/runtime/objects/print_r_spl.rs
  - tests/codegen/runtime_gc.rs
  - tests/codegen/runtime_gc/spl_list_debug_view.rs
  - ".artifacts/acceptance-v2/append.expected"
  - ".artifacts/acceptance-v2/append.php"
  - ".artifacts/acceptance-v2/base_property_views.expected"
  - ".artifacts/acceptance-v2/base_property_views.php"
---

## Decided
Keep the native print_r adapter, preserved property/export rows and strict rendering/lifetime controls. Interpret the seven findings in dll-print-r-trail-check-v1.log against the actual emitted fields and matched outputs. The observations in the earlier diagnostic refer to old artifacts; they do not restrict support for those native objects.

## Tried and ruled out
The checker connects class names, test names and repaired renderer comments to an earlier empty-output report. It also connects current clean-heap assertions to an older lvalue lifetime diagnostic. Current fixtures compare complete PHP output for capture, inherited fields, dynamic-property ordering and user overrides, then require a clean heap. Their native counterparts match on both instruction sets. Those assertions are evidence for the repair.

## Watch out
The first check has seven lexical findings and is non-green. Explain these categories in the commit message while preserving existing diagnostics and controls. Recursion rendering is validated with explicit root retirement; the original no-unset shutdown-cycle source remains a separate failed GC artifact. Full replay remains on immutable 38621, prior to this change, and native iOS validation remains open. No note, test or expected php-src output is removed to silence this check.
