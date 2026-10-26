---
title: Borrowed method receivers need independent lifetime roots when callbacks overwrite bindings
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
An ordinary method call must hold a separate reference to a borrowed receiver before evaluating arguments, through callee execution and argument cleanup. Publish that reference in an OwnedTemp call-operand ledger slot and use the lifetime-pin Acquire marker, then retire it at the call boundary. Nullable and nullsafe dispatch share the same helper. Independently owned receiver temporaries keep their existing cleanup path.

## Tried and ruled out
Main 09c634061a reproduces the failure with an ordinary class: a method callback clears the only global binding, the destructor runs early, and a later property read returns heap poison. A final clean heap report does not prove lifetime safety. Pinning only independent SSA owners, as the earlier receiver-unwind repair does, cannot protect a borrowed global/local/ref-cell receiver because such a pin acquires nothing.

## Watch out
Six generic regressions pass in both optimizer modes on fix/main-borrowed-method-receiver: callback global clearing, argument global clearing, callee throw, nullsafe dispatch, local reference mutation and property receiver mutation. This bounded fix is copied into the SPL integration worktree to address its receiver_unset crash; full target and PHPT gates remain open until fresh artifacts are checked. Preserve the separate baseline worktree. Shared Cargo targets can reuse another checkout's builtin-contract archive; refresh all current-checkout crate roots without content changes and verify source/archive provenance before trusting a rebuild.
