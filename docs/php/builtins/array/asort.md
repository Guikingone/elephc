---
title: "asort()"
description: "Sorts an array and maintains index association."
sidebar:
  order: 48
---

## asort()

```php
function asort(array $array, int $flags = 0): bool
```

Sorts an array and maintains index association.

**Parameters**:
- `$array` (`array`), passed by reference
- `$flags` (`int`), default `0`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/asort.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/asort.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `asort` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/asort.md).
