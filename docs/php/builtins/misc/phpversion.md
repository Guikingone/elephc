---
title: "phpversion()"
description: "Returns the targeted PHP language version, or one extension's version."
sidebar:
  order: 300
---

## phpversion()

```php
function phpversion(?string $extension = null): string|false
```

Returns the PHP **language** version elephc targets, or — when `$extension` is
given — that same version string for a loaded extension and `false` for anything
else.

**Parameters**:

| Name | Type | Default | Description |
|---|---|---|---|
| `$extension` | `?string` | `null` | Extension name, compared case-insensitively. Omit for the PHP version itself. |

**Returns**: `string|false`

elephc reports `8.<minor>.0` for the `--php-version` profile (`8.5.0` by
default), not an upstream patch release — reference PHP 8.5.6 reports `8.5.6`.
See [the PHP version surface](../../system-and-io.md#php-version-surface) for the
rule, the full constant table and the divergences.

Extension membership is exactly `extension_loaded()`'s, so `phpversion($e) !==
false` and `extension_loaded($e)` can never disagree. Reference PHP reports the
interpreter's own version for every bundled extension, which is why a loaded
extension reports the PHP version rather than a version of its own.

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `phpversion` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/phpversion.md).
