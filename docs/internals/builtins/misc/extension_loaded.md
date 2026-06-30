---
title: "extension_loaded() — internals"
description: "Compiler internals for extension_loaded(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 265
---

## `extension_loaded()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:995](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L995) (`lower_extension_loaded`)
- **Function symbol**: `lower_extension_loaded()`


### Lowering notes

- Lowers `extension_loaded("name")` to a compile-time constant boolean.
- In the closed-world AOT model the loaded-extension set is fixed at compile
- time, so the result is materialized as a static `0`/`1` rather than a
- runtime query.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_enum_exists`

## Signature summary

```php
function extension_loaded(mixed $extension): bool
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `extension_loaded()`](../../../php/builtins/misc/extension_loaded.md)

