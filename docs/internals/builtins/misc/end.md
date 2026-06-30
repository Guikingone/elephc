---
title: "end() — internals"
description: "Compiler internals for end(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 264
---

## `end()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:1193](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L1193) (`lower_end`)
- **Function symbol**: `lower_end()`


### Lowering notes

- Lowers `end($array)` by reading the last element of a boxed Mixed array.
- The IR lowering boxes the array argument into a Mixed cell (or passes an
- already-boxed `Mixed`/union value through), so the single operand is always a
- Mixed receiver here. `__rt_end_boxed` dispatches on the runtime array kind and
- returns the last element — or boxed `false` for an empty/non-array receiver — as
- an owned Mixed cell, matching PHP's `end()` (the internal array pointer is not
- modeled, only the value read).

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_end_boxed`
- `__rt_mixed_cast_string`

## Signature summary

```php
function end(array $array): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **By-reference parameters**: `$array`.

## Cross-references

- [User reference for `end()`](../../../php/builtins/misc/end.md)

