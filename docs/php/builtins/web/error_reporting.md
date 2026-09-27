---
title: "error_reporting()"
description: "Implemented by the compiler-injected error-handling prelude."
sidebar:
  order: 919
---

## error_reporting()

```php
function error_reporting(?int $error_level = null): int
```

Implemented by the compiler-injected error-handling prelude.

**Parameters**:
- `$error_level` (`?int`), default `null`, optional

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected error_handling prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `error_reporting` is implemented in the compiler, see [the internals page](../../../internals/builtins/web/error_reporting.md).
