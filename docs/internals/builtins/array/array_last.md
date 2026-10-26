---
title: "array_last() - internals"
description: "Compiler internals for array_last(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 30
---

## `array_last()` - internals

## Where it lives

- **Signature**: [`crates/elephc-builtin-contract/src/catalog_data.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-builtin-contract/src/catalog_data.rs)
- **Lowering**: [`src/array_edge_prelude.rs`:61](https://github.com/illegalstudio/elephc/blob/main/src/array_edge_prelude.rs#L61) (`array_last`)
- **Function symbol**: `array_last()`


### Lowering notes

- Implemented by the compiler-injected array-edge prelude.

## Semantic descriptor

Shared contract implemented by an injected elephc-PHP prelude.

## EIR and runtime boundary

_Implemented by an injected elephc-PHP prelude._

## Signature summary

```php
function array_last(array $array): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/array/array_first.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_first.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `dynamic-language-surface`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `array_last()`](../../../php/builtins/array/array_last.md)
