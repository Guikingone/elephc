---
title: "implode() — internals"
description: "Compiler internals for implode(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 380
---

## `implode()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/implode.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/implode.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:462](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L462) (`lower_implode`)
- **Function symbol**: `lower_implode()`


### Lowering notes

- Lowers `implode(glue, array)` by selecting the string or integer array helper.
- A union-boxed `array` argument (the `$hosts = $x ?: false`-style gradual-typing idiom — the
- checker's `implode` branch never restricts arg 2's type at all, so ANY value can reach here)
- arrives with codegen-erased `PhpType::Mixed`/`Union` and is routed to `lower_implode_dynamic`
- instead of the STATIC-array path below, which assumes a compile-time-known element layout.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_implode`
- `__rt_mixed_from_array_kind`

## Signature summary

```php
function implode(string $separator, array $array = null): string
```

## What the type checker enforces

- **Arity**: takes 1–2 arguments (1 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/implode.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/implode.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `implode()`](../../../php/builtins/string/implode.md)
