---
title: "strpbrk() — internals"
description: "Compiler internals for strpbrk(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 379
---

## `strpbrk()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:186](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L186) (`lower_strpbrk`)
- **Function symbol**: `lower_strpbrk()`


### Lowering notes

- Lowers `strpbrk(string, characters)` and boxes its `string|false` result as Mixed.
- `__rt_strpbrk` returns the suffix of `string` starting at the first byte that
- occurs in `characters` (pointer/length in the string-result registers), or a
- null pointer when no character matches. The null sentinel is boxed as PHP
- `false`, mirroring `strstr`/`grapheme_strrev` search builtins.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_explode`
- `__rt_hexdec`
- `__rt_strpbrk`

## Signature summary

```php
function strpbrk(string $string, string $characters): mixed
```

## What the type checker enforces

- **Arity**: takes exactly 2 arguments.

## Cross-references

- [User reference for `strpbrk()`](../../../php/builtins/string/strpbrk.md)

