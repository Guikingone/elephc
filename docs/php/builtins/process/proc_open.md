---
title: "proc_open()"
description: "Execute a command and open file pointers for I/O."
sidebar:
  order: 308
---

## proc_open()

```php
function proc_open(string $descriptor_spec, string $command, array $pipes): mixed
```

Execute a command and open file pointers for I/O.

**Parameters**:
- `$descriptor_spec` (`string`)
- `$command` (`string`)
- `$pipes` (`array`), passed by reference

**Returns**: `mixed`

_No examples yet — check `examples/` and `showcases/` for usage patterns._







## Internals

For how `proc_open` is implemented in the compiler, see [the internals page](../../../internals/builtins/process/proc_open.md).

