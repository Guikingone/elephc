---
title: SPL storage subclasses inherit backing-object mutation requirements
author: Guillaumeloulier
date: 2026-10-14
branch: feat/spl-opcache-php-src-1806
touches:
  - src/types/checker/inference/objects/constructors.rs
  - src/types/checker/inference/objects/methods.rs
  - tests/codegen/spl/storage_subclasses.rs
---

## Decided
An ArrayObject or ArrayIterator descendant retains the builtin's object-backing mutation capability. The checker must reserve dynamic-property storage and widen untyped backing-object fields to boxed Mixed for descendant construction and exchangeArray as well as direct builtin construction. Use the existing Checker::is_subclass_of relation; do not infer this capability from an exact receiver name.

## Tried and ruled out
The exact-name predicate missed MyArrayObject and MyArrayIterator. Three regressions crashed in __rt_str_persist after indirect offsetUnset: the removed string property's high word held the unset sentinel, while an ordinary property read still treated it as a string length. Direct unset of the same property already works and does not reproduce the failure. The pinned php-src ArrayObject/bug54323.phpt uses the descendant shape and exposes this missed layout preparation.

## Watch out
Both constructor and exchangeArray inference consume the predicate. Preserve unrelated concrete property layouts and declared types; the existing program-usage-gated reservation handles that distinction. Inherited physical-slot views and custom constructors forwarding backing arguments remain separate cases to validate. The focused regressions intercept E_WARNING messages for stable oracle comparison; this does not establish internal deprecation severity parity.
