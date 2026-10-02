---
title: "get_class_vars() — internals"
description: "Compiler internals for get_class_vars(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 99
---

## `get_class_vars()` — internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_surfaces.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_surfaces.rs)
- **Lowering**: [`src/class_vars_prelude.rs`:241](https://github.com/illegalstudio/elephc/blob/main/src/class_vars_prelude.rs#L241) (`get_class_vars`)
- **Function symbol**: `get_class_vars()`


### Lowering notes

- Implemented by the compiler-injected class-vars prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function get_class_vars(mixed $class): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/symbols/get_class_vars.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/symbols/get_class_vars.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `callable-or-reflection`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `get_class_vars()`](../../../php/builtins/class/get_class_vars.md)
