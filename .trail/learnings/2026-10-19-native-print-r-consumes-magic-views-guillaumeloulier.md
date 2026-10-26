---
title: Native print_r consumes magic views with detached callback captures
author: Guillaumeloulier
date: 2026-10-19
branch: work/spl-opcache-main-integration
cost: { minutes: 2186, tokens: 13158608 }
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
Native print_r now invokes the existing magic-debug callable descriptor after recursion detection and before marking the receiver. Retain the logical view array independently from its result/reference cell, retire that cell, traverse keys and value triples with __rt_array_iter_next, and destroy temporary views outside capture before emitting the completed root string. Recursive values carry independent input leases. Child destructor and null-deprecation exceptions belong to the owning render scope until text is complete. Detach all print_r capture/scope globals into the native callback frame before arbitrary PHP, restoring them after callbacks or Fiber suspension.

## Tried and ruled out
Sharing only generated metadata did not consume the hook. Sharing only a result-cell owner did not retain the active container independently. x86 recursion push needs the C ABI rdi receiver, unlike internal helpers taking rax. The key decoder is now a shared pure borrowed-slice parser for print_r and var_dump, following Zend's anonymous-source delimiters and any leading-star protected class segment. A minimal Linux runtime fixture omitted handler_state and failed handler-shutdown linking; enabling the required feature corrected the fixture without a compiler change. Fatal stdout-only comparison missed native stderr: preserve and compare merged user streams. Heap instrumentation can start immediately after user bytes without a newline, so strip only explicit HEAP DEBUG records at any byte offset in instrumented custom controls; the upstream PHPT matcher is unchanged.

## Watch out
M16 print_r suite passes72/72; its pinned executable also passes20/20 existing object debug rendering regressions. All-feature runtime assembles on five targets. Two unchanged upstream debug PHPTs pass, complete=false. Linked full core runtime magic controls execute on both Linux architectures, with handler_state/fiber enabled in the corrected fixture. Custom merged streams are17/19 exact: visible hook-exception warning/trace/location and isset after throwing result assignment remain gaps. Fiber text matches but strings leak; independent M14 probes without print_r reproduce both generic isset behavior and a Fiber method leak, but exact-main attribution still requires the clean-base build. Upstream bug69264 is still compile_error for AssocArray<Str,Array(Mixed)> to Array(Mixed); pinned M14 also reproduces it, so it predates this consumer. Keep all negative evidence and the full1710 corpus gate open. No publication or full-compliance claim.
