---
title: "preg_match_all() — internals"
description: "Compiler internals for preg_match_all(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 333
---

## `preg_match_all()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/preg_match_all.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/preg_match_all.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/regex.rs`:94](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/regex.rs#L94) (`lower_preg_match_all`)
- **Function symbol**: `lower_preg_match_all()`


### Lowering notes

- Lowers `preg_match_all(pattern, subject)` through the shared regex runtime helper.
- Lowers `preg_match_all(pattern, subject, &matches?, flags?, offset?)` through the regex runtime.
- The optional `$matches` out-parameter is populated through `__rt_preg_match_capture` (the same
- helper `preg_match` uses), so the caller's variable is defined and readable after the call. The
- capture helper records the first match and its capture groups; full `preg_match_all` semantics
- (nested per-match arrays) require a dedicated runtime helper and are not yet implemented —
- `$flags` and `$offset` are accepted so calls type-check and lower but behave as the defaults.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_preg_match_all`
- `__rt_preg_match_capture`

## Signature summary

```php
function preg_match_all(string $pattern, string $subject, array $matches = [], int $flags = 0, int $offset = 0): int
```

## What the type checker enforces

- **Arity**: takes 2–5 arguments (3 optional).
- **By-reference parameters**: `$matches`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/regex/preg_match_all.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/regex/preg_match_all.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`
- **By-reference parameters**: `$matches`.

## Cross-references

- [User reference for `preg_match_all()`](../../../php/builtins/regex/preg_match_all.md)
