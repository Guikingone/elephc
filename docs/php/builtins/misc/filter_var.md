---
title: "filter_var()"
description: "Filters a variable with a specified filter."
sidebar:
  order: 325
---

## filter_var()

```php
function filter_var(mixed $value, int $filter = 516, mixed $options = 0): mixed
```

Filters a variable with a specified filter.

**Parameters**:
- `$value` (`mixed`)
- `$filter` (`int`), default `516`, optional
- `$options` (`mixed`), default `0`, optional

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `filter_var` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/filter_var.md).
