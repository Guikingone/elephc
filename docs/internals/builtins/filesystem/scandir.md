---
title: "scandir() — internals"
description: "Compiler internals for scandir(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 147
---

## `scandir()` — internals

## Where it lives

- **Signature**: [`src/builtins/io/scandir.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/io/scandir.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/io.rs`:4533](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/io.rs#L4533) (`lower_scandir`)
- **Function symbol**: `lower_scandir()`


### Lowering notes

- Lowers `scandir(path)` through the target-aware runtime directory listing helper.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function scandir(string $directory, int $sorting_order = 0, mixed $context = null): array
```

## What the type checker enforces

- **Arity**: takes 1–3 arguments (2 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/filesystem/scandir.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/filesystem/scandir.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `scandir()`](../../../php/builtins/filesystem/scandir.md)
