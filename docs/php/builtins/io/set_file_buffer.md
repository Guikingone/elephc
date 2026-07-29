---
title: "set_file_buffer()"
description: "Sets file buffering on the given stream (alias of stream_set_write_buffer)."
sidebar:
  order: 209
---

## set_file_buffer()

```php
function set_file_buffer(mixed $stream, int $size): int
```

Sets file buffering on the given stream (alias of stream_set_write_buffer).

**Parameters**:
- `$stream` (`mixed`)
- `$size` (`int`)

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `set_file_buffer` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/set_file_buffer.md).
