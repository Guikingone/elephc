---
title: "get_debug_type()"
description: "Returns a debug-oriented PHP type or class name."
sidebar:
  order: 509
---

## get_debug_type()

```php
function get_debug_type(mixed $value): string
```

Returns a debug-oriented PHP type or class name.

**Parameters**:
- `$value` (`mixed`)

**Returns**: `string`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `get_debug_type` is implemented in the compiler, see [the internals page](../../../internals/builtins/type/get_debug_type.md).
