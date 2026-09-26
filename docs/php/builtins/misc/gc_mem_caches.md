---
title: "gc_mem_caches()"
description: "Reclaims memory used by the Zend Engine memory manager and returns the number of bytes freed."
sidebar:
  order: 616
---

## gc_mem_caches()

```php
function gc_mem_caches(): int
```

Reclaims memory used by the Zend Engine memory manager and returns the number of bytes freed.

**Parameters**: none.

**Returns**: `int`

## Availability

- **Compiled (AOT)**: supported by the Elephc code generator.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/gc.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `gc_mem_caches` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/gc_mem_caches.md).
