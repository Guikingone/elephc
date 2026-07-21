---
title: "sort() — internals"
description: "Compiler internals for sort(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 60
---

## `sort()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/sort.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/sort.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1328](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1328) (`lower_sort`)
- **Function symbol**: `lower_sort()`


### Lowering notes

- Lowers `sort()` for indexed integer arrays by mutating the source array in place.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_arsort`
- `__rt_asort`
- `__rt_krsort`
- `__rt_ksort`
- `__rt_rsort_int`
- `__rt_rsort_str`
- `__rt_sort_int`
- `__rt_sort_str`

## Signature summary

```php
function sort(array $array, int $flags = 0): bool
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).
- **By-reference parameters**: `$array`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/sort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/sort.rs) (`eval_builtin!`)
- **Dispatch hooks**: `values`
- **By-reference parameters**: `$array`.

## Cross-references

- [User reference for `sort()`](../../../php/builtins/array/sort.md)
