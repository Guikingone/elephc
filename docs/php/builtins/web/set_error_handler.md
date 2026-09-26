---
title: "set_error_handler()"
description: "Installs a user-defined error handler and returns the previous one."
sidebar:
  order: 948
---

## set_error_handler()

```php
function set_error_handler(mixed $callback, int $error_levels = E_ALL): mixed
```

Installs a user-defined error handler and returns the previous one.

**Parameters**:
- `$callback` (`mixed`)
- `$error_levels` (`int`), default `E_ALL`, optional

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `set_error_handler` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/set_error_handler.md).
