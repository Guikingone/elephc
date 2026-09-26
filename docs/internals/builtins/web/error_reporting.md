---
title: "error_reporting() — internals"
description: "Compiler internals for error_reporting(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 917
---

## `error_reporting()` — internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/error_handling_prelude.rs`:171](https://github.com/illegalstudio/elephc/blob/main/src/error_handling_prelude.rs#L171) (`error_reporting`)
- **Function symbol**: `error_reporting()`


### Lowering notes

- Implemented by the compiler-injected error-handling prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function error_reporting(?int $error_level = null): int
```

## What the type checker enforces

- **Arity**: takes 0–1 arguments (1 optional).

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `error_reporting()`](../../../php/builtins/web/error_reporting.md)
