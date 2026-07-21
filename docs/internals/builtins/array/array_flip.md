---
title: "array_flip() — internals"
description: "Compiler internals for array_flip(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 13
---

## `array_flip()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/array_flip.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/array_flip.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:180](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L180) (`lower_array_flip`)
- **Function symbol**: `lower_array_flip()`


### Lowering notes

- Lowers `array_flip()` through the hash-building runtime helpers.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_hash_is_list`
- `__rt_mixed_array_is_list`

## Signature summary

```php
function array_flip(array $array): array
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/array_flip.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_flip.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `array_flip()`](../../../php/builtins/array/array_flip.md)
