---
title: "hash_init()"
description: "Opens an incremental hashing context, returning a HashContext object. Provided by the compiler-injected hash prelude in compiled code; the eval interpreter still returns a resource."
sidebar:
  order: 379
---

## hash_init()

```php
function hash_init(mixed $algo, mixed $flags = '0', mixed $key = '""'): mixed
```

Opens an incremental hashing context, returning a HashContext object. Provided by the compiler-injected hash prelude in compiled code; the eval interpreter still returns a resource.

**Parameters**:
- `$algo` (`mixed`)
- `$flags` (`mixed`), default `'0'`, optional
- `$key` (`mixed`), default `'""'`, optional

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: not available — compiled programs cannot call this builtin yet.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_init.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._
