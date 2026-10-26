---
title: Error handler args box and container return shortcuts
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 39, tokens: 162521 }
touches:
  - src/codegen/lower_inst/core_builtins/handlers.rs
  - src/ir_lower/stmt/return_coercions.rs
---

## Decided
- lower_diagnostic (trigger_error / __elephc_deprecated with a user handler) must PUSH the owned args box after emit_invoker_arg_mixed: the call reads arg1 from temporary slot 0 and pops 16 bytes afterwards. The branch had replaced main's emit_push_reg with a reg_move, so every handled trigger_error/engine deprecation read a stale stack word (segfault, or 'callable descriptor invoker expected an indexed or associative argument array'). This predates the main merge (the M45 pinned compiler crashes too).
- coerce_heap_return_source: main's mixed-slot shortcut (unbox to array<_, never> then ArrayToMixed/HashToMixed) is removed. A Mixed cell can hold either container kind (SPL storage declares `array` as an associative contract while holding a list), and the plain unbox handed a list to __rt_hash_to_mixed (crash in hash_iter_next from ArrayObject::getArrayCopy). Container returns now always use the promoting converters plus the rooted cell release; main's leak test test_match_call_and_parameter_arms_returned_as_containers_heap_clean still passes.

## Watch out
- A runtime-call conversion to array<_, never> is the EMPTY-array contract (plain unbox); never use it as a generic 'payload of whatever container' unbox.
