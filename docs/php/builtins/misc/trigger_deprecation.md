---
title: "trigger_deprecation()"
description: "Lowers `trigger_deprecation(package, version, message, ...args)` as a sound no-op."
sidebar:
  order: 271
---

## trigger_deprecation()

```php
function trigger_deprecation(mixed $package, mixed $version, mixed $message, ...$args): void
```

Lowers `trigger_deprecation(package, version, message, ...args)` as a sound no-op.

**Parameters**:
- `$package` (`mixed`)
- `$version` (`mixed`)
- `$message` (`mixed`)
- `...$args` — variadic: collects excess arguments into `$args`.

**Returns**: `void`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `trigger_deprecation` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/trigger_deprecation.md).

