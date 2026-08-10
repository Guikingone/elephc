---
title: "hash_init() — internals"
description: "Compiler internals for hash_init(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 396
---

## `hash_init()` — internals

## Where it lives

- **Signature**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs)
- **Lowering**: [`(not lowered)`:0]()
- **Function symbol**: `(none — type-checker only)()`


## Semantic descriptor

_Compiler-resident construct; this name is intentionally outside the builtin registry._

## EIR and runtime boundary

_Compiler-resident lowering; no registry-backed typed runtime target applies._

## Signature summary

```php
function hash_init(mixed $algo, mixed $flags = '0', mixed $key = '""'): mixed
```

## What the type checker enforces

- **Arity**: takes 1–3 arguments (2 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `dynamic-language-surface`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `hash_init()`](../../../php/builtins/string/hash_init.md)
