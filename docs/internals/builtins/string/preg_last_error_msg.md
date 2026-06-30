---
title: "preg_last_error_msg() — internals"
description: "Compiler internals for preg_last_error_msg(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 371
---

## `preg_last_error_msg()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:248](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L248) (`lower_preg_last_error_msg`)
- **Function symbol**: `lower_preg_last_error_msg()`


### Lowering notes

- Lowers `preg_last_error_msg()` — always returns the static string `"No error"`.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_explode`
- `__rt_preg_last_error`
- `__rt_preg_last_error_msg`

## Signature summary

```php
function preg_last_error_msg(): string
```

## What the type checker enforces

- **Arity**: takes no arguments.

## Cross-references

- [User reference for `preg_last_error_msg()`](../../../php/builtins/string/preg_last_error_msg.md)

