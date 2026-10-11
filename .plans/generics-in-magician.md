# Generics in Magician

- [x] Share generic binding rules between AOT and runtime evaluation.
- [x] Parse native generic declarations, written arguments, and typed arrays.
- [x] Instantiate functions, classes, interfaces, traits, and methods, including enum methods in persistent eval contexts.
- [x] Retain PHPDoc generic annotations and enforce bounds and type contracts.
- [x] Cover inference, explicit arguments, defaults, namespaces, persistence, errors, ownership, and strict mode with focused tests.
- [x] Document runtime behavior and add an executable example.

Magician resolves type arguments from evaluated runtime values. Templates remain syntax until a call or class reference requests an instantiation, which is registered as ordinary EvalIR under its concrete name. Argument evaluation and ownership stay on the existing call paths. Generic binding rules must agree with AOT, while runtime execution continues to use boxed cells and catchable type errors.

Keep source locations and namespace imports when retaining templates. Script-cache serialization must invalidate older layouts. Native syntax obeys strict PHP mode; PHPDoc annotations remain valid in strict mode. Specialization must recurse through bodies and nested types without changing literal strings or variable names.

## Implementation

`elephc-generics` provides dependency-neutral type terms, binding conflicts, written arguments, dependent defaults, and depth limits. AOT adapts its existing type expressions through `src/generics/shared.rs`; Magician infers corresponding terms from runtime values.

The parser retains declaration prototypes and tokens marked by lexical type-parameter identity. Replay preserves imports, namespaces, literal bytes, attributes, source lines, and constructor-local shadowing. Native syntax and PHPDoc provenance remain distinct for strict mode. File-cache format 12 covers the additional serialized metadata.

Concrete functions, class-like declarations, and methods use ordinary EvalIR execution. Function and class templates occupy separate PHP namespaces, and normal declarations reject conflicting templates. Persistent-context entry points, first-class callables, callback aliases, and call argument arrays resolve templates before normal binding. `interpreter/context_calls.rs` keeps the interpreter dispatcher small.

Runtime contracts validate bounds, variance, structural arrays, typed callbacks, local assignments, and returns. Reference parameters use dedicated reference wrappers; by-value parameters detach array boxes before COW writes. Duplicate reference targets compare normalized array keys. These paths preserve the existing boxed-cell ABI on every target.

## Validation

Focused checks cover Magician generics, lexer/parser behavior, cache serialization, callable dispatch, ownership, and reference binding. Real runtime-selected eval tests cover persistence, includes, strict PHPDoc, native caller access, the example, and heap-debug cleanup. AOT unit/error tests and focused constructor, namespace, and default regressions cover the shared binding changes.

`cargo build`, `git diff --check`, the generated builtin-doc workflow, site compatibility validation, and the target-architecture EIR boundary audit pass. Generated builtin registries and pages remain unchanged. User documentation lives in `docs/beyond-php/generics.md`, `docs/php/eval.md`, and `docs/internals/eval-runtime.md`; the executable example is `examples/eval-generics/main.php`.
