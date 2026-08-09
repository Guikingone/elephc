---
title: "set_socket_blocking()"
description: "Set blocking mode on a socket stream (alias of stream_set_blocking)."
sidebar:
  order: 217
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
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/filesystem/set_socket_blocking.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/filesystem/set_socket_blocking.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `set_socket_blocking` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/set_socket_blocking.md).
