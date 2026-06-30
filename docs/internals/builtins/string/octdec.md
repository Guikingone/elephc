---
title: "octdec() — internals"
description: "Compiler internals for octdec(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 368
---

## `octdec()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:76](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L76) (`lower_unary_string_runtime`)
- **Function symbol**: `lower_unary_string_runtime()`


### Lowering notes

- Lowers a one-argument string builtin that directly delegates to a runtime helper.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_base64_decode`

## Signature summary

```php
function octdec(mixed $octal_string): int
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `octdec()`](../../../php/builtins/string/octdec.md)

