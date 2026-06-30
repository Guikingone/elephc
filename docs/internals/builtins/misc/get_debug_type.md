---
title: "get_debug_type() — internals"
description: "Compiler internals for get_debug_type(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 266
---

## `get_debug_type()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins.rs`:753](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins.rs#L753) (`lower_get_debug_type`)
- **Function symbol**: `lower_get_debug_type()`


### Lowering notes

- Lowers `get_debug_type(value)` — PHP 8's type-name helper. Like `gettype()` but with the short
- scalar spellings (`int`/`float`/`string`/`bool`/`null`) and an object's class name in place of
- the literal "object".

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function get_debug_type(mixed $value): string
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `get_debug_type()`](../../../php/builtins/misc/get_debug_type.md)

