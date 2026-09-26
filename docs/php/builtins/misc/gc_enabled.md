---
title: "gc_enabled()"
description: "Returns whether the circular reference collector is active."
sidebar:
  order: 615
---

## gc_enabled()

```php
function gc_enabled(): bool
```

Returns whether the circular reference collector is active.

**Parameters**: none.

**Returns**: `bool`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `gc_enabled` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/gc_enabled.md).
