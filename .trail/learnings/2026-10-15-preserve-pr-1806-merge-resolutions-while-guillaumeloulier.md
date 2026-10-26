---
title: Preserve PR 1806 merge resolutions while rebasing main
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - tests/codegen/optimizer/concat_reset.rs
---

## Decided
Use a clean isolated checkout and history-preserving rebase with no-rebase-cousins. The original merge 33678cea93 joins the feature cousin to main 62cfeb85ef; recreating it against dbea935750 conflicts in three old resolution files. Preserve those files from that original merge only after proving none belongs to the new upstream delta.

## Tried and ruled out
Flattening the historical branch replayed old feature commits against a new base and stopped on previously resolved conflicts. The isolated merge-preserving attempt instead retained the cousin and replayed later commits cleanly after the three resolutions. Do not infer success from a completed Git command alone.

## Watch out
Rebased 978a0810f109d2a67a96b63786a6837c4263b5d7 has exactly the clean git merge-tree of prior checkpoint 7441615e93 plus main dbea935750: tree d551240bd7a9a4b41db4762c134bb17f611692a9, with no unexpected changed path. A named backup retains the old tip. New main changes concat-reset optimization, so rerun compiler/target controls on rebuilt artifacts; old binary evidence stays a separate checkpoint. No publication follows from a local rebase.
