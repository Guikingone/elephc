---
title: "get_extension_funcs() — internals"
description: "Compiler internals for get_extension_funcs(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 328
---

## `get_extension_funcs()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/get_extension_funcs.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/get_extension_funcs.rs)
- **Argument lowering**: [`src/ir_lower/expr/array_builtin_args.rs`](https://github.com/illegalstudio/elephc/blob/main/src/ir_lower/expr/array_builtin_args.rs) (`lower_get_extension_funcs_args`)
- **Direct-AST coercion helper**: [`src/get_extension_funcs_prelude.rs`](https://github.com/illegalstudio/elephc/blob/main/src/get_extension_funcs_prelude.rs)


### Lowering notes

- Keeps the registry's typed `runtime.get_extension_funcs` target for extension lookup.
- Lowers the first operand through a pay-for-use direct Rust AST helper before the runtime call.
- The helper implements PHP's weak string-parameter binding: scalar coercion, the null deprecation, Stringable objects, and catchable `TypeError`s for arrays, resources, and non-Stringable objects. In strict-types code it rejects every non-string value.
- Production code never embeds PHP source or routes this helper through the PHP parser.

## Semantic descriptor

- **Target strategy**: `runtime_call`
- **Validation**: `checker_hook`
- **Result type source**: `checked`
- **Result ownership**: `may_alias_arguments`
- **Effects**: `static (16 declared effects)`
- **Requirements**: `static (0 requirements)`
- **Callable policy**: `static_only`
- **Target support**: `macos-aarch64`, `ios-arm64`, `ios-sim-arm64`, `linux-aarch64`, `linux-x86_64`

## EIR and runtime boundary

- **Typed EIR target**: `runtime.get_extension_funcs`
- **Backend boundary**: `src/codegen/lower_inst/runtime_calls.rs` resolves the typed target without PHP-name dispatch after the direct-AST argument binder has produced the extension string.

## Signature summary

```php
function get_extension_funcs(string $extension): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.
- **Binding**: PHP-compatible weak coercion is applied at lowering time because the public registry signature remains usable by typed callers; strict-types calls and invalid runtime values throw the same catchable errors as PHP.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/network_env/get_extension_funcs.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/get_extension_funcs.rs) (`eval_builtin!`)
- **Execution**: Magician interpreter adapter.
- **Adapter reason**: `runtime-state-or-resource`.
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `get_extension_funcs()`](../../../php/builtins/misc/get_extension_funcs.md)
