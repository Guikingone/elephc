---
title: "hash_copy()"
description: "Clones an incremental hashing context into an independent HashContext object. Provided by the compiler-injected hash prelude in compiled code."
sidebar:
  order: 382
---

## hash_copy()

```php
function hash_copy(mixed $context): mixed
```

Clones an incremental hashing context into an independent HashContext object. Provided by the compiler-injected hash prelude in compiled code.

**Parameters**:
- `$context` (`mixed`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: not available — compiled programs cannot call this builtin yet.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/string/hash_copy.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_copy.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._
