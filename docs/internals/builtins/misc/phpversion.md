---
title: "phpversion() — internals"
description: "Compiler internals for phpversion(): lowering path, type checks, and runtime helpers."
sidebar:
  order: 300
---

## `phpversion()` — internals

## Where it lives

- **Signature**: [`src/builtins/system/phpversion.rs`](https://github.com/illegalstudio/elephc/blob/main/src/builtins/system/phpversion.rs)
- **Lowering**: [`src/codegen/lower_inst/builtins.rs`](https://github.com/illegalstudio/elephc/blob/main/src/codegen/lower_inst/builtins.rs) (`lower_phpversion`)
- **Function symbol**: `lower_phpversion()`


### Lowering notes

- Fully const-folded: there is no `__rt_*` helper. The version string comes from
  `codegen::compile_php_version().version_string()`, i.e. the `--php-version`
  profile of this compilation, seeded into a codegen thread-local by
  `pipeline::compile` (`codegen::set_compile_profile`).
- `phpversion()` emits a static string. `phpversion($extension)` emits either the
  version string or `false`, boxed into a `Mixed` cell by
  `emit_box_current_value_as_mixed` — that is how the `string|false` union is
  represented at the backend boundary.
- Extension membership reuses `extension_is_loaded()` /
  `dynamic_extension_loaded_candidates()`, the same predicate and the same baked
  candidate table `extension_loaded()` uses, so the two cannot disagree. A
  non-literal name is scanned against the table with `__rt_strcasecmp`, matching
  PHP's case-insensitive extension lookup.
- The loaded/not-loaded DECISION cannot move earlier than codegen: the effective
  set is core ∪ `codegen::linked_extensions()`, and the linked set is only
  complete after type checking (it includes bridges auto-detected from
  `check_result.required_libraries`). That is why the checker types the whole
  one-argument arity as `string|false` instead of folding per call site — a
  checker-side fold would see a smaller set and could contradict the backend.

## Semantic descriptor

- **Target strategy**: `runtime_call`
- **Validation**: `checker hook`
- **Result type source**: `shared` (`Str` for zero arguments, `Mixed` for one)
- **Result ownership**: `may_alias_arguments`
- **Effects**: `static (16 declared effects)`
- **Requirements**: `static (0 requirements)`
- **Callable policy**: `static_only`
- **Target support**: `macos-aarch64`, `linux-aarch64`, `linux-x86_64`

## EIR and runtime boundary

- **Typed EIR target**: `runtime.phpversion`
- **Backend boundary**: `src/codegen/lower_inst/runtime_functions/group_11.rs` routes the typed target to `lower_phpversion`; PHP builtin names do not participate in dispatch.

> The shared result-type hook keys off `BuiltinSemanticInput::arg_types`, NOT
> `args`: `semantics::lower_registry_call` re-resolves it with `args: &[]` (it
> only holds lowered EIR operands there), while
> `ir_lower::expr::registry_builtin_result_type` resolves it with real AST args.
> `arg_types` is derived from the operand list on both paths, so it is the only
> field that reports the same arity to both.

## Signature summary

```php
function phpversion(?string $extension = null): string|false
```

## What the type checker enforces

- **Arity**: `phpversion() takes 0 or 1 arguments`.
- **Argument type**: the extension name must infer as `string`, otherwise
  `phpversion() extension argument must be string`.
- **Result type**: `string` for the zero-argument form, `string|false` for the
  one-argument form (backend repr: boxed `Mixed`).

## Eval interpreter (magician)

- **Declaration**: [`crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs`](https://github.com/illegalstudio/elephc/blob/main/crates/elephc-magician/src/interpreter/builtins/network_env/phpversion.rs) (`eval_builtin!`)
- **Dispatch hooks**: `direct`, `values`

## Cross-references

- [User reference for `phpversion()`](../../../php/builtins/misc/phpversion.md)
