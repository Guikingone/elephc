---
title: "setlocale() — internals"
description: "Compiler internals for setlocale(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 273
---

## `setlocale()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:1208](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L1208) (`lower_setlocale`)
- **Function symbol**: `lower_setlocale()`


### Lowering notes

- Lowers `setlocale($category, $locale, ...)` as a minimal sound stub.
- elephc has no locale machinery, so the call changes nothing and returns the
- requested locale string (boxed as a `string|false` Mixed cell, matching PHP's
- return type). The locale comes from the second argument: a string is returned
- verbatim, a boxed value is coerced to a string, and anything else falls back to
- the `"C"` locale.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_mixed_cast_string`

## Signature summary

```php
function setlocale(int $category, string $locales, ...$rest): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 2 arguments.
- **Variadic**: collects excess arguments into `$rest`.

## Cross-references

- [User reference for `setlocale()`](../../../php/builtins/misc/setlocale.md)

