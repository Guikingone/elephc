---
title: "get_defined_functions() — internals"
description: "Compiler internals for get_defined_functions(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 294
---

## `get_defined_functions()` — internals

## Where it lives

- **Signature**: [`src/builtins/callables/get_defined_functions.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/callables/get_defined_functions.rs)
- **Lowering**: [`src/builtins/semantics.rs`:433](https://github.com/illegalstudio/elephc/blob/main/src/builtins/semantics.rs#L433) (`lower_registry_call`)
- **Function symbol**: `lower_registry_call()`


### Lowering notes

- Uses the `runtime_call` strategy from the single-source builtin descriptor.
- Emits the typed EIR target `runtime.get_defined_functions` through `BuiltinLoweringContext`.
- The backend resolves that typed target through `src/codegen/lower_inst/runtime_calls.rs`; PHP builtin names do not participate in dispatch.

## Semantic descriptor

- **Target strategy**: `runtime_call`
- **Validation**: `checker_hook`
- **Result type source**: `checked`
- **Result ownership**: `may_alias_arguments`
- **Effects**: `static (16 declared effects)`
- **Requirements**: `static (0 requirements)`
- **Callable policy**: `static_only`
- **Target support**: `macos-aarch64`, `linux-aarch64`, `linux-x86_64`

## EIR and runtime boundary

- **Typed EIR target**: `runtime.get_defined_functions`
- **Backend boundary**: `src/codegen/lower_inst/runtime_calls.rs` resolves the typed target without PHP-name dispatch.

## Signature summary

```php
function get_defined_functions(): array
```

## What the type checker enforces

- **Arity**: takes no arguments.

## Eval interpreter (magician)

_Not callable from eval'd code — the magician interpreter has no entry for this builtin._

## Cross-references

- [User reference for `get_defined_functions()`](../../../php/builtins/misc/get_defined_functions.md)
