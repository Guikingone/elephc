---
title: "preg_grep()"
description: "Returns entries whose values match a regular expression while preserving keys."
sidebar:
  order: 334
---

## preg_grep()

```php
function preg_grep(string $pattern, mixed $array, int $flags = 0): array
```

Returns entries whose values match a regular expression while preserving keys.

**Parameters**:
- `$pattern` (`string`)
- `$array` (`mixed`)
- `$flags` (`int`), default `0`, optional

**Returns**: `array`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `preg_grep` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/preg_grep.md).
