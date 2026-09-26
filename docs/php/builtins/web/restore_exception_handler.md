---
title: "restore_exception_handler()"
description: "Restores the previous user-defined exception handler."
sidebar:
  order: 924
---

## restore_exception_handler()

```php
function restore_exception_handler(): bool
```

Restores the previous user-defined exception handler.

**Parameters**: none.

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `restore_exception_handler` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/restore_exception_handler.md).
