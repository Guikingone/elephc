---
title: "extension_loaded()"
description: "Lowers `extension_loaded(\"name\")` to a compile-time constant boolean."
sidebar:
  order: 262
---

## extension_loaded()

```php
function extension_loaded(mixed $extension): bool
```

Lowers `extension_loaded("name")` to a compile-time constant boolean.

**Parameters**:
- `$extension` (`mixed`)

**Returns**: `bool`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `extension_loaded` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/extension_loaded.md).

