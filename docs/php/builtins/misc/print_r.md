---
title: "print_r()"
description: "Prints human-readable information about a variable."
sidebar:
  order: 265
---

## print_r()

```php
function print_r(mixed $value): void
```

Prints human-readable information about a variable.

> **Note**: PHP also accepts an optional `bool $return` second argument that returns the output as a string instead of printing it. This `$return` mode is not yet supported by elephc; only the single-argument form that writes to stdout is implemented.

**Parameters**:
- `$value` — `mixed`: the value to print.

**Returns**: `void`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `print_r` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/print_r.md).

