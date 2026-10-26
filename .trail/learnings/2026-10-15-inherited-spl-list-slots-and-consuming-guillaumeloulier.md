---
title: Inherited SPL list slots and consuming method adapters preserve native ownership
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-list-subclass-storage
touches:
  - examples/spl-containers/main.php
  - src/codegen/lower_inst/callable_descriptors.rs
  - src/codegen/lower_inst/objects/clone_and_spl.rs
  - src/codegen/lower_inst/objects/dynamic_factory.rs
  - src/codegen/lower_inst/objects/dynamic_mixed_candidates.rs
  - src/codegen/lower_inst/objects/fixed_new.rs
  - src/codegen/lower_inst/objects/reflection/class_members.rs
  - src/codegen/runtime_callable_invoker/string_return.rs
  - src/codegen/runtime_metadata/callables.rs
  - src/codegen/runtime_metadata/mod.rs
---

## Decided
Give SplDoublyLinkedList three private inherited physical property slots: owned Array(Mixed) storage at byte 8, iterator index at 24, and mode at 40. Generic class allocation/default thunks and GC descriptors then work for native bases and descendants, including custom constructors, dynamic new and user fields. Exclude the reserved NUL names from PHP-visible property metadata. Native method adapters acquire an independent reference for parameters declared consuming by IntrinsicCall, because PHP method arguments are borrowed.

PHP clone expressions use RuntimeFnId::CloneWith and the boxed clone adapter, not ObjectCloneShallow. Finish DLL clones there by splitting the retained array owner and setting physical index zero before __clone, including in LIFO mode. Load ancestor ids after ABI symbol loads: those helpers may overwrite x9/r10. Iterator key reports the signed traversal position, invalid next/prev retain that position, FIFO prev can exhaust at -1, and LIFO delete exhaustion is -1.

Native container first-class callables use the existing emitted wrapper inventory. IntrinsicCall declares the native serialize string's owned return. Captured instance-descriptor calls use the existing owned result producer; raw closure contracts and the strict backend unowned-string rejection stay intact.

## Tried and ruled out
Exact main 114e514 initially failed all seven storage probes. Merely enlarging child allocations still stored a freed Mixed cell: the wrapper passed a borrowed box into a consuming helper. Balanced final heap counts did not catch the premature payload destruction; the marker after push does. A first clone prototype changed an inactive lowerer and did not affect the fixtures; it was removed. Clone initialization is not rewind: PHP's Stack clone starts at physical node zero, while rewind selects the tail. A length-based exhausted sentinel broke key() after LIFO next.

## Watch out
Twenty-three focused PHP-cross-checked controls and twenty-seven existing native class controls pass. The immutable matrix compiler 2264e7981740b5c4867b9ba10b9553f63c0f6bcac0f6b74185d0f5936fb45d0c plus its exact CLI-linked rlib pass 69 native stdout/clean-heap cases across macOS ARM64 and both Linux architectures, and assembly checks on all five targets. Real iOS SDK/native evidence is still open.

This is a campaign checkpoint, not full SPL compliance. Two strict tests remain registered outside that green focused filter: default-heap Reflection needs the separate owner/capacity prerequisites, and self-cycle destructor order differs on unchanged main even for an ordinary class. Preserve both expected PHP outcomes. Append syntax for descendants is a feature-branch prerequisite, not part of this main storage slice. On integration into the object-projection feature, its serial-property visibility masks must also skip reserved NUL fields before assigning mask positions. Do not call this checkpoint, its source snapshot, or the frozen 639/1710 corpus result full compliance.
