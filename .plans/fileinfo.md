# PHP fileinfo

- [ ] Add a pinned static libmagic native package with its embedded magic database on every target.
- [ ] Add the optional fileinfo bridge and PHP finfo class, six functions and eleven constants.
- [ ] Expose the same declarations to dynamic eval and preserve ownership and failure behavior.
- [ ] Add focused native, codegen, error and target coverage plus a practical example.
- [x] Generate and audit builtin documentation in CI without local builds or heavy tests.
- [ ] Open a draft pull request, follow its exact-head CI and correct failures.

## Implementation

Use upstream file/libmagic 5.46, pinned by source size and SHA-256. Build a host database compiler when cross-compiling and embed the compiled database in a target C shim, so deployed programs need neither a system libmagic nor an external database. Custom text and compiled databases remain supported through magic_load.

A small Rust staticlib owns opaque handles. The PHP surface uses ordinary synthetic declarations and shared contracts, following the existing XML prelude and native-function eval bridge. All generated code uses the existing EIR and extern ABI paths.

PHP 8 semantics include no-op finfo_close, per-call flag restoration, binary-safe buffers, false results on lookup failures, constructor exceptions, stream input and contexts, and destruction without duplicate release. CI validates the supported target matrix. The user explicitly requested no local heavy builds or tests.

## Current validation

Draft PR: https://github.com/illegalstudio/elephc/pull/2061.
The remote compiler and exporter build successfully. CI found stale function-count
snapshots and a missing docs prelude-source mapping; those are being corrected.
Native behavior and the full target matrix remain under CI verification.
Native ownership uses a private final owner object so a finfo subclass destructor
cannot bypass release of its libmagic instance.

The exact-head CI exporter and target architecture audit passed. The docs pipeline's
16 tests, static docs audit (zero errors), site validation and PHP comparison tests
passed. Generated artifacts were downloaded and copied without local compilation.
The comparison now reports fileinfo as 6/6 functions, 1/1 classes and 11/11 constants.
