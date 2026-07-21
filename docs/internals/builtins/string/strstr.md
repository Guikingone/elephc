---
title: "strstr() — internals"
description: "Compiler internals for strstr(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 414
---

## `strstr()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/strstr.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/strstr.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:1232](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L1232) (`lower_strstr`)
- **Function symbol**: `lower_strstr()`


### Lowering notes

- Lowers `strstr(haystack, needle[, before_needle])`.
- With `before_needle = false` (default) returns the matching suffix starting at the first
- occurrence of the needle.  With `before_needle = true` returns the prefix that precedes
- the first occurrence.  The `before_needle` argument must be a compile-time constant.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function strstr(string $haystack, string $needle, bool $before_needle = false): string
```

## What the type checker enforces

- **Arity**: takes 2–3 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/strstr.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/strstr.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `strstr()`](../../../php/builtins/string/strstr.md)
