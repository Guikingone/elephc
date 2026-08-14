---
title: "extract()"
description: "Imports array entries as variables in the current scope."
sidebar:
  order: 55
---

## extract()

```php
function extract(array $array, int $flags = 0, string $prefix = ''): int
```

Imports array entries as variables in the current scope.

**Parameters**:
- `$array` (`array`)
- `$flags` (`int`), default `0`, optional
- `$prefix` (`string`), default `''`, optional

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `extract` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/extract.md).
