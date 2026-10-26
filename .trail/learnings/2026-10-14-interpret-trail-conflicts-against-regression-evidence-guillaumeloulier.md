---
title: Interpret Trail conflicts against regression evidence semantically
author: Guillaumeloulier
date: 2026-10-14
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - src/types/checker/inference/objects/constructors.rs
  - tests/codegen/spl/storage_subclasses.rs
---

## Decided
A Trail finding needs a semantic comparison with the changed implementation. The descendant-storage change deliberately includes the ancestry relation, and its fixtures exercise inputs that previously failed. The main shutdown correction is isolated on a branch cut from main; its feature-dependent regression remains on the SPL branch. These changes follow the recorded ownership and branch boundaries.

## Tried and ruled out
The whole-branch scan was capped at 400 KB. Its text matcher treated narrative evidence and regression fixtures as prohibited designs. A second check against HEAD covered the complete new increment and reported the same class of lexical matches. Both check results are retained and explained in the commit; neither is claimed as a clean automated gate.

## Watch out
Do not delete useful regression fixtures or rewrite immutable learnings to make a textual checker green. Preserve the finding, verify the actual behavior, and document the reason. Native validation and the unchanged pinned PHPT provide evidence independent of the learning matcher. A large whole-branch check alone cannot establish complete review coverage when it reports truncation.
