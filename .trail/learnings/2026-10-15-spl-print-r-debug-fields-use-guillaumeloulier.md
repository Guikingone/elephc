---
title: SPL print_r debug fields use a separate inherited render view
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-list-subclass-storage
touches:
  - examples/spl-containers/main.php
  - src/codegen_support/runtime/data/user.rs
  - src/codegen_support/runtime/objects/print_r_object.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/acceptance-v2/append.expected"
  - ".artifacts/acceptance-v2/append.php"
  - ".artifacts/acceptance-v2/base_property_views.expected"
  - ".artifacts/acceptance-v2/base_property_views.php"
  - ".artifacts/acceptance-v2/custom_constructor.expected"
  - ".artifacts/acceptance-v2/custom_constructor.php"
---

## Decided
Render inherited native DLL/Stack/Queue debug fields through a print_r-only append adapter. The preserved method-declaring map selects the default DLL handler and suppresses this view for a user override. Declared and dynamic user properties render first; flags and physical-order dllist storage follow. Shared print_r writers retain capture, indentation and seen-stack behavior. Property/cast/export descriptors are unchanged.

## Tried and ruled out
The unchanged SplDoublyLinkedList_debug-info PHPT printed an empty object although PHP renders two native fields. Those fields cannot simply be added to shared property rows: that would expose runtime state to casts and var_export. The native handler must also respect a user __debugInfo override and preserve dynamic-property ordering. Six focused fixtures cover those boundaries.

A recursive output fixture already matches PHP, but leaving its root alive until main return exposes the separate existing shutdown-cycle lifetime gap (three retained blocks). Preserve its original source and failure artifact. Explicitly unsetting the root after rendering lets this renderer test assert clean normal cleanup without claiming implicit shutdown collection is fixed.

## Watch out
Six renderer heap controls and 47 existing print_r-related controls pass with PHP cross-check. An independent compiler/linked rlib pin passes 18 native stdout/clean-heap cases across macOS ARM64 and both Linux architectures and assembly checks on all five targets. The general GC timing/shutdown gates, real iOS SDK/native execution and full corpus remain separate. The full 1710 run uses an older immutable 38621 artifact and must not be attributed to this renderer change.
