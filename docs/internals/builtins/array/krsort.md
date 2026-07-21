---
title: "krsort() — internals"
description: "Compiler internals for krsort(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 53
---

## `krsort()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/krsort.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/krsort.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1353](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1353) (`lower_krsort`)
- **Function symbol**: `lower_krsort()`


### Lowering notes

- Lowers `krsort()` through the reverse key-sort helper surface.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_krsort`
- `__rt_natcasesort`
- `__rt_natsort`

## Signature summary

```php
function krsort(array $array, int $flags = 0): bool
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).
- **By-reference parameters**: `$array`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/krsort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/krsort.rs) (`eval_builtin!`)
- **Dispatch hooks**: `values`
- **By-reference parameters**: `$array`.

## Cross-references

- [User reference for `krsort()`](../../../php/builtins/array/krsort.md)
