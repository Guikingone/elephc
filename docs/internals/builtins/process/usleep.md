---
title: "usleep() — internals"
description: "Compiler internals for usleep(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 304
---

## `usleep()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/system.rs`:638](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/system.rs#L638) (`lower_usleep`)
- **Function symbol**: `lower_usleep()`


### Lowering notes

- Lowers `usleep(microseconds)` through the target's C library symbol.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_serialize_unsupported`

## Signature summary

```php
function usleep(int $microseconds): void
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `usleep()`](../../../php/builtins/process/usleep.md)

