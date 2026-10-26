---
title: "Post-merge reconciliation: double releases, receiver roots, stale main tests"
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 91, tokens: 381777 }
touches:
  - src/codegen/lower_inst/arrays.rs
  - src/codegen/lower_inst/objects/reflection/collection_emit.rs
  - src/ir_lower/expr/nullable_method_calls.rs
  - src/codegen/lower_inst/objects/runtime_property_writes.rs
  - src/docblock/bindings.rs
  - src/ir_lower/stmt/typed_foreach.rs
  - src/codegen/lower_inst/builtins/type_predicates.rs
  - src/ir/runtime_fn.rs
---

## Decided
- codegen lower_array_push: removed this branch's extra release_mutated_source_local_owner before ArrayPush; main retires the source owner itself (734ca7ddec), so the pair double-released an untaken reference view (bad refcount in test_ref_local_append_untaken_reference_native_views).
- ReflectionClass static property array: a Mixed static already holds a box owned by the static storage; the hash now gets __rt_mixed_clone instead of that shared owner (double free at exit once any Reflection object of the class was freed; main leaks instead).
- Method receivers: no borrowed-receiver root and no in-flight pin when receiver_needs_no_root ($this = ExprKind::This, or a plain frame local with trivially pure args). Restores main's method inlining and precise closed-world effect summaries (READS_FS leaked in through release_local_slot).
- unset() of an owned reference property releases the cell and installs a fresh uninitialized owned cell; a borrowed one only gets the uninitialized marker. Needed because main's _class_propinit thunks PropUnset reference slots and the runtime-name unset dispatch reaches them.
- docblock bindings try .lfc first for a tagless source: since the inline-HTML lexer change, tokenize() succeeds on code without <?php (one echo) and never fell back.
- is_object() on an iterable reads the payload heap kind (4 = object) instead of a static false; by-ref foreach over an iterable branches at run time (object -> Error, array -> iterate).
- __elephc_object_restore_property / spl_object_append effects exclude BLOCKING_IO|NETWORK_IO (destructors do their own I/O), so no monitoring policy is needed.
- Stale main tests updated to this branch's verified semantics: spl_classes() count 55 (php 8.5.10 includes RecursiveTreeIterator), fgetcsv 5 params, SplObjectStorage::__unserialize boxed array contract, runtime-name untyped stores boxed (dynamic properties), backtrace materializer excluded from asm counts.

## Watch out
- Every user function carries a backtrace frame materializer on this branch (uncaught trace parity); asm-count tests must count only the body before 'backtrace_frame_materialize'.
- Four SPL runtime_gc tests were red before the merge (bead elephc-4ml.11).
