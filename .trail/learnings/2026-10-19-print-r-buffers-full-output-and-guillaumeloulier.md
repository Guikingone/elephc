---
title: Print_r buffers full output and capture ownership must survive callbacks
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
Follow PHP 8.5.10 Zend/zend.c rather than var_dump's streaming model: print_r builds its whole output before emitting it, even in echo mode. Hooks and temporary debug-view destructors therefore produce their side effects before the final text. Source-argument ownership lasts through the output flush. Grow capture storage as an owned string allocation, retain a terminator, retire old growth allocations and transfer the final allocation at finish. Active pointer, capacity, length and mode must later be scoped per call for nested print_r callbacks.

## Tried and ruled out
M8's static 64 KiB buffer truncates PHP's 70007-byte result to 65540 bytes. The old test asserting that cap was an Elephc limitation, not the PHP oracle; replace it with a complete-byte comparison. Adding __debugInfo directly to the old object streamer would still get hook/destructor ordering and nested captures wrong. M9/M10 fix capture growth, not magic-view integration. The before-change print_r suite had 47/49 passes; the new suite has 48/50, with only the same two unimplemented magic-view tests failing.

## Watch out
All six large/empty/sequential/runtime-flag controls match every PHP byte. Multiple-growth direct str_repeat producers retain two strings totaling 220032 bytes on both pinned M8 and M9: preserve that negative evidence and diagnose producer ownership separately. The bound-value growth control is exact and clean. Complete runtime assembly passes all five targets; extracted buffer helpers execute on both Linux architectures with real ABI adapters and C heap/release stubs, comparing 220021 bytes including NULs and balancing 53 allocations/releases. This is separate from full Linux runtime or PHPT compliance. Null-debug deprecation-handler exceptions render the empty view before propagating; a throwing hook is fatal. A pending debug-view destructor exception followed by another magic hook is also fatal before flushing, so clearing it and invoking later hooks would change PHP behavior. Oracles and checkpoints are under .artifacts/integration/print-r-oracles and main-m9/main-m10. No 100% or publication claim.
