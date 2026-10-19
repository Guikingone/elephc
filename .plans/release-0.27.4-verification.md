# Release 0.27.4 verification

- [x] Verify the published base, exact candidate and complete source inventory.
- [x] Run static release checks and a warning-free build without running tests.
- [x] Correct documentation and regenerate the builtin references.
- [ ] Present the audited changelog draft for approval.
- [ ] Apply and commit the approved numbered release notes.

## Scope and evidence

Date: 2026-10-19. Checkout: `release/0.27.4`, initially clean.
Published base: `v0.27.3`, commit `132b4da6f74da2da0956bc25026a9d5570f49689`.
Candidate: `df57a9800f4fb61bd4e876f0ddf309b9d08109f2`, equal to refreshed
`origin/main` when the audit started. Local release-preparation commits are
tracked separately and do not expand that shipped-code range.

GitHub's latest stable release endpoint and `gh release list` both identify
`v0.27.3`. The base is an ancestor of the candidate. The requested fetch updated
main but refused to replace the moving local `nightly` tag; the stable base and
candidate were verified independently without replacing that tag.

The bundled release inventory reconciled 37 merged PRs and 5 direct commits,
covering all 301 commits with zero errors and zero unaccounted commits. All PRs
target main and their merge commits are reachable from the candidate. The
semantic ledger and proposed notes are recorded separately before approval.

The user requested verification, documentation updates, then final changelog
preparation, with progressive commits and no test execution. The changelog
inventory prerequisite was performed first; final changelog approval and writing
remain the last phase. Package versions, tags and publishing are outside scope.

## Pre-release verification report

### 0. Release changelog

Status: PENDING APPROVAL. Source reconciliation passes: 37 PRs, 5 direct
commits, 301 total commits, 0 unresolved. `[Unreleased]` is empty. The numbered
0.27.4 section is not written before approval.

### 1. README

Status: PASS for builtin counts, supported constructs, constants, type model,
targets and current backend paths. Its generated reference contains 1,068
public entries, matching the 1,194-contract registry after excluding 126
internal entries. Dedicated references cover all supported builtin signatures.
The project structure describes the active EIR and shared runtime boundaries.

### 2. Language reference and internals

Status: PASS after documentation updates. The initial findings corrected were:

- Both the pipeline reference and architecture diagram omit the new
  `dom-html-prelude` and `dom-html-autoload-prelude` timing phases.
- Architecture runtime counts are stale for arrays, strings, system helpers,
  exceptions, objects and runtime data. The module map omits the DOM prelude.
- The parser's supporting-type table omits `ClassProperty.set_visibility`,
  `ClassProperty.is_promoted` and `ClassConst.type_expr`.
- The optimization reference describes fresh checked-integer assignment
  transfer but omits the dynamic numeric add/sub/mul extension.

All 2,356 tracked Markdown pages have the required frontmatter, no duplicate
top-level body title, and no missing relative link targets outside code. The
only nonexistent explicit source path detected, `src/deep/thing.php` in the
profiling guide, is an illustrative filename, not a broken source reference.
Every literal Token, ExprKind and StmtKind variant is mentioned in its internals
reference: 166, 62 and 48 variants respectively. The operator reference documents
the spellings of all 25 BinOp variants; enum names are not its presentation API.
The PhpType reference matches all 19 variants, including internal TaggedScalar.
CLI flags and the three parsed environment overrides are documented. The old
`--instrument` and `--probe` strings occur only in a source comment, not in the
accepted option parser. Defaults and aliases are checked against current CLI
parsing and target selection.

Generated references were rebuilt with `cargo build --example gen_builtins
--features curl -j 4`, `extract_builtins.py --render --force`,
`gen_module_sections.py` and `gen_php_comparison.py`. The committed generated
pages and registries were already reproducible, so no generated changes were
needed. `audit_builtins.py`, `validate_site_compat.py` and
`audit_builtin_eir_boundary.py --enforce-target-architecture` all pass with zero
errors. The exporter build also completed without warnings.

The runtime reference now names all 1,004 literal global `__rt_*` labels found
in shared emitters, including 20 previously omitted helpers. The memory model
covers all 234 literal BSS names from the data emitters, including the weak
reference identity index and its size. These inventories check explicit symbol
names; they do not claim to exhaust dynamically formatted metadata labels.

### 3. Roadmap

Status: PASS after documentation updates. Existing completed v0.27.x items are still
under a `Planned` heading before this update, and the introductory claim that v0.27.x contains only
unshipped work contradicted those items. They now remain in v0.27.x under
their delivered release. Tail self/mutual recursion is implemented by
`src/ir_passes/tail_call.rs`; its existing item is already checked. The remaining
compute-performance, DOOM benchmark and real-world CLI validation goals cannot
be marked complete from this static audit. No benchmark or showcase was run.

### 4. Test coverage

Status: PARTIAL. Static inventory finds 16,236 literal `#[test]` declarations in
852 integration Rust files. This is a source count, not a harness or pass count;
macros can generate additional cases. Per-file counts and candidate gaps are in
[the static inventory](release-0.27.4-static-inventory.json).

The shipped feature and fix PRs contain regression coverage, including BCMath
Number, Core symbols, DOM HTML, JSON enums, append writes, asymmetric property
visibility, static receivers and extern bool returns. Lexer/parser coverage
exists for the affected syntax. No test was executed in this task.

The broader builtin scan found 115 public names without an explicit mention
anywhere in integration Rust/PHP/Python/shell sources. These are concentrated in
Cairo/GD, mysqli procedural wrappers, session aliases and XMLWriter wrappers.
The inventory lists every name. Add direct wrapper fixtures or document a
concrete generated fixture mapping before claiming exhaustive per-builtin
coverage. Method-level and bridge-unit coverage alone does not establish a
direct PHP wrapper test. Error-only scans also contain candidates that require
manual classification because parameterized fixtures, runtime-error assertions,
name-resolver rewrites and prelude methods can share validation. Those raw
candidates are not asserted to be 360 proven missing error tests.

### 5. Examples

Status: PASS for major feature categories. There are 249 example directories.
Types, loops, recursion, strings, casts, math, includes, FFI, objects, eval,
generics, BCMath Number, Core symbols, DOM HTML and tail recursion have source
examples. Their execution is skipped by request.

### 6. Code style

Status: PASS for current source gates. All 5,562 tracked Rust files start with
complete Purpose / Called from / Key details module preambles. The repository
assembly-comment checker reports zero missing comments and zero misaligned
comments across `src/codegen/` and `src/codegen_support/`, including multiline
calls. Reviewed runtime groups retain intent headings. Cohesive large leaf
files are not violations solely because of line count.

The audited history contains 18 existing Co-authored-by trailers. This is a
historical convention deviation, not a reason to rewrite published ancestry.
Release-preparation commits must contain no such trailers.

### 7. Build and test suite

Build: PASS. `cargo build -j 4` completed successfully in 4m 15s, with zero
warnings. This builds the default workspace members without executing tests.

Tests: SKIPPED BY USER REQUEST. No Cargo test, Nextest, Docker target suite,
Python unit test, PHP behavioral probe or example execution was invoked.
Cross-target execution and regression freedom are therefore not newly verified.

## Release readiness

Release ready: NOT YET ESTABLISHED.

1. Documentation updates and non-test audits are complete.
2. Approve, apply and commit the numbered changelog with complete traceability.
3. Resolve or explicitly accept the direct-wrapper coverage gaps before claiming
   exhaustive coverage; retain the distinction between static presence and
   executed verification. This task intentionally provides no new test result.
