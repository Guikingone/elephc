---
title: Unwinding compiler calls must root owned operands
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 1, tokens: 1155 }
touches:
  - src/ir_lower/expr/compiler_calls.rs
  - src/names/compiler_calls.rs
  - src/codegen/eval_property_helpers/native_restore.rs
---

## Decided
- A compiler-only call whose runtime helper raises through `__rt_throw_current` (today only `CompilerCall::NativePropertyRestore`: readonly, reference-detachment, invalid-receiver errors) skips the post-call release loop in `ir_lower/expr/compiler_calls.rs`. Its owning temporaries leaked: `strval($propertyName)` in the heap `__unserialize` prelude left one 48-byte string ("locked") per readonly failure.
- `CompilerCall::may_unwind()` marks those operations; the lowering roots owning operands with `root_owned_call_operand` (PushCallOperandOwner) before the call and retires the slots after it, skipping them in the ordinary release loop so nothing is released twice.

## Tried and ruled out
- Returning the readonly Error as a pending result like destructor failures: php stops the property loop at the readonly error (after stays 1), so a real throw is required.
- Hoisting strval into a prelude local: fixes one site, not the operation contract.

## Watch out
- A new compiler call whose helper jumps to `__rt_throw_current` must return true from `may_unwind()`; helpers that return a pending Throwable (NativePropertyReplace, dynamic set behind emit_guarded_cleanup_call) do not need it.
- Evidence: test_spl_heap_unserialize_property_readonly_chain heap-clean; 44 spl_heap, 9 spl::heaps, 65 operand_owners tests pass on macOS ARM64.
