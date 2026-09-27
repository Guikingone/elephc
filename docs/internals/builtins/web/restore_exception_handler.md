---
title: "restore_exception_handler() — internals"
description: "Compiler internals for restore_exception_handler(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 926
---

## `restore_exception_handler()` — internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/error_handling_prelude.rs`:412](https://github.com/illegalstudio/elephc/blob/main/src/error_handling_prelude.rs#L412) (`restore_exception_handler`)
- **Function symbol**: `restore_exception_handler()`


### Lowering notes

- Implemented by the compiler-injected error-handling prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function restore_exception_handler(): bool
```

## What the type checker enforces

- **Arity**: takes no arguments.

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `restore_exception_handler()`](../../../php/builtins/web/restore_exception_handler.md)
