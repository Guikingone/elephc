---
title: "array_udiff()"
description: "Computes the difference of arrays using a callback comparator."
sidebar:
  order: 49
---

## array_udiff()

```php
function array_udiff(array $array1, array $array2, callable $callback): array
```

Computes the difference of arrays using a callback comparator.

**Parameters**:
- `$array1` (`array`)
- `$array2` (`array`)
- `$callback` (`callable`)

**Returns**: `array`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/array_ucompare.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_ucompare.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `array_udiff` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/array_udiff.md).
