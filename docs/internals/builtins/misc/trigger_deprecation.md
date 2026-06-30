---
title: "trigger_deprecation() — internals"
description: "Compiler internals for trigger_deprecation(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 274
---

## `trigger_deprecation()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/system.rs`:322](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/system.rs#L322) (`lower_trigger_deprecation`)
- **Function symbol**: `lower_trigger_deprecation()`


### Lowering notes

- Lowers `trigger_deprecation(package, version, message, ...args)` as a sound no-op.
- elephc suppresses Symfony deprecation notices: the call's arguments are already
- evaluated (for their side effects) during argument lowering, so this emitter emits
- no instructions and produces no value. Any owning argument temporaries are released
- by the shared call-argument cleanup, exactly as for any other builtin call.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_gmmktime`
- `__rt_mktime`

## Signature summary

```php
function trigger_deprecation(mixed $package, mixed $version, mixed $message, ...$args): void
```

## What the type checker enforces

- **Arity**: takes exactly 3 arguments.
- **Variadic**: collects excess arguments into `$args`.

## Cross-references

- [User reference for `trigger_deprecation()`](../../../php/builtins/misc/trigger_deprecation.md)

