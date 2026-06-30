---
title: "strpos() — internals"
description: "Compiler internals for strpos(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 391
---

## `strpos()` — internals

## Where it lives

- **Signature**: [`src/types/signatures.rs`](https://github.com/illegalstudio/elephc/blob/main/src/types/signatures.rs)
- **Lowering**: [`src/codegen_ir/lower_inst/builtins/strings.rs`:919](https://github.com/illegalstudio/elephc/blob/main/src/codegen_ir/lower_inst/builtins/strings.rs#L919) (`lower_strpos`)
- **Function symbol**: `lower_strpos()`


### Lowering notes

- Lowers `strpos(haystack, needle[, offset])` with optional starting offset.
- The 2-arg form delegates to the shared binary-string-runtime path.  The 3-arg form
- adjusts the haystack pointer and length by the given offset, calls `__rt_strpos`, and
- then adds the offset back to the returned position so the result is absolute.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_strpos`
- `__rt_substr_count`

## Signature summary

```php
function strpos(string $haystack, string $needle, int $offset): mixed
```

## What the type checker enforces

- **Arity**: takes 2–3 arguments (1 optional).

## Cross-references

- [User reference for `strpos()`](../../../php/builtins/string/strpos.md)

