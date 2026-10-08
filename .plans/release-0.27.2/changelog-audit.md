# Release changelog audit for 0.27.2

- [x] Verify the published base release and exact candidate.
- [x] Reconcile every integration and introduced commit.
- [x] Preserve the 16 previously approved bullets.
- [x] Apply the four additional user-approved bullets.

Range: `24f68a7b03d4db9660515ad17e711a385e0d596a..a65ce2f1bc18cdf0caad0d7bf5eed74dfa95241b`.
Published base: [v0.27.1](https://github.com/illegalstudio/elephc/releases/tag/v0.27.1).
Candidate: `a65ce2f1bc18cdf0caad0d7bf5eed74dfa95241b`, refreshed `main`.
Coverage: 56 merged PRs, 9 direct first-parent integrations, 212 commits; zero unresolved.
Decisions: 1 covered, 32 include, 32 omit.
Approval: 20 bullets applied under 0.27.2, date 2026-10-06. Unreleased remains empty.

## Features and improvements

- Added a runtime OPcache script cache for dynamic `include` and `require` through `eval()`, with optional persistent file caching, timestamp revalidation, admission limits, blacklists, preloading, live statistics, and API restrictions; ordinary compiled code remains ahead of time and no tracing JIT is provided.
- Added project INI directives in the `[ini]` table of `elephc.toml`, using the same settings as `--ini` and allowing command-line overrides.
- Improved scalar code generation by promoting eligible `int`, `bool`, and `float` locals into SSA values across branches and loops, and refining effects for resolved static calls.
- Added EIR integer range and induction-variable analysis to remove proven-safe overflow checks and simplify boxed numeric values while preserving PHP overflow-to-float behavior.
- Added compile-time generics for functions, classes, interfaces, methods, and traits, with inference, bounds, defaults, variance, typed arrays and callables, and PHPDoc template support; `--strict-php` accepts PHPDoc templates and rejects native generic syntax.

## Fixes

- Fixed serialization and unserialization of PHP classes that forbid it, with catchable exceptions, and omitted uninitialized typed properties from serialized property counts.
- Fixed `implode()` and `join()` on nullable, mixed, and runtime-promoted arrays, preserving actual element layouts, catchable type errors, and safe integer formatting near the string scratch-buffer limit.
- Fixed ownership of callable and reference entries in associative `array_chunk()` results, bound closures on Linux x86_64, and strings stored in static properties.
- Fixed `PDOException` subclass validation and reflected method ownership, isolated driver connection diagnostics per thread, and preserved SQLSRV constructor SQLSTATE and native error codes.
- Fixed qualified predefined names, named `::class` constants in defaults and attribute keys, nested array spread types, interface reflection order, strict-PHP callable guards, runtime `boolval()` callables, and the exception class and messages for invalid mixed throws.
- Fixed deeply nested compiler entry points, unreachable EIR continuations, reference-return callable metadata, and dynamic include/eval handling of templates, argument counts, scoped returns, and pending exceptions across `finally`.
- Improved diagnostics for unparenthesized `instanceof` call targets, missing class-constant names, disabled `class_alias()` autoload, and conflicting include-variant signatures.
- Fixed source identities for literal eval fragments so type and binding metadata cannot collide with equal coordinates in root or included files.
- Fixed heap leaks when eval writes back replacements through native typed by-reference variadic parameters.
- Fixed constant propagation and control-flow handling across switch fallthrough, side-effecting case labels, and default blocks that appear before later cases.
- Fixed `urlencode()` and `rawurlencode()` to leave ASCII digits unescaped.
- Fixed crashes when reading untyped null properties from `get_object_vars()` or an object-to-array cast.
- Fixed sparse array writes, typed-to-mixed array argument ownership, bare-array value builtins, mapped result types, `array_map(null, ...)`, associative filtering and sorting, internal-pointer values, and callable string-transform leaks.
- Fixed descriptor wrapper placement to keep generated functions contiguous, avoiding AArch64 branch-range assembly failures and broken ELF function extents under `--debug-info`.
- Fixed argument-unpack validation to reject positional entries after named entries within the same unpacked array.

## Coverage ledger

| Source | Shipped behavior | Decision | Bullets |
|---|---|---|---|
| [`d093426b8a`](https://github.com/illegalstudio/elephc/commit/d093426b8a1b0ca70d43c32d1be842fa82c4d523) | chore: bump version to 0.27.1 [skip ci] Release version automation only. | omit |  |
| [`49732894c6`](https://github.com/illegalstudio/elephc/commit/49732894c69e7d8daec2d027d22b0bf1c0b96edf) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #1665](https://github.com/illegalstudio/elephc/pull/1665) | test(regressions): pin #1565 and two shapes main already matches php on Regression coverage for behavior already present on the base. | omit |  |
| [PR #1514](https://github.com/illegalstudio/elephc/pull/1514) | fix(types): let classes extend PDOException like PHP | include | X4 |
| [PR #1510](https://github.com/illegalstudio/elephc/pull/1510) | fix(serialize): refuse not-serializable classes in both directions | include | X1 |
| [PR #1667](https://github.com/illegalstudio/elephc/pull/1667) | feat(cli): read INI directives from the project's `elephc.toml` | include | F2 |
| [`7ce73d028d`](https://github.com/illegalstudio/elephc/commit/7ce73d028d0f3f5218ff3622f43c4774808d79b6) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #1670](https://github.com/illegalstudio/elephc/pull/1670) | fix(eir): refine static effects and keep concat resets out of call summaries | include | F3 |
| [PR #1671](https://github.com/illegalstudio/elephc/pull/1671) | refactor(eir): move array_splice lowering out of the expression dispatcher Cohesive module extraction preserves existing array_splice behavior. | omit |  |
| [PR #1681](https://github.com/illegalstudio/elephc/pull/1681) | test(arrays): cover closure map results owned by literals Tests only for already-fixed array literal ownership. | omit |  |
| [PR #1682](https://github.com/illegalstudio/elephc/pull/1682) | fix(ci): verify and retry missing nextest artifacts CI artifact validation and retry only. | omit |  |
| [PR #1684](https://github.com/illegalstudio/elephc/pull/1684) | fix(optimizer): seed SPL hierarchy for AST-only exception analysis AST-only analysis API hierarchy correction; normal compilation already supplies checked metadata. | omit |  |
| [PR #1685](https://github.com/illegalstudio/elephc/pull/1685) | fix(stack): budget target folding and autoload reference scans | include | X6 |
| [PR #1686](https://github.com/illegalstudio/elephc/pull/1686) | fix(codegen): preserve reference-return fallback metadata | include | X6 |
| [PR #1687](https://github.com/illegalstudio/elephc/pull/1687) | fix(pdo): isolate driver open diagnostics per thread | include | X4 |
| [PR #1691](https://github.com/illegalstudio/elephc/pull/1691) | test(reflection): pin static abstractness after body trimming Emitter test only for existing reflection behavior. | omit |  |
| [PR #1692](https://github.com/illegalstudio/elephc/pull/1692) | test(locals): execute guarded and divergent mixed stores Executes additional existing mixed-storage fixture branches. | omit |  |
| [PR #1695](https://github.com/illegalstudio/elephc/pull/1695) | fix(types): scope PDO SQLSTATE exemption to Throwable contract | include | X4 |
| [PR #1697](https://github.com/illegalstudio/elephc/pull/1697) | test: share retype fixtures between checker and codegen Shares test fixtures without changing production behavior. | omit |  |
| [PR #1705](https://github.com/illegalstudio/elephc/pull/1705) | test(iconv): verify Apple discard mode without fallback skipping Apple iconv test only. | omit |  |
| [PR #1706](https://github.com/illegalstudio/elephc/pull/1706) | test(mbstring): split slow ownership fixtures by backend Splits test fixtures to stay within time limits. | omit |  |
| [PR #1707](https://github.com/illegalstudio/elephc/pull/1707) | fix(runtime): pass x86 retain pointers in the helper input register | include | X3 |
| [PR #1708](https://github.com/illegalstudio/elephc/pull/1708) | test(types): cover documented null-coalesce result contracts Checker test coverage only. | omit |  |
| [PR #1745](https://github.com/illegalstudio/elephc/pull/1745) | CI: pin Rust toolchain to 1.98 Pins CI and packaging toolchain; no PHP-visible change. | omit |  |
| [`293c02545c`](https://github.com/illegalstudio/elephc/commit/293c02545c2e348664366652134a8b44ca918a73) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #1567](https://github.com/illegalstudio/elephc/pull/1567) | feat: promote scalar php locals to SSA | include | F3 |
| [PR #1672](https://github.com/illegalstudio/elephc/pull/1672) | fix(types): preserve nested element types in mixed array spreads | include | X5 |
| [PR #1677](https://github.com/illegalstudio/elephc/pull/1677) | fix(parser): parse qualified predefined constant and class names | include | X5 |
| [PR #1680](https://github.com/illegalstudio/elephc/pull/1680) | docs(generators): clarify named and closure metadata paths Corrects generator metadata comments and documentation only. | omit |  |
| [PR #1744](https://github.com/illegalstudio/elephc/pull/1744) | fix: keep INI identity allocation warning-free across Rust toolchains Equivalent atomic implementation avoids a toolchain warning. | omit |  |
| [`198d0b79db`](https://github.com/illegalstudio/elephc/commit/198d0b79dbbef3a0e0422afdb4c157d34207f0ad) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #968](https://github.com/illegalstudio/elephc/pull/968) | feat(opcache): give the OPcache surface real behaviour | include | F1, X2, X3, X6 |
| [PR #1702](https://github.com/illegalstudio/elephc/pull/1702) | Stop adding unplanned work to the roadmap Repository roadmap management policy only. | omit |  |
| [PR #1054](https://github.com/illegalstudio/elephc/pull/1054) | fix(strings): implode() reads its array's real element layout, not an assumed one | include | X2 |
| [PR #1673](https://github.com/illegalstudio/elephc/pull/1673) | fix(reflection): list interface methods in PHP declaration order | include | X5 |
| [`127947c385`](https://github.com/illegalstudio/elephc/commit/127947c3859d306a3cfb96cc7885d0cc525a5e19) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [`703528c563`](https://github.com/illegalstudio/elephc/commit/703528c563bde7d6ecb66351334339d5c078a4c0) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #1767](https://github.com/illegalstudio/elephc/pull/1767) | ci: move macOS jobs from macos-14 to macos-15 runners CI runner migration only. | omit |  |
| [PR #1679](https://github.com/illegalstudio/elephc/pull/1679) | fix(strict-php): honor hidden extensions in guards and callables | include | X5 |
| [PR #1688](https://github.com/illegalstudio/elephc/pull/1688) | docs(tests): align retype and array-growth fixture comments Corrects stale test comments only. | omit |  |
| [PR #1693](https://github.com/illegalstudio/elephc/pull/1693) | test(arrays): cover static and ternary spread ownership Adds spread ownership coverage and splits MIME tests. | omit |  |
| [PR #1704](https://github.com/illegalstudio/elephc/pull/1704) | test(math): cover mt_rand and rand mixed bounds Tests existing random-bound coercion behavior. | omit |  |
| [PR #1709](https://github.com/illegalstudio/elephc/pull/1709) | fix: fold named class constants before schema construction | include | X5 |
| [PR #1711](https://github.com/illegalstudio/elephc/pull/1711) | fix: support object boolval runtime callables | include | X5 |
| [PR #1713](https://github.com/illegalstudio/elephc/pull/1713) | fix: distinguish invalid mixed throw errors | include | X5 |
| [PR #1770](https://github.com/illegalstudio/elephc/pull/1770) | docs(types): note the by-ref variadic limitation for a ref-aliased local Documents an existing by-reference variadic limitation. | omit |  |
| [`13c15d5f43`](https://github.com/illegalstudio/elephc/commit/13c15d5f43be4fc3c3b936ccf44ec784dc69c61d) | chore: update repository stats [skip ci] Generated repository traffic statistics only. | omit |  |
| [PR #1749](https://github.com/illegalstudio/elephc/pull/1749) | fix(pdo): publish the SQLSRV CLI diagnostic on constructor failure | include | X4 |
| [PR #1752](https://github.com/illegalstudio/elephc/pull/1752) | fix(arrays): array_chunk owns callable and reference elements | include | X3 |
| [PR #1669](https://github.com/illegalstudio/elephc/pull/1669) | fix(eir): validate uses across unreachable continuations | include | X6 |
| [PR #1644](https://github.com/illegalstudio/elephc/pull/1644) | Range proofs specialize safe arithmetic and loop induction values, with conservative consumer and null-representation guards. | include | F4 |
| [PR #1750](https://github.com/illegalstudio/elephc/pull/1750) | DBLIB pre-handle connection diagnostics are thread-local and preserve SQLSTATE and native error codes. The approved PDO bullet already covers per-thread driver connection diagnostics. | covered | X4 |
| [PR #1683](https://github.com/illegalstudio/elephc/pull/1683) | Static method instanceof targets receive a parentheses hint; missing constant-list names report the offending token. | include | X7 |
| [PR #1689](https://github.com/illegalstudio/elephc/pull/1689) | Corrects array documentation for already supported indexed unset through references and preserved next integer keys. Documentation clarification of existing behavior; no runtime change. | omit |  |
| [`afbb2af85c`](https://github.com/illegalstudio/elephc/commit/afbb2af85c0004bd72e08cfbb30e312e8333240e) | chore: update repository stats [skip ci] Repository traffic statistics only. | omit |  |
| [PR #1759](https://github.com/illegalstudio/elephc/pull/1759) | Literal AOT eval fragments use a separate source identity counter that survives include-resolution resets. | include | X8 |
| [PR #1769](https://github.com/illegalstudio/elephc/pull/1769) | Ref-marker writeback releases overwritten concrete string, array, hash, and object occupants in both architecture emitters. | include | X9 |
| [PR #1690](https://github.com/illegalstudio/elephc/pull/1690) | Documents incomplete compiled-enum interface metadata across eval and the distinct eval-declared enum behavior. Documents a remaining limitation; does not implement interface parity. | omit |  |
| [PR #1714](https://github.com/illegalstudio/elephc/pull/1714) | Eligible class_alias source forms diagnose explicitly false autoload literals before constant folding. | include | X7 |
| [PR #1632](https://github.com/illegalstudio/elephc/pull/1632) | Switch propagation invalidates earlier label and fallthrough writes and preserves default ordering and structured exits. | include | X10 |
| [PR #1661](https://github.com/illegalstudio/elephc/pull/1661) | The capture_hash test fixture quarantines freed allocation addresses until each case finishes. Test-fixture reliability fix; shipped runtime behavior is unchanged. | omit |  |
| [PR #1778](https://github.com/illegalstudio/elephc/pull/1778) | Both URL encoder emitters classify ASCII digits before the alphabetic lower-bound rejection. | include | X11 |
| [PR #1779](https://github.com/illegalstudio/elephc/pull/1779) | Object property projection converts a Mixed null slot to canonical PHP null before boxing. | include | X12 |
| [PR #1712](https://github.com/illegalstudio/elephc/pull/1712) | Declared include-variant contracts are validated before recursive provisional signatures and errors point to conflicting declarations. | include | X7 |
| [PR #1221](https://github.com/illegalstudio/elephc/pull/1221) | Compile-time generics and typed array storage, including native and PHPDoc templates; accompanying array layout, ownership, callable thunk and argument-unpack correctness fixes. The new feature and fixes are not covered by the existing 16 approved bullets. | include | F5, X13, X14, X15 |

## Source evidence and open decisions

No open changelog decisions remain. Every integration has a decision in [ledger.json](ledger.json), including per-path evidence and the 22 introduced commits from PR #1221. The bundled inventory is in [inventory.json](inventory.json). Its raw log and earlier refresh snapshots are retained locally as historical evidence.

Tests were inspected as source only. None was executed.

The audit is pinned to the source candidate above. The four separately authorized
local preparation commits record comments, diagnostic fixtures, documentation,
and the approved release notes; they introduce no additional shipped feature
requiring a new changelog bullet. Validation results are in [validation.json](validation.json).
