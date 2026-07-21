---
title: "str_ireplace() — internals"
description: "Compiler internals for str_ireplace(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 401
---

## `str_ireplace()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/str_ireplace.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/str_ireplace.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:1292](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L1292) (`lower_string_replace`)
- **Function symbol**: `lower_string_replace()`


### Lowering notes

- Lowers `str_replace()`/`str_ireplace()` with three operands.
- Handles the common all-string form directly, and the PHP array-`$search` form (with an array or
- single-string `$replace`) against a string `$subject` through the `__rt_*_array` runtime helpers.
- Array operands must currently be indexed `Array(Str)` (string slots); other array shapes return a
- clear unsupported error rather than miscompiling.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function str_ireplace(string $search, string $replace, string $subject, int $count = null): string
```

## What the type checker enforces

- **Arity**: takes 3–4 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/str_ireplace.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/str_ireplace.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `str_ireplace()`](../../../php/builtins/string/str_ireplace.md)
