---
title: "fputs()"
description: "Binary-safe file write (alias of fwrite)."
sidebar:
  order: 183
---

## fputs()

```php
function fputs(mixed $stream, string $data): mixed
```

Binary-safe file write (alias of fwrite).

**Parameters**:
- `$stream` (`mixed`)
- `$data` (`string`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/filesystem/fputs.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/filesystem/fputs.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `fputs` is implemented in the compiler, see [the internals page](../../../internals/builtins/io/fputs.md).
