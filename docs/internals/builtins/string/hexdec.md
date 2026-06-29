---
title: "hexdec() — internals"
description: "Compiler internals for hexdec(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 348
---

## `hexdec()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:197](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L197) (`lower_hexdec`)
- **Function symbol**: `lower_hexdec()`


### Lowering notes

- Lowers `hexdec(hex_string)` through the runtime hex-string parser.
- The runtime helper ignores non-hexadecimal bytes (matching PHP) and folds the
- hex digits into a 64-bit integer returned in the int result register.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_hexdec`

## Signature summary

```php
function hexdec(mixed $hex_string): int
```

## What the type checker enforces

- **Arity**: takes exactly 1 argument.

## Cross-references

- [User reference for `hexdec()`](../../../php/builtins/string/hexdec.md)

