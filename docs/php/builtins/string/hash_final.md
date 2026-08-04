---
title: "hash_final()"
description: "Finalizes an incremental hashing context and returns the digest (hex, or raw bytes when $binary). Provided by the compiler-injected hash prelude in compiled code."
sidebar:
  order: 384
---

## hash_final()

```php
function hash_final(mixed $context, mixed $binary = 'false'): mixed
```

Finalizes an incremental hashing context and returns the digest (hex, or raw bytes when $binary). Provided by the compiler-injected hash prelude in compiled code.

**Parameters**:
- `$context` (`mixed`)
- `$binary` (`mixed`), default `'false'`, optional

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: not available — compiled programs cannot call this builtin yet.
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/hash_final.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._
