# Pre-release verification report for 0.27.2

- [x] Rebase main and release/0.27.2 onto the refreshed main candidate.
- [x] Restore release preparation and resolve the one docblock conflict.
- [x] Reconcile all release sources and apply all approved changelog additions.
- [x] Refresh affected documentation, generated references, and style corrections.
- [x] Complete non-test validation and remove the preparation stash.
- [x] Record the preparation in four local commits without publishing the branch.

Checkout: `/home/nahime/Developer/.worktrees/illegalstudio/elephc/release-0-27-2`.
Branch: `release/0.27.2`. Candidate: `a65ce2f1bc18cdf0caad0d7bf5eed74dfa95241b`.
The clean main checkout identifies the same SHA, matching the live remote recheck.
Preparation changes are recorded in four local commits. The audited source base
remains the candidate above; these preparation commits are listed separately below.
No tests, Docker suites, pushes, tags, workflow triggers, or package-version changes
were performed.

## 0. Release changelog

Status: PASS.
Published base: [v0.27.1](https://github.com/illegalstudio/elephc/releases/tag/v0.27.1),
`24f68a7b03d4db9660515ad17e711a385e0d596a`.
Coverage: 56 merged PRs, 9 direct first-parent integrations, 212 commits, zero unresolved.
All 65 integration rows have a decision: 32 include, 1 covered, 32 omit.
The bundled inventory script passes on this exact candidate. PR #1221 contributes
22 introduced commits, including the merge, and four additional approved bullets.

CHANGELOG.md contains 20 approved bullets under 0.27.2, dated 2026-10-06, with
features before fixes. All 16 earlier bullets are preserved, Unreleased is empty,
and the compare link remains v0.27.1 to v0.27.2. See
[changelog-audit.md](changelog-audit.md), [ledger.json](ledger.json), and
[the current inventory](inventory.json).

## 1. README

Status: PASS for the refreshed changes and prior audited content.
The type table, construct highlights, pipeline, and source map now include generic
specialization, typed arrays and callables, and per-file PHPDoc application.
Strict-PHP acceptance of PHPDoc templates and rejection of native generic syntax
are explicit. Earlier INI, OPcache, bridge, and optimization corrections remain.

## 2. Language reference and internals

Status: PASS for the refreshed changes and static documentation checks.
The pipeline and architecture describe specialization inside the typecheck phase,
without inventing a separate timing label or backend pass. Parser metadata includes
generic declarations, associative type annotations, generic construction, and
callable signatures. The checker page describes the specialization fixed point
and scope-aware site keys.

Two stale array descriptions are corrected: known string callbacks may use typed
array_map result storage, and appending concrete values can build a typed result;
an incremented int|float counter needs a cast for array<int>. Builtin extraction,
module-section generation, PHP comparison generation, builtin audits, site
compatibility, and the EIR builtin/target architecture gate all pass.

All 2,339 Markdown pages pass metadata, body-heading, and relative destination
checks, covering 5,206 links. External URLs and heading fragments are outside this
static scan. The earlier complete CLI, operator, runtime, memory, and target audits
are retained; no new CLI option or target was introduced by the refreshed PR.

## 3. Roadmap

Status: PASS for the current planned scope.
Prior scalar local promotion and integer range deliveries, pass order corrections,
and remaining planned items are preserved. No unplanned generics item was invented;
the delivered feature is recorded in the release changelog.

## 4. Test coverage

Status: REVIEW REQUIRED for complete per-surface coverage sign-off.
There are 20,605 test declaration matches across 1,597 files. Tests for native and
PHPDoc generics, constructor and method templates, strict-PHP diagnostics, array
storage and ownership, named spreads, and callable wrappers were inspected as source.
The two preparation diagnostic functions still contain 24 invalid-call fixtures;
`cargo check --test error_tests --features curl -j 2` passes with zero warnings.
This compiles the fixtures and does not execute them.

The previous static literal-name candidates remain: 116 procedural surfaces lack
a direct positive name fixture and 360 lack a direct arity-diagnostic name fixture.
Aliases, methods, and wrappers may provide semantic coverage, so these counts do
not establish missing behavior tests. See
[coverage-findings.json](coverage-findings.json) and
[validation.json](validation.json). Detailed raw fixture mappings remain local.

## 5. Examples

Status: PASS for static layout and the refreshed feature examples.
There are 234 example entry files, each with a local .gitignore. The new generics
and generics-docblock examples cover native and PHPDoc syntax. No examples were
compiled or executed as runtime checks in this refresh.

## 6. Code style

Status: PASS for current static checks.
All 5,310 owned Rust files have complete initial module preambles. The function
Rustdoc scan finds zero missing docblocks across 50,702 declaration matches.
Five missing docblocks introduced by PR #1221 were added without behavior changes.
The one stash conflict was two equivalent docblocks; the upstream comment was kept.

All 1,205 files under src/codegen and src/codegen_support pass assembly-comment
checks. One new comment in array_internal_pointer.rs was moved from column 82 to
column 81 without changing the instruction. git diff --check passes, no new em
dash characters were introduced, and package version files are unchanged.
Three historical commits contain Co-Authored-By trailers; no history was rewritten
and the preparation commits have no Co-Authored-By trailers.

## 7. Build, test execution, and CI

Build: PASS, zero warnings for cargo build -j 2 and the builtin exporter build.
Diagnostic fixture compilation: PASS, zero warnings.
Tests: SKIPPED by explicit user request. No runtime verification is claimed.

Existing GitHub workflows for this candidate were inspected read-only. CI was
queued, while PDO Live Databases and Push on main were in progress at inspection.
These runs cover the committed source candidate, not the local preparation commits.
No workflow was triggered by the agent. The raw read-only CI snapshot remains
local in refresh-a65ce2f1bc/candidate-ci.json.

## Completion and limits

The requested rebase and approved changelog refresh are complete. The temporary
preparation stash was removed by its exact SHA; other stashes were preserved.
No task-owned temporary directory or patch remains. Raw evidence and the final
state remain local under refresh-a65ce2f1bc. The reviewable check results are
recorded in [validation.json](validation.json).

Full release readiness is not claimed: runtime tests were skipped by request,
complete per-surface coverage mappings still need review, and the candidate CI
was not yet complete when inspected. These limits do not block the requested
checkout refresh.

## Local preparation commits

- `93516a9694df1238574ad377277af70aa68e594e`: style: complete Rust and assembly comments
- `133438a7ee5c61dcf3f16344b7a77da60e934a94`: test: cover PCNTL and XML registry arity
- `18ac29ffe6878581a8f569b3fa33b71f728c1bb2`: docs: align release references with shipped behavior
- The final release-notes commit records CHANGELOG.md and the current audit artifacts.

The branch was confirmed unpublished and was not pushed, following the user
instruction of October 8. PR creation remains with the user. Raw logs and
historical working snapshots are retained locally rather than committed.
