---
title: "constant()"
description: "Returns the value of a constant given its name."
sidebar:
  order: 298
---

## constant()

```php
function constant(string $name): mixed
```

Returns the value of a constant given its name.

**Parameters**:
- `$name` (`string`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `constant` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/constant.md).
