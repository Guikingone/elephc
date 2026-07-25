---
title: "sort()"
description: "Sorts an array in ascending order."
sidebar:
  order: 61
---

## sort()

```php
function sort(array $array, int $flags = 0): bool
```

Sorts an array in ascending order.

**Parameters**:
- `$array` (`array`), passed by reference
- `$flags` (`int`), default `0`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/sort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/sort.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `sort` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/sort.md).
