---
title: SplObjectStorage magic serialization preserves native entries and user slots
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
Represent the storage's native objects, infos and cursor as reserved NUL-prefixed fields. Magic __serialize returns interleaved object/info entries plus get_mangled_object_vars user properties. Magic __unserialize validates shape, merges entries, updates duplicates, preserves the cursor and retains partial mutation before a later invalid key. Restore mangled properties through the internal borrowed typed-slot writer, which retains replacements and releases previous values; preserve distinct parent/child private physical slots. Builtin reachability must keep serialization hooks even when only serialize/unserialize references the class.

## Tried and ruled out
A plain get_object_vars projection loses private names; native backing slots must not appear in user property data. Calling the user's overridable attach method changes native hydration behavior. Treating get_mangled_object_vars as a statically specialized array loses boxed PHP array representation. Independent hash-helper emission collides with the parser writer; shared nullable stores need caller-specific labels. Renderer filtering must resolve the declaring physical slot before hiding Reflection backing fields, so private user __names remain visible.

## Watch out
The integrated tree passes 41 focused strict PHP/heap controls, including six magic-protocol cases, ordinary hydration, untyped GC/JSON and Reflection. Native feature matrix and unchanged full PHPT replay remain separate acceptance gates. The legacy Serializable string protocol, custom getHash semantics and exact Reflection object IDs remain open. This implements the modern magic protocol, not full SplObjectStorage or campaign completion. Preserve generated builtin docs and the independent main prerequisites.
