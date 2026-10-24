# PHP fileinfo

- [x] Add a pinned static libmagic native package with its embedded magic database on every target.
- [x] Add the optional fileinfo bridge and PHP finfo class, six functions and eleven constants.
- [x] Expose the same declarations to dynamic eval and preserve ownership and failure behavior.
- [x] Add focused native, codegen, error and target coverage plus a practical example.
- [x] Generate and audit builtin documentation in CI without local builds or heavy tests.
- [ ] Open a draft pull request, follow its exact-head CI and correct failures.

## Implementation

Use upstream file/libmagic 5.46, pinned by source size and SHA-256. Build a host database compiler when cross-compiling and embed the compiled database in a target C shim, so deployed programs need neither a system libmagic nor an external database. Custom text and compiled databases remain supported through magic_load.

A small Rust staticlib owns opaque handles. The PHP surface uses ordinary synthetic declarations and shared contracts, following the existing XML prelude and native-function eval bridge. All generated code uses the existing EIR and extern ABI paths.

PHP 8 semantics include no-op finfo_close, per-call flag restoration, binary-safe buffers, false results on lookup failures, constructor exceptions, stream input and contexts, and destruction without duplicate release. CI validates the supported target matrix. The user explicitly requested no local heavy builds or tests.

## Current validation

Draft PR: https://github.com/illegalstudio/elephc/pull/2061.
The Linux x86_64 and ARM64 archive builds passed, including the three native bridge
tests against libmagic and its embedded database. CI caught a missing generated
magic.h header for Automake's named targets; the recipe now generates it explicitly
for both host and target builds.

The CI exporter, builtin architecture audit, documentation pipeline tests, static
docs audit and site validation passed. Generated artifacts were downloaded without
local compilation. The comparison reports fileinfo as 6/6 functions, 1/1 classes
and 11/11 constants. PHP 8.5 deprecations follow the selected PHP version profile.

The CI archive jobs now execute all thirteen focused PHP fileinfo tests before
publishing test archives. PHP execution, the broader regression matrix and Apple
target validation are still pending. No local heavy builds or tests are authorized.
