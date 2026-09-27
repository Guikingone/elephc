---
title: "set_exception_handler()"
description: "Installs a user-defined exception handler and returns the previous one."
sidebar:
  order: 951
---

## set_exception_handler()

```php
function set_exception_handler(mixed $callback): mixed
```

Installs a user-defined exception handler and returns the previous one.

**Parameters**:
- `$callback` (`mixed`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `set_exception_handler` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/set_exception_handler.md).
