---
title: "setlocale()"
description: "Sets locale information for the process."
sidebar:
  order: 324
---

## setlocale()

```php
function setlocale(int $category, string $locales, ...$rest): mixed
```

Sets locale information for the process.

**Parameters**:
- `$category` (`int`)
- `$locales` (`string`)
- `...$rest` — variadic: collects excess arguments into `$rest`.

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `setlocale` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/setlocale.md).
