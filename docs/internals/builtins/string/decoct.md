---
title: "decoct() — internals"
description: "Compiler internals for decoct(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 336
---

## `decoct()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:234](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L234) (`lower_int_to_base_string`)
- **Function symbol**: `lower_int_to_base_string()`


### Lowering notes

- Lowers an integer-valued builtin that delegates directly to a named runtime helper.
- Loads the first operand into the integer result register (the int ABI input for the helper),
- calls the runtime, and stores the string-typed result (ptr/len in string result registers).

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_preg_last_error`
- `__rt_preg_last_error_msg`

## Signature summary

```php
function decoct(mixed $value): string
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `decoct()`](../../../php/builtins/string/decoct.md)

