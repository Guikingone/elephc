---
title: "array_search() — internals"
description: "Compiler internals for array_search(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 35
---

## `array_search()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/array_search.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/array_search.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:1998](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L1998) (`lower_array_search`)
- **Function symbol**: `lower_array_search()`


### Lowering notes

- Lowers `array_search()` for indexed arrays with integer-like payloads.
- Lowers `array_search(needle, haystack[, strict])` for indexed arrays.
- When `strict` is a compile-time `true` constant and the needle type differs from the
- element type, returns `false` immediately without searching (PHP strict-comparison
- semantics: a string needle never equals an integer element).  For `strict=false` (the
- default) or same-type strict comparisons the existing loose-comparison path is used.
- A runtime-dynamic `strict` argument emits an `unsupported` error.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function array_search(mixed $needle, array $haystack, bool $strict = false): mixed
```

## What the type checker enforces

- **Arity**: takes 2–3 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/array_search.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_search.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `array_search()`](../../../php/builtins/array/array_search.md)
