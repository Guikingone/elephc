---
title: "memory_get_usage()"
description: "Returns current Elephc heap memory usage in bytes."
sidebar:
  order: 298
---

## memory_get_usage()

```php
function memory_get_usage(bool $real_usage = false): int
```

Returns current Elephc heap memory usage in bytes.

**Parameters**:
- `$real_usage` (`bool`), default `false`, optional

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `memory_get_usage` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/memory_get_usage.md).
