---
title: "proc_open() — internals"
description: "Compiler internals for proc_open(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 308
---

## `proc_open()` — internals

## Where it lives

- **Signature**: [`src/builtins/io/proc_open.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/io/proc_open.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/io.rs`:3651](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/io.rs#L3651) (`lower_proc_open`)
- **Function symbol**: `lower_proc_open()`


### Lowering notes

- Lowers `proc_open(descriptor_spec, command, pipes)` and boxes the process as
- `resource|false`. The `pipes` array is passed by reference so the runtime can
- populate it with the child's pipe descriptors.
- Runtime ABI: AArch64 `x0` = descriptor_spec array pointer, `x1` = command
- pointer, `x2` = command length, `x3` = pipes array pointer; x86_64 `rdi` =
- descriptor_spec pointer, `rsi` = command pointer, `rdx` = command length,
- `rcx` = pipes array pointer.

## Runtime helpers

_No direct `__rt_*` helpers captured — the lowering is inlined or routes through another builtin._

## Signature summary

```php
function proc_open(string $descriptor_spec, string $command, array $pipes): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 3 arguments.
- **By-reference parameters**: `$pipes`.

## Cross-references

- [User reference for `proc_open()`](../../../php/builtins/process/proc_open.md)
