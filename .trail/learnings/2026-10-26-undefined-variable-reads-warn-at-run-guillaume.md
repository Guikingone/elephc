---
title: Undefined variable reads warn at run time (php semantics)
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 110, tokens: 395823 }
touches:
  - src/undefined_reads.rs
  - src/ir_lower/expr/compiler_calls.rs
  - src/main.rs
  - src/ir/instr.rs
  - tests/codegen/undefined_variable_reads.rs
---

## Decided
- A read of a variable that NO statement in its scope can write is php's run-time `Warning: Undefined variable $x` + null, not a compile error. `src/undefined_reads.rs::rewrite` (called first in `func_args::desugar_on_compiler_stack`, so both pipeline and the codegen harness see it) rewrites such reads to `CompilerCall::UndefinedVariable("x")`, lowered to `Op::Warn` + Null.
- `Op::Warn` must carry `E::MAY_WARN`: that is what makes `prepare_call_site` publish `_php_diagnostic_file/_line`; without it the warning named the wrong line.
- A scope is skipped entirely (no rewrite) when it is dynamic: extract/compact/get_defined_vars/parse_str/eval/include/func_get_args/$GLOBALS/arrow fn. Probes (isset/empty/unset, `??` left side) are untouched; `@$x` becomes plain null.
- Call arguments count as writes (by-ref params may define them) EXCEPT global builtins whose registry entry has no `ref_params` (`is_by_value_builtin`), so `explode(':', $undef)` warns.
- `extern global` names are visible in every scope (thread-local EXTERN_GLOBALS seeds each ScopeFacts); missing this broke examples/ffi in the lowering corpus.
- error_tests that asserted compile-time `Undefined variable: $x` for never-written names were moved to codegen `tests/codegen/undefined_variable_reads.rs` (php behavior). The checker's own `Undefined variable:` diagnostics remain for flow-sensitive cases (read after unset, read before a later write).

## Watch out
- The rewritten read is typed Void, so a scalar builtin parameter still refuses it at compile time (`strlen($undef)` -> "strlen() argument must be string"); php passes null with a Deprecated. Same gap as `strlen(null)`, open.
- compile_and_run's stdout filters php display-diagnostic lines; assert warnings via compile_and_run_capture().stderr.
