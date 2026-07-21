---
title: "array_is_list() — internals"
description: "Compiler internals for array_is_list(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 17
---

## `array_is_list()` — internals

## Where it lives

- **Signature**: [`src/builtins/array/array_is_list.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/array/array_is_list.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/arrays.rs`:205](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/arrays.rs#L205) (`lower_array_is_list`)
- **Function symbol**: `lower_array_is_list()`


### Lowering notes

- Lowers `array_is_list(array $array): bool`.
- A bare `PhpType::Array(_)` static type does NOT prove the runtime payload is a packed
- indexed array: gradual typing lets a runtime-built associative hash occupy that slot
- (e.g. a plain `array $a` parameter, or any array value narrowed back down from
- `Mixed`), so folding straight to the compile-time constant `true` there is unsound —
- see `lower_array_is_list_dynamic_kind`, which probes the actual heap kind instead. A
- statically known associative hash (`PhpType::AssocArray`) dispatches directly to
- `__rt_hash_is_list`, which walks the insertion-order keys and returns `1` only when
- they are exactly `0, 1, .., count-1`. A boxed `Mixed`/union operand goes through
- `__rt_mixed_array_is_list`, which unboxes the runtime tag before scanning the same way.
- The operand and boolean result share the single-arg int-result register (`x0` / `rax`).

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_hash_is_list`
- `__rt_heap_kind`
- `__rt_mixed_array_is_list`

## Signature summary

```php
function array_is_list(mixed $array): bool
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `array_is_list()`](../../../php/builtins/array/array_is_list.md)
