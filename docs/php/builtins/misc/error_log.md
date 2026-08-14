---
title: "error_log()"
description: "Writes a message to the configured error log destination."
sidebar:
  order: 310
---

## error_log()

```php
function error_log(string $message, int $message_type = 0, string $destination = '', string $additional_headers = ''): bool
```

Writes a message to the configured error log destination.

**Parameters**:
- `$message` (`string`)
- `$message_type` (`int`), default `0`, optional
- `$destination` (`string`), default `''`, optional
- `$additional_headers` (`string`), default `''`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `error_log` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/error_log.md).
