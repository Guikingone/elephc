---
title: "gc_disable()"
description: "Deactivates the circular reference collector."
sidebar:
  order: 613
---

## gc_disable()

```php
function gc_disable(): void
```

Deactivates the circular reference collector.

**Parameters**: none.

**Returns**: `void`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `gc_disable` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/gc_disable.md).
