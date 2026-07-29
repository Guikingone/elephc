---
title: "socket_set_timeout()"
description: "Set timeout period on a socket stream (alias of stream_set_timeout)."
sidebar:
  order: 214
---

## socket_set_timeout()

```php
function socket_set_timeout(mixed $stream, int $seconds, int $microseconds = 0): bool
```

Set timeout period on a socket stream (alias of stream_set_timeout).

**Parameters**:
- `$stream` (`mixed`)
- `$seconds` (`int`)
- `$microseconds` (`int`), default `0`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `socket_set_timeout` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/socket_set_timeout.md).
