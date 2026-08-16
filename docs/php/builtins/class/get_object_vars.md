---
title: "get_object_vars()"
description: "Returns visible properties for an object."
sidebar:
  order: 90
---

## get_object_vars()

```php
function get_object_vars(mixed $object): mixed
```

Returns visible properties for an object.

**Parameters**:
- `$object` (`mixed`)

**Returns**: `mixed`

## Availability

- **Compiled (AOT)**: not available — compiled programs cannot call this builtin (`eval-only-reflection`).
- **`eval()` (magician interpreter)**: supported — declarative interpreter builtin ([`crates/elephc-magician/src/interpreter/builtins/symbols/get_object_vars.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/symbols/get_object_vars.rs)).

_No examples yet — check `examples/` and `showcases/` for usage patterns._
