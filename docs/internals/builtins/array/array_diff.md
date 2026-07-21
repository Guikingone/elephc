---
title: "array_diff() — internals"
description: "Compiler internals for array_diff(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 6
---

## `array_diff()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/array_diff.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/array_diff.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1119](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1119) (`lower_array_diff`)
- **Function symbol**: `lower_array_diff()`


### Lowering notes

- Lowers `array_diff()` for two compatible indexed arrays with pointer-sized payload slots.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_array_diff`
- `__rt_array_diff_key`
- `__rt_array_diff_refcounted`
- `__rt_array_intersect`
- `__rt_array_intersect_refcounted`

## Signature summary

```php
function array_diff(array $array, ...$arrays): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **Variadic**: collects excess arguments into `$arrays`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/array_diff.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_diff.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`
- **Variadic**: collects excess arguments into `$arrays`.

## Cross-references

- [User reference for `array_diff()`](../../../php/builtins/array/array_diff.md)
