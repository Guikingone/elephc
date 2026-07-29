---
title: "socket_get_status()"
description: "Retrieves header/meta data from streams/file pointers (alias of stream_get_meta_data)."
sidebar:
  order: 211
---

## socket_get_status()

```php
function socket_get_status(mixed $stream): mixed
```

Retrieves header/meta data from streams/file pointers (alias of stream_get_meta_data).

**Parameters**:
- `$stream` (`mixed`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `socket_get_status` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/socket_get_status.md).
