---
title: "bindec()"
description: "Lowers a one-argument string builtin that directly delegates to a runtime helper."
sidebar:
  order: 329
---

## bindec()

```php
function bindec(mixed $binary_string): int
```

Lowers a one-argument string builtin that directly delegates to a runtime helper.

**Parameters**:
- `$binary_string` (`mixed`)

**Returns**: `int`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `bindec` is implemented in the compiler, see [the internals page](../../../internals/builtins/string/bindec.md).

