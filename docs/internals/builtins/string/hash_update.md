---
title: "hash_update() — internals"
description: "Compiler internals for hash_update(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 397
---

## `hash_update()` — internals

## Where it lives

- **Signature**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs)
- **Lowering**: [`(not lowered)`:0]()
- **Function symbol**: `(none — type-checker only)()`


## Semantic descriptor

_Compiler-resident construct; this name is intentionally outside the builtin registry._

## EIR and runtime boundary

_Compiler-resident lowering; no registry-backed typed runtime target applies._

## Signature summary

```php
function hash_update(mixed $context, mixed $data): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 2 arguments.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `hash_update()`](../../../php/builtins/string/hash_update.md)
