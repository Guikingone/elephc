---
title: "preg_last_error() — internals"
description: "Compiler internals for preg_last_error(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 364
---

## `preg_last_error()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:235](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L235) (`lower_preg_last_error`)
- **Function symbol**: `lower_preg_last_error()`


### Lowering notes

- Lowers `preg_last_error()` — always returns 0 (PREG_NO_ERROR) in this implementation.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_explode`
- `__rt_preg_last_error`
- `__rt_sscanf`

## Signature summary

```php
function preg_last_error(): int
```

## What the type checker enforces

- **Arity**: takes no arguments.

## Cross-references

- [User reference for `preg_last_error()`](../../../php/builtins/string/preg_last_error.md)

