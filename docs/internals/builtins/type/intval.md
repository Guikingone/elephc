---
title: "intval() — internals"
description: "Compiler internals for intval(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 436
---

## `intval()` — internals

## Where it lives

- **Signature**: [`src/builtins/types/intval.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/types/intval.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins.rs`:1396](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins.rs#L1396) (`lower_intval`)
- **Function symbol**: `lower_intval()`


### Lowering notes

- Lowers `intval()` for concrete scalar operands.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_mixed_cast_int`
- `__rt_str_to_int`

## Signature summary

```php
function intval(mixed $value, int $base = 10): int
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/types/intval.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/types/intval.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `intval()`](../../../php/builtins/type/intval.md)
