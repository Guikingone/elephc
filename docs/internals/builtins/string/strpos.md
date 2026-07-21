---
title: "strpos() — internals"
description: "Compiler internals for strpos(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 411
---

## `strpos()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/strpos.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/strpos.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:1151](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L1151) (`lower_string_position`)
- **Function symbol**: `lower_string_position()`


### Lowering notes

- Lowers `strpos()`/`strrpos()` and boxes position-or-false results as Mixed.
- `strrpos()` accepts an optional third `$offset` argument (2–3 args); `strpos()`
- is lowered through its own `lower_strpos` entry point. When three operands are
- present, the haystack start is adjusted by the offset and the runtime result is
- shifted back to an absolute position, mirroring `strpos`'s offset path.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function strpos(string $haystack, string $needle, int $offset = 0): mixed
```

## What the type checker enforces

- **Arity**: takes 2–3 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/strpos.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/strpos.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `strpos()`](../../../php/builtins/string/strpos.md)
