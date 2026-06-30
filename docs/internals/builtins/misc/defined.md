---
title: "defined() — internals"
description: "Compiler internals for defined(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 262
---

## `defined()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:869](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L869) (`lower_defined`)
- **Function symbol**: `lower_defined()`


### Lowering notes

- Lowers `defined($name)` against the closed-world constant registry.
- A constant string name folds to a static boolean (the cheap literal path
- normally resolved during EIR lowering never reaches the runtime helper); a
- non-literal name is lowered to the `__rt_defined` registry lookup.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_constant`
- `__rt_defined`

## Signature summary

```php
function defined(string $constant_name): bool
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `defined()`](../../../php/builtins/misc/defined.md)

