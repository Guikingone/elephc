---
title: "mkdir() — internals"
description: "Compiler internals for mkdir(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 137
---

## `mkdir()` — internals

## Where it lives

- **Signature**: [`src/builtins/io/mkdir.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/io/mkdir.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/io.rs`:4435](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/io.rs#L4435) (`lower_mkdir`)
- **Function symbol**: `lower_mkdir()`


### Lowering notes

- Lowers `mkdir($directory, $permissions = 0777, $recursive = false, $context = null)`.
- The 1-arg form keeps the existing wrapper-aware dispatch (`__rt_mkdir`,
- which now passes the real PHP default mode 0777 instead of a hardcoded
- 0755 — see `crate::codegen_support::runtime::io::fs`). Once `$permissions` is
- explicitly passed, this bypasses stream-wrapper dispatch and calls the
- mode-aware native helpers directly (`__rt_mkdir_mode` / `__rt_mkdir_recursive`
- per the runtime-evaluated `$recursive` flag) — a scoped, documented
- residual: `mkdir($wrapperUrl, $mode, ...)` against a registered userspace
- stream wrapper is not implemented (native filesystem paths only for the
- mode/recursive-aware form). `$context` (checker-validated to be a
- compile-time `null`) is never materialized as an operand here.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_mkdir`
- `__rt_mkdir_mode`
- `__rt_mkdir_recursive`

## Signature summary

```php
function mkdir(string $directory, int $permissions = 511, bool $recursive = false, bool $context = null): bool
```

## What the type checker enforces

- **Arity**: takes 1–4 arguments (3 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/filesystem/mkdir.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/filesystem/mkdir.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `mkdir()`](../../../php/builtins/filesystem/mkdir.md)
