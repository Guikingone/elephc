---
title: "preg_match() — internals"
description: "Compiler internals for preg_match(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 332
---

## `preg_match()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/preg_match.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/preg_match.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/regex.rs`:36](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/regex.rs#L36) (`lower_preg_match`)
- **Function symbol**: `lower_preg_match()`


### Lowering notes

- Lowers `preg_match(pattern, subject, &matches?, flags?, offset?)` via the regex runtime.
- The optional `$matches` out-parameter is populated through
- `__rt_preg_match_capture`. The `$flags` and `$offset` arguments are accepted
- (so calls type-check and lower) but are not yet honored by the EIR capture
- runtime; non-default flags/offset therefore behave as the defaults.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_preg_match`
- `__rt_preg_match_capture`

## Signature summary

```php
function preg_match(string $pattern, string $subject, array $matches = [], int $flags = 0, int $offset = 0): int
```

## What the type checker enforces

- **Arity**: takes 2–5 arguments (3 optional).
- **By-reference parameters**: `$matches`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/regex/preg_match.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/regex/preg_match.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`
- **By-reference parameters**: `$matches`.

## Cross-references

- [User reference for `preg_match()`](../../../php/builtins/regex/preg_match.md)
