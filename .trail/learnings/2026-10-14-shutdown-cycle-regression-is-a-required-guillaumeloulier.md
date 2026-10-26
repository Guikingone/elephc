---
title: Shutdown cycle regression is a required test fixture
author: Guillaumeloulier
date: 2026-10-14
branch: fix/main-shutdown-cycle-collection
touches:
  - tests/codegen/runtime_gc/cow_and_cycles.rs
---

## Decided
The two test_gc_main_return regressions intentionally construct a cyclic object and let its last external owner expire. Keeping those test fixtures is required: they fail before explicit shutdown collection and pass after it. They do not add new production GC roots.

## Tried and ruled out
Trail check reported a lexical conflict between the regression fixture and the earlier shutdown learning's description of the failing baseline. That description ruled out relying on teardown alone as the production solution; it did not rule out regression coverage of the failing input. Retain the tests and explain the false positive in the commit message.

## Watch out
Compare the generated epilogue with the test assertions before treating a textual learning match as a semantic conflict. Main-return ordering and balanced heap output are the acceptance criteria.
