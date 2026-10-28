---
title: Run-time SPL autoload stack for standalone programs
author: Guillaume
date: 2026-10-28
branch: work/spl-opcache-main-integration
touches:
  - src/spl_autoload_prelude.rs
---

## Decided
- Standalone programs (no PSR-4 index, no consumed closure rule) get php's run-time loader stack as an AST prelude in src/spl_autoload_prelude.rs, built with synthetic_class builders. spl_autoload_* and the {class,interface,trait,enum}_exists probes are rewritten to hidden __elephc_* helpers; calls with named or spread args are left alone.
- The gate is `!autoload_registry.is_empty()`, not just PSR-4: spl_autoload_functions() must keep counting the compile-time rules, otherwise it returns an empty list.
- Loaders live in an untyped static prop of a hidden class (__elephc_spl_autoload_state). A function static widened badly, and a t_array()-typed prop read back garbage.
- A closure rule whose body has no include/require resolves to Resolved::Unknown in autoload/rule.rs, so it stays a run-time registration instead of being stripped.
- class_exists accepts a non-literal Str name only inside the exists helper (builtins/callables/support.rs).

## Tried and ruled out
- unset() of a static prop element is unsupported: copy to a local, unset there, store array_values back.
- ltrim() on a param that was once an array refused: the stack call helper takes `string $name` separately.

## Watch out
- A program that mixes a file-loading closure with a run-time-only loader keeps the old compile-time path; the run-time loader is not registered.
- Still open: `new Unknown` must run the loaders and then throw Class "X" not found; the "non-static method cannot be called statically" message; Closure var_dump; prelude frames (line 1000032) leak into stack traces.
- The REPL terminal tests time out (host build ~186 s vs a 150 s deadline) when other suites load the machine; that is load, not this change.
