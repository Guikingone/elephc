---
title: "html_entity_decode() — internals"
description: "Compiler internals for html_entity_decode(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 377
---

## `html_entity_decode()` — internals

## Where it lives

- **Signature**: [`src/builtins/string/html_entity_decode.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/string/html_entity_decode.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/strings.rs`:75](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/strings.rs#L75) (`lower_unary_string_runtime`)
- **Function symbol**: `lower_unary_string_runtime()`


### Lowering notes

- Lowers a one-argument string builtin that directly delegates to a runtime helper.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_htmlspecialchars`

## Signature summary

```php
function html_entity_decode(string $string, int $flags = 11, string $encoding = null): string
```

## What the type checker enforces

- **Arity**: takes 1–3 arguments (2 optional).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/string/html_entity_decode.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/string/html_entity_decode.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `html_entity_decode()`](../../../php/builtins/string/html_entity_decode.md)
