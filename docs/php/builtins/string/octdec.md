---
title: "octdec()"
description: "Lowers a one-argument string builtin that directly delegates to a runtime helper."
sidebar:
  order: 359
---

## octdec()

```php
function octdec(mixed $octal_string): int
```

Lowers a one-argument string builtin that directly delegates to a runtime helper.

**Parameters**:
- `$octal_string` (`mixed`)

**Returns**: `int`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `octdec` is implemented in the compiler, see [the internals page](../../../internals/builtins/string/octdec.md).

