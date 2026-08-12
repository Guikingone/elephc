# Design: PHP `bcmath` procedural functions (AOT + Magician)

**Date:** 2026-08-12
**Status:** approved
**Baseline:** PHP 8.4 / 8.5 (elephc default profile)

## Goal

Ship the 14 PHP 8.4 `bcmath` procedural functions on both compilation paths,
with one PHP-compatible decimal engine and no `BcMath\Number` class.

| Path | Mechanism |
|---|---|
| AOT | Registry `builtin!` → `RuntimeFnId::Bc*` → target-aware lowering → `elephc_bcmath_*` C ABI |
| Magician | `eval_builtin!` homes that call the same crate ABI |

## Non-goals

- `BcMath\Number` and operator overloading (`+`, `-`, `*`, `/`, `**`, `%`, comparisons). Elephc has no operator-overloading dispatch; the class is deferred.
- `ini_get` / `ini_set('bcmath.scale')`. Default scale is `0`; `bcscale()` is the supported setter/getter.
- Linking php-src `libbcmath` or any system decimal library.
- Re-implementing decimal arithmetic in assembly.
- Version-gating the four PHP 8.4-only names (`bcceil`, `bcfloor`, `bcround`, `bcdivmod`) behind `--php-version`. All 14 names are always in the catalog (elephc does not version-gate individual builtins today).

## Architecture

A new workspace bridge crate `crates/elephc-bcmath` (`staticlib` + `rlib`):

- The engine is pure Rust and PHP-compatible. Internal representation is a base-10
  number (`sign`, decimal digits, `scale`), not `bigdecimal` / `rust_decimal`.
  Those crates are **not** the source of truth: PHP scale is “digits after the
  point, then truncate”, and the output string form is specified by scale.
- `num-bigint` may be used **only** as an implementation detail of integer
  modular exponentiation inside `bcpowmod`. It must not decide output scale or
  string formatting.
- C ABI `elephc_bcmath_*` is panic-free (catch/encode; never unwind into
  generated code). Pointer + length for strings. Integer status codes for errors.
- Magician depends on the crate the same way it depends on `elephc-crypto` and
  calls the same `extern "C"` functions.
- AOT never implements decimal math in `__rt_*` assembly. Runtime helpers only
  marshal PHP strings, publish function-pointer slots, map status codes onto
  catchable `\ValueError` / `\DivisionByZeroError`, and persist result strings.

Linking follows hash/openssl:

- Each `RuntimeFnId::Bc*` declares `BuiltinRequirement::Bridge("elephc_bcmath")`.
- Usage auto-links `libelephc_bcmath.a`.
- `--with-bcmath` force-links the whole archive.
- `BRIDGES` entry: `php_extension: Some("bcmath")`, `flag_name: "bcmath"`,
  `needs_libdl: true`, `whole_archive: false`.

### Process scale

- Default scale is `0`.
- Stored in the crate as an `AtomicI32` behind `elephc_bcmath_get_scale` /
  `elephc_bcmath_set_scale`.
- Every arithmetic operation receives an explicit scale **or** a “use global”
  flag. The engine itself is otherwise stateless.
- `bcscale($n)` sets and returns the previous scale. `bcscale()` / `bcscale(null)`
  returns the current scale without changing it.
- AOT and Magician in the **same process** must observe the same scale.

Mandatory mixed test:

```php
bcscale(4);
eval('echo bcmul("1", "1");');
```

must print `1.0000`. If linking both `libelephc_magician.a` (which would otherwise
bake a second copy of the crate) and `libelephc_bcmath.a` produces two scale
atoms or duplicate `no_mangle` symbols, Magician must stop embedding the crate
and call `extern "C"` symbols provided by the single linked staticlib. In that
fallback, any program that embeds Magician and can evaluate `bc*` must also link
`elephc_bcmath` (same idea as `--with-regex` for eval regex).

## PHP contract

Authoritative surface: PHP 8.4 manuals. Cross-check every edge with `php -r`
during implementation; fixture tables win over this prose if they disagree.

### Signatures

```php
function bcadd(string $num1, string $num2, ?int $scale = null): string
function bcsub(string $num1, string $num2, ?int $scale = null): string
function bcmul(string $num1, string $num2, ?int $scale = null): string
function bcdiv(string $num1, string $num2, ?int $scale = null): string
function bcmod(string $num1, string $num2, ?int $scale = null): string
function bcdivmod(string $num1, string $num2, ?int $scale = null): array
function bcpow(string $num, string $exponent, ?int $scale = null): string
function bcpowmod(string $num, string $exponent, string $modulus, ?int $scale = null): string
function bcsqrt(string $num, ?int $scale = null): string
function bccomp(string $num1, string $num2, ?int $scale = null): int
function bcscale(?int $scale = null): int
function bcceil(string $num): string
function bcfloor(string $num): string
function bcround(string $num, int $precision = 0, int $mode = 1): string
```

`$scale = null` or an omitted scale argument means “use `bcscale()`”.
An explicit `0` is scale zero, not “use global”.

`bcround` `$mode` is the same integer enumeration already used by `round()`
(`1..=8`, default `1` = `PHP_ROUND_HALF_UP` / `RoundingMode::HalfAwayFromZero`).
Elephc does not need the `RoundingMode` enum type; integers are enough, matching
the existing `round()` lowering.

`bcdivmod` returns a 2-element indexed array of strings `[quotient, remainder]`.

### Scale range

`0..=2147483647`. Any other integer, including negatives, is `\ValueError`.

### Well-formed numeric strings

After trimming ASCII whitespace:

- optional leading `+` or `-`
- zero or more digits
- optional `.`
- zero or more digits
- at least one digit must be present

Valid: `"0"`, `"00"`, `".5"`, `"5."`, `"+1.20"`, `"  -3.0  "`.
Invalid: `""`, `"+"`, `"-"`, `"."`, `"1e2"`, `"1.2.3"`, `"abc"`.

Scientific notation is never accepted.

### Arithmetic semantics

- `bcadd` / `bcsub` / `bcmul` / `bcdiv` / `bcmod` **truncate** to `$scale`.
  They do not round. Only `bcround` / `bcceil` / `bcfloor` round.
- Result strings honor `$scale` with trailing zeros (`bcadd('1.234', '5', 4)` →
  `"6.2340"`). Default scale `0` drops the fractional part
  (`bcadd('1.234', '5')` → `"6"`).
- Positive results have no `+` prefix. Zero is not `"-0"`.
- Leading zeros on the integer part are normalized (`"007"` → `"7"`), except the
  required single `0` before a fractional-only value when scale > 0
  (`"0.50"`).

Lock the exact string form against PHP fixtures. Do not invent a “prettier”
normalization.

### Errors (catchable)

| Condition | Exception |
|---|---|
| Not a well-formed BCMath string | `\ValueError` |
| Scale outside `0..=2147483647` | `\ValueError` |
| `bcdiv` / `bcmod` / `bcdivmod` divisor `0` | `\DivisionByZeroError` |
| `bcsqrt` of a negative number | `\ValueError` |
| `bcpow` exponent has a fractional part, or is out of the documented integer range | `\ValueError` |
| `bcpow('0', negative)` | `\DivisionByZeroError` (PHP 8.4+) |
| `bcpowmod` non-integral num/exponent/modulus, negative exponent, or modulus `0` | `\ValueError` or `\DivisionByZeroError` as in PHP 8.4 |
| `bcround` `$mode` outside `1..=8` | `\ValueError` |

PHP messages are copied verbatim from php-src (argument names, function prefix).
The crate owns a typed error plus `php_message()` so AOT and Magician cannot
drift.

### Effects

| Function | Effects |
|---|---|
| `bcscale` | `READS_PROCESS \| WRITES_PROCESS \| MAY_THROW` |
| `bccomp` | `READS_PROCESS \| MAY_THROW` |
| every other `bc*` | `READS_PROCESS \| ALLOC_HEAP \| MAY_THROW` |

Omitted-scale calls read process state, so they must not be treated as pure.
Even explicit-scale calls `MAY_THROW`, so unused calls must not be DCE’d.

### Ownership

- String results: `Fresh` (new persisted string, no alias of inputs).
- `bccomp` / `bcscale`: `NonHeap`.
- `bcdivmod`: `Fresh` indexed array of two fresh strings.

### `extension_loaded('bcmath')`

- AOT: true when the bridge is linked (any `bc*` use or `--with-bcmath`).
- Magician: add `"bcmath"` to eval’s `CORE_LOADED_EXTENSIONS` so
  `function_exists('bcadd')` and `extension_loaded('bcmath')` agree inside eval.

## Components

### Crate (`crates/elephc-bcmath`)

| File | Responsibility |
|---|---|
| `src/lib.rs` | C ABI, status codes, `elephc_bcmath_free` |
| `src/parse.rs` | well-formed scan + trim |
| `src/num.rs` | `BcNum` (sign, base-10 digits, scale) |
| `src/format.rs` | PHP result string (scale padding, sign, zero) |
| `src/ops.rs` | add, sub, mul, div, mod, comp, divmod |
| `src/pow.rs` | pow, powmod, sqrt |
| `src/round.rs` | ceil, floor, round (`1..=8` modes) |
| `src/scale.rs` | `AtomicI32` global scale |
| `src/error.rs` | typed error + PHP message text |
| `tests/php_fixtures.rs` | table-driven parity |
| `tests/gen_bcmath_fixtures.php` | optional generator run under host `php` |

### AOT

| File | Responsibility |
|---|---|
| `src/builtins/math/bcadd.rs` … `bcround.rs` | one `builtin!` home per name, `Area::Math` |
| `src/ir/runtime_fn.rs` | `BcAdd` … `BcRound`, effects, requirements, eir names, ownership |
| `src/codegen_support/bcmath.rs` | publish `_elephc_bcmath_*_fn` slots (hash pattern) |
| `src/codegen/lower_inst/builtins/bcmath.rs` | target-aware marshal + throw mapping |
| `src/codegen/lower_inst/runtime_functions/group_13.rs` | dispatch the 14 IDs |
| `src/codegen_support/runtime/bcmath/` | `__rt_bcadd` etc.: call slots, persist strings, throw |
| `src/linker/bridges.rs` | `BRIDGES` row |
| root `Cargo.toml` | workspace member + dev-dependency |

Registry declaration shape (all binary ops):

```rust
builtin! {
    name: "bcadd",
    area: Math,
    params: [
        num1: Str,
        num2: Str,
        scale: Int = DefaultSpec::Null
    ],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::BcAdd,
    ),
    summary: "Add two arbitrary-precision numbers.",
    php_manual: "https://www.php.net/manual/en/function.bcadd.php",
}
```

`bcscale` returns `Int`. `bccomp` returns `Int`. `bcdivmod` returns `Mixed`
(checker-facing array of two strings; a `check` hook may refine to
`PhpType::Array(Str)`). `bcceil` / `bcfloor` take one string. `bcround` takes
`num`, `precision = 0`, `mode = 1`.

### Magician

| File | Responsibility |
|---|---|
| `crates/elephc-magician/src/interpreter/builtins/math/bcadd.rs` … | `eval_builtin!` + coerce to string + crate ABI + map errors |
| `hooks/direct.rs` / `hooks/values.rs` | one shared `Bcmath` hook (not 14 hook variants) |
| `network_env/extension_loaded.rs` | add `"bcmath"` |
| `Cargo.toml` | `elephc-bcmath = { path = "../elephc-bcmath" }` |

PHP coercions (`int`/`float` → numeric string) happen in the Magician/AOT
wrappers, **before** the crate sees the bytes. The crate only accepts already
stringified operands, matching PHP’s `string $num` signature after Zend
coercion.

### Docs / example

- `docs/php/bcmath.md` (Astro frontmatter, no top-level `#` title)
- link from `docs/README.md` and a short pointer from `docs/php/math.md`
- `examples/bcmath/main.php` + `.gitignore` (`*.s`, `*.o`, `main`)
- generated builtin docs via `update-builtin-docs`
- README builtin list
- do **not** add a ROADMAP row (no planned item exists; implemented work is not
  added to the roadmap)

## C ABI (locked)

Status codes (stable, used by AOT and Magician):

```text
0  BCMATH_OK
1  BCMATH_ERR_MALFORMED
2  BCMATH_ERR_SCALE_RANGE
3  BCMATH_ERR_DIV_ZERO
4  BCMATH_ERR_SQRT_NEGATIVE
5  BCMATH_ERR_POW_FRACTIONAL
6  BCMATH_ERR_POW_RANGE
7  BCMATH_ERR_POWMOD
8  BCMATH_ERR_ROUND_MODE
```

`scale_is_null != 0` means “use global scale”. An explicit scale of `0` is
`scale=0, scale_is_null=0`.

String-result functions write a freshly allocated UTF-8 buffer and length.
Caller frees with `elephc_bcmath_free`. `bcdivmod` writes two buffers.
`bccomp` writes an `i32` (`-1` / `0` / `1`). `bcscale` get/set returns the
previous or current `i32` scale.

Function-pointer slots (AOT, hash pattern):

```text
elephc_bcmath_add            → _elephc_bcmath_add_fn
elephc_bcmath_sub            → _elephc_bcmath_sub_fn
… one slot per exported C entry, including get/set scale and free
```

Publishing happens at the call site so unused programs do not reference the
staticlib.

## Testing

1. **Crate unit tests** (`cargo test -p elephc-bcmath`): parse, ops, scale,
   rounding modes, error kinds. Seed table from PHP 8.4 manuals plus extra
   cases (whitespace, leading zeros, truncate-vs-round, negative zero,
   `bcpow('0','-1')`, `bcdivmod` sign table from the PHP manual).
2. **AOT codegen** (`tests/codegen/math/bcmath.rs`): compile_and_run stdout
   parity, named args, case-insensitive names, `bcscale` then omitted scale,
   `try/catch` for `\ValueError` and `\DivisionByZeroError`.
3. **AOT errors** (`tests/error_tests/math_builtins.rs`): arity diagnostics.
4. **Magician** (`crates/elephc-magician/.../tests/builtins_bcmath.rs`): same
   seeds through `execute_program`.
5. **Mixed AOT + eval scale** (codegen test with `eval()`).
6. **`extension_loaded('bcmath')`**: false in a program with no `bc*`; true
   when `bcadd` is used or `--with-bcmath` is passed.
7. **Docs**: `update-builtin-docs` workflow before the PR.

Do not run the full suite locally. Focused filters only.

## Implementation order

1. Crate engine + fixtures for parse / add / sub / mul / div / mod / comp / scale.
2. Crate pow / powmod / sqrt / ceil / floor / round / divmod.
3. Workspace + `BRIDGES` + `--with-bcmath`.
4. `RuntimeFnId` + registry homes.
5. AOT slots, `__rt_*` helpers, lowering, codegen + error tests.
6. Magician homes, hooks, tests, mixed scale test.
7. Docs, example, generated builtin pages, README.

Each step must leave `cargo build` warning-free and its own focused tests green.
The feature may land as more than one PR; the suite stays green at every step.
