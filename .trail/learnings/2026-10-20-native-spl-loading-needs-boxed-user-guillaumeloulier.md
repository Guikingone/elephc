---
title: Native SPL loading needs boxed user storage with original property contracts
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
ClassInfo.native_property_contracts preserves original PHP declarations by physical index while non-reference user properties of native heap descendants use Mixed storage. Reserved NUL-prefixed backing fields retain their ABI. Ordinary weak writes, strict hydration masks/class checks, Reflection, eval registration and uninitialized diagnostics must consult the original declaration; getters and GC use physical storage. The backend guard explicitly boxes coerced scalar results for these slots and preserves literal false. M31 verifies this prerequisite with 50 native Rust tests and 90 property cases across five targets (54 exact/clean executions, 36 iOS assembly-only cases). The frozen heap selection remains 27/36; no native __unserialize or property-load operation exists yet.

## Tried and ruled out
Simply boxing fields made property [] writes fail checker admission. Admit compiler-marked native boxed fields and use the existing PropGetForWrite/PHP-array mutation path; keyed COW and nullable arrays then pass both optimizer modes on every supported target. Reference-backed fields retain their current typed shared-cell ABI instead of silently treating caller int cells as Mixed; native detachment still needs implementation. M28 already failed assignment-expression result normalization and dynamic readonly error ordering, and could not construct a false default. M31 fixes the false default without claiming the two independent baseline mismatches are solved. Preserve their complete artifacts.

## Watch out
Pinned PHP native loading bypasses property type checks but enforces initialized readonly fields. Additional measured controls require old|new during a property destructor, array|new during an array element destructor, and integer:1|string:bad when native loading detaches a reference. Ordinary early assignment changes the visible value; a payload pin postpones destruction. Ordinary Mixed property reads retain the box, so release-before-publication needs an explicit borrowed-view path. mixed_free_deep releases the payload before the cell; array_free_deep retains its struct until child release completes. Keep PHP insertion-order hash cleanup and protected exception boundaries. Two trailing-comment spaces were fixed after M31 functional validation; raw source hashes differ only at those recorded sites, and the next semantic build must repin them. No commit/push or full compliance claim was made.
