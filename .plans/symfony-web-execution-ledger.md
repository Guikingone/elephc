# Symfony --web: active implementation plan

Last consolidated: 2026-09-26. This is the single current-status entry point for
the Symfony campaign. Update the relevant section instead of appending session
narration or creating another checkpoint plan. Logs, probes and review outputs
belong under `target/`; historical notes are read-only archives.

## Acceptance checklist

- [x] Capture the stock application's native PHP/Xdebug request baseline.
- [x] Preserve typed source snapshots before include folding and declaration stripping.
- [x] Share actual include-once state between native and dynamic execution.
- [ ] Separate all compiled declaration metadata from request-active bindings.
- [ ] Execute known compiled source units through native entries, preserving caller
  scope, references, return values and exception propagation.
- [x] Compile the unchanged application with `--web` and serve `GET /` with HTTP
  **404** and **Welcome to Symfony!**, without worker failures.
- [ ] Compile `bin/console` without errors or warnings and validate the command
  surface; every command must run fully.
- [ ] Remove all compile warnings from both `--web` and CLI console builds; do not
  suppress them or alter Symfony/vendor sources.
- [ ] Validate repeated requests, ownership/cleanup, autoload and target behavior.
- [ ] Meet the build-time objective of approximately 1–2 minutes or less and the
  final executable-size limit of 250 MiB. Measure throughout implementation.
- [ ] Complete independent implementation audits, resolve findings to the requested
  consensus, and obtain the final Kimi K3 audit.

Do not modify `examples/symfony-app`, vendor or generated framework sources to make
tests pass. No framework/package-manager/layout special cases or semantic stubs.
Compare encountered behavior with PHP/php-src, add regressions, and implement the
standard behavior through AST/EIR/native code. Reduced tests are not HTTP acceptance.

## Active specifications

- [Native/dynamic inclusion specification](native-eval-inclusion-spec.md), frozen
  revision 3, SHA-256 `459e29a0c61812134bd5fd53565f56f6dccce0290509874f1af122134353883a`.
- [Exact-hash consensus record](native-eval-inclusion-consensus.md): root, GLM 5.2,
  Kimi K2.7 and Kimi K3 locked the contract. The frozen spec's old header is retained
  deliberately; the separate consensus record is authoritative. This is not a
  completed-code audit or proof of execution.
- [Class alias draft](20260908-native-class-alias-spec.md): still open and unlocked.
  Keep alias identity semantics in that draft; do not implement aliases as subclasses.

## Latest end-to-end evidence

Measured 2026-09-26 on the example application (its own routes and controllers, preload
closed world, `--web` entry `public/index.php`), release compiler from the working tree.

`--web`: builds in about 95-165 s wall (machine shared with other jobs), **11.2 GiB peak
RSS**, **177 MiB** executable, **1.36 GiB** assembly. All **7/7** probed routes are
byte-identical to `php -S` (`/` twig, `/plain`, `/greet/world`, `/greet/accented`, GET and
POST `/echo`, and the 404 page). One-worker latency at load ~9: `/` p50 **4.74 ms**
(p99 7.45), `/plain` p50 3.10 ms, `/greet/world` p50 3.26 ms; `php -S` is ~1 ms.

Console (`bin/console`, compiled): **79/82** command cases match `php bin/console`
(stdout, rc and file side effects). Open: `cache:clear` (container dump diverges:
by-value array arguments written through in the interpreter, see below) and
`secrets:generate-keys` / `secrets:set` (no sodium). Console build: 60-110 s, 8.0 GiB peak
RSS, 133 MiB executable.

Native PHP/Xdebug baseline (2026-08-29): HTTP 404, 39,542 response bytes,
72,544 trace events, 570 distinct targets, 1,421 edges and 215 include targets.
Normalized artifacts: `target/symfony-native-xdebug/ledger/`. The generic tools are
`scripts/normalize_xdebug_trace.py` and `scripts/classify_execution_ledger.py`.
Their historical capability classifications require revalidation against current
code; keep the full edge inventory rather than assuming a trace proves support.

## Immediate implementation order

1. Interpreter by-value ARRAY parameters are bound `Borrowed` and array writes then
   mutate the caller's array (`f(array $p){ $p[] = 1; f($p); }`). This is what empties
   `PhpDumper::collectCircularReferences()`'s loop detection and breaks `cache:clear`.
   Separate array arguments at bind time (`copy_value`, Owned), generators included.
2. Sodium: `crypto_box_keypair/publickey/seal/seal_open` for the two secrets commands,
   as a managed native package plus builtins on both execution paths.
3. Return boundary: `array|null` and `array|false` into a declared `array` return
   neither raise php's TypeError at run time; `array|false` is refused at compile time
   (blocks compiling `symfony/polyfill-mbstring` statically).
4. Conditionally declared compiled functions (`if (!function_exists())`) are not
   callable from eval by direct name.
5. preg: `preg_split`/`preg_match_all` failure results, `preg_last_error()` builtins.
6. Then the build itself: compile time, executable size and peak RSS (11 GiB).

Relevant producers/consumers: `src/resolver/declarations.rs`, `engine_includes.rs`,
`src/ir/source_units.rs`, `src/ir_lower/program.rs`, `stmt/mod.rs`,
`src/codegen_support/runtime/data/const_registry.rs`, runtime `rt_interface_exists.rs`,
`src/codegen/lower_inst/builtins/member_queries.rs`, `src/codegen/web.rs`, and
Magician `interpreter/statements/trait_declarations.rs` / `ffi/symbols.rs`.

## Evidence and open regressions

| Boundary | Current evidence | Remaining work |
| --- | --- | --- |
| Native/dynamic once state | Three of four `native_eval_once` cases pass; native state callbacks executed on macOS ARM64 and Linux ARM64/x86_64 | Skipped native interface first included dynamically still fails; native source entry/provider remains absent |
| File-level functions | Three early-binding regressions pass; parent exception does not activate an unentered child | Conditional functions and same-file alternative implementations need distinct identities and activation |
| Direct native variant calls | Inactive calls throw catchable Error before arguments; methods, closures and 21 existing variant tests pass | Indirect-call timing, genuine redeclaration and complete request-lifetime audit |
| First-class native variant callables | Inactive creation is caught, later creation works, selected variant is invoked, unused creation retains Error | Full target audit and broader binding implementation still open |
| First-class neighbors | 108 passed, two failures; exact PHP reducers reproduce both errors with the older release compiler | Existing `sort` Mixed-receiver and associative-spread Heap(Array)/Heap(Hash defects remain open |
| Class-like activation | Interface event/table tranche is implemented but has not completed executable validation. The pre-fix probe was native `1:1:0` versus PHP `1:0:0`; event generation distinguishes plain/parent-linked/conditional interface sites | Validate corrected `1:0:0`; add class/trait/enum tables, declaration events, active queries, reset and non-existence consumers |
| Activation representation | Six AST/EIR tests pass: namespace/source preservation, no symbol reservation, reachability use, typed identity validation, explicit unsupported non-interface backend on all five targets | Interface runtime emission needs executable/cross-target validation; all non-interface native bindings remain pending |
| Interface diagnostics | Redeclaration diagnostic regression and 57 interface-filtered Magician tests pass | Diagnostic correction is not an activation fix |

Primary regression module: `tests/codegen/type_builtins/includes/declaration_activation.rs`.
Evidence logs: `target/first-class-binding-catch-tests.log`,
`first-class-binding-unused-test.log`, `first-class-binding-neighbors.log`,
`file-early-binding-includes-tests.log`, and `interface-activation-oracle/test.log`.
Activation representation checks: `target/classlike-activation-tests.log` (six pass).
Interface runtime attempt: `target/interface-activation-overlay-test.log` first
exposed a missing runtime feature; `overlay-build.log` exposed a case-normalized
cell mismatch. A later executable run still returned `1:1:0`: the activation table
lost the child marker after `throw` was eliminated, exposing a provenance gap. The
table now uses real declaration spans (not only surviving EIR events), and nested
control marker rewriting is broader. That correction is unexecuted: the subsequent
CLI build was terminated before linking at 398 MiB free, then its 944 MiB partial
debug artifacts were cleaned. This is not a functional pass.
Old-compiler comparison reducers/logs: `target/first-class-neighbor-oracles/`.

## Wider backlog retained, not declared closed

- Conditional functions: availability/callability mismatch, duplicate native symbols
  for same-name alternatives, and different signatures requiring the selected ABI.
- Conditional `define`, deferred enums, aliases, autoload order and source identities.
- Remaining GLOBALS/global-reference/hydration cases; successful reduced callbacks
  do not prove complete PHP symbol-table or request-lifetime parity.
- Reflection alias identity and method filename, closure/callable return representation,
  constructor parity, inaccessible mixed-property reads, GC/error-path ownership and
  strict heap leftovers documented in the historical implementation journal.
- Build reproducibility and per-function output nondeterminism; complete native/eval
  trace classification and missing capabilities must be rechecked, not inferred from
  old builtin-count tables.
- Compiler executable module duplication was removed: one measured release rebuild
  fell from 9m45 to 5m04. This is compiler rebuild evidence, not Symfony application
  compilation time or a statistically controlled benchmark.

## Validation and resource rules

Use one build/test job at a time: `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`RUST_MIN_STACK=67108864`; use `ELEPHC_ASM_JOBS=1` for reducers. Check active processes
and free disk before builds. Do not stop unrelated processes or discard dirty work.
Prefer focused tests and existing binaries; do not run the full suite by default.
Do not confuse emission checks, host execution, cross-target execution and HTTP proof.
Every runtime-affecting change needs coherent supported-target behavior.

No owned build/test is pending. Disk protection is currently mandatory: two CLI
rebuilds exhausted the available free space, even after debug cleanup. Historical
Symfony outputs `r130` through `r166-retry` were moved to the system Trash, preserving
`r167`, root logs and the release compiler; the Trash is on the same volume, so this
did not release its occupied bytes. Empty only those recoverable outputs or reclaim
space outside the worktree before another Cargo build. The AST patch was
completed and corrected after its first check exposed 27 omitted walkers; focused
AST/EIR tests pass. Kage recall for this stage timed out after 300 seconds; current
code/tests were used.
Recheck live state before resuming; do not infer completion from a log.

## Historical evidence

The following snapshots were archived without changing their bytes. References
inside them describe their historical locations and measurements, not current state.

- [Detailed implementation and test journal](archive/symfony/globals-native-eval-parity.md).
- [Original execution-map plan and trace inventory](archive/symfony/symfony-web-execution-ledger.md).
- [r149 debugger notebook](archive/symfony/symfony-r149-double-free-debugger.md).
- [Pre-implementation source inventory](archive/symfony/native-eval-inclusion-implementation-map.md).
- Superseded inclusion specs v1/v2 and five completed review-request documents are
  retained in `archive/symfony/`. Do not resume them as active plans.

Maintain only this current plan and distinct active specifications at `.plans/` root.
Record new measurements in `target/` and summarize their consequence here; do not
grow another append-only session journal or create one plan per build/review.
`globals-native-eval-parity.md` remains only as a short pointer for an existing Kage
citation; it is not an additional active plan.
