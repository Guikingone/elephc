---
title: "ksort()"
description: "Sorts an array by key in ascending SORT_REGULAR order; PHP sort flags are not yet supported."
sidebar:
  order: 59
---

## ksort()

```php
function ksort(array $array, int $flags = 0): bool
```

Sorts an array by key in ascending SORT_REGULAR order; PHP sort flags are not yet supported.

**Parameters**:
- `$array` (`array`), passed by reference
- `$flags` (`int`), default `0`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/ksort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/ksort.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `ksort` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/ksort.md).
