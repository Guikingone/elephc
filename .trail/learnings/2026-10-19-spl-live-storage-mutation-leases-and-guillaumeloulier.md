---
title: SPL live storage mutation leases and asymmetric debug-view COW
author: Guillaumeloulier
date: 2026-10-19
branch: work/spl-opcache-runtime-integration
cost: { minutes: 2080, tokens: 12732993 }
touches:
  - crates/elephc-magician/src/context.rs
  - crates/elephc-magician/src/context/classlike_objects.rs
  - crates/elephc-magician/src/context/closure_metadata.rs
  - crates/elephc-magician/src/context/closure_owners.rs
  - crates/elephc-magician/src/context/core.rs
  - crates/elephc-magician/src/context/reflection_registry.rs
  - crates/elephc-magician/src/ffi/dynamic_destructors.rs
  - crates/elephc-magician/src/interpreter/expressions.rs
  - crates/elephc-magician/src/interpreter/expressions/closures.rs
  - crates/elephc-magician/src/interpreter/expressions/evaluation.rs
---

## Decided
Native ArrayObject and ArrayIterator writes must update the existing private storage Mixed cell. Borrowing its identity preserves debug-view reads across payload COW, growth and packed-to-hash promotion; writing the view itself still detaches normally. A scoped EIR leaf in ir_lower/stmt/spl_storage_writes.rs accepts only these builtin method bodies and their private storage field. Key writes use the existing Mixed RuntimeCall writer, append uses MixedArrayAppend, and removal uses OffsetUnset on every target. PropGet of a Mixed field acquires an owner unconditionally: keep that owner under an exception argument guard, then unguard and release it on normal completion. This leases an existing cell; it does not allocate a second cell merely for a deferred key.

## Tried and ruled out
Copying storage into a local and assigning the whole field back loses the debug-view link. A bare PropertyArrayAssign AST has no named-Mixed-property EIR fallback for these classes. The first live-cell prototype matched the value but leaked its property-read owner. The twenty pinned PHP controls now produce nineteen exact outputs, with eighteen clean heaps; the remaining value mismatch is ArrayObject clone, and both sort cases retain 208 bytes. PHP asort detaches an old debug view. PHP ArrayObject clone duplicates its terminal storage, while ArrayIterator clone wraps the original iterator. Do not implement one shared clone rule or replace the live view with a PHP reference.

## Watch out
Six executed lifetime cases cover set, unset, and throwing reentrant destruction across both classes; reentry uses a typed local receiver because the old compiler refuses the original static mixed write form. That refusal remains an open compatibility gap. A one-insert non-SPL reproducer leaks one forty-byte Mixed owner, so the sort problem has a generic ownership root. Reconcile fetched main before implementing that root again. Five target user assemblies pass, including the guard and mutation paths; Linux execution and iOS runtime are separate gates. Keep filtered PHPT passes and all remaining clone, sort, filesystem, print_r, diagnostic, Magician and full 1710-PHPT gates explicit. Evidence is under .artifacts/integration/v7-live-cell-owners and the integration plan; nothing has been published.
