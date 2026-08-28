---
name: prepare-release-changelog
description: Audit and prepare Elephc's release changelog from the last published GitHub release through an exact candidate main SHA. Use before any release, version tag, release Pull Request, or pre-release verification, and when reconstructing `CHANGELOG.md` from merged work. Refresh live release/main state, reconcile every merged PR and direct first-parent commit, inspect actual shipped behavior, draft concise user-facing bullets with complete source traceability, and update `[Unreleased]` only after user approval.
---

# Prepare Release Changelog

Build a complete, evidence-backed release ledger before writing release notes. Treat missing or unresolved commits as release blockers; do not infer coverage from PR titles or the existing changelog alone.

## 1. Establish the exact release range

1. Read `CONTRIBUTING.md`, `AGENTS.md`, and the top and link-list portions of `CHANGELOG.md`.
2. Record the checkout, branch, `HEAD`, worktree status, remotes, and divergence from `origin/main`. Preserve unrelated or pre-existing changes.
3. Refresh live state with `git fetch --prune origin main --tags` and identify the latest published, non-draft, non-prerelease GitHub release with `gh release list` / `gh release view`. Do not substitute the numerically highest local tag without checking GitHub. If preparing a prerelease line, require the user to name its base release explicitly.
4. Resolve the release tag and candidate head to full SHAs. Default the candidate to refreshed `origin/main`; if the user names a SHA/ref, use that exact head. Report local commits ahead of `origin/main` separately and do not include them without explicit direction.
5. Require the release tag to be an ancestor of the candidate head. Stop if the tag, candidate, repository identity, or GitHub release cannot be verified.

Do not edit `CHANGELOG.md` during this discovery phase.

## 2. Build and validate the coverage ledger

Run the bundled inventory script from the repository root:

```bash
python3 .claude/skills/prepare-release-changelog/scripts/release_inventory.py \
  --from-tag <published-tag> \
  --head <candidate-sha> \
  --repo illegalstudio/elephc \
  --format json
```

The script walks the first-parent history, recognizes GitHub merge and squash commits, queries associated PRs for ambiguous commits, expands every merge's introduced commits, and reconciles the result against the complete Git commit range.

Require all of these before drafting:

- `unaccounted_commits` is empty;
- `errors` is empty;
- every first-parent integration is a verified merged PR or a verified direct commit;
- every PR targets `main`, is merged, and has a merge commit reachable from the candidate;
- the reported base and candidate SHAs match the refs selected above.

Never silently classify a commit as direct when GitHub association lookup failed. Investigate unresolved or ambiguous rows and rerun the inventory until coverage is complete.

## 3. Inspect what actually shipped

For every PR and direct commit in the ledger:

1. Inspect its metadata, changed paths, diff, tests, documentation, linked issues, and relevant source. Titles and bodies are orientation, not semantic proof.
2. Describe the shipped user-visible behavior, compatibility change, performance effect, security impact, or developer-facing interface change.
3. Mark internal chores, repository statistics, generated data, tests-only changes, and refactors with no observable effect as `omit`, with a concise reason.
4. Detect follow-ups, partial fixes, reverts, and several PRs implementing one coherent outcome. Consolidate only when the combined wording remains accurate; retain every source PR/commit in the ledger.
5. Compare every candidate semantically with the current `[Unreleased]` bullets. Mark existing coverage, duplicates, stale bullets, and claims not supported by the audited range.

Every ledger row must end as `include`, `covered`, `omit`, `reverted`, or `needs decision`. No source may disappear merely because it does not deserve its own bullet.

## 4. Draft the changelog

Draft one concise bullet per notable outcome, not per commit. Follow the existing Elephc voice:

- lead with `Added`, `Changed`, `Fixed`, `Removed`, or `Security` where natural;
- say what users can now do or what failure is corrected;
- mention implementation detail only when it explains a constraint or material compatibility effect;
- do not claim broader PHP parity, target support, issue closure, or security coverage than the code and tests prove;
- group features and improvements before fixes, and order each group by user impact;
- preserve accurate existing wording unless consolidation or correction is explicitly approved.

Present a draft before editing, using this structure:

```markdown
## Release changelog audit

Range: <tag-sha>..<candidate-sha>
Coverage: <N> PRs, <M> direct commits, <K> total commits; 0 unresolved

### Proposed features and improvements
- ...

### Proposed fixes
- ...

### Coverage ledger
| Source | Shipped behavior | Existing coverage | Decision | Proposed bullet |
|---|---|---|---|---|

### Open decisions
- ...
```

If any row needs a decision, ask for it before modifying the file.

## 5. Apply only the approved draft

After user approval:

1. Update only the top-level `## [Unreleased]` section. Do not create the numbered release section, change version links, tag, commit, push, or publish unless separately requested.
2. Keep approved features/improvements before fixes and preserve intentional existing bullets.
3. Recount the before/after bullets and verify that every removal or consolidation was approved.
4. Run `git diff --check`, inspect the bounded changelog diff, and rerun the ledger against the same candidate SHA if the source range changed while drafting.
5. Report the exact release tag, candidate SHA, PR/direct-commit counts, omitted sources, remaining decisions, and whether the changelog is ready for `verify-release`.

Do not declare the changelog complete from a green diff alone: complete source reconciliation is the gate.
