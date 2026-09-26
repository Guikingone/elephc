---
title: "set_error_handler() — internals"
description: "Compiler internals for set_error_handler(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 948
---

## `set_error_handler()` — internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/error_handling_prelude.rs`:294](https://github.com/illegalstudio/elephc/blob/main/src/error_handling_prelude.rs#L294) (`set_error_handler`)
- **Function symbol**: `set_error_handler()`


### Lowering notes

- Implemented by the compiler-injected error-handling prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function set_error_handler(mixed $callback, int $error_levels = E_ALL): mixed
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `set_error_handler()`](../../../php/builtins/web/set_error_handler.md)
