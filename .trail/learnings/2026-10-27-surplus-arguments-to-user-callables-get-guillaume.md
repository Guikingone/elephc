---
title: Surplus arguments to user callables get the func_get_args collector
author: Guillaume
date: 2026-10-27
branch: work/spl-opcache-main-integration
cost: { minutes: 176, tokens: 951416 }
touches:
  - src/codegen/lower_inst/objects/runtime_property_writes.rs
  - src/func_args/mod.rs
  - src/func_args/surplus.rs
  - src/func_args/walk.rs
  - src/undefined_constants.rs
  - tests/codegen/mod.rs
  - tests/codegen/surplus_arguments.rs
  - tests/error_tests/classes_traits.rs
  - tests/error_tests/type_system.rs
  - src/optimize/reachability/usage.rs
---

## Decided
- php ignores (but evaluates) arguments a user function/method/constructor does not declare; elephc refused them in the checker arity check. `src/func_args/surplus.rs` counts the largest positional call per callee (functions by name, methods by name, constructors by instantiated class + `extends` ancestors) and `walk_function_scope(collect_surplus)` installs the same hidden collector func_get_args uses, so the checker sees a variadic and lowering accepts the tail.
- Internal (prelude, `SourceMode::Internal`) declarations keep their arity: php's builtins throw on surplus args.
- `undefined_constants::walk_program_exprs_mut` is the shared pre-order mutable expression visitor (visit node, then children; through bodies, methods, closures). Reuse it instead of writing another walker.

## Tried and ruled out
- Keying `__construct` by method name: every `new X(a,b)` (preludes included) gave all user constructors a collector and changed too-few-args diagnostics to "expects at least". Key constructors by class.
- A codegen-time refusal inside runtime-name unset() ladder arms: ladders carry an arm for every class, so it failed builds that never unset (promoted_reference_guards_accept_materialized_cells). Raise in the arm instead.

## Watch out
- `new static`/`new self` and spread/named calls are not counted: those keep the compile-time arity check.
