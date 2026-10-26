---
title: Retire temporary keys across array writes and RHS exceptions
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-array-write-key-owners
touches:
  - src/ir_lower/expr/mod.rs
  - src/ir_lower/stmt/array_write_core.rs
  - src/ir_lower/stmt/nested_array_writes.rs
  - src/ir_lower/stmt/property_array_writes.rs
  - src/ir_lower/stmt/static_property_writes.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/constructor-unwind-owners/build.log"
  - ".artifacts/constructor-unwind-owners/commit-message.txt"
  - ".artifacts/constructor-unwind-owners/constructor-owners-trace-probe.rs"
  - ".artifacts/constructor-unwind-owners/fused.php"
---

## Decided
Array and hash writes borrow their key representation. Capture the PHP key value before later expressions run, detach mutable Mixed cells and stabilize string storage, then expose a borrowed view from one unwind-visible owner slot. Preserve PHP's delayed read for a bare variable key. Give borrowed runtime setters a separate value policy; nested writes park their existing parent lease without cloning the attached parent.

## Tried and ruled out
On clean main dbea935750, a returned Mixed integer key retained one 40-byte cell, and a string key retained that cell plus its string. An RHS exception left the evaluated key behind too. Binding the key to a PHP local happened to hide the leak but does not fix the writer. The original nested control used a concrete type rejected by this checker; its preserved source is not executed evidence. Returning Mixed explicitly exercises the intended nested COW/unwind path without weakening its assertions.

## Watch out
All sixteen key-owner controls pass; the broader selection has 492 pass, one ignored and no failures. The warning-free build passes. All 48 PHP-matching native executions have clean heaps on macOS ARM64 and both Linux architectures; every program/runtime assembles for all five targets. The initial shared-filesystem ARM link hit a terminal linker bus error. Disk and memory were available; replaying identical assembly from container-local files succeeds. Keep both attempts and their input hashes. The current cast prototype is stashed separately; restore it with its recorded hashes after integrating this shared prerequisite. Keep callback values borrowed so existing coercion does not release the scoped key twice. No full PHP compliance or publication follows from these focused results.
