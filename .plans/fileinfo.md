# PHP fileinfo

- [x] Add a pinned static libmagic native package with its embedded magic database on every target.
- [x] Add the optional fileinfo bridge and PHP finfo class, six functions and eleven constants.
- [x] Expose the same declarations to dynamic eval and preserve ownership and failure behavior.
- [x] Add focused native, codegen, error and target coverage plus a practical example.
- [x] Generate and audit builtin documentation in CI without local builds or heavy tests.
- [x] Open a draft pull request and configure exact-head CI coverage for all supported targets.

## Implementation

Use upstream file/libmagic 5.46, pinned by source size and SHA-256. Build a host database compiler when cross-compiling and embed the compiled database in a target C shim, so deployed programs need neither a system libmagic nor an external database. Custom text and compiled databases remain supported through magic_load.

A small Rust staticlib owns opaque handles. The PHP surface uses ordinary synthetic declarations and shared contracts, following the existing XML prelude and native-function eval bridge. All generated code uses the existing EIR and extern ABI paths.

PHP 8 semantics include no-op finfo_close, per-call flag restoration, binary-safe buffers, false results on lookup failures, constructor exceptions, stream input and contexts, and destruction without duplicate release. CI validates the supported target matrix. The user explicitly requested no local heavy builds or tests.

## Validation and acceptance

PR: https://github.com/illegalstudio/elephc/pull/2061, ready for review.
All required checks and both review verdicts must apply to the final head before
the task is complete. The user authorized subsequent fixes and pushes while ready.

Published head `40587ab6f69c7ccbf8480325cd634187947b9bce` passed the complete
CI matrix in run 40189861691 and PDO live suites in run 40189861721. Nineteen
fileinfo execution tests and four native bridge tests passed on Linux x86_64,
Linux ARM64 and macOS. iOS device and Simulator final links passed. Greptile
scored that exact head 5/5 with all eight review threads resolved.

The user subsequently requested independent read-only Grok review through
`me-delegate-review`. Five rounds returned changes-requested:

| Round | Delegation |
| --- | --- |
| 1 | `9b0b53f9-5658-4cae-9187-09764c4b5757` |
| 2 | `d6945bcd-da22-4df9-8f4e-7dd3b014ea63` |
| 3 | `be97a074-822b-4aad-900e-fbed0fca7ce3` |
| 4 | `274f8b3a-5f70-4006-84d7-3f6e3eed0ee1` |
| 5 | `30b23899-3231-461a-9afa-4ed155a81343` |

Valid findings were fixed locally with regression coverage. These include PHP
argument diagnostics, data URL grammar, filter parsing and warning frames,
descriptor-checked directories, captured network causes and callback-safe flag
restoration. The fifth-round fixes preserve directory classification through
nested filters and PHP's resource-segment tokenization. Evidence-backed declines
retain ValueError for database NULs, platform-dependent flag validity, warned
unknown-scheme local fallback and the illegal-parameter data URL reason. Later
review rounds accepted those decisions.

There are now 46 fileinfo execution tests, eight native bridge tests and a TLS
regression. New tests remain unexecuted until CI. PHP 8.5.12 reference oracles,
static documentation audits and diff checks pass. The six generated builtin
source anchors are unchanged; the compatibility table still reports 6/6
functions, 1/1 classes and 11/11 constants.

- [x] Keep the PR ready for review.
- [ ] Obtain a clean Grok verdict on the final candidate.
- [x] Resolve integration conflicts with the moved main branch, retaining tokenizer and fileinfo wiring.
- [ ] Push only after the external review is clean, then follow CI and Greptile.
- [ ] Confirm CI, clean Grok review and Greptile 5/5 on one final head.

The user explicitly authorized continuing beyond the five-round limit. Further
read-only Grok rounds continue until clean. The latest main integration retains
both tokenizer and fileinfo and regenerates documentation from the merged JSON
registries without a local build. Audits pass with 1082 catalog functions and
2316 generated pages. Exporter parity and executable validation remain CI gates.
Local corrections have not been pushed during this external review phase.
No local builds or compiled tests were run; task-owned oracle files were removed.


Grok round 6 (9660cadf-7936-4842-a6c5-405c8f875e80) accepted the integration
and earlier fixes, then identified two P2 stream-stat defects. URL queries now
create filters before stat and read user wrappers only after stat succeeds. A
missing stat method uses the active public function and concrete wrapper class
in its warning; failed stat is silent false. Directory identity bypasses magic.
Already-open mime_content_type resources retain their direct read behavior.
Successful network and compressed opens without stat return false. Pending
streams and prepared filter stages are query-local and close on all paths;
native exception cleanup suppresses stream-close callbacks as PHP does and
restores normal close behavior afterward. New codegen coverage and PHP reference
oracles cover stat results, callback ordering, exceptions, mode coercion and
all HTTP URL entry points. Compiled tests and exporter parity remain CI gates.

Native descriptor classification uses the same PHP directory-bit rule as user
wrapper metadata. An anonymous socket-pair metadata regression covers the
non-directory POSIX mode whose directory bit PHP nevertheless recognizes; no
physical device is opened or read.

Grok round 7 (a6d36c36-3c59-40b0-8c33-58be6dd486a7) accepted the prior fixes
and identified missing magic-stream stat and cast callbacks. URL identification
now performs the second stat after its initial gate; resource identification
performs one stat without that gate. Active bit 512 returns false on stat failure.
Unfiltered wrapper reads call stream_cast(0) after obtaining bytes, preserving
missing-method warnings, falsy results, resource identity and invalid-return
errors. Per-class ABI metadata distinguishes raw and boxed callback results.
Read and write filters suppress casts only when attachment succeeds; write-only
filter owners retain creation/close ordering without modifying read bytes.
Exception-time diagnostics match PHP's registered-handler suppression, and
normal reporting applies when no handler is installed. Three new execution
fixtures have matching PHP 8.5.12 oracle output; compiled coverage awaits CI.


The first round-8 delegation (65deb75e-24b4-4ee7-9d95-74b20621e800)
terminated with exit143 and no result. Both processes stopped before edits.
Three host-verified cases were corrected before restarting the full review:
read and read-filter exceptions retain the cannot-read diagnostic; exception-time
suppression checks the handler mask for E_WARNING; invalid cast owners survive
until warning and TypeError construction, then retire in PHP finally. A guarded
opaque return record transfers the owner across native allocation and is freed
before its typed value destructor, preserving reentry and exception chaining.
The new regression matches PHP8.5.12 across handler masks, resource reads,
filter failures, destructor changes to error_reporting and a throwing destructor
whose previous exception remains TypeError. Compiled tests still await CI.

The completed round-8 restart (e425ab41-b7b1-48b8-87e0-b82868b3f36f)
accepted the exception and owner corrections, then found one ARM64 string-result
ABI defect. Cast string capture now uses the shared string_result_regs helper,
preserving x1/x2 on ARM64 and rax/rdx on x86 before truthiness and ownership
processing. Existing zero-string, substring and fresh-string regression cases
cover the fix. A new full read-only review remains required before publication.
