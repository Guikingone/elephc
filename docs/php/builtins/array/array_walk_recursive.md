---
title: "array_walk_recursive()"
description: "Applies a user function recursively to every member of an array."
sidebar:
  order: 47
---

## array_walk_recursive()

```php
function array_walk_recursive(array $array, callable $callback, mixed $arg = null): bool
```

Applies a user function recursively to every member of an array.

**Parameters**:
- `$array` (`array`), passed by reference
- `$callback` (`callable`)
- `$arg` (`mixed`), default `null`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/array_walk_recursive.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_walk_recursive.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `array_walk_recursive` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/array_walk_recursive.md).
