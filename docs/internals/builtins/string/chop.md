---
title: "chop() — internals"
description: "Compiler internals for chop(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 359
---

## `chop()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/chop.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/chop.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:132](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L132) (`lower_trim_like`)
- **Function symbol**: `lower_trim_like()`


### Lowering notes

- Lowers `trim()`/`ltrim()`/`rtrim()`/`chop()` for default and explicit masks.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_str_persist`

## Signature summary

```php
function chop(string $string, string $characters = ' \n\r\t\x0b\x0c\x00'): string
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/chop.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/chop.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `chop()`](../../../php/builtins/string/chop.md)
