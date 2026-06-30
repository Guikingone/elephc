---
title: "base64_decode() — internals"
description: "Compiler internals for base64_decode(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 329
---

## `base64_decode()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:93](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L93) (`lower_base64_decode`)
- **Function symbol**: `lower_base64_decode()`


### Lowering notes

- Lowers `base64_decode(string $string, bool $strict = false)`.
- PHP accepts an optional second `$strict` argument. elephc decodes the same
- way regardless of `$strict`, so the second operand (already evaluated for its
- side effects during argument lowering) is ignored here; only the first string
- operand is materialized into the runtime call's argument registers.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_base64_decode`
- `__rt_grapheme_strrev`
- `__rt_strcopy`

## Signature summary

```php
function base64_decode(string $string, bool $strict): string
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Cross-references

- [User reference for `base64_decode()`](../../../php/builtins/string/base64_decode.md)

