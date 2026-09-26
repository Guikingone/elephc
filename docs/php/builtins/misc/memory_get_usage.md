---
title: "memory_get_usage()"
description: "Returns the bytes currently held by live elephc heap blocks."
sidebar:
  order: 626
---

## memory_get_usage()

```php
function memory_get_usage(bool $real_usage = false): int
```

Returns the bytes currently held by live elephc heap blocks.

**Parameters**:
- `$real_usage` (`bool`), default `false`, optional

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/memory_usage.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/memory_usage.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `memory_get_usage` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/memory_get_usage.md).
