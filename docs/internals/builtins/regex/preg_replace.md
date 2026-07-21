---
title: "preg_replace() — internals"
description: "Compiler internals for preg_replace(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 334
---

## `preg_replace()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/preg_replace.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/preg_replace.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins/regex.rs`:130](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins/regex.rs#L130) (`lower_preg_replace`)
- **Function symbol**: `lower_preg_replace()`


### Lowering notes

- Lowers `preg_replace(pattern, replacement, subject, limit?, &count?)`.
- The optional `$count` out-parameter is populated with the number of
- replacements performed, computed via `__rt_preg_match_all` over the same
- pattern/subject before the replacement runs (the unlimited `limit = -1` case,
- which matches every supported call). The optional `$limit` argument is
- accepted but not yet enforced; replacement always processes every match.

## Runtime helpers

The following runtime helpers are referenced:
- `__rt_preg_match_all`
- `__rt_preg_replace`

## Signature summary

```php
function preg_replace(string $pattern, string $replacement, string $subject, int $limit = -1, int $count = null): string
```

## What the type checker enforces

- **Arity**: takes 3–5 arguments (2 optional).
- **By-reference parameters**: `$count`.

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/regex/preg_replace.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/regex/preg_replace.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `preg_replace()`](../../../php/builtins/regex/preg_replace.md)
