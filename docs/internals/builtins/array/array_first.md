---
title: "array_first() - internals"
description: "Compiler internals for array_first(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 18
---

## `array_first()` - internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/array_edge_prelude.rs`:48](https://github.com/illegalstudio/elephc/blob/main/src/array_edge_prelude.rs#L48) (`array_first`)
- **Function symbol**: `array_first()`


### Lowering notes

- Implemented by the compiler-injected array-edge prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function array_first(array $array): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/array_first.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_first.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `dynamic-language-surface`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `array_first()`](../../../php/builtins/array/array_first.md)
