---
title: SPL debug views require physical heap order and mutation phase ownership
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
SPL heap storage must maintain Zend binary-heap order, not append order plus a best-element scan at extraction. Preserve parent/incoming and sibling/bottom comparison order, mask queue extraction flags, and choose queue output after comparison callbacks can change flags. Track live count separately from temporary sift slots and retain pending comparison exceptions until state bookkeeping finishes. Debug snapshots use the PHP packed-or-hash array contract, copy only the live prefix, preserve user members and hide implementation fields. MultipleIterator reuses SplObjectStorage's private storage key and obj/inf schema; its debug body is also an indirect metadata edge.

## Tried and ruled out
The array contract repair alone reached an unresolved MultipleIterator::__debugInfo symbol in unchanged bug69264. Whitelisting its body and fixing its getter schema makes that PHPT pass unchanged. M19's 15 custom fixtures pass 14 complete behaviors in both IR modes; all thirty runs have clean heaps after the main-rooted receiver pin fix, but next_destructor still exposes the wrong count. Seventy-five target compile/assembly/link controls succeed, including thirty complete core-runtime Linux executions. Twenty-eight Linux results match PHP; the two next cases retain the same value mismatch. Do not count them as passes.

## Watch out
next must release the root before decrementing count and before comparisons. Current generic array/property replacement releases it later; a callback reading current can hit a zero-refcount boxed cell. Do not fake the counter during destructors or manually call __destruct while retaining the object. Five pinned retirement oracles and the explicit root-retirement plan cover identity, callback order, snapshots, clones, queue data/priority order and throws. An artifact-only raw borrowed-cell snapshot plus __rt_mixed_clone demonstrates correct owned resurrection and a live saved object, but production scratch lifetime, clone handling and all targets remain unimplemented. Full corpus 1710, diagnostics, Fiber, generic property/Reflection and clone gates remain open. No publication is authorized.
