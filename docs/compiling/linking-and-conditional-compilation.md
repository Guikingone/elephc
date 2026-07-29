---
title: "Linking, heap, and conditional compilation"
description: "Linking native libraries and frameworks for FFI, sizing the runtime heap, and defining compile-time symbols for ifdef branches."
sidebar:
  order: 8
---

These flags control how the binary is linked, how much heap the program gets, and
which compile-time branches are taken.

## Linking native libraries

When a program calls into C libraries through [extern/FFI](../beyond-php/extern.md),
those libraries must be linked into the binary. Raw link flags, managed native
packages, Composer source, Rust bridge crates, and toolchains are distinct
mechanisms:

| Mechanism | Use it for | Do not use it for |
|---|---|---|
| `elephc native` | Reviewed runtime/builtin-oriented C packages with exact source, lock, recipe, and cached static outputs | Arbitrary FFI libraries, PHP packages, Rust crates, or tool installation |
| Composer/autoload | PHP source dependencies resolved ahead of time | C archives or Rust bridges |
| Auto-detected bridge / `--with-CRATE` | Optional Elephc Rust `staticlib` implementations | Catalogued C sources |
| User/OS toolchain | `cc`, `ar`, `ranlib`, assembler, linker, Make, SDK, and cross tools | Project dependency locking |

Raw `extern` linking is a separate user-supplied workflow layered onto the
linker. In particular, DOOM and the SDL examples require a user-installed SDL
plus `extern` and `--link`/`--link-path`; SDL is **not** installed or versioned
by `elephc native`.

### Managed native packages

Curated C/C++ dependencies are declared and installed with `elephc native`, not
with raw linker flags. During a final link, the compiler resolves logical
requirements against the nearest project's `elephc.toml`, deterministic
`elephc.lock`, and verified target/toolchain cache receipt. It passes exact
static archive paths to the linker; compilation never downloads or builds them.

The current catalog contains PCRE2 10.47 and zlib 1.3.2. Regex use links PCRE2's
managed archives in the fixed shim/POSIX/8-bit order and has no production
system-library fallback:

```bash
elephc native add pcre2
elephc app.php
```

Declaring PCRE2 does not force it into a program that does not use regex. Exact
managed archives remain compatible with Linux's static-link preference. zlib is
the second exact pure-C recipe; it is available for curated runtime/builtin
integration, not an automatic replacement for arbitrary `extern "z"` and `-lz`
workflows. See [Native dependencies](native-dependencies.md) for the full
workflow.

### `--link` / `-l`

Links an extra native library. Accepts the spaced form, the short flag, and the
attached form; repeat it for multiple libraries.

```bash
elephc app.php --link sqlite3
elephc app.php -l sqlite3
elephc app.php -lsqlite3
```

### `--link-path` / `-L`

Adds a directory to the library search path. Repeatable.

```bash
elephc app.php -l sqlite3 -L /opt/homebrew/lib
elephc app.php --link-path /usr/local/lib
```

### `--framework`

Links a macOS framework. Repeatable.

```bash
elephc app.php --framework Cocoa --framework Metal
```

`extern "libname" { ... }` blocks in source add their own `-l` flags
automatically; the flags above are for libraries not already named in the source.
They do not override or satisfy a missing managed-package requirement such as
PCRE2.
See [FFI & Extern](../beyond-php/extern.md).

## Bridge crates and `--with-CRATE`

Some optional features are implemented as Rust *bridge crates* (`staticlib`
archives) that elephc links into the program: `pdo` (database access), `tls`
(`https://`/`ftps://` streams), `crypto` (the `hash()`/`md5()`/`sha1()` family),
`phar` (Phar archives), `tz` (timezone introspection), `image` (GD/Imagick image
processing), `eval` (the Magician interpreter fallback for dynamic `eval()`),
and `web` (the `--web` server).

By default a bridge is linked **only when the program uses it** — using a hash
function pulls in `crypto`, opening an `https://` stream pulls in `tls`,
referencing `PDO` pulls in `pdo`, and so on. An `eval()` call pulls in Magician
only when it needs runtime parsing: eligible literal fragments can be parsed at
compile time and lowered to native EIR without the interpreter bridge. Programs
that do not need a feature never link its crate, so binaries stay small.

`--with-CRATE` force-enables a bridge regardless of that auto-detection. It
force-links the staticlib (whole-archived, so it is retained even if no symbol
references it) and, for crates whose PHP surface comes from an injected prelude
(`pdo`, `tz`, `image`), force-injects that prelude so the classes/functions are
available. This is useful when a program reaches a feature through indirection
that detection cannot see. The flag is repeatable:

```bash
elephc app.php --with-pdo
elephc app.php --with-crypto --with-tls
elephc app.php --with-eval
```

`--with-eval` force-links `elephc_magician`; it does not enable new syntax or
change which fragments are eligible for AOT lowering. Normal eval usage is
detected automatically. See [Eval](../php/eval.md) for language semantics and
[Eval Runtime Architecture](../internals/eval-runtime.md) for the AOT/fallback
decision and scope ABI.

`--with-web` is an alias for [`--web`](../beyond-php/web.md) (the full server
mode, which owns the program entry point). An unknown crate name is rejected with
the list of valid crates. Forcing a crate increases binary size, since the whole
archive is included.

Bridge crates are Elephc's optional Rust workspace components. They are not
installed or versioned by `elephc native`, and `--with-CRATE` is not a native
package command. Composer dependencies are PHP source handled by the compile-time
autoload pipeline; they are separate from both mechanisms.

## Heap size

The compiled program uses a fixed-size runtime heap, **8 MB** by default. Programs
that allocate a lot of arrays, strings, or objects may need more.

### `--heap-size`

Sets the heap size in bytes. The minimum is `65536` (64 KB).

```bash
elephc --heap-size=16777216 heavy.php   # 16 MB
```

If a program exhausts its heap it aborts with a fatal "heap memory exhausted"
error; raising `--heap-size` is the fix. See [Memory Model](../internals/memory-model.md).

## Runtime dead stripping

The compiler ships a single runtime with helpers for every supported builtin, but
a given program only uses a few of them. When linking an **executable**, the
linker keeps only the runtime helpers reachable from the program and drops the
rest, so a small program does not carry the whole runtime. This is automatic —
there is no flag — and never changes behavior, only binary size.

It works the same on every supported target, using each platform's native
mechanism:

- **Linux** emits each runtime helper into its own section and links with
  `--gc-sections`.
- **macOS** emits the runtime object with `.subsections_via_symbols` so each
  helper is a separately collectable atom, and links with `-dead_strip`.

Shared libraries (`--emit cdylib`) keep the full runtime, since any exported
symbol may be reached by a host the linker cannot see.

## Conditional compilation

elephc supports compile-time feature branches with `ifdef`. Symbols are defined
on the command line and the branches are resolved before optimization and code
generation, so unused branches are never compiled.

### `--define` / `--define=`

Defines a compile-time symbol. Repeatable. It may be combined with
[`--strict-php`](cli-reference.md#strict-php-mode), but strict auditing still
rejects every `ifdef` in physical PHP source. The combination exists for mixed
projects and LFC source: LFC `ifdef` consumes the symbol while PHP source
remains audited.

```bash
elephc --define DEBUG app.php
elephc --define=DEBUG --define=METAL app.php
elephc --strict-php --define DEBUG app.lfc
```

```php
ifdef (DEBUG) {
    echo "debug build\n";
}
```

See [Conditional Compilation](../beyond-php/ifdef.md) for the full `ifdef` syntax
and semantics.
