---
title: Scoped SPL value snapshots must not expose raw handles through Reflection
author: Guillaumeloulier
date: 2026-10-20
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

A scoped borrowed value snapshot owns only a 32-byte raw-kind heap allocation: original dereferenced cell, logical tag, low word and high word. Capture must not retain the PHP payload. An ordinary read boxes an acquired owner through mixed_from_value; resource reads retain the original canonical cell. Free releases scratch only. Typed EIR targets and the complete runtime cover both architectures.

Keep these operations internal to EIR. The experimental private SPL method wrappers exposed raw integer handles: ReflectionMethod found SplHeap::__elephcHeapSnapshot in the M20 native binary. Those wrappers and their declarations were removed; they are not an acceptable PHP surface. Any future heap binding must keep scratch inside a validated instance/lifetime boundary and preserve PHP-visible metadata.

## Tried and ruled out

Calling mixed_clone on a raw copied resource cell can return the raw scratch pointer as an owned value, since resource cloning retains and returns its input cell. Store the original concrete cell separately instead.

The C observer initially called a PHP global-return function as though every return were owned. Native PHP returns can be borrowed (the hidden ownership flag is not a C contract). The final fixture obtains an acquired result through array_pop of a fresh local array before C releases it. Its reachability branch uses var_dump so source optimization cannot eliminate C-accessed function bodies. iOS CLI emission requires --emit staticlib.

## Watch out

M20 primitive controls have 12 actual executable runs on macOS ARM64 and Linux ARM64/x86_64, plus 8 iOS device/simulator assembly controls, using both IR optimizer modes. Executed controls have exact expected output and clean heap summaries; the C observer covers scalar bits, binary strings, arrays, hashes, objects, resource identity, capture refcounts and destructor resurrection. No runtime ownership/allocation helper is stubbed. These are internal primitive controls, not additional php-src PHPT passes.

Reads require a still-valid captured value. This does not prove post-destruction reads, clone callbacks, exception cleanup or SPL next root-retirement order. The M19 next regression remains open; do not connect a borrowed pointer to arbitrary callbacks without proving lifetime, resurrection and clone behavior. Evidence: .artifacts/integration/main-m20-borrow-primitives/targets-results-3.json; final source validation and the follow-up checkpoint are recorded separately.
