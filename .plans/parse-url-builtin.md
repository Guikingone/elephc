# Plan: `parse_url()` builtin (AOT + magician)

## Goal

Ship PHP-compatible `parse_url()` on **both** compilation paths:

1. **AOT** — registry builtin → EIR `RuntimeFnId` → target-aware lowering → `__rt_parse_url*` runtime helpers for every supported target (`macos-aarch64`, `linux-aarch64`, `linux-x86_64`).
2. **Magician** — `eval_builtin!` home file + pure-Rust PHP-parity implementation used by `eval()` / the eval bridge.

Also ship the matching predefined constants:

| Constant | Value |
|---|---|
| `PHP_URL_SCHEME` | `0` |
| `PHP_URL_HOST` | `1` |
| `PHP_URL_PORT` | `2` |
| `PHP_URL_USER` | `3` |
| `PHP_URL_PASS` | `4` |
| `PHP_URL_PATH` | `5` |
| `PHP_URL_QUERY` | `6` |
| `PHP_URL_FRAGMENT` | `7` |

## PHP contract (authoritative)

Signature (PHP 8.x):

```php
function parse_url(string $url, int $component = -1): mixed
```

### Return shapes

| Call form | Success | Missing component | Invalid URL |
|---|---|---|---|
| `parse_url($url)` / `parse_url($url, -1)` | associative array of present components | key **omitted** | `false` |
| `parse_url($url, PHP_URL_PORT)` | `int` | `null` | `false` |
| `parse_url($url, PHP_URL_{SCHEME,HOST,USER,PASS,PATH,QUERY,FRAGMENT})` | `string` (may be `""`) | `null` | `false` |

Array keys (string), in PHP insertion order when present:

`scheme`, `host`, `port`, `user`, `pass`, `path`, `query`, `fragment`

- `port` is always **int** in the array form.
- All other present keys are **string** (including empty user `""` for `http://:pass@host`).
- IPv6 hosts keep brackets: `http://[::1]:8080/` → `host` = `"[::1]"`.

### Component validation

- Valid `$component`: any negative integer (full-array mode) or `0..=7`.
- Any integer greater than `7` throws catchable `\ValueError`:
  `parse_url(): Argument #2 ($component) must be a valid URL component identifier, N given`
- Constants are **not** bitmasks (unlike `PATHINFO_*`). `PHP_URL_SCHEME | PHP_URL_HOST` happens to equal `1` (`PHP_URL_HOST`) and is therefore accepted as host.

### Representative PHP 8.4 behaviors (fixture seeds)

Cross-check every case with `php -r` during implementation; the suite must pin at least:

```text
"https://user:pass@example.com:8080/path/to?query=1&x=2#frag"
  → full array with all 8 keys; port int 8080

"http://example.com" / PHP_URL_PORT → null
"http://example.com:80" / PHP_URL_PORT → 80
"http://" → false (all modes)
"//example.com/path" → {host, path}  (scheme-relative)
"/just/path?q=1#f" → {path, query, fragment}
"?query=1" → {query}
"#frag" → {fragment}
"" → {path: ""}
"http://example.com:-1" → false
"http://example.com:99999" → false   (port > 65535)
"http://example.com:0" → port 0 present
"http://[::1]:8080/path" → host "[::1]", port 8080
"http://:pass@host" → user "", pass "pass"
"http://example.com#" / PHP_URL_FRAGMENT → ""
"http://example.com/path?q" / PHP_URL_QUERY → "q"
"////example.com" → false
"http:///example.com" → false
```

Algorithm note: do **not** use a general-purpose URI crate (`url`, `urllib`, …) as the source of truth. PHP’s parser is intentionally quirky. Implement a dedicated PHP-compatible scanner (mirror `php-src` `ext/standard/url.c` behavior for the supported PHP version surface) and lock it with the fixture table.

## Non-goals

- `http_build_query`, `parse_str`, stream wrappers, or full URI normalization.
- Query-string decoding inside `parse_url` (PHP leaves `query` raw).
- IDNA / non-ASCII host conversion.
- Making magician depend on the main `elephc` crate (or vice versa) just to share types. Shared **fixtures** and a documented algorithm are enough; optional later extraction of a pure-Rust helper crate is out of scope for the first landing unless dual implementation starts to diverge badly.

## Area / placement

| Path | Location | Why |
|---|---|---|
| AOT registry | `src/builtins/string/parse_url.rs` | Sits with `urlencode` / `urldecode` / `rawurl*` |
| Magician | `crates/elephc-magician/src/interpreter/builtins/string/parse_url.rs` | Same family |
| Runtime AOT | `src/codegen_support/runtime/strings/parse_url*.rs` | Assembly emitters for `__rt_parse_url*` |
| Lowering | `src/codegen/lower_inst/builtins/string.rs` (or a focused sibling) + `RuntimeFnId` dispatch group | Typed EIR target |
| Constants | same places as `PATHINFO_*` | See phase 1 |

## Design

### AOT registry declaration

```rust
builtin! {
    name: "parse_url",
    area: String,
    params: [url: Str, component: Int = DefaultSpec::Int(-1)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ParseUrl,
    ),
    summary: "Parses a URL and returns its components.",
    php_manual: "function.parse-url",
}
```

**`check` hook** (pattern: `pathinfo` + `realpath`):

1. Optional second arg must be `Int` when present; diagnostic: `parse_url() component must be int`.
2. Resolve static component via a private helper (int literal / `PHP_URL_*` / unary `-` on literals). No bitmask combinators required beyond what constant folding already produces.
3. Refined return types:

| Static component | Return type |
|---|---|
| absent / `-1` | `Union(AssocArray{Str, Mixed}, False)` |
| `PHP_URL_PORT` (`2`) | `Union(Int, Null, False)` |
| other `0..=7` | `Union(Str, Null, False)` |
| unknown dynamic | `Mixed` (or the full union of all arms — prefer `Mixed` for simplicity and match `json_decode`) |

`AssocArray{Str, Mixed}` is required because `port` is int while other values are strings.

**Semantics / ownership / effects**

- Effects: pure (no I/O, no globals) — whatever `runtime_fn_semantics` already assigns; confirm `RuntimeFnId::ParseUrl` is listed as pure if the table is hand-maintained.
- Result ownership: **`Fresh`** (owned hash and/or owned Mixed cell; never alias `$url`).
- Add `RuntimeFnId::ParseUrl` to the `Fresh` ownership bucket in `src/ir/runtime_fn.rs` (same rationale as `Pathinfo`/`Explode`/`Strstr`).

### Runtime shape (AOT)

Prefer **one primary Mixed-returning helper** plus thin specializations if lowering can prove the shape:

| Helper | Contract |
|---|---|
| `__rt_parse_url` | `(url_ptr, url_len, component:i64) → Mixed*` |
| (optional) `__rt_parse_url_array` | array-only path when component is statically `-1` and result type is assoc |
| (optional) component helpers | only if they shrink hot paths; not required for v1 |

**Mixed cell results** (must match existing boxed Mixed contract):

- Success array → owned hash pointer with Mixed-tagged values (`port` as int tag, strings as string tags), boxed as Mixed array/hash.
- Success string → Mixed string (or unboxed `Str` when checker refined to `Str|Null|False` and lowering selects a non-Mixed path).
- Success int port → Mixed int / unboxed int.
- Missing component → Mixed null / unboxed null.
- Invalid URL → Mixed false / unboxed false.
- Invalid component → throw `\ValueError` via the existing exception path (`__rt_throw_current` + ValueError class id), same style as hash/mb_strlen ValueErrors.

v1 recommendation: **always lower through Mixed**, then let store/result typing unbox when the call-site type is a refined union. This mirrors `json_decode` and avoids a combinatorial explosion of helpers. If refined non-Mixed returns are easy (static component), add them as a second step.

Both **AArch64** and **x86_64** emitters are mandatory in the same change.

### Magician

```rust
eval_builtin! {
    name: "parse_url",
    area: String,
    params: [url, component = EvalBuiltinDefaultValue::Int(-1)],
    direct: ...,
    values: ...,
}
```

Implementation sketch:

1. Coerce/read `$url` as bytes (PHP string).
2. Default `component = -1` when omitted.
3. If component > `7` → eval `ValueError` (or the project’s established fatal/exception mapping for ValueError in eval); any negative component selects the array form, matching PHP 8.4.
4. Run the pure-Rust PHP URL parser → `Option<ParsedUrl>`.
5. If `None` → return `false`.
6. If `component < 0` → build assoc array, inserting only present keys (`port` as int cell).
7. Else extract the requested component or `null`.

Register the module in `string/mod.rs` (AOT + magician). No hand tables for names/signatures — registry only.

### Constants wiring (mirror `PATHINFO_*`)

Touch every path that already knows `PATHINFO_*`:

| Surface | File(s) |
|---|---|
| Checker constant types | `src/types/checker/driver/init.rs` |
| Prescan values | `src/codegen_support/prescan.rs` |
| Name-resolver known names | `src/name_resolver/names.rs` |
| Autoload symbolic interpreter (if still relevant) | `src/autoload/interpret.rs` |
| Magician predefined constants | `crates/elephc-magician/src/interpreter/constants.rs` + `constant_eval.rs` |
| Docs / README constant lists | `docs/php/*`, `README.md` |
| Examples that exercise `defined('PHP_URL_…')` | eval example / new example |

### Algorithm ownership

To keep AOT and magician from drifting:

1. Put a **fixture table** (inputs → expected JSON/PHP dumps) in one place used by both test suites, e.g. `tests/fixtures/parse_url_cases.json` **or** duplicated tables with a comment “keep in sync”, preferring a single shared fixture file if the test harness can load it from both crates.
2. Implement magician first against that table (fast iteration in pure Rust).
3. Port the same decision tree into AOT assembly (or a small set of runtime helpers) and run the **same** table through `compile_and_run` codegen tests.
4. Document non-obvious branches in the runtime/module preambles (empty user, port range, scheme-relative URLs, false vs empty path).

Optional later refactor (not required for landing): extract `php_parse_url(&[u8]) -> Option<Parts>` into a tiny workspace rlib used by magician and linked into AOT via a bridge. That is heavier (linker/BRIDGES entry) and only justified if assembly parity becomes unmaintainable.

## Implementation phases

### Phase 0 — Spec & fixtures (no production code)

- [x] Freeze the fixture corpus from PHP 8.x (`php -r` / `ELEPHC_PHP_CHECK` later).
- [x] Document ValueError message text exactly.
- [x] Confirm target matrix obligations (all three targets same PR).

### Phase 1 — Constants

- [x] Add `PHP_URL_*` to checker init, prescan, name resolver, magician constant tables, README/docs lists.
- [x] Focused tests: `defined('PHP_URL_HOST')`, value equality, use in both AOT and `eval()`.

### Phase 2 — Magician builtin

- [x] `eval_builtin!` home file + pure-Rust parser.
- [x] Direct + values dispatch paths.
- [x] Unit/integration tests under magician and/or `tests/codegen/eval.rs` (pattern: `test_eval_dispatches_pathinfo_builtin_call`).
- [x] `function_exists('parse_url')`, named args, `call_user_func`.

### Phase 3 — AOT registry + EIR wiring

- [x] `src/builtins/string/parse_url.rs` + `mod.rs` export.
- [x] `RuntimeFnId::ParseUrl` (+ `as_eir`, effects, ownership, any exhaustive matches).
- [x] Dispatch arm in the appropriate `runtime_functions/group_*.rs`.
- [x] Lowering function (Mixed-first).
- [x] Checker refinement tests / error tests for non-int component.

### Phase 4 — AOT runtime helpers

- [x] Emit `__rt_parse_url` (AArch64 + x86_64) under `src/codegen_support/runtime/strings/`.
- [x] Register in `strings/mod.rs` + `emitters.rs`.
- [x] Runtime data: string key literals (`scheme`, `host`, …), ValueError message bytes if needed.
- [x] Hash construction with Mixed values; false/null Mixed boxing; string persistence.
- [x] Ownership/GC: release only owned intermediates; Fresh result.

### Phase 5 — Tests (quality gate)

Minimum coverage (codegen + magician + errors):

| Area | Cases |
|---|---|
| Full array | full URL, scheme-relative, path-only, query-only, fragment-only, empty string |
| Components | each `PHP_URL_*` present + missing → null |
| Failure | `false` for invalid URLs; port out of range |
| ValueError | components `8`, `99`; negative values remain array selectors |
| Types | `port` is int in array; empty user string |
| IPv6 | bracketed host + port |
| Defaults | omitted component ≡ `-1` |
| Named args | `parse_url(url: ..., component: PHP_URL_HOST)` |
| Case / namespace | `Parse_Url(...)`, `\parse_url(...)` |
| Callables | `function_exists`, `call_user_func` |
| Eval | same matrix under `eval('...')` |
| Errors | non-int component at compile time |

Suggested files:

- `tests/codegen/strings/parse_url.rs` (new)
- `tests/codegen/eval.rs` (eval dispatch section)
- `tests/error_tests/string_builtins.rs` (or dedicated file)
- magician unit tests if parser helpers are pure and unit-testable

Run focused:

```bash
cargo build
cargo test --test codegen_tests parse_url
cargo test --test error_tests parse_url
cargo test -p elephc-magician parse_url
# optional parity:
ELEPHC_PHP_CHECK=1 cargo test --test codegen_tests parse_url
git diff --check
```

Do **not** run the full suite locally unless requested; CI owns the matrix.

### Phase 6 — Docs, example, changelog, builtins registry

- [x] User docs: `docs/php/strings.md` table row + short semantics note (array vs component, constants, false/null).
- [x] Generated builtins: run `update-builtin-docs` skill / CI sequence (`gen_builtins` → `extract_builtins.py --render --force` → audits). Commit `docs/php/builtins/**`, `docs/internals/builtins/**`, `scripts/docs/builtin_registry.json`.
- [x] Internals: brief mention in `docs/internals/the-runtime.md` for `__rt_parse_url*` if that page lists helpers.
- [x] Example: extend `examples/string-ops` or add `examples/parse-url/main.php` (+ `.gitignore` with `*.s`, `*.o`, `main`).
- [x] `CHANGELOG.md` under `[Unreleased]`: one user-facing bullet (`parse_url()` + `PHP_URL_*`).
- [x] `ROADMAP.md`: only mark `[x]` if an existing open item matches; otherwise leave untouched.
- [x] README constant/builtin lists if they enumerate URL helpers / predefined constants.

### Phase 7 — Parity gates

- [x] `src/builtins/parity_tests.rs` stays green (registry-driven; no manual name list edits unless an exception list exists).
- [x] Magician ↔ AOT builtin name parity tests still pass.

## Key file checklist

### AOT

- `src/builtins/string/parse_url.rs` (**new**)
- `src/builtins/string/mod.rs`
- `src/ir/runtime_fn.rs` (`ParseUrl`, ownership, name, effects if needed)
- `src/codegen/lower_inst/runtime_functions/group_*.rs`
- `src/codegen/lower_inst/builtins/...` (new lowerer)
- `src/codegen_support/runtime/strings/parse_url*.rs` (**new**)
- `src/codegen_support/runtime/strings/mod.rs`
- `src/codegen_support/runtime/emitters.rs`
- `src/codegen_support/runtime/data/fixed.rs` (keys / messages if required)
- `src/types/checker/driver/init.rs`
- `src/codegen_support/prescan.rs`
- `src/name_resolver/names.rs`

### Magician

- `crates/elephc-magician/src/interpreter/builtins/string/parse_url.rs` (**new**)
- `crates/elephc-magician/src/interpreter/builtins/string/mod.rs`
- `crates/elephc-magician/src/interpreter/constants.rs`
- `crates/elephc-magician/src/interpreter/constant_eval.rs`

### Docs / project meta

- `docs/php/strings.md`
- generated builtins docs + `scripts/docs/builtin_registry.json`
- `examples/...`
- `CHANGELOG.md`
- `README.md` (constants list)

## Risks & decisions

| Risk | Mitigation |
|---|---|
| PHP parser edge cases diverge from general URI libs | Hand-written PHP-compatible scanner + PHP-derived fixtures; no crate as oracle |
| Dual AOT/magician implementations drift | Shared fixture file; implement magician first; port decisions 1:1 |
| Heterogeneous array (`port` int) | Always Mixed-valued hash; checker uses `AssocArray{Str, Mixed}` |
| ValueError for bad component | Reuse existing ValueError throw path in AOT + eval exception mapping |
| Large assembly helper | Keep one Mixed entrypoint; split only if file-size/cohesion policy requires (`strings/parse_url.rs` leaf may exceed 500 LoC if mono-feature) |
| Target coverage | Emit both arches in the same PR; focused local macOS tests + CI for Linux |

## Suggested PR split (optional)

1. **Constants + fixtures + magician** — useful standalone for `eval()`.
2. **AOT registry + runtime + codegen tests + docs** — completes the feature.

Prefer a **single PR** if the runtime helper stays manageable; split only if review size becomes painful.

## Done criteria

- [x] `parse_url` exists in AOT registry and magician registry.
- [x] `PHP_URL_*` constants resolve in AOT and eval.
- [x] Fixture corpus green on AOT codegen and magician/eval.
- [x] Invalid component raises ValueError (AOT + eval).
- [x] All three supported targets handled in runtime/lowering.
- [x] Docs, example, CHANGELOG, generated builtin docs committed.
- [x] Focused tests + `git diff --check` clean; no new compiler warnings.

## Implementation order (recommended)

1. Fixtures + constant plumbing.
2. Magician pure-Rust parser + eval tests.
3. AOT `builtin!` + checker refinement + error tests.
4. `RuntimeFnId` + Mixed lowering skeleton (can fail/link missing symbol briefly in WIP).
5. Assembly runtime both arches + codegen fixture tests.
6. Docs / example / changelog / `update-builtin-docs`.
7. Final focused verification commands above.
