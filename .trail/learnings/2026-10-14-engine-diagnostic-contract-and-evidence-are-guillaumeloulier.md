---
title: Engine diagnostic contract and evidence are selected behavior
author: Guillaumeloulier
date: 2026-10-14
branch: feat/spl-opcache-php-src-1806
touches:
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - crates/elephc-builtin-contract/src/support.rs
  - scripts/docs/builtin_registry.json
  - docs/internals/builtins/_internal/__elephc_deprecated.md
  - ".plans/spl-opcache-php-src-1806.md"
---

## Decided
The engine-diagnostic contract, its AOT/eval homes and generated documentation are the selected implementation. Public trigger_error continues rejecting engine levels. Historical red results and current green controls are evidence, not prohibited input patterns.

## Tried and ruled out
The current checker interprets word triples from successful outcome sentences in a learning section as rejected code. It also matches generated documentation and the provenance ledger against old fixture narratives. Review each flagged source change against its assertions and contract rather than deleting metadata, regression coverage or unchanged generator output.

## Watch out
The current increment has twenty lexical matches, not twenty reproduced behavioral regressions. Contract coverage, compiler/eval parity, native severity and heap checks, generated-doc audits and the unchanged PHPT status delta are the supporting evidence. Record the exception in the commit and retain the raw check output. The check itself is not green; full PHP compliance remains open. Never rewrite an immutable learning to make a textual check pass.
