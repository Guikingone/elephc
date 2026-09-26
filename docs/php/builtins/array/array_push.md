---
title: "array_push()"
description: "Pushes zero or more elements onto the end of array and returns the new count."
sidebar:
  order: 30
---

## array_push()

```php
function array_push(array $array, ...$values): int
```

Pushes zero or more elements onto the end of array and returns the new count.

**Parameters**:
- `$array` (`array`), passed by reference
- `...$values` — variadic: collects excess arguments into `$values`.

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/array_push.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_push.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `array_push` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/array_push.md).
