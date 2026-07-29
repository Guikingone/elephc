---
title: "stream_register_wrapper()"
description: "Register a URL wrapper implemented as a PHP class (alias of stream_wrapper_register)."
sidebar:
  order: 235
---

## stream_register_wrapper()

```php
function stream_register_wrapper(string $protocol, string $class, int $flags = 0): bool
```

Register a URL wrapper implemented as a PHP class (alias of stream_wrapper_register).

**Parameters**:
- `$protocol` (`string`)
- `$class` (`string`)
- `$flags` (`int`), default `0`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `stream_register_wrapper` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/stream_register_wrapper.md).
