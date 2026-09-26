---
title: "get_error_handler()"
description: "Returns the current user-defined error handler."
sidebar:
  order: 918
---

## get_error_handler()

```php
function get_error_handler(): mixed
```

Returns the current user-defined error handler.

**Parameters**: none.

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `get_error_handler` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/get_error_handler.md).
