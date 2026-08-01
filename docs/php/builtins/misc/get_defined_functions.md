---
title: "get_defined_functions()"
description: "Returns an array of all defined functions, split into 'internal' and 'user'."
sidebar:
  order: 294
---

## get_defined_functions()

```php
function get_defined_functions(): array
```

Returns an array of all defined functions, split into 'internal' and 'user'.

**Parameters**: none.

**Returns**: `array`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `get_defined_functions` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/get_defined_functions.md).
