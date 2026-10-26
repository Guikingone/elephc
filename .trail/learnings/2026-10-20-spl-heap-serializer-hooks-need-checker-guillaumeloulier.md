---
title: SPL heap serializer hooks need checker bodies and EIR support registration
author: Guillaumeloulier
date: 2026-10-20
branch: work/spl-opcache-main-integration
---

## Decided
SplHeap and SplPriorityQueue __serialize export the native indexed [user properties, state] pair. Use the mangled member snapshot with exact hidden-slot removal; preserve physical binary heap order, queue data-before-priority records and extraction flags. Reject corruption before mutation lock, with the distinct Cannot serialize heap while it is being modified. message. The writer uses the existing boxed packed-or-hash PHP array contract. M28 accepts this write-side slice only: 41 focused native tests, four writer controls on both optimizer modes across macOS ARM64 and both Linux architectures, both iOS assembly targets, and a runnable example. Exact-artifact selected upstream PHPT improves from 25/36 to 27/36; full compliance remains false.

## Tried and ruled out
Adding checker method bodies alone left magic serialize using plain object properties and direct __serialize calls jumping to address zero. Add __serialize to each heap owner's EIR supported builtin SPL method whitelist; generic serialize discovery already retains supported magic hooks. M26's failing public-pipeline evidence is preserved. A ReflectionMethod foreach over dynamic class names is unsupported. A full literal Reflection probe also exposed getReturnType returning array instead of PHP's null tentative type and four leaked strings. The same count-method probe reproduces wrong int output and exactly four leaked strings on both M25 and M27, proving this predates the writer. Do not count that probe as passed: the committed-source regression scope is arity/visibility only, and the full failing artifacts remain explicit campaign gates.

## Watch out
M27 and M28 CLI, rlib and default Magician artifacts are byte-identical; only the new regression's scope changed. Reuse exact-artifact PHPT/matrix evidence only after checking those hashes and the sole source delta. Eight loading PHPT failures and one by-reference compare error remain in the 36-case heap selection; 1674 corpus cases are unselected. Native heap __unserialize has a type-bypassing property loader with initialized readonly enforcement, so it needs a distinct authority/representation design rather than weakening the checked generic writer. Preserve the accepted M25 root-retirement checkpoint and M28 writer checkpoint while addressing that next family.
