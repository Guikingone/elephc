---
title: SPL object stores need separate raw named and public views
author: Guillaumeloulier
date: 2026-10-15
branch: feat/spl-opcache-php-src-1806
touches:
  - ".plans/spl-opcache-php-src-1806.md"
  - crates/elephc-builtin-contract/src/catalog_data.rs
  - crates/elephc-builtin-contract/src/support.rs
  - docs/internals/builtins/_internal/__elephc_strtotime_raw.md
  - docs/internals/builtins/image/_cairo_clamp8.md
  - docs/internals/builtins/image/_cairo_color.md
  - docs/internals/builtins/image/_cairo_fx.md
  - docs/internals/builtins/image/_cairo_pack.md
  - docs/internals/builtins/image/_elephc_img_output.md
  - docs/internals/builtins/image/_gmagick_norm_color.md
---

## Decided
Match php-src 8.5.10 (34308a6666b2d489c509541ea9befea9e2b42348): object-backed [] and offsetSet(null, value) append a real integer property bucket. A named property "0" is distinct. Offset lookup ignores integer buckets; getArrayCopy preserves the complete visibility-mangled raw table; iteration and count filter public properties while preserving raw key kinds. Ordinary get_object_vars and object-array casts keep their separate normalization contract.

Four compiler-internal typed contracts drive append/raw/named/iteration helpers through EIR and the shared target runtime. Append consumes one extra owned Mixed lease; projection results are fresh. Nested wrappers forward to terminal storage, self-backed views filter only exact mangled compiler fields, and object-tail metadata remains authoritative. Static object [] uses ordinary offsetSet call lowering so argument/unwind ownership matches method calls.

## Tried and ruled out
A universal get_object_vars projection dropped private properties and merged key kinds. A universal raw copy made array_012 iterate protected/private fields. A named "0" write cannot emulate integer append. Matrix-v1 omitted object_clone; v2 enabled eval_bridge unnecessarily. Preserve those invalid configurations. Main return-box and widened-container ownership defects are fixed on their own branches and cherry-picked separately, rather than weakening heap checks.

## Watch out
Ten new heap-clean regressions and the 373-test SPL suite pass. Thirty native executions match PHP on macOS ARM64 and both Linux architectures; every program/runtime assembles for all five targets. iOS SDK/native acceptance remains open. Four-helper generated docs and all three audits, three contract-support and seven builtin-parity checks pass.

The unchanged 108-case ArrayObject replay remains 35 pass / 46 mismatch / 26 compile_error / one oracle_error, exactly the preceding deprecation checkpoint. It recovers only the intermediate prototype's lost array_012 pass; do not invent a net corpus gain. The full frozen 1710-case aggregate is still 635 passes. Grouped PHP-check logs identify 18 output differences beyond Rust assertions: four canonical temporary-path differences and fourteen semantic/diagnostic gaps. Wrapper array-cast projection is the next proven PHPT root; do not expose compiler state or claim complete SPL/OPcache parity.
