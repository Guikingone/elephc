---
title: "unset() — internals"
description: "Compiler internals for unset(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 304
---

## `unset()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/types.rs`:52](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/types.rs#L52) (`lower_unset_builtin`)
- **Function symbol**: `lower_unset_builtin()`


### Lowering notes

- Rejects `unset()` calls that were not converted into direct EIR unbind operations.
- Reaching this lowering means `crate::ir_lower::expr` could not turn the target
- into a slot clear, a hash/array removal, an `offsetUnset()` call or a `__unset()`
- call, so the message lists the shapes that do lower directly.

## Semantic descriptor

_Compiler-resident construct; this name is intentionally outside the builtin registry._

## EIR and runtime boundary

_Compiler-resident lowering; no registry-backed typed runtime target applies._

## Signature summary

```php
function unset(mixed $var, ...$vars): void
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **Variadic**: collects excess arguments into `$vars`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/symbols/unset.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/symbols/unset.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`
- **Variadic**: collects excess arguments into `$vars`.

## Cross-references

- [User reference for `unset()`](../../../php/builtins/misc/unset.md)
