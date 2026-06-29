---
title: "strrpos() — internals"
description: "Compiler internals for strrpos(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 389
---

## `strrpos()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:794](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L794) (`lower_string_position`)
- **Function symbol**: `lower_string_position()`


### Lowering notes

- Lowers `strpos()`/`strrpos()` and boxes position-or-false results as Mixed.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function strrpos(string $haystack, string $needle, int $value): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 3 arguments.

## Cross-references

- [User reference for `strrpos()`](../../../php/builtins/string/strrpos.md)

