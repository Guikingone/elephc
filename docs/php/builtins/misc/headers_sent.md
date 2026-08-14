---
title: "headers_sent()"
description: "Reports whether output has already committed response headers."
sidebar:
  order: 316
---

## headers_sent()

```php
function headers_sent(mixed $filename = null, mixed $line = null): bool
```

Reports whether output has already committed response headers.

**Parameters**:
- `$filename` (`mixed`), passed by reference, default `null`, optional
- `$line` (`mixed`), passed by reference, default `null`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `headers_sent` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/headers_sent.md).
