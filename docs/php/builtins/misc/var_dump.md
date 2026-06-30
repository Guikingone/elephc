---
title: "var_dump()"
description: "Dumps information about a variable, including its type and value."
sidebar:
  order: 269
---

## var_dump()

```php
function var_dump(mixed $value, mixed ...$values): void
```

Dumps information about one or more variables, including each type and value. Each argument is dumped independently in source order, so `var_dump(1, "two", 3.5)` emits three separate dumps.

**Parameters**:
- `$value` — `mixed`: the first value to dump (required).
- `...$values` — `mixed` (variadic): additional values to dump, each rendered as its own dump output.

**Returns**: `void`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `var_dump` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/var_dump.md).

