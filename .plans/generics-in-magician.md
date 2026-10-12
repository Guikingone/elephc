# Generics in Magician

- [x] Share generic binding rules between AOT and runtime evaluation.
- [x] Parse native generic declarations, written arguments, and typed arrays.
- [x] Instantiate functions, classes, interfaces, traits, and methods, including enum methods in persistent eval contexts.
- [x] Retain PHPDoc generic annotations and enforce bounds and type contracts.
- [x] Cover inference, explicit arguments, defaults, namespaces, persistence, errors, ownership, and strict mode with focused tests.
- [x] Document runtime behavior and add an executable example.
- [x] Match PHPDoc inheritance annotations by qualified name and clause.
- [x] Validate nullable and union array writes against their structural arms.
- [x] Preserve indexed array key/storage contracts on writes and value boundaries.
- [x] Preserve class variance while distinguishing constructor specializations.
- [x] Resolve fully written class and constructor slots before any cached instantiation.
- [x] Validate float and bool array keys using their normalized integer type.
- [x] Release failed construction receivers while suppressing their destructors.
- [x] Reproduce the second review findings and validate native cleanup.

Magician resolves type arguments from evaluated runtime values. Templates remain syntax until a call or class reference requests an instantiation, which is registered as ordinary EvalIR under its concrete name. Argument evaluation and ownership stay on the existing call paths. Generic binding rules must agree with AOT, while runtime execution continues to use boxed cells and catchable type errors.

Keep source locations and namespace imports when retaining templates. Script-cache serialization must invalidate older layouts. Native syntax obeys strict PHP mode; PHPDoc annotations remain valid in strict mode. Specialization must recurse through bodies and nested types without changing literal strings or variable names.

## Implementation

`elephc-generics` provides dependency-neutral type terms, binding conflicts, written arguments, dependent defaults, and depth limits. AOT adapts its existing type expressions through `src/generics/shared.rs`; Magician infers corresponding terms from runtime values.

The parser retains declaration prototypes and tokens marked by lexical type-parameter identity. Replay preserves imports, namespaces, literal bytes, attributes, source lines, and constructor-local shadowing. Native syntax and PHPDoc provenance remain distinct for strict mode. File-cache format 13 covers the additional serialized metadata and invalidates older PHPDoc inheritance rewrites.

Concrete functions, class-like declarations, and methods use ordinary EvalIR execution. Function and class templates occupy separate PHP namespaces, and normal declarations reject conflicting templates. Persistent-context entry points, first-class callables, callback aliases, and call argument arrays resolve templates before normal binding. `interpreter/context_calls.rs` keeps the interpreter dispatcher small.

Runtime contracts validate bounds, variance, structural arrays, typed callbacks, local assignments, and returns. Reference parameters use dedicated reference wrappers; by-value parameters detach array boxes before COW writes. Duplicate reference targets compare normalized array keys. These paths preserve the existing boxed-cell ABI on every target.

## Validation

Focused checks cover Magician generics, lexer/parser behavior, cache serialization, callable dispatch, ownership, and reference binding. Real runtime-selected eval tests cover persistence, includes, strict PHPDoc, native caller access, the example, and heap-debug cleanup. AOT unit/error tests and focused constructor, namespace, and default regressions cover the shared binding changes.

`cargo build`, `git diff --check`, the generated builtin-doc workflow, site compatibility validation, and the target-architecture EIR boundary audit pass. Generated builtin registries and pages remain unchanged. User documentation lives in `docs/beyond-php/generics.md`, `docs/php/eval.md`, and `docs/internals/eval-runtime.md`; the executable example is `examples/eval-generics/main.php`.

## PR 1819 review follow-up

Reproduce each of the four findings on commit `7dd6c551b9` before applying the correction. Cover qualified inheritance clauses with parser and runtime tests, nullable/union local writes and rejected associative keys with native runtime tests, and class-plus-constructor variance with distinct concrete identities. Re-run focused Magician generic tests, native eval regressions, a warning-free build, and diff hygiene before publishing the correction to the draft PR.

Six added regressions fail on the original implementation and cover all four findings. The corrections match full written parent names within their own clauses, select coherent array union arms, enforce indexed keys and storage, and encode all class/constructor bindings under the original class base. Inference and variance compare the declared class slots while constructor slots keep concrete registry identities distinct. Native zero-property objects contribute type evidence; only actual empty arrays are skipped. File-cache format 13 discards parsed PHPDoc inheritance produced by the old rewrite without changing serialized shapes.

Validation passes: 46 generic parser/interpreter tests, 226 parser tests, 150 cache tests (one unrelated ignored), and 12 native eval integration tests. The native regressions cover strict PHPDoc, normalized array keys, rejected writes, structural boundaries, AOT object hierarchy, and clean heap-debug reports. `cargo build`, Rust documentation/added-text hygiene, and `git diff --check` pass.

## Second PR 1819 review

The review of `5590f4eb50` identifies two remaining regressions: a concrete class-plus-constructor spelling fails before inference creates its cache entry, and typed array writes reject float/bool keys that PHP stores as integers. Add cold-cache, inheritance, default/bound, and normalized-key regressions before correcting either path. Keep operand evaluation and native array-key diagnostics on their existing storage paths.

Both new native regressions fail on the reviewed head. Combined written lists now bind class and constructor parameters; class-only lists retain constructor inference. Float/bool keys use a temporary integer cell for validation, with balanced release and the original storage operand preserved.

Constructor rejection additionally exposed an allocated receiver leak. Construction and property initialization now release partial receivers on error and suppress their destructors, preserving metadata for escaped references. Tests cover constructor failures, native parents, by-reference receiver escape, and property-default failure cleanup.

Validation passes: 51 generic parser/interpreter tests, 20 object lifecycle tests, 33 reference tests, and 15 native eval integration tests with clean heap reports on the regression fixtures. `cargo build`, Rust documentation, diff hygiene, and task-owned temporary cleanup pass. The recompiled example prints `42|Magician|8` followed by `type checked`. A separate follow-up records pre-existing static-property and class-constant cell retention during eval context teardown.
