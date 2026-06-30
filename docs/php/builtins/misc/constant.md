---
title: "constant()"
description: "Lowers `constant($name)` against the closed-world constant registry."
sidebar:
  order: 257
---

## constant()

```php
function constant(string $name): mixed
```

Lowers `constant($name)` against the closed-world constant registry.

**Parameters**:
- `$name` (`string`)

**Returns**: `mixed`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `constant` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/constant.md).

