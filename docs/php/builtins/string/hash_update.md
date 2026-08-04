---
title: "hash_update()"
description: "Feeds data into an incremental hashing context. Provided by the compiler-injected hash prelude in compiled code."
sidebar:
  order: 396
---

## hash_update()

```php
function hash_update(mixed $context, mixed $data): mixed
```

Feeds data into an incremental hashing context. Provided by the compiler-injected hash prelude in compiled code.

**Parameters**:
- `$context` (`mixed`)
- `$data` (`mixed`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: not available — compiled programs cannot call this builtin yet.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_update.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._
