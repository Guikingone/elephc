---
title: "set_socket_blocking()"
description: "Set blocking mode on a socket stream (alias of stream_set_blocking)."
sidebar:
  order: 210
---

## set_socket_blocking()

```php
function set_socket_blocking(mixed $stream, bool $enable): bool
```

Set blocking mode on a socket stream (alias of stream_set_blocking).

**Parameters**:
- `$stream` (`mixed`)
- `$enable` (`bool`)

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `set_socket_blocking` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/set_socket_blocking.md).
