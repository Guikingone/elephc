# Kage Memory Review

Project: /Users/guillaumeloulier/PhpstormProjects/oss/elephc/.claude/worktrees/streams-csv-signature
Pending packets: 2
Branch summaries: 1

Review with:

```bash
kage review --project /Users/guillaumeloulier/PhpstormProjects/oss/elephc/.claude/worktrees/streams-csv-signature
```

## Branch Summary 1: worktree-streams-csv-signature

- Head: `034ac0793606439d8aa746a800a1323674793d0a`
- Merge base: `68cb75479a5a19b08d18e4efe19ee9a2a6366a5a`
- Changed files: .agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md, .agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md, .codebase-memory/artifact.json, .codebase-memory/graph.db.zst
- Generated: 2026-08-14T14:10:50.554Z

```text
.codebase-memory/artifact.json |  12 ++++++------
 .codebase-memory/graph.db.zst  | Bin 37947115 -> 38854582 bytes
 2 files changed, 6 insertions(+), 6 deletions(-)
.agent_memory/packets/repo_map-streams-csv-signature-repo-overview-649cee4c.md | untracked
.agent_memory/packets/repo_map-streams-csv-signature-repo-structure-757b1268.md | untracked
```

## 1. Runbook: cargo test p elephc builtin contract 2 &1 | grep E "FAILED|panicked|assertion|left|right|test result" | head 20: surfaces ... FAILED test support::tests::every contract has a backend support record ... FAILED test suppor

- ID: `repo:streams-csv-signature:runbook:runbook-cargo-test-p-elephc-builtin-contract-2-1-grep-e-failed-panicked-assertio`
- Type: `runbook`
- Tags: observed-session, commands, runbook, auto-distill
- Paths: (none)
- Summary: Observed commands: cargo test -p elephc-builtin-contract 2>&1 | grep -E "FAILED|panicked|assertion|left|right|test result" | head -20, cargo nextest run --test codegen_tests -E 'test(/csv|getcsv|putcsv|spl_file|SplFile/)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5, cargo nextest run --test codegen_tests -E 'test(/stream|filter|wrapper/) and not test(imagefilter) and not test(eval)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5 && cargo test -p elephc-magician --lib 2>&1 | tail -1
- Admission: candidate (58/100, candidate)
- Admission reasons: durable memory type, has provenance, has durable trigger, rationale, issue context, or explanation, substantive enough to reuse
- Admission risks: (none)
- Quality score: 74/100
- Quality reasons: high-value memory type, has source evidence, tagged, actionable rationale or verification
- Review risks: not grounded to paths
- Estimated tokens saved: 1248
- Duplicate candidates: (none)

Reusable command observation distilled from session c8dbe38f-1f79-484f-9b83-62bacab9ca56:

- cargo test -p elephc-builtin-contract 2>&1 | grep -E "FAILED|panicked|assertion|left|right|test result" | head -20: cargo test -p elephc-builtin-contract 2>&1 | grep -E "FAILED|panicked|assertion|left|right|test result" | head -20: surfaces ... FAILED test support::tests::every_contract_has_a_backend_support_record ... FAILED test support::tests::every_eval_binding_has_a_documented_execution_route ... FAILED thread 'registry::tests: cargo test -p elephc-builtin-contract 2>&1 | grep -E "FAILED|panicked|assertion|left|right|test result" | head -20: surfaces ... FAILED test support::tests::every_contract_has_a_backend_support_record ... FAILED test support::tests::every_eval_binding_has_a_documented_execution_route ... FAILED thread 'registry::tests::catalog_is_valid_and_complete_for_all_contract_surfaces' (117809611) panicked at crates/elephc-builtin-contract/src/registry.rs:121:9: assertion `left == right` failed left: 554 right: 544 thread 'support::tests::every_contract_has_a_backend_support_record' (117809615) panicked at crates/elephc-builtin-contract/src/support.rs:281:9: assertion `left == right` failed left: 484 right: 474 thread 'support::tests::every_eval_binding_has_a_documented_execution_route' (117809616) panicked at crates/elephc-builtin-contract/src/support.rs:332:9: assertion `left… [+137 chars truncated]
- cargo nextest run --test codegen_tests -E 'test(/csv|getcsv|putcsv|spl_file|SplFile/)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5: cargo nextest run --test codegen_tests -E 'test(/csv|getcsv|putcsv|spl_file|SplFile/)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5: Summary [ 47.634s] 47 tests run: 47 passed, 8242 skipped cargo nextest run --test codegen_tests -E 'test(/csv|getcsv|putcsv|spl_file|SplFile/)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5: Summary [ 47.634s] 47 tests run: 47 passed, 8242 skipped
- cargo nextest run --test codegen_tests -E 'test(/stream|filter|wrapper/) and not test(imagefilter) and not test(eval)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5 && cargo test -p elephc-magician --lib 2>&1 | tail -1: cargo nextest run --test codegen_tests -E 'test(/stream|filter|wrapper/) and not test(imagefilter) and not test(eval)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5 && cargo test -p elephc-magician --lib 2>&1 | tail -1: Summary [ 270.457s] 622 tests run: 622 passed, 7667 skipped cargo nextest run --test codegen_tests -E 'test(/stream|filter|wrapper/) and not test(imagefilter) and not test(eval)' --no-fail-fast 2>&1 | grep -E "FAIL |TIMEOUT |Summary" | head -5 && cargo test -p elephc-magician --lib 2>&1 | tail -1: Summary [ 270.457s] 622 tests run: 622 passed, 7667 skipped
- cargo test -p elephc-magician --lib 2>&1 | grep -E "test result|error" | head -4: cargo test -p elephc-magician --lib 2>&1 | grep -E "test result|error" | head -4: test errors::tests::parse_error_status_distinguishes_unsupported_constructs ... ok test ffi::tests::scope_execution::execute_rejects_php_opening_tags_as_parse_errors ... ok test interpreter::tests::builtins_arrays_type_errors::execute_pro cargo test -p elephc-magician --lib 2>&1 | grep -E "test result|error" | head -4: test errors::tests::parse_error_status_distinguishes_unsupported_constructs ... ok test ffi::tests::scope_execution::execute_rejects_php_opening_tags_as_parse_errors ... ok test interpreter::tests::builtins_arrays_type_errors::execute_program_array_builtin_type_error_names_the_eval_declared_class ... ok test interpreter::tests::builtins_arrays_type_errors::execute_program_array_key_exists_validates_its_second_argument ... ok
- cargo test -p elephc-magician --lib 2>&1 | grep -c "^test .* ok$"; cargo test -p elephc-magician --lib 2>&1 | grep -cE "FAILED|panicked": cargo test -p elephc-magician --lib 2>&1 | grep -c "^test .* ok$"; cargo test -p elephc-magician --lib 2>&1 | grep -cE "FAILED|panicked": 1172 0 cargo test -p elephc-magician --lib 2>&1 | grep -c "^test .* ok$"; cargo test -p elephc-magician --lib 2>&1 | grep -cE "FAILED|panicked": 1172 0
- cargo test --test codegen_tests test_eval_declared_enum_marker_interface_inheritance 2>&1 | grep -E "test result|ok\b" | head -2: cargo test --test codegen_tests test_eval_declared_enum_marker_interface_inheritance 2>&1 | grep -E "test result|ok\b" | head -2: test codegen::eval::test_eval_declared_enum_marker_interface_inheritance ... ok test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8288 filtered out; finished in 158.80s cargo test --test codegen_tests test_eval_declared_enum_marker_interface_inheritance 2>&1 | grep -E "test result|ok\b" | head -2: test codegen::eval::test_eval_declared_enum_marker_interface_inheritance ... ok test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8288 filtered out; finished in 158.80s

Review before approving as a durable runbook.

## 2. Workflow: changed /Users/guillaumeloulier/.claude/projects/ Users guillaumeloulier PhpstormProjects oss elephc/memory/MEMORY.md: Active work — Streams branche feat/streams csv signatures Active work — Streams branche feat/streams

- ID: `repo:streams-csv-signature:workflow:workflow-changed-users-guillaumeloulier-claude-projects-users-guillaumeloulier-p`
- Type: `workflow`
- Tags: observed-session, workflow, auto-distill
- Paths: /Users/guillaumeloulier/.claude/projects/-Users-guillaumeloulier-PhpstormProjects-oss-elephc/memory/MEMORY.md
- Summary: changed /Users/guillaumeloulier/.claude/projects/ Users guillaumeloulier PhpstormProjects oss elephc/memory/MEMORY.md: Active work — Streams branche feat/streams csv signatures Active work — Streams branche feat/streams
- Admission: candidate (70/100, candidate)
- Admission reasons: durable memory type, has provenance, repo scoped or path grounded, has durable trigger, rationale, issue context, or explanation, substantive enough to reuse
- Admission risks: (none)
- Quality score: 74/100
- Quality reasons: high-value memory type, has source evidence, grounded to repo paths, tagged, concise but substantive
- Review risks: all referenced paths are missing: /Users/guillaumeloulier/.claude/projects/-Users-guillaumeloulier-PhpstormProjects-oss-elephc/memory/MEMORY.md
- Estimated tokens saved: 311
- Duplicate candidates: (none)

Reusable file observation distilled from session c8dbe38f-1f79-484f-9b83-62bacab9ca56:

- /Users/guillaumeloulier/.claude/projects/-Users-guillaumeloulier-PhpstormProjects-oss-elephc/memory/MEMORY.md: changed /Users/guillaumeloulier/.claude/projects/-Users-guillaumeloulier-PhpstormProjects-oss-elephc/memory/MEMORY.md: ## Active work — Streams (branche `feat/streams-csv-signatures`) -> ## Active work — Streams (branche `feat/streams-csv-signatures`) - 🎯🔴🔴 [AUDIT COMPLET surface streams 2026-08-14](streams-audit-full- changed /Users/guillaumeloulier/.claude/projects/-Users-guillaumeloulier-PhpstormProjects-oss-elephc/memory/MEMORY.md: ## Active work — Streams (branche `feat/streams-csv-signatures`) -> ## Active work — Streams (branche `feat/streams-csv-signatures`) - 🎯🔴🔴 [AUDIT COMPLET surface streams 2026-08-14](streams-audit-full-surface-2026-08-14.md) — 12 P0 silencieux (STREAM_CLIENT_* permutées · stream_metadata=décalage registres · READ_CSV=explode · fputcsv escape asym · strip_tags fantôme · php://output vs ob · filter défaut 3≠0 · buffers non dispatchés · mkdir 1-param · &$consumed · ValueError single-char · fscanf &vars) ; 2 tests épinglent du FAUX (:2191, :2559)

Review before approving as durable repo memory.
