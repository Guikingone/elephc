# Per-file declaration attribution for Reflection Implementation Plan

**Goal:** `ReflectionClass::getFileName()` must report the file a class was **declared** in, as PHP
does, instead of silently falling back to the entry file.

**Architecture:** Declaration→file attribution is currently computed *after* `require`/`include`
targets have been spliced into their parent program, so every declaration in a physically separate
file inherits its parent's path — and declarations pulled in by the entry program are absent from
the table entirely. The physical loaders already know each file's path; this plan records the
attribution there, per physical file, before any splicing.

**Tech Stack:** Rust. `src/resolver/files.rs`, `src/resolver/engine*.rs`, `src/autoload/mod.rs`,
`src/pipeline.rs`, `src/ir/module.rs`, `tests/codegen/oop/reflection*.rs`.

---

## Measured defect

```php
// Widget.php
namespace Inc;
class Widget {}

// main.php
require __DIR__ . '/Widget.php';
$r = new \ReflectionObject(new \Inc\Widget());
echo $r->getFileName();
```

| | result |
|---|---|
| `php -n` | `…/Widget.php` — the **declaring** file |
| elephc | `…/main.php` — the **entry** file |

`is_file()` is true either way, so nothing downstream notices. Symfony's
`AbstractKernel::getProjectDir()` walks up from this path looking for `composer.json`; it happens to
recover because the walk-up compensates, which is precisely why this stayed invisible.

## Mechanism

`src/autoload/mod.rs:458`:

```rust
let (resolved, nested_includes) = resolve_collecting_includes_with_defines(parsed, ...)?;
let canonicalized = name_resolver::resolve(resolved)?;
let declaration_sources = declaration_source_files(&canonicalized, &file_label);
```

`declaration_source_files` walks the program **after** includes were spliced and attributes every
declaration it finds to the single `file_label`. Consequences:

- a class declared in `B.php` and `require`d by an autoloaded `A.php` is attributed to `A.php`;
- a class `require`d by the **entry** program is attributed nowhere, so
  `reflection_source_file` (`codegen/lower_inst/objects/reflection/owner_emission.rs:426`) takes its
  `.or_else(|| ctx.module.source_path.clone())` fallback and reports the entry file.

The fallback is the part that makes this silent: it always produces a real, existing path.

## Fix

Attribute at the physical loader, where the path is already in hand:

- `src/resolver/files.rs:56` loads one included file and calls `finalize_physical_program(parsed,
  path, ...)`. Record that file's *own* class-likes, interfaces, traits, enums and functions against
  `path` here, before its own includes are resolved.
- Thread the collected map out of `resolve_collecting_includes*` alongside the existing
  `nested_includes`, and merge in `autoload/mod.rs` and `pipeline.rs`.
- Per-file attribution must win over the current post-splice attribution when both exist.

Note `finalize_physical_program` is also where `__FILE__` is substituted
(`magic_constants::substitute_file_and_scope_constants`), which is why `__FILE__` is already correct
per file while `getFileName()` is not: the same knowledge exists, it is just not recorded for
declarations.

## Semantic invariants

1. A class declared in the entry file keeps reporting the entry file.
2. `ReflectionFunction::getFileName()` follows the same rule via `declared_function_source_files`.
3. `__FILE__`, `__DIR__` and `getFileName()` must agree for a class declared in an included file.
4. Paths stay canonicalized exactly as `__FILE__` bakes them (`Path::canonicalize`), so
   OPcache script-manifest comparisons keep matching.

## Task checklist

- [ ] Task 1: collect per-file declarations in `resolver/files.rs`, unit-tested in isolation
- [ ] Task 2: thread the map through `resolve_collecting_includes*`
- [ ] Task 3: merge in `autoload/mod.rs` + `pipeline.rs`, per-file attribution winning
- [ ] Task 4: end-to-end tests (below)
- [ ] Task 5: re-check `getProjectDir()` on the Symfony boot

## Acceptance criteria

1. The two-file reproduction above prints the declaring file, matching `php -n`.
2. A class declared in the entry file still reports the entry file.
3. A class in a file `require`d by an autoloaded file reports its own file, not the autoloaded one.
4. `__FILE__` inside that class's method and `getFileName()` on it return the same string.
5. `codegen::oop::reflection*` stays at its current pass set (set comparison, not counts).
