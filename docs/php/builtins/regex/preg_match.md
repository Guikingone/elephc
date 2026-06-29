---
title: "preg_match()"
description: "Lowers `preg_match(pattern, subject, &matches?, flags?, offset?)` via the regex runtime."
sidebar:
  order: 296
---

## preg_match()

```php
function preg_match(string $pattern, string $subject, array $matches, mixed $flags, mixed $offset): int
```

Lowers `preg_match(pattern, subject, &matches?, flags?, offset?)` via the regex runtime.

**Parameters**:
- `$pattern` (`string`)
- `$subject` (`string`)
- `$matches` (`array`), passed by reference, optional
- `$flags` (`mixed`), optional
- `$offset` (`mixed`), optional

**Returns**: `int`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `preg_match` is implemented in the compiler, see [the internals page](../../../internals/builtins/regex/preg_match.md).

