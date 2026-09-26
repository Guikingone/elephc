---
title: "gc_collect_cycles()"
description: "Forces collection of any existing garbage cycles and returns how many were collected."
sidebar:
  order: 612
---

## gc_collect_cycles()

```php
function gc_collect_cycles(): int
```

Forces collection of any existing garbage cycles and returns how many were collected.

**Parameters**: none.

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `gc_collect_cycles` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/gc_collect_cycles.md).
