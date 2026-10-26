---
title: Scoped print_r calls keep argument lifetime and native address semantics
author: Guillaumeloulier
date: 2026-10-19
branch: work/spl-opcache-main-integration
cost: { minutes: 2147, tokens: 13043052 }
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
Route direct and callable PrintR lowering through one owned capture boundary, with return flag, physical Bool/Str/Mixed result kind and an explicit native-address-format selector. Buffer echo as well as return mode. Flush the completed echo string before retiring its source argument; guard output-handler/destructor errors and restore outer capture state. Preserve missing-container null semantics before boxing. Native Pointer/Buffer/Packed values need the original hexadecimal formatter: their generic Mixed representation is integer-shaped and loses that display information.

## Tried and ruled out
M12's generic boxing changed typed-buffer addresses from hexadecimal to decimal; pinned M10 versus M12 exposed the regression, and M13 restores the original formatter on all return paths. Validate complete echo/return/dynamic strings against each other and validate their hex spelling, since absolute addresses vary between processes. Echo Buffer and int casts are unsupported in the old baseline, so those initial probes could not be reference oracles. ctype_xdigit is also undefined here; the registered compatibility fixture now classifies ASCII hex digits directly. Two old assembly-position assertions required a direct walker that the scoped lowering no longer emits; replace them with full PHP-output tests for missing arrays and objects, preserving the actual sentinel requirement.

## Watch out
The M14 scoped lifecycle group passes 6/6, including source mutation, native formatting, output-handler reentry/throw, resources and argument destruction. The production binaries match M13. M13's complete-runtime assembly passes all five targets; linked core runtime plus user programs execute 16/16 PHP-oracle controls on Linux ARM64/x86_64 with clean heaps, plus two separate native-extension compatibility checks. These are not full corpus results. Native print_r still does not consume magic debug views, so its two original failures remain open. PHP permits Fiber suspension inside __debugInfo: do not merely flip capture mode around future callbacks. Detach active capture/scope globals into the callback's saved native frame while PHP executes, then restore them on return; also audit the global seen stack for out-of-order Fiber resumes. The Fiber switch examined saves exception/call-frame/magic-set guard state, so capture state must not be assumed fiber-local. No commit or publication has occurred.
