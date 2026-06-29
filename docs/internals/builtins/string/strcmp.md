---
title: "strcmp() — internals"
description: "Compiler internals for strcmp(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 382
---

## `strcmp()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:139](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L139) (`lower_binary_string_runtime`)
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

## Cross-references

- [User reference for `strcmp()`](../../../php/builtins/string/strcmp.md)

