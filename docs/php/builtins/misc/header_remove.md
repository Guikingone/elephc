---
title: "header_remove()"
description: "Removes one or all pending HTTP response headers."
sidebar:
  order: 315
---

## header_remove()

```php
function header_remove(string $name = null): void
```

Removes one or all pending HTTP response headers.

**Parameters**:
- `$name` (`string`), default `null`, optional

**Returns**: `void`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `header_remove` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/header_remove.md).
