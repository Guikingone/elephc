# Unified task API and periodic heartbeat implementation

## Checklist

- [x] Confirm owned worktree, draft PR #958, current head and inherited WIP boundaries.
- [ ] Close the existing native heap/ownership regressions without weakening assertions.
- [ ] Introduce `Elephc\Task\Execution::{Async, Parallel}` with native/eval/strict/target tests.
- [ ] Move public task creation and scope ownership to `Elephc\Task\run` / `TaskGroup`.
- [ ] Implement explicit mode submission and common nonblocking task-result `await()`.
- [ ] Integrate worker completion wakeups without blocking the cooperative scheduler.
- [ ] Implement scope-owned `TaskGroup::every()` registrations with independent intervals.
- [ ] Implement per-registration `stop()` and `setInterval()` plus safe Parallel control proxies.
- [ ] Verify cancellation, failure, queue bounds, stale handles, teardown and AST/oracle parity.
- [ ] Test relevant optimizer modes and every supported target; complete exact-artifact review gates.
- [ ] Update public/internal/generated documentation after the API verification gate.
- [ ] Commit verified checkpoints and push to the existing draft PR with exact remote provenance.

## Authority and current evidence

The maintainer authorized implementation on 2026-10-11: unify the public namespace
as `Elephc\Task`, retain an explicit Async/Parallel mode, clean up the current work,
implement heartbeat and document the fully tested API. Push regular verified
checkpoints to https://github.com/illegalstudio/elephc/pull/958; do not create a new PR.
This reopens the old separate-public-namespace/result-handle choices deliberately,
not worker memory isolation or structured cleanup. Existing independent-review
acceptance remains a completion gate; a draft checkpoint is not consensus.

Owned branch: `spike/runtime-ctx-register` in `hazy-watching-moler`.
Starting local head: `fb95c1bfba6b965dcfaa17ec43b4977b12ef3a49`.
Live PR head checked at task entry: `c933b7c714ea5df714de770b0d1f28cfa91a90be`, draft/open.
The prior main rebase is local; publication requires a verified explicit lease.
Seven vendor deletions, local memory/config/scratchpad state and inherited source
WIP must not be swept into a checkpoint with `git add -A`.

## Required end state

One structured public group owns both execution domains. `spawn()` submits one
execution; `every()` registers recurring actions, each with its own positive finite
seconds interval. Registration is nonblocking and returns a control distinct from
an individual task result. Control is explicit in callbacks and externally usable
inside its live owning scope. Callbacks and task inputs use explicit arguments,
not required userland captures. No public heartbeat accessor or extra event loop.

Async keeps same-context cooperative execution. Parallel keeps isolated worker
arenas and checked copy transfer; worker controls require typed command proxies.
Common await must suspend/wake the coordinator, never conceal blocking join.
Periodic registrations cannot detach or restart an old completed execution.

Maintainer decision (2026-10-12): completed task results remain accessible after
`run()` exits, for both execution modes. A terminal handle owns the parent-side
value/failure/cancellation outcome, not a live group, worker, worker arena or native
job. Repeated awaits preserve ordinary PHP by-value semantics; native worker
results are materialized/copied before release. Scope closure still invalidates
group mutation and periodic control authority. This deliberately extends the old
Async closed-handle rule while preserving Parallel's terminal-result capability.

Working timing defaults from discussion, to pin with tests: first execution after
the interval; monotonic fixed cadence; at most one queued/running invocation per
registration; skip missed deadlines; no overlapping invocation of one action and
no unbounded catch-up. Stop prevents further invocations and drains active work
cooperatively. Define rearming under setInterval, stop/change ordering, scope-close
behavior and worker command acknowledgement in the implementation contract.

## First code checkpoint — execution enum foundation

The first slice adds compiler-owned AST declarations for the execution enum and
the neutral class contract, with matching frontend injection routes and focused
native/eval/diagnostic tests. It does not yet claim unified run/spawn/await or
heartbeat implementation. Production must never parse the PHP oracle to build it.
Generated user-facing docs remain behind the requested full-API testing gate.

## Native completion-wake foundation checkpoint

`crates/elephc-parallel/src/completion_wake.rs` now provides per-coordinator,
native-owned nonblocking socket pairs. Terminal transitions broadcast one coalesced
pending byte to each live subscriber. Draining one subscriber cannot steal another
scope's notification. Handles are monotonic, constrained to PHP's positive signed
integer range, and retired IDs fail closed even if the OS recycles a descriptor.
Exported integer status/FD results use signed 64-bit ABI slots. No PHP pointer or
arena lease crosses threads; no socket/registry is allocated before a subscription.

Create the subscription before inspecting job state; recheck job state after every
wake. Registry operations never acquire a job mutex. The new signal augments the
existing generation/condition-variable path; it does not replace legacy join yet.
The coordinator must release both endpoints only after its waits are settled.

Current final-source evidence (2026-10-11):

- macOS ARM64: 17/17 focused completion-wake/generation/cancellation/wait/worker tests.
  The 2026-10-12 precommit filter expands that evidence to 19/19 with controlled
  bridge panic and rejection of raw non-EPV job envelopes; the bridge build is clean.
- Linux ARM64: 17/17 in image 3f506a232838c660a0258049609b99a32be19beeaff50f93bd691da0c599a96e.
- Linux x86_64: 17/17 in image db9f3dce9005b21801ae553692d7c5c69fac998efd890766faedeccfcbd47312.
- iOS device and Simulator ARM64: focused bridge cargo check succeeds on both targets.
  The local iPhoneOS SDK is absent; no iOS link or device execution is claimed.
- Native Execution enum controls: 3/3, including alias/type transport and AOT/eval
  singleton identity. The initial CLI module-registration failure was corrected;
  the successful rerun is distinct evidence, not a green reinterpretation of that failure.
- Extended enum checks revealed missing EnumDecl support in the AST printer; it
  is corrected without excluding the declaration from the contract gates. The
  2026-10-12 resumed run passes 8/8 tests (including five-target EIR lowering,
  unit/backed enum print round trips and prelude contracts), then 1/1 diagnostic test.

Logs are `/tmp/elephc-task-completion-wake-{host,linux-arm,linux-x86}-integrated-final-20261011.log`,
`/tmp/elephc-task-completion-wake-precommit-20261012.log`, the two
`/tmp/elephc-task-completion-wake-ios{,-sim}-final-20261012.log` checks, and
`/tmp/elephc-task-execution-{target-parity,errors}-resume-20261012.log`.
Only the three native bridge source files form the first publishable code slice.
The enum integration and inherited heap/ABI WIP remain separate until their own
verification gates. This checkpoint does not implement public unified run/await,
worker command proxies or every, and does not close the existing heap regressions.

Kioku risk review completed after a long whole-store verification despite the path
filter. It flags historical broad ctx/serialization/panic packets as drifted.
The current bridge diff does not change worker storage or release/transfer ABI;
the focused worker/quarantine/panic/envelope controls above pass. Wider ctx/C-host
claims remain explicitly un-reverified rather than silently refreshing their packets.
