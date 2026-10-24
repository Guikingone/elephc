# PHP fileinfo

- [ ] Add a pinned static libmagic native package with its embedded magic database on every target.
- [ ] Add the optional fileinfo bridge and PHP finfo class, six functions and eleven constants.
- [ ] Expose the same declarations to dynamic eval and preserve ownership and failure behavior.
- [ ] Add focused native, codegen, error and target coverage plus a practical example.
- [ ] Generate and audit builtin documentation in CI without local builds or heavy tests.
- [ ] Open a draft pull request, follow its exact-head CI and correct failures.

## Implementation

Use upstream file/libmagic 5.46, pinned by source size and SHA-256. Build a host database compiler when cross-compiling and embed the compiled database in a target C shim, so deployed programs need neither a system libmagic nor an external database. Custom text and compiled databases remain supported through magic_load.

A small Rust staticlib owns opaque handles. The PHP surface uses ordinary synthetic declarations and shared contracts, following the existing XML prelude and native-function eval bridge. All generated code uses the existing EIR and extern ABI paths.

PHP 8 semantics include no-op finfo_close, per-call flag restoration, binary-safe buffers, false results on lookup failures, constructor exceptions, stream input and contexts, and destruction without duplicate release. CI validates the supported target matrix. The user explicitly requested no local heavy builds or tests.
