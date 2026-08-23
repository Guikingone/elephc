---
title: "unpack()"
description: "Unpacks binary data according to a format string."
sidebar:
  order: 484
---

## unpack()

```php
function unpack(string $format, string $string, int $offset = 0): mixed
```

Unpacks binary data according to a format string.

**Parameters**:
- `$format` (`string`)
- `$string` (`string`)
- `$offset` (`int`), default `0`, optional

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `unpack` is implemented in the compiler, see [the internals page](../../../internals/builtins/string/unpack.md).
