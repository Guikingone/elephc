---
title: Shared Cargo targets can reuse another worktree catalog as fresh
author: Guillaumeloulier
date: 2026-10-20
branch: work/isset-assignment-main-baseline
touches:
  - src/ir_lower/context.rs
  - src/ir_lower/context/local_presence.rs
  - src/ir_lower/expr/lazy_isset.rs
  - src/ir_lower/function.rs
  - src/types/checker/builtins/language_constructs.rs
  - src/types/checker/driver/init.rs
  - src/types/checker/driver/top_level.rs
  - src/types/checker/mod.rs
  - src/types/checker/null_probe.rs
  - tests/codegen/runtime_gc.rs
---

## Decided
When switching dirty worktrees on the campaign's shared Cargo target, bind validation to actual current-source artifacts and force a source-root refresh if Cargo falsely reuses a crate. Touch only current worktree crate roots without changing their bytes, record hashes and timestamps, serialize the rebuild and preserve every failed attempt. Do not clean a shared target or remove another task's artifacts. Pin the complete CLI/test executable, the selected empty-feature Magician archive and their source manifests; an old rlib cannot safely regenerate test drivers after its shared dependency files have been overwritten by another worktree.

## Tried and ruled out
M17 failed because schema validation could not find method_return_is_tentative, although the current builtin-contract lib.rs exported it and its source hashes were unchanged since the successful M16 build. Cargo reported the current integration source path and fresh=true while reusing metadata produced during clean-main work. An explicit package selection and matching source-path record alone were insufficient. The refreshed M17 build reports fresh=false for both builtin-contract and Magician, verifies one empty-feature Magician archive, compiles successfully and preserves source hashes.

## Watch out
The combined accepted checkpoint is main-m17-refresh under the SPL/OPcache main integration worktree. CLI SHA 384ca1409febdb85b91a1559f213bde10f394f91b9826378d0e61641cd768922; Magician SHA 71e21d8ccf03305acc335ce2687866a50fd353161a2f41514ffddb0683f02dd0. The print_r destructor capture now matches PHP with a clean heap in both IR modes on macOS and in actual complete-core-runtime Linux ARM64/x86_64 executions. The two selected debug PHPT pass unchanged; bug69264 still fails compilation. Fatal visible diagnostics, Fiber leaks, generic nullable-branch capture and the full 1710-test campaign remain open. Trail check's three-word overlap warnings are retained and reviewed in .plans/isset-main-fix-review.md; no commit or publication is authorized.
