---
title: Test native SPL ownership together with complete thrown messages
author: Guillaumeloulier
date: 2026-10-16
branch: fix/main-intrinsic-callback-owners
cost: { minutes: 917, tokens: 8136364 }
touches:
  - tests/codegen/runtime_gc.rs
  - ".artifacts/intrinsic-callback-owners/baseline-array-callable"
  - ".artifacts/intrinsic-callback-owners/baseline-array-callable.compile.stderr"
  - ".artifacts/intrinsic-callback-owners/baseline-array-callable.json"
  - ".artifacts/intrinsic-callback-owners/baseline-array-callable.php"
  - ".artifacts/intrinsic-callback-owners/baseline-array-callable.s"
  - ".artifacts/intrinsic-callback-owners/baseline-build-retry.log"
  - ".artifacts/intrinsic-callback-owners/baseline-build.log"
  - ".artifacts/intrinsic-callback-owners/baseline-local-key/compile.stderr"
  - ".artifacts/intrinsic-callback-owners/baseline-local-key/fixed_local_key_type_error"
---

## Decided
Keep the existing nonheap method alias summary fix together with consumed-Mixed owner acquisition in generated intrinsic method wrappers. The twelve additional regression controls cover reads, writes, local object keys, list add, queue enqueue, and valid nested values through dynamic SPL methods and ArrayAccess callbacks. Assert the entire PHP exception message as well as a clean heap.

## Tried and ruled out
Retaining only the wrapper arguments prevented the use-after-free but left literal-array owners alive when void methods lacked an alias summary. Reuse the existing c6c35eb7 fix; do not add a second implementation. Clean main 259877ca reproduced the ownership failure before that prerequisite was restored.

## Watch out
A balanced final heap does not prove diagnostic bytes remained alive: the local stdClass key reproducer corrupted the exception message before the complete prerequisite. The twelve controls passed all three native architectures, with assembly checks on both iOS targets. First-class native method callables remain a separate unresolved gate; this test set makes no claim about them. Manual cross-target links of dynamic SPL programs can need Phar plus zlib and bzip2 even when the source uses only SPL.
