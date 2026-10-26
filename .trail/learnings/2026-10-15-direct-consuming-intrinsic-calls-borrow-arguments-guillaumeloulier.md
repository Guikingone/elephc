---
title: Direct consuming intrinsic calls borrow arguments and snapshots retire producers
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-list-subclass-storage
touches:
  - examples/spl-containers/main.php
  - src/codegen/lower_inst/method_intrinsics.rs
  - src/codegen_support/runtime/spl/doubly_linked_list.rs
  - tests/codegen/runtime_gc/spl_list_subclass_storage.rs
  - ".artifacts/acceptance-v2/append.expected"
  - ".artifacts/acceptance-v2/append.php"
  - ".artifacts/acceptance-v2/base_property_views.expected"
  - ".artifacts/acceptance-v2/base_property_views.php"
  - ".artifacts/acceptance-v2/custom_constructor.expected"
  - ".artifacts/acceptance-v2/custom_constructor.php"
---

## Decided
Direct native instance calls with consuming Mixed parameters use their generated PHP method adapters, just like inherited and first-class calls. The shared IntrinsicCall consumption descriptor selects that path. Track and retire caller-created boxing temporaries through the normal direct-call materializer; the adapter supplies each helper's independent owner. Non-consuming direct intrinsic calls keep their existing path.

Native DLL serialized-state construction retains fresh items/properties arrays in Mixed boxes, then retires each producer reference with the ordinary array release helper on both architectures. The result boxes are their sole owners. Update the example to pass a mixed lvalue into a native queue and unset that variable before reading the queued payload.

## Tried and ruled out
On the frozen b74 artifact, a base list holding a mixed string variable prints 1: after unset instead of 1:a while the final heap is balanced. The inherited-wrapper fix did not cover the direct intrinsic bypass. A hydration hook could restore two element slots but lose both strings after the parser's data array was retired. Serialized-state snapshots separately retained two array allocations after the outer state was dropped; retaining box creation had stranded the original producers.

## Watch out
The two lvalue and two snapshot controls pass with PHP output and clean heaps. The expanded main list filter passes 27 controls, and the existing 27 class controls including throwing diagnostics pass after the direct-call fix. An independently copied CLI/linked rlib validate 81 native stdout/clean-heap cases on macOS ARM64 and both Linux architectures, plus assembly on all five targets. Native iOS evidence and the registered default-heap Reflection/general cycle-timing prerequisites remain distinct. The feature's hydration reachability change is separate and still needs its own pinned PHPT replay. Do not infer full corpus completion from these controls.
