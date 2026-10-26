---
title: Parser and explicit property restore share helper emission ownership
author: Guillaumeloulier
date: 2026-10-16
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - docs/internals/builtins/_internal/__elephc_opcache_rt_blacklist_entry.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_compile.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_discard.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_in_file_cache.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_is_cached.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_reset.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_script_field.md
  - docs/internals/builtins/_internal/__elephc_opcache_rt_script_path.md
---

## Decided
The feature's parser writer and explicit internal restoration writer share one property-hash helper definition. Eval owns it when linked; otherwise the parser owns it when hydrate::is_used is true, and an active explicit writer emits it only without that parser capability. The capability scan covers the same lowered function collections as parser emission. The parser-specific TypeError path remains separate from explicit restoration because their context lifetimes differ.

## Tried and ruled out
Emitting the hash-slot helper independently from both writers duplicates its assembler symbol. That deduplication alone still leaves nullable-property branch labels colliding; the dedicated main correction namespaces those by writer entry. Keeping parsed scalar boxes or overwritten string defaults alive does not fix either symbol conflict. The six SplObjectStorage protocol controls, including the former 40-byte magic-pair leak, pass after the ordinary-owner prerequisite is integrated.

## Watch out
Preserve feature WIP when integrating main prerequisites: the parser helper is committed separately from the internal builtin, prelude protocol and generated registry/docs. Keep every runtime_gc module during union merges. The working feature snapshot passes 40 focused controls, while the final main prerequisite passes 12 controls and 36 native checks; two combined-writer configurations assemble on all five targets. Full 1710 PHPT acceptance, defaultless-property GC/JSON metadata and real iOS execution remain separate gates. Trail's historical-plan and generated-registry token overlaps describe earlier failures, not a reason to erase notes or weaken fixtures.
