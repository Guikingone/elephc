---
title: "hash_final() — internals"
description: "Compiler internals for hash_final(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 394
---

## `hash_final()` — internals

## Where it lives

- **Signature**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs)
- **Lowering**: [`(not lowered)`:0]()
- **Function symbol**: `(none — type-checker only)()`


## Semantic descriptor

_Compiler-resident construct; this name is intentionally outside the builtin registry._

## EIR and runtime boundary

_Compiler-resident lowering; no registry-backed typed runtime target applies._

## Signature summary

```php
function hash_final(mixed $context, mixed $binary = 'false'): mixed
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `dynamic-language-surface`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `hash_final()`](../../../php/builtins/string/hash_final.md)
