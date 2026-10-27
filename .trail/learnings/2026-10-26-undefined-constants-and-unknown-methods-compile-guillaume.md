---
title: Undefined constants and unknown methods compile; php decides at run time
author: Guillaume
date: 2026-10-26
branch: work/spl-opcache-main-integration
cost: { minutes: 115, tokens: 340066 }
touches:
  - src/codegen/lower_inst/builtins/system.rs
  - src/types/checker/builtins/language_constructs.rs
  - tests/codegen/scalar_strings.rs
  - src/undefined_constants.rs
  - src/names/compiler_calls.rs
  - src/ir_lower/expr/compiler_calls.rs
  - src/types/checker/mod.rs
  - src/types/checker/driver/init.rs
  - src/types/result.rs
  - src/types/checker/inference/expr/calls_objects.rs
---

## Decided
- An undefined constant read is php's run-time `Error: Undefined constant "X"`, not a compile error. The checker records `(span, name)` in `Checker::undefined_constant_reads`, types the read `mixed`, and `check_types_on_compiler_stack` keeps only names still missing from the FINAL constant table (the table grows while walking). `CheckResult::undefined_constant_reads` is applied by `src/undefined_constants.rs::rewrite`, called at the end of `generics::monomorphize` -- library code both the pipeline and the codegen/error harnesses run, so no plumbing into LoweringContext was needed. The read becomes `CompilerCall::UndefinedConstant`, lowered with `lower_throw_access_error_expr`.
- A method the receiver's declared class/interface lacks (`Iterator $it; $it->hasChildren()`, `A $a; $a->subclassOnly()`, union/static|Parent returns) is deferred the same way: `defer_unknown_method_call` (methods.rs) records `(span, class, method)`, types the call `mixed`; finalization keeps sites whose final class table still lacks the method; the rewrite wraps the receiver in `CompilerCall::MixedReceiver` (a `MixedBox`), so lowering takes `lower_mixed_method_call`. Its no-match arm now throws `Call to undefined method <runtime class>::m()` via `conversions::emit_error_with_receiver_class` (was "Call to a member function m() on null").
- Built-in (is_internal) classes still get a compile WARNING "Undefined method: C::m is not implemented..." so the Cairo "report, don't stub" decision stays visible (tests/error_tests/image.rs).
- `exit(string)` prints and exits 0; Mixed status dispatches on tag 1 (system.rs lower_exit).
- `LoadGlobal` fallback for a runtime-registered constant must read `Mixed`: `StoreGlobal` always boxes. Reading it as the syntactic `int` printed the box address for `const B = 2;` placed after an `if` (pre-existing).

## Watch out
- Removing tests with a regex over `fn name() {.*?\n}\n` stops at a `}` at column 0 inside a PHP raw string; check the diff.
- Rebuilding while `cargo test --test codegen_tests` runs makes eval tests fail en masse (libelephc_*.a replaced under them). Never build during a full run.
- Pre-existing leak (elephc-4ml.17): a throw out of a Mixed-receiver call inside a typed-return function leaks one block.
