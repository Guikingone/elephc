---
title: "set_exception_handler() — internals"
description: "Compiler internals for set_exception_handler(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 949
---

## `set_exception_handler()` — internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/error_handling_prelude.rs`:401](https://github.com/illegalstudio/elephc/blob/main/src/error_handling_prelude.rs#L401) (`set_exception_handler`)
- **Function symbol**: `set_exception_handler()`


### Lowering notes

- Implemented by the compiler-injected error-handling prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function set_exception_handler(mixed $callback): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `set_exception_handler()`](../../../php/builtins/web/set_exception_handler.md)
