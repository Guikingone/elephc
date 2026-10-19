# Repository plans and working specifications

Reviewed on 2026-10-19 against release code candidate `df57a9800f`. This index
distinguishes remaining work, retained specifications and active release
evidence. Implementation status comes from source and recorded integration
evidence; this cleanup ran no tests or behavioral probes.

## Active follow-up work

| Plan | Remaining scope |
|---|---|
| [Register allocation](eir-06-linear-scan-register-allocator.md) | Position-specific allocation, interval splitting and spill/reload transitions. The default linear-scan allocator is already shipped. |
| [Core ownership and callables](core-final-review-fixes.md) | Dynamic reference-return transfer and retained descriptor/interface reports. Consolidates both former Core logs; PR #893 is merged. |
| [Eval and Magician](elephc-eval-magician-plan.md) | Shared grammar corpus, dynamic-name boundaries, lexical specialization and further literal AOT families. |
| [Mbstring qualification](mbstring-extension.md) | Exact probe-to-test mapping and remaining ownership/strictness/host qualification. All 65 baseline function bindings are shipped on both backends. |
| [Native dependencies](native-dependencies-v1.md) | Explicit future providers, cache migration decision and recipe-revision diagnostics. Managed dependencies v1 is shipped. |
| [PHP extension bridge](bidirectional-php-extension-bridge.md) | Producer stack integration on open PRs and the separate consumer-hosting design. The producer is not in this release code candidate. |
| [PDO parity](pdo-full-maintained-php-parity.md) | Broader registry migration, unintended common divergences and proprietary-client/cross-target live qualification. |
| [PhpStorm plugin](phpstorm-plugin.md) | Unimplemented IDE proposal; SDK choice, language integration and qualification remain open. |

Detailed execution plans, current task ownership and investigation evidence also
live in Code Journal. The individual files link the relevant plan/task IDs.

## Retained working specification

[iOS target and library ABI](ios-target.md) describes shipped behavior and host
integration. It was deliberately moved into `.plans` and is still referenced by
the SwiftUI examples. Keep its stable path; it is a working specification with
separate host/device qualification and library-boundary follow-ups.

## Active release evidence

| Artifact | Purpose / state |
|---|---|
| [0.27.4 changelog audit](release-0.27.4-changelog-audit.md) | Complete integration ledger and proposed release bullets; changelog application still awaits approval. |
| [0.27.4 verification](release-0.27.4-verification.md) | Static verification, permitted build evidence and disclosed validation limits. |
| [0.27.4 static inventory](release-0.27.4-static-inventory.json) | Machine-readable inventory supporting the verification record. |

The cleanup preserves these artifacts. It does not update the numbered release
section or replace the release approval gate.

## Retained source reproducers

[Mbstring/eval probe inventory](mbstring-probes/README.md) describes all 12
retained PHP sources and their related permanent coverage. A matching class
name or partial lexer case is insufficient evidence to delete a reproducer.
Keep each source until its exact output, mutations, cleanup and failure behavior
are preserved in a permanent regression or resolved with current evidence.

## Completed-plan history and maintenance

The cleanup removed 46 completed/superseded planning records and merged the
second Core log into its surviving backlog. The complete original collection is
recoverable from Git commit `0316affac0`, for example:

```bash
git show 0316affac0:.plans/<original-filename>.md
```

Use permanent `docs/` pages for shipped contracts and update live references
when retiring a plan. Preserve immutable historical learning records. New or
continued plans should state their source baseline, remaining decisions and
acceptance criteria; move historical terminal transcripts out of the active
backlog by retaining them in Git or Code Journal. Keep generated binaries,
assembly, logs and temporary files out of this directory.
