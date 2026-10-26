---
title: Merging origin/main into the SPL main-integration WIP
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 42, tokens: 171923 }
touches:
  - src/ir_lower/stmt/array_write_core.rs
  - src/ir_lower/stmt/static_property_writes.rs
  - src/ir_lower/expr/method_calls.rs
  - src/ir_lower/stmt/return_coercions.rs
  - src/ir_lower/stmt/control_exit.rs
  - src/codegen/lower_inst/objects/fixed_new.rs
  - src/codegen/block_emit.rs
  - src/codegen_support/runtime/arrays/decref_any.rs
  - src/codegen_support/runtime/system/unserialize/storage_aarch64.rs
  - crates/elephc-builtin-contract/src/support.rs
---

## Decided
- Merge of origin/main (810 commits) into work/spl-opcache-main-integration (WIP checkpoint f978abe9c7): 104 conflicts, 66 generated docs/internals/builtins pages taken from main (regenerate, do not hand-merge).
- Array writes: KEEP the branch's `with_write_key_and_value` (closure + `capture_array_write_key` roots, StrPersist covers main's persist_string_byte_view) for local and static writes; main's `lower_write_key_then` / `lower_concrete_array_set_values` stay for buffer_element_updates. Main's middle code that passes `index_pins` runs inside our closure with an empty pin vector (the key is already rooted). Main's `retype_value_for_container_element` was added inside our closure.
- Method receivers: main's deferred-default receiver root (`root_owned_call_operand`) comes first; our borrowed-receiver root and in-flight pins apply only when that root is absent, never both (double release).
- Return coercions: main's `coerce_heap_return_source` kept, with our unwind root of the owning Mixed cell inside it (the unbox can throw TypeError).
- fixed_new: our FixedObjectLayout (php construction order) now carries main's `PropertyInitialization` (thunk fallback for non-literal defaults); the unconstructed-object guard also covers thunk initialization.
- block_emit frame identity stays `true` (uncaught report prints traces); main's module_uses_backtrace / Module::uses_php_backtrace became dead and were removed.
- decref_any: ours (only element kinds 4..7 are swept graph nodes) is a superset of main's kind-0 exemption.
- __rt_obj_store_prop: our removal kept (typed writers in eval_property_helpers/hydrate.rs own previous-value release); main's runtime previous-value release fix is superseded.
- Builtin contract counts = base + our delta + main delta (1176, 692, 131, 765, 604, 484).

## Watch out
- origin/main needs rustc >= 1.97 (mago-* crates): build with `cargo +1.98`.
- scripts/php_src/replay.py calls --elephc unfrozen: never rebuild or merge while a replay runs.
