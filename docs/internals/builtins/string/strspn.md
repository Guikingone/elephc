---
title: "strspn() — internals"
description: "Compiler internals for strspn(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 383
---

## `strspn()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:156](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L156) (`lower_span`)
- **Function symbol**: `lower_span()`


### Lowering notes

- Lowers `strcspn()`/`strspn()` initial-segment-span builtins to a runtime helper.
- Both builtins scan `string` for the longest leading run of bytes that are
- (`strspn`) or are not (`strcspn`) members of `characters`, returning that
- run's length. The optional `offset`/`length` arguments are accepted by the
- type checker but are not yet supported in the EIR backend.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_strpbrk`

## Signature summary

```php
function strspn(mixed $string, mixed $characters, mixed $offset, mixed $length): int
```

## What the type checker enforces

- **Arity**: takes 2–4 arguments (2 optional).

## Cross-references

- [User reference for `strspn()`](../../../php/builtins/string/strspn.md)

