---
title: Native object comparison follows PHP handlers and preserves borrowed payloads
author: Guillaumeloulier
date: 2026-10-16
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - docs/internals/the-codegen.md
  - src/codegen_support/runtime/compare/mod.rs
  - src/codegen_support/runtime/compare/obj_loose_eq.rs
  - src/codegen_support/runtime/data/user/comparison_props.rs
  - src/types/checker/builtin_spl_classes/object_storage.rs
  - src/types/mod.rs
  - tests/codegen/runtime_gc/spl_object_storage_comparison.rs
  - examples/spl-object-comparison/.gitignore
  - examples/spl-object-comparison/main.php
---

## Decided
Separate PHP property equality from compiler-private NUL slots. Exact SplObjectStorage compares default object-key identity and loosely equal info values, ignoring insertion order and cursor. Its distinct subclasses are uncomparable; identity still succeeds first. SplDoublyLinkedList uses standard PHP properties and ignores native entries/cursor. Pin php-src spl_observer.c 385-408 as the handler authority. Share storage slot names in types::spl_storage_layout instead of duplicating the physical ABI.

## Tried and ruled out
Comparing every physical slot fixed observer_007 but incorrectly compared DLL native state and storage cursor/order. The first native-dispatch run passed 13/17: ARM64 emit_load_symbol_to_reg borrows x9, so loading class identity before the helper made its range check delegate every time. Load the class id after the symbol helper, matching its documented clobber contract. Borrowed stack array cells may be normalized/read but never decref'd; retire only owned key/info reads. Read key and nested info payloads again after comparison to check lifetime, not only final heap balance.

## Watch out
The native comparison controls pass 17/17. The broader run passes 117/118; its cycle-destructor timing failure reproduces unchanged on frozen a08f7f111c (drop before done), so it remains a separate gate. Full a08 replay measured 633/1710: all seventeen lost passes were missing the same locked PCRE2 artifact. Locked offline installation restored it, and all seventeen pass unchanged on the same compiler. Do not extrapolate a new full score or overwrite frozen evidence. The target matrix and another complete replay are still required; recursive-comparison errors, dynamic property ordering and other builtin handlers remain open.
