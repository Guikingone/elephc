---
title: Reconcile SPL compliance on main without losing hydration contracts or archive provenance
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
Port the complete final feature state as one three-way delta from its common base onto a new branch rooted at a pinned main SHA. Keep the original integration worktree and immutable artifacts intact. The main reconciliation at df57a9800f4fb61bd4e876f0ddf309b9d08109f2 resolves 106 conflicts; per-file decisions are under .artifacts/main-reconciliation. Preserve main's single-evaluation receiver/operand-root paths and insert SPL live-cell writes into their already-lowered value helpers. Keep feature deferred-key capture together with main sparse/reference/invoker and readonly behavior.

## Tried and ruled out
Building only -p elephc did not rebuild the Magician staticlib. An absent compiler-artifact list is not evidence of an empty feature set: it allowed an old archive to be pinned. The pinner now explicitly selects elephc-magician, checks its current source path, requires one artifact with features=[], and fails closed otherwise. M2 verifies the default archive and all pure AOT insertion, renderer and lifetime controls; array-view outputs are 19/20 and every heap is clean. Reconciliation removes the earlier generic insertion and sort leaks without another bespoke ownership patch.

Per-property rejected-object diagnostics expanded every class and produced 41 MB / 1.15 million assembly lines, overflowing ARM64 conditional branches. Replace that cross-product with the shared runtime property TypeError helper, nearby store-failure trampolines and inverse-condition/full-range public-name dispatch. M3 hydration user assembly passes all five targets at about 5 MB. Its native test exposed bridge_storage_type erasure: false and float|bool must retain their original logical contract separately from the physical bridge type. M4 adds that separate declared-type option; its runtime validation remains pending at this note.

## Watch out
The decoder transfers parsed producer ownership to the active context before calling the property writer. Shared property-error diagnostics must receive rejected-box owner zero and leave context closure to the decoder boundary. Preserve strict tag masks, false-only bool membership, canonical type spelling and integer widening when moving a raw runtime writer to a generated retaining writer. Do not replace physical storage types with source contracts. Keep all generated pages pending a canonical curl-enabled exporter/audit run, and keep pinned default Magician archives isolated from that curl build. Clone semantics, eval integration, print_r, filesystem views, full diagnostic protocol, target execution and all 1710 unchanged PHPTs remain open. No commit or publication was performed.
