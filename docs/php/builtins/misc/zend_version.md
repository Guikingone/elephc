---
title: "zend_version()"
description: "Returns the version of the Zend engine the runtime reports."
sidebar:
  order: 664
---

## zend_version()

```php
function zend_version(): string
```

Returns the version of the Zend engine the runtime reports.

**Parameters**: none.

**Returns**: `string`

## Availability

- **Compiled (AOT)**: supported through the compiler-injected version prelude.
- **`eval()` (magician interpreter)**: not available inside eval'd code.

_No examples yet — check `examples/` and `showcases/` for usage patterns._

## Internals

For how `zend_version` is implemented in the compiler, see [the internals page](../../../internals/builtins/misc/zend_version.md).
