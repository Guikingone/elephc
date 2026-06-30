---
title: "substr_count() — internals"
description: "Compiler internals for substr_count(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 401
---

## `substr_count()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:944](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L944) (`lower_substr_count`)
- **Function symbol**: `lower_substr_count()`


### Lowering notes

- Lowers `substr_count(haystack, needle[, offset[, length]])`.
- The 2-arg form counts all non-overlapping occurrences via `__rt_substr_count`.
- The 3- and 4-arg forms (offset/length) are not yet supported in AOT mode.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_substr_count`

## Signature summary

```php
function substr_count(string $haystack, string $needle, mixed $offset, mixed $length): int
```

## What the type checker enforces

- **Arity**: takes 2–4 arguments (2 optional).

## Cross-references

- [User reference for `substr_count()`](../../../php/builtins/string/substr_count.md)

