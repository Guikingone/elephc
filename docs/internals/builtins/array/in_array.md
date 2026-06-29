---
title: "in_array() — internals"
description: "Compiler internals for in_array(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 35
---

## `in_array()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/arrays.rs`:1202](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/arrays.rs#L1202) (`lower_in_array`)
- **Function symbol**: `lower_in_array()`


### Lowering notes

- Lowers `in_array()` for indexed arrays with scalar or string payloads.
- Accepts the optional 3rd `strict` argument. When `strict` is statically true
- and the needle can never be `===` an element (disjoint scalar/string types),
- the result is unconditionally `false`. For the supported same-type
- scalar/string cases, strict (`===`) membership reduces to the existing exact
- comparison, so the strict flag does not change the lowering otherwise.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function in_array(mixed $needle, array $haystack, bool $strict): mixed
```

## What the type checker enforces

- **Arity**: takes 2–3 arguments (1 optional).

## Cross-references

- [User reference for `in_array()`](../../../php/builtins/array/in_array.md)

