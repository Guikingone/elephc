---
title: "array_change_key_case()"
description: "Changes the case of all string keys in an array."
sidebar:
  order: 3
---

## array_change_key_case()

```php
function array_change_key_case(array $array, int $case = 0): array
```

Changes the case of all string keys in an array.

**Parameters**:
- `$array` (`array`)
- `$case` (`int`), default `0`, optional

**Returns**: `array`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `array_change_key_case` is implemented in the compiler, see [the internals page](../../../internals/builtins/array/array_change_key_case.md).
