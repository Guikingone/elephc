# PHP fileinfo

- [x] Add a pinned static libmagic native package with its embedded magic database on every target.
- [x] Add the optional fileinfo bridge and PHP finfo class, six functions and eleven constants.
- [x] Expose the same declarations to dynamic eval and preserve ownership and failure behavior.
- [x] Add focused native, codegen, error and target coverage plus a practical example.
- [x] Generate and audit builtin documentation in CI without local builds or heavy tests.
- [x] Open a draft pull request and configure exact-head CI coverage for all supported targets.

## Implementation

Use upstream file/libmagic 5.46, pinned by source size and SHA-256. Build a host database compiler when cross-compiling and embed the compiled database in a target C shim, so deployed programs need neither a system libmagic nor an external database. Custom text and compiled databases remain supported through magic_load.

A small Rust staticlib owns opaque handles. The PHP surface uses ordinary synthetic declarations and shared contracts, following the existing XML prelude and native-function eval bridge. All generated code uses the existing EIR and extern ABI paths.

PHP 8 semantics include no-op finfo_close, per-call flag restoration, binary-safe buffers, false results on lookup failures, constructor exceptions, stream input and contexts, and destruction without duplicate release. CI validates the supported target matrix. The user explicitly requested no local heavy builds or tests.

## Validation and acceptance

Draft PR: https://github.com/illegalstudio/elephc/pull/2061.
Its check summary is the live record for the published head. All required checks
must pass before this task is complete; the PR remains a draft.

CI run 40123074238 at 8cd6740c32 passed all fourteen fileinfo execution tests on
Linux x86_64, including all six functions through opaque eval, HTTP context options,
custom databases, PHP 8.4/8.5 deprecations, subclass restrictions and heap ownership.
The separate FFI copied-string-return and throwing-handler heap regressions passed.
Linux x86_64 and ARM64 both passed the three native libmagic bridge tests.

Both iOS device and Simulator final links passed in run 40121874821. Their fixture
retains both buffer and file queries and links the complete native/stream graph.
The macOS archive job runs the same focused executable cases before publishing its
archive. The broader CI matrix checks integration with the remaining compiler.

The CI exporter, builtin architecture audit, documentation pipeline, static docs
audit and site validation passed. Generated artifacts were downloaded without local
compilation. The comparison reports 6/6 functions, 1/1 classes and 11/11 constants.
No local heavy builds or test runs are authorized for this task.
