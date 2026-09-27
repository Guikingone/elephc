---
title: "restore_error_handler()"
description: "Restores the previous user-defined error handler."
sidebar:
  order: 925
---

## restore_error_handler()

```php
function restore_error_handler(): bool
```

Restores the previous user-defined error handler.

**Parameters**: none.

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `restore_error_handler` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/restore_error_handler.md).
