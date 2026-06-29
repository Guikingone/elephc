---
title: "strspn()"
description: "Lowers `strcspn()`/`strspn()` initial-segment-span builtins to a runtime helper."
sidebar:
  order: 387
---

## strspn()

```php
function strspn(mixed $string, mixed $characters, mixed $offset, mixed $length): int
```

Lowers `strcspn()`/`strspn()` initial-segment-span builtins to a runtime helper.

**Parameters**:
- `$string` (`mixed`)
- `$characters` (`mixed`)
- `$offset` (`mixed`), optional
- `$length` (`mixed`), optional

**Returns**: `int`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `strspn` is implemented in the compiler, see [the internals page](../../../internals/builtins/string/strspn.md).

