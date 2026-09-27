---
title: "trigger_error()"
description: "Implemented by the compiler-injected error-handling prelude."
sidebar:
  order: 954
---

## trigger_error()

```php
function trigger_error(string $message, int $error_level = E_USER_NOTICE): bool
```

Implemented by the compiler-injected error-handling prelude.

**Parameters**:
- `$message` (`string`)
- `$error_level` (`int`), default `E_USER_NOTICE`, optional

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `trigger_error` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/trigger_error.md).
