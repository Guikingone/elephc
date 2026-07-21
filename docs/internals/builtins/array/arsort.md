---
title: "arsort() — internals"
description: "Compiler internals for arsort(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 47
---

## `arsort()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/arsort.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/arsort.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1343](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1343) (`lower_arsort`)
- **Function symbol**: `lower_arsort()`


### Lowering notes

- Lowers `arsort()` for indexed integer arrays through the descending value-sort wrapper.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_arsort`
- `__rt_krsort`
- `__rt_ksort`
- `__rt_natcasesort`
- `__rt_natsort`

## Signature summary

```php
function arsort(array $array, int $flags = 0): bool
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).
- **By-reference parameters**: `$array`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/arsort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/arsort.rs) (`eval_builtin!`)
- **Dispatch hooks**: `values`
- **By-reference parameters**: `$array`.

## Cross-references

- [User reference for `arsort()`](../../../php/builtins/array/arsort.md)
