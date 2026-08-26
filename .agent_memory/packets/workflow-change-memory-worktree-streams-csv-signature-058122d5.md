---
type: "Workflow"
title: "Change memory: worktree-streams-csv-signature"
description: "Repo-local context for 2 changed repo paths on worktree-streams-csv-signature."
resource: ".codebase-memory/artifact.json"
tags: ["change-memory", "diff-proposal", "repo-local", "branch:worktree-streams-csv-signature"]
timestamp: "2026-08-14T14:10:50.557Z"
x-kage-id: "repo:streams-csv-signature:workflow:change-memory-worktree-streams-csv-signature"
x-kage-type: "workflow"
x-kage-status: "approved"
x-kage-scope: "repo"
x-kage-visibility: "team"
x-kage-verified: "verified"
x-kage-paths: [".codebase-memory/artifact.json", ".codebase-memory/graph.db.zst"]
---

# Change memory: worktree-streams-csv-signature

> Repo-local context for 2 changed repo paths on worktree-streams-csv-signature.

Repo-local change memory generated from the current git diff.

Goal: preserve the durable context another agent should receive when it works in this repo later.

What changed:
- .codebase-memory/artifact.json
- .codebase-memory/graph.db.zst

Diff summary:
```text
.codebase-memory/artifact.json |  12 ++++++------
 .codebase-memory/graph.db.zst  | Bin 37947115 -> 38854582 bytes
 2 files changed, 6 insertions(+), 6 deletions(-)
.agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md | untracked
.agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md | untracked
```

How to verify:
- Add the exact test, build, or manual verification command when you refine this memory.

Improve this packet when more context is known:
- The actual feature, fix, or refactor rationale.
- Why the change was made, including relevant bugs, issues, decisions, and code explanations.
- The package, API, command, or architectural pattern future agents should understand, verify, or reuse.
- Any gotchas, follow-up risks, or branch-specific assumptions.

Promote beyond this repo only after explicit org/global review.

## Why

Branch change memory gives future agents durable context from the git diff when they continue, review, or verify this work.

## Trigger

Recall when asking what changed on this branch, preparing a PR review, or resuming this work.

## Action

Use the changed file list and diff summary as orientation, then inspect the actual diff and source files before making further edits.

## Verification

Generated from git diff and refreshed by kage pr summarize or kage propose --from-diff.

## Risk if forgotten

Future agents may repeat orientation work, miss branch-specific assumptions, or ignore files touched by this change.

## Stale when

The branch diff changes substantially, the branch is merged, or a newer change-memory packet supersedes it.

# Citations

[1] git_diff

## Kage state

Machine state for lossless round-trip; OKF consumers can ignore it.

```json kage-state
{"schema_version":2,"id":"repo:streams-csv-signature:workflow:change-memory-worktree-streams-csv-signature","title":"Change memory: worktree-streams-csv-signature","summary":"Repo-local context for 2 changed repo paths on worktree-streams-csv-signature.","body":"Repo-local change memory generated from the current git diff.\n\nGoal: preserve the durable context another agent should receive when it works in this repo later.\n\nWhat changed:\n- .codebase-memory/artifact.json\n- .codebase-memory/graph.db.zst\n\nDiff summary:\n```text\n.codebase-memory/artifact.json |  12 ++++++------\n .codebase-memory/graph.db.zst  | Bin 37947115 -> 38854582 bytes\n 2 files changed, 6 insertions(+), 6 deletions(-)\n.agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md | untracked\n.agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md | untracked\n```\n\nHow to verify:\n- Add the exact test, build, or manual verification command when you refine this memory.\n\nImprove this packet when more context is known:\n- The actual feature, fix, or refactor rationale.\n- Why the change was made, including relevant bugs, issues, decisions, and code explanations.\n- The package, API, command, or architectural pattern future agents should understand, verify, or reuse.\n- Any gotchas, follow-up risks, or branch-specific assumptions.\n\nPromote beyond this repo only after explicit org/global review.","type":"workflow","scope":"repo","visibility":"team","sensitivity":"internal","status":"approved","confidence":0.62,"tags":["change-memory","diff-proposal","repo-local","branch:worktree-streams-csv-signature"],"paths":[".codebase-memory/artifact.json",".codebase-memory/graph.db.zst"],"stack":[],"source_refs":[{"kind":"git_diff","branch":"worktree-streams-csv-signature","head":"034ac0793606439d8aa746a800a1323674793d0a","merge_base":"68cb75479a5a19b08d18e4efe19ee9a2a6366a5a","changed_files":[".agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md",".agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md",".codebase-memory/artifact.json",".codebase-memory/graph.db.zst"],"summary_path":"/Users/guillaumeloulier/PhpstormProjects/oss/elephc/.claude/worktrees/streams-csv-signature/.agent_memory/review/branch-summary-worktree-streams-csv-signature.json"}],"context":{"fact":"Current branch worktree-streams-csv-signature changes 2 repo paths.","why":"Branch change memory gives future agents durable context from the git diff when they continue, review, or verify this work.","trigger":"Recall when asking what changed on this branch, preparing a PR review, or resuming this work.","action":"Use the changed file list and diff summary as orientation, then inspect the actual diff and source files before making further edits.","verification":"Generated from git diff and refreshed by kage pr summarize or kage propose --from-diff.","risk_if_forgotten":"Future agents may repeat orientation work, miss branch-specific assumptions, or ignore files touched by this change.","stale_when":"The branch diff changes substantially, the branch is merged, or a newer change-memory packet supersedes it."},"freshness":{"last_verified_at":"2026-08-14T14:10:50.557Z","ttl_days":180,"path_fingerprints":[{"path":".codebase-memory/artifact.json","sha256":"78db95b7ba5b472ca164daf6de8b2c9693cba850ddbb1807d2bc6de620f3b10f","size":370},{"path":".codebase-memory/graph.db.zst","sha256":"458c8af6254a06d1a35e120b22ceb18022c070aaa189be15babe4473b1cc88b4","size":38854582}],"path_fingerprint_policy":"source_hash_staleness","verification":"git_diff"},"edges":[{"relation":"changes_path","to":"path:.agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md","evidence":"git_diff"},{"relation":"changes_path","to":"path:.agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md","evidence":"git_diff"},{"relation":"changes_path","to":"path:.codebase-memory/artifact.json","evidence":"git_diff"},{"relation":"changes_path","to":"path:.codebase-memory/graph.db.zst","evidence":"git_diff"}],"quality":{"score":100,"reasons":["high-value memory type","has source evidence","grounded to repo paths","tagged","concise but substantive","actionable rationale or verification"],"risks":[],"duplicate_candidates":[],"stale_reasons":[],"estimated_tokens_saved":287,"admission":{"admit":true,"class":"candidate","score":70,"reasons":["durable memory type","has provenance","repo scoped or path grounded","has durable trigger, rationale, issue context, or explanation","substantive enough to reuse"],"risks":[]},"candidate_kind":"change_memory","review_boundary":"git_or_pr","promotion_requires_review":true},"created_at":"2026-08-14T14:10:50.557Z","updated_at":"2026-08-14T14:10:50.557Z"}
```

