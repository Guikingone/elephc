---
title: DLL cursor and lifetime controls explain the Trail checkpoint findings
author: Guillaumeloulier
date: 2026-10-15
branch: fix/main-spl-list-subclass-storage
touches:
  - examples/spl-containers/main.php
  - src/codegen/lower_inst/callable_descriptors.rs
  - src/codegen/lower_inst/objects/clone_and_spl.rs
  - src/codegen/lower_inst/objects/dynamic_factory.rs
  - src/codegen/lower_inst/objects/dynamic_mixed_candidates.rs
  - src/codegen/lower_inst/objects/fixed_new.rs
  - src/codegen/lower_inst/objects/reflection/class_members.rs
  - src/codegen/runtime_callable_invoker/string_return.rs
  - src/codegen/runtime_metadata/callables.rs
  - src/codegen/runtime_metadata/intrinsic_wrappers.rs
---

## Decided
Keep the DLL implementation and all strict controls. The three findings in dll-trail-check-v1.log are lexical matches against historical diagnostics. The ancestor walker uses a negative class-id terminator. Cursor stepping validates the existing index, then decrements zero to -1; subsequent movement preserves an invalid position. The clone starts at physical index zero. These facts are verified by the native outputs on all three executable targets.

## Tried and ruled out
The checker grouped unrelated comments, old failure narratives and the current regression preamble. The actual control asserts held|drop|done as well as a clean heap; it detects destruction before the post-call marker even when the final allocation totals balance. The boundary controls assert negative positions, repeated movement and deletion behavior directly against PHP. Removing those checks would lose the evidence for the repair.

## Watch out
The first check has three findings. Explain their categories in the checkpoint commit and preserve the original learning. Private checkpoint publication does not close the two registered dependency/GC gates or the 1710-test campaign. Example verification initially exposed an existing PHP 8.5 attach deprecation; the example now uses offsetSet, while the original mismatch stays in its artifact. No php-src test source or expected output was changed.
