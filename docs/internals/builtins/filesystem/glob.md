---
title: "glob() — internals"
description: "Compiler internals for glob(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 124
---

## `glob()` — internals

## Where it lives

- **Signature**: [`src/builtins/io/glob.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/io/glob.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/io.rs`:4549](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/io.rs#L4549) (`lower_glob`)
- **Function symbol**: `lower_glob()`


### Lowering notes

- Lowers `glob($pattern, $flags = 0)` through the target-aware runtime glob
- expansion helper. `$flags` must be a compile-time integer literal (after EIR
- constant folding — `GLOB_NOSORT`, `GLOB_MARK`, `GLOB_BRACE`, `GLOB_ONLYDIR`,
- and OR-combinations of them all fold to `Op::ConstI64`): only literal flags
- can be validated against the supported bit set, so a non-literal `$flags`
- stays loud instead of silently passing an unvalidated runtime value to libc.
- `GLOB_ONLYDIR` is split out and never reaches libc `glob()` — see the
- runtime helper's module doc for why.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function glob(string $pattern, int $flags = 0): array
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/filesystem/glob.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/filesystem/glob.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `glob()`](../../../php/builtins/filesystem/glob.md)
