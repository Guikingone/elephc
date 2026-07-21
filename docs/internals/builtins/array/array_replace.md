---
title: "array_replace() — internals"
description: "Compiler internals for array_replace(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 32
---

## `array_replace()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/array_replace.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/array_replace.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:284](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L284) (`lower_array_replace`)
- **Function symbol**: `lower_array_replace()`


### Lowering notes

- Lowers `array_replace(array $array, array ...$replacements): array` for associative
- hashes.
- The result starts as an owned shallow clone of the first argument, then each later
- argument is overlaid onto it via `__rt_hash_replace_into` (last-wins by key, preserving
- insertion order). Because the clone is uniquely owned, the overlays mutate it in place
- with no intermediate allocations to leak. Only associative-hash operands are supported;
- packed/indexed or boxed-Mixed operands fall through to a loud unsupported error rather
- than risk a representation mismatch.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_hash_replace_into`
- `__rt_hash_union`

## Signature summary

```php
function array_replace(array $array, ...$replacements): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **Variadic**: collects excess arguments into `$replacements`.

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `array_replace()`](../../../php/builtins/array/array_replace.md)
