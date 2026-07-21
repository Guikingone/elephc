---
title: "strcmp() — internals"
description: "Compiler internals for strcmp(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 408
---

## `strcmp()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/strcmp.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/strcmp.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:167](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L167) (`lower_binary_string_runtime`)
- **Function symbol**: `lower_binary_string_runtime()`


### Lowering notes

- Lowers a two-argument string builtin that directly delegates to a runtime helper.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function strcmp(string $string1, string $string2): int
```

## What the type checker enforces

- **Arity**: takes exactly 2 arguments.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/strcmp.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/strcmp.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `strcmp()`](../../../php/builtins/string/strcmp.md)
