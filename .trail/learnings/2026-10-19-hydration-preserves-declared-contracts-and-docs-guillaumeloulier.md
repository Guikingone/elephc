---
title: Hydration preserves declared contracts and docs use the canonical exporter
author: Guillaumeloulier
date: 2026-10-19
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
Keep the original declared property contract separate from EvalPropertySlot.ty, which is the physical bridge representation. bridge_storage_type erases literal false and heterogeneous scalar unions, so strict hydration masks, expected-type text and integer widening must read the optional declared_type captured from ClassInfo before that erasure. Untyped fields have no declared contract. The M4 native control matches PHP with a clean heap, all five hydration user assemblies pass, and all 54 focused unserialize codegen regressions pass on the reconciled main-based branch.

## Tried and ruled out
Validating bridge types instead of declared types silently accepted true for false and left int payloads unchanged in float|bool. Per-slot rejected-object class expansion caused ARM64 branch range errors; the shared runtime property TypeError builder and short conditional/full-range public-name dispatch fix it without reducing semantics. Decoder-owned rejected values remain under the active parser context; pass no extra rejected-box owner to the property error helper.

## Watch out
The canonical docs exporter must be built with curl enabled. The extractor only recognizes a fresh repo/target/{debug,release}/examples/gen_builtins, so copy the exact fresh exporter there when using another Cargo target directory. Host Python 3.9 lacks tomllib; the installed tomli works through a task-local runpy alias without changing repository scripts. Regeneration and all builtin/site/EIR boundary audits pass under .artifacts/builtin-docs-m4. Preserve pinned default Magician archives before a curl exporter build, then explicitly build and verify current-source staticlib artifacts for Magician, tz, phar and crypto before eval replay. The current opaque eval driver still fails code generation at debug metadata claiming PHP-visible __debugInfo parameters; M5 adds the exact class/signature to identify that defect. Four unchanged ArrayObject clone PHPTs remain mismatches. Do not mark eval, clone or the full 1710-PHPT goal complete.
