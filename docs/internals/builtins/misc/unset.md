---
title: "unset() — internals"
description: "Compiler internals for unset(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 319
---

## `unset()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/types.rs`:139](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/types.rs#L139) (`lower_unset_builtin`)
- **Function symbol**: `lower_unset_builtin()`


### Lowering notes

- Rejects `unset()` calls that were not converted into direct EIR unbind operations.
- Reaching this lowering means `crate::ir_lower::expr` could not turn the target
- into a slot clear, a hash/array removal, an `offsetUnset()` call, a `__unset()`
- call or a dynamic-property removal, so the message lists the shapes that do lower
- directly and then names the one shape users hit most.
- THE UNTYPED FIXED SLOT is that shape. `unset($obj->untypedProp)` on a property
- declared without a type (`public $foo = 1;`) truly REMOVES it in PHP: a later read
- warns `Undefined property` and answers `null`, and a later write recreates it.
- elephc gives each declared property a fixed, monomorphically typed slot, so a
- property the checker typed `Int` has no encoding for "removed and reading as null"
- — every candidate encoding answers `int(0)` or a raw marker word instead. A loud
- error beats a wrong value, so the shape is refused here. Untyped properties whose
- storage is a DYNAMIC hash (`stdClass`, undeclared names on
- `#[AllowDynamicProperties]` classes) are genuinely removable and lower fine.

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
