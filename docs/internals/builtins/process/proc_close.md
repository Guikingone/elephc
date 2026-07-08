---
title: "proc_close() — internals"
description: "Compiler internals for proc_close(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 307
---

## `proc_close()` — internals

## Where it lives

- **Signature**: [`src/builtins/io/proc_close.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/io/proc_close.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/io.rs`:3692](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/io.rs#L3692) (`lower_proc_close`)
- **Function symbol**: `lower_proc_close()`


### Lowering notes

- Lowers `proc_close(process)` and returns the child process exit status.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_proc_close`

## Signature summary

```php
function proc_close(resource $process): int
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `proc_close()`](../../../php/builtins/process/proc_close.md)
