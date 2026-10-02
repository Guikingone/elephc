---
title: "array_first()"
description: "Gets the first value of an array, or null when it is empty."
sidebar:
  order: 18
---

## array_first()

```php
function array_first(array $array): mixed
```

Gets the first value of an array, or null when it is empty.

**Parameters**:
- `$array` (`array`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected array_edge prelude.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/array/array_first.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/array/array_first.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `array_first` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/array_first.md).
