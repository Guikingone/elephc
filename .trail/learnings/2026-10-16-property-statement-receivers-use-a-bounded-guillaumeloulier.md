---
title: Property statement receivers use a bounded owner scope
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-nested-property-receiver-owners
touches:
  - docs/internals/the-codegen.md
  - src/ir_lower/stmt/instance_property_writes.rs
  - tests/codegen/runtime_gc.rs
  - ".artifacts/nested-property-receiver-owners/build-final.jsonl"
  - ".artifacts/nested-property-receiver-owners/build-final.log"
  - ".artifacts/nested-property-receiver-owners/chain_assignment"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.compile"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.php"
  - ".artifacts/nested-property-receiver-owners/chain_assignment.s"
  - ".artifacts/nested-property-receiver-owners/magic_receiver"
---

## Decided
Publish an owning property-statement receiver before RHS evaluation. Inner ordinary, hooked and magic setter paths borrow a load from that published slot. Retire the receiver once after successful or guarded completion; existing operand cleanup retires it during unwind. Preserve source evaluation order and borrowed-local behavior.

## Tried and ruled out
Releasing the RHS does not retire the lease produced by a property-chain read or a function-returned receiver. Publishing the receiver but forwarding the same owned SSA value into inner setter pins can register one lease twice; pass a borrowed slot view instead. The original hydration back-reference fixture is retained unchanged and uses this independent setup correction.

## Watch out
Exact main 132b4da6f7 leaks the chained Peer object/string before serialization, and misses temporary receiver destruction on ordinary and magic writes. Four initial strict controls pass; seven PHP-derived fixtures pass twenty-one native macOS ARM64/Linux ARM64/Linux x86_64 executions with clean heaps plus five-target assembly in .artifacts/nested-property-receiver-owners/target-matrix. That matrix also validates store TypeError and throwing magic setters. The extended forty-six setter/hook/eval controls are still running. Ordinary hydration ownership, source interface reference-return mismatch, shutdown cycle timing, full php-src acceptance and real iOS execution remain separate gates.
