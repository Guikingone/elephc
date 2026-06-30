---
title: "constant() — internals"
description: "Compiler internals for constant(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 260
---

## `constant()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:888](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L888) (`lower_constant`)
- **Function symbol**: `lower_constant()`


### Lowering notes

- Lowers `constant($name)` against the closed-world constant registry.
- Literal names that resolve at compile time are folded during EIR lowering and
- never reach here; every name that does reach this lowering (non-literal, or a
- literal that did not resolve) is sent to `__rt_constant`, which returns an
- owned boxed Mixed on a hit and throws a catchable `\Error` on a miss.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_constant`
- `__rt_defined`
- `__rt_enum_exists`
- `__rt_mixed_cast_string`

## Signature summary

```php
function constant(string $name): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `constant()`](../../../php/builtins/misc/constant.md)

