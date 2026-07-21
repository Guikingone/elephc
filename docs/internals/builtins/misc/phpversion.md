---
title: "phpversion() — internals"
description: "Compiler internals for phpversion(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 295
---

## `phpversion()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/phpversion.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/phpversion.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins.rs`:614](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins.rs#L614) (`lower_phpversion`)
- **Function symbol**: `lower_phpversion()`


### Lowering notes

- Lowers `phpversion()` as the compiler package version string.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_constant`
- `__rt_defined`

## Signature summary

```php
function phpversion(string $extension = null): string
```

## What the type checker enforces

- **Arity**: takes 0–1 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `phpversion()`](../../../php/builtins/misc/phpversion.md)
