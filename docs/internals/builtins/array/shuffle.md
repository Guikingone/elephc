---
title: "shuffle() — internals"
description: "Compiler internals for shuffle(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 59
---

## `shuffle()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/shuffle.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/shuffle.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1368](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1368) (`lower_shuffle`)
- **Function symbol**: `lower_shuffle()`


### Lowering notes

- Lowers `shuffle()` for indexed arrays with 8-byte slots by mutating the source array in place.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function shuffle(array $array): bool
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **By-reference parameters**: `$array`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/shuffle.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/shuffle.rs) (`eval_builtin!`)
- **Dispatch hooks**: `values`
- **By-reference parameters**: `$array`.

## Cross-references

- [User reference for `shuffle()`](../../../php/builtins/array/shuffle.md)
