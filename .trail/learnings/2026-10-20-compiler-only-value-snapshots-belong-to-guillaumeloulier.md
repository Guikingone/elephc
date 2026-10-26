---
title: Compiler-only value snapshots belong to typed runtime operations
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
touches:
  - ".plans/spl-opcache-php-src-1806-checkpoint.md"
  - ".plans/spl-opcache-php-src-1806-diagnostics.md"
  - ".plans/spl-opcache-php-src-1806.md"
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - crates/elephc-builtin-contract/src/lib.rs
  - crates/elephc-builtin-contract/src/method_returns.rs
  - crates/elephc-builtin-contract/src/registry.rs
  - crates/elephc-builtin-contract/src/support.rs
  - crates/elephc-magician/src/context.rs
  - crates/elephc-magician/src/context/classlike_objects.rs
---

## Decided

Compiler-private borrowed-value primitives belong to RuntimeCallTarget, with fixed storage signatures: Heap(Mixed) to I64 for capture, I64 to Heap(Mixed) for an acquired read, and I64 to Void for scratch retirement. Their backend is src/codegen/lower_inst/runtime_calls/borrowed_value.rs, and the emitted runtime is arrays/borrowed_value.rs. No PHP builtin contract, synthetic reflected method or fake semantic home is needed.

## Tried and ruled out

The RuntimeFnId prototype failed scripts/audit_builtin_eir_boundary.py --enforce-target-architecture with three errors: each new function was not referenced by builtin semantics. That enum is for registry-backed functions. Do not weaken that audit, add exceptions, or invent PHP catalog entries for compiler operations. The prototype additions and runtime-function dispatcher changes were removed byte-for-byte; the corrected architecture audit exits 0. M21 was interrupted after the audit rejection and is not a valid checkpoint.

## Watch out

The new typed-target unit test validates and lowers all three fixed operations on all five supported targets. Native lifetime/resource tests use real runtime helpers and both optimizer modes; source guards and final pinned artifacts are separate gates recorded under borrow-primitives-validation-5 and main-m22-borrow-primitives. Only accepted per-case results count; intermediate prototype runs are not current source proof. The SPL next mutation still has not been connected to these operations and its destructor-order regression remains open.

A calculated class name in method_exists is a separate existing compiler gap: the preserved clean-main base-0 CLI ffddabddb3570fdcc9a57b71bb6d3d3bd8d3dc1b2590f7987dc97b64e373fb6a rejects it with builtin requires a literal string name, while pinned PHP prints 111 for the three SPL class names and count. Keep that reproducer; the primitive invisibility regression uses nine literal pairs without modifying any upstream PHPT.
