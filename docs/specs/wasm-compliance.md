---
title: "WebAssembly and PHP Compliance"
description: "Normative implementation and acceptance specification for the wasm32-wasi target."
sidebar:
  order: 13
---

# WebAssembly and PHP Compliance Specification

This document is the acceptance contract for Elephc's `wasm32-wasi` target. It
replaces feature-count claims with externally observable requirements. A phase,
test count, or generated artifact is not evidence of compliance unless every
applicable acceptance gate in this document passes.

## Evidence status

This table distinguishes implementation progress from acceptance. **Satisfied**
means that the requirement has durable, automated evidence in the repository.
**Partially satisfied** means that an implementation or a local observation
exists but at least one acceptance gate remains open. **Open** means that the
available evidence is insufficient.

| Area | Status | Durable evidence and remaining work |
|---|---|---|
| Normative contract | Partially satisfied | Core 3.0 and WASI Preview 1 are identified, with the WASI ABI pinned below. Every PHP acceptance run must still record immutable php-src revisions and the complete oracle environment. |
| Independent implementation audits | Partially satisfied | Three independent Codex audits covered EIR/runtime semantics, hosts/npm/artifacts, and ownership/cleanup after the requested Ollama reviewers became unavailable and the project owner accepted the substitution. Their findings inform this specification; the final revision has not yet received three independent approvals. |
| Initial baseline | Partially satisfied | Counts and corpus results were recorded at one compiler revision. They are historical observations, not a generated current-revision coverage report. |
| Production validation and artifact integrity | Partially satisfied | Production in-process assembly and Core validation plus focused artifact tests exist. External validation in the shared host job and transaction-safe publication of every output remain acceptance gates. |
| Capability audit | Partially satisfied | Exhaustive identity classification and shape checks for an audited P0 subset exist. Other admitted operations still rely on late lowerer diagnostics, so operand/result types, immediates, representations, ownership, callable shapes, and control-flow shapes are not yet completely classified. |
| Typed transfer and control flow | Partially satisfied | Focused transfer and regression tests exist. Exhaustive representation, branch, switch, ownership, and cross-host matrices remain open. |
| WASI startup, arguments/environment, I/O, and process status | Partially satisfied | Focused implementation tests exist. The complete byte contracts, status boundaries, partial-write behavior, and three-host evidence remain open. |
| Numeric and PHP error semantics | Partially satisfied | Selected regressions have local evidence. The versioned php-src differential matrix remains open. |
| Allocator, ownership, COW, and adversarial safety | Partially satisfied | Focused implementation and cleanup tests exist. Exhaustive resource, malformed-state, aliasing, and failure-path evidence remains open. |
| Deterministic artifacts | Partially satisfied | Deterministic IDs have focused tests, and two local CLI invocations produced identical WAT and WASM. A durable subprocess gate covering WAT, WASM, npm sources, and any normalized distribution archive remains open. |
| PHP differential parity | Open | No committed full-version oracle matrix covers the declared reachable surface. |
| Wasmer, Wasmtime, Node, and external-validator CI | Open | The last established CI evidence installs Wasmer 4.3.7 for WASM-backed tests. A pending shared same-artifact job pins the wider toolchain, but it must be committed and observed green before it becomes evidence. |
| Executable npm package matrix | Open | Package execution and JavaScript/TypeScript toolchain gates are not yet durable CI evidence. |
| Final exact-revision review | Open | Three independent reviewers must inspect the same final source revision and its recorded acceptance logs, with no unresolved blocker. |

### Audit provenance

The originally requested GLM 5.2, Kimi K2.7, and MiniMax M3 reviewers were no
longer available through Ollama. With the project owner's approval, three
independent Codex reviews replaced them for the initial audit. This substitution
changes the reviewer identities, not the completion threshold: three
independent reviewers must still approve the exact revision proposed for merge
and the same durable evidence bundle.

## Normative references

The following sources define separate parts of the contract:

1. [WebAssembly Core Specification 3.0](https://webassembly.github.io/spec/core/intro/introduction.html),
   including its [validation rules](https://webassembly.github.io/spec/core/valid/index.html)
   and [numeric execution semantics](https://webassembly.github.io/spec/core/exec/numerics.html),
   is normative for module validation and Core execution.
2. The official WASI Preview 1
   [API specification](https://github.com/WebAssembly/WASI/blob/e840fe45e63b4f227a29fa87df94ab3bbe3d5efb/legacy/preview1/docs.md)
   and
   [`wasi_snapshot_preview1.witx`](https://github.com/WebAssembly/WASI/blob/e840fe45e63b4f227a29fa87df94ab3bbe3d5efb/legacy/preview1/witx/wasi_snapshot_preview1.witx)
   at WebAssembly/WASI commit
   `e840fe45e63b4f227a29fa87df94ab3bbe3d5efb` are normative for the command ABI
   used by this target. Updating that snapshot requires an explicit spec change;
   a moving branch is not reproducible evidence. Preview 2 or component-model
   support requires a separate contract.
3. Exact commits of [php-src](https://github.com/php/php-src) are normative for
   PHP-visible behavior. The maintained Elephc profiles are PHP 8.2, 8.3, 8.4,
   and 8.5; the evidence manifest must map each tested `--php-version` profile
   to its php-src commit and build configuration.

The [Wasmtime WASI tutorial](https://github.com/bytecodealliance/wasmtime/blob/main/docs/WASI-tutorial.md)
and [Node.js WASI API](https://nodejs.org/api/wasi.html) are informative host
integration references and host-conformance targets; they do not replace the
WASI ABI definition. Host and tool versions must be pinned in the evidence
manifest rather than inferred from these moving pages.

The Core specification defines module validity and execution. It does not
define files, arguments, standard streams, or exit behavior; those are WASI
host contracts. PHP semantics remain authoritative even where the native
Elephc backend has the same bug.

## Compliance boundary

The target is compliant only when all three layers below are simultaneously
true:

### Core WebAssembly

Every `.wat`, `.wasm`, and npm-contained module emitted by Elephc:

- assembles and validates under WebAssembly Core 3.0;
- has type-correct stack, local, block, call, result, memory, and table use;
- has deterministic, collision-free identifiers and exports;
- does not depend on engine-specific acceptance or undefined behavior.

Elephc need not generate every Core instruction. It must generate valid Core
semantics for every instruction it does generate.

### WASI command ABI

Every command module:

- imports Preview 1 functions with their exact signatures;
- checks every errno-bearing call;
- handles partial writes until completion or a defined failure;
- obtains arguments through `args_sizes_get` and `args_get`;
- preserves PHP's `$argc` and `$argv`, including the script-name element;
- terminates with a deterministic status and no callable fall-through after
  `proc_exit`;
- runs without host-specific imports on Wasmer, Wasmtime, and Node.

### PHP-to-WASM semantic parity

For every EIR construct reachable from Elephc's supported PHP subset, the WASM
target must do one of the following before it publishes an artifact:

1. emit PHP-equivalent behavior, including values, types, output, errors,
   ownership, and observable ordering; or
2. fail compilation with a precise target-capability diagnostic.

The second outcome prevents silent corruption but does not constitute 100%
target support. Final compliance requires closing all reachable capability
gaps exercised by the supported PHP surface.

### Reachable surface

The reachable surface is every EIR module that Elephc can produce from
command-mode PHP source accepted under the maintained `--php-version` profiles.
Compiler extensions or host-inapplicable features may be excluded only through
a reviewed, machine-readable WASM exclusion catalog that names the PHP/EIR
surface and gives the reason. A capability identifier admitted by the
pre-emission audit but rejected later because of operand types, immediates,
representations, ownership, callable shape, or control-flow shape remains a
reachable capability gap. The unsupported count cannot be reduced by silently
shrinking this definition.

## Initial audited baseline

The historical audit measured revision
`48b3bdf9ca0d2c19e6949a5f5e89a6055db43b24`. The figures below describe that
revision only. They are not current compliance evidence and must not be copied
forward without regenerating them from a recorded command and tool manifest.

| Surface | Total | WASM dispatch | Gap |
|---|---:|---:|---:|
| EIR `Op` variants | 236 | 90 | 146 |
| `RuntimeFnId` variants | 437 | 8 | 429 |
| `RuntimeCallTarget` variants | 3 | 1 partial | 2 |
| `UnaryStringRuntime` variants | 15 | 0 | 15 |
| `Terminator` variants | 8 | 5 | 3 |

A compile-and-validate pass over the checked-in example corpus produced:

| Outcome | Count |
|---|---:|
| Compiled | 8 |
| Valid modules | 6 |
| Invalid modules | 2 |
| Diagnosed unsupported backend surface | 176 |
| Frontend rejection | 4 |

The invalid examples were `destructor` and `string-builder`. Therefore artifact
generation, focused unit counts, and a green native matrix were insufficient
evidence of target validity at the audit baseline.

## Required implementation work

### P0 — artifact integrity

#### WASM-ART-001: validate production binaries

`emit_wasm_artifacts` must generate in memory, assemble to bytes, and validate
those bytes with a production dependency such as `wasmparser` before writing
any user-visible artifact.

Acceptance:

- malformed and type-invalid WAT are rejected by a production test;
- `--emit-asm` performs the same assembly and binary validation even when only
  `.wat` is requested;
- `.wat`, `.wasm`, npm directories, and any package archive are written only
  after successful validation;
- failed generation leaves no new or partially overwritten artifact;
- a final external `wasm-tools validate` or equivalent remains in CI as an
  implementation-independent check.

#### WASM-ART-002: capability validation before emission

Add a target-capability pass over the complete EIR module before WAT generation.
It must report all unsupported reachable operations with function and EIR
context instead of failing after earlier artifacts or partial output exist.

Acceptance:

- every `Op`, `RuntimeCallTarget`, `RuntimeFnId`, `UnaryStringRuntime`,
  `IrType`, and `Terminator` variant is classified by an exhaustive Rust match;
- the classification checks operand and result arity/types, immediates,
  representations, ownership modes, callable shapes, and control-flow shapes
  wherever those properties change support;
- a module accepted by the capability pass cannot later fail with a backend
  `Unsupported` result during lowering;
- adding a new enum variant fails compilation or the parity gate until its WASM
  capability is classified and tested;
- the parity gate derives its totals and per-variant report from the
  current-revision enums and descriptors; no literal historical count is an
  acceptance threshold;
- no backend `Unsupported` error is first discovered after output publication.

The current shape audit covers checked integer add/subtract/multiply, selected
casts, indexed `ArrayGet`, `NullsafeMethodCall`, and the admitted `GetClass`,
`ArrayMap`, `Usort`, and `ArrayReduce` runtime forms. It does not make the
remaining admitted opcodes shape-complete. In particular, PHP warning/null
behavior for out-of-range array reads and PHP errors for dynamic nullsafe
receivers outside the proven closed class set remain runtime semantic gates;
Core traps are not substitutes.

### P0 — representation and control-flow validity

#### WASM-REP-001: typed value transfer

Create one authoritative transfer/materialization layer for SSA values, local
slots, block parameters, call arguments, call results, and returns. Comparing
component counts is insufficient: the layer must compare concrete WebAssembly
types and apply explicit PHP/EIR conversions.

It must cover:

- concrete `I64`, `F64`, strings, tagged scalars, and every heap kind;
- boxing concrete values into `Mixed`/union slots;
- safe unboxing or target diagnostics in the reverse direction;
- ownership transfer versus borrow, including strings and heap objects;
- multi-value strings and returns;
- block-argument transfers and dispatch-loop merges.

Acceptance:

- the real `while ($i < 3) { $i++; }` regression validates and prints `3`;
- `examples/string-builder` validates and matches PHP;
- a generated test matrix covers every source/destination representation pair;
- Wasmer, Wasmtime, Node, and `wasmparser` agree on validity.

#### WASM-REP-002: void call results

Mirror the native `store_call_result` contract. When EIR assigns the result of a
void callee to a result slot, materialize Elephc's null sentinel and box it when
the destination is `Mixed`; never emit `local.set` against an empty WASM stack.

Acceptance:

- `examples/destructor` validates and matches native/PHP output;
- tests cover ignored void results, concrete result slots, and mixed result
  slots.

#### WASM-CFG-001: typed branches and switches

Validate every branch argument and switch scrutinee against its target
representation. Do not assume an `i64` scrutinee.

Acceptance:

- branch and switch matrices cover scalar, string, tagged, mixed, and heap
  values where EIR permits them;
- invalid EIR is rejected with an EIR diagnostic rather than emitted.

### P0 — entry point, symbols, PHP arguments, and environment

#### WASM-ABI-001: initialize `$argc` and `$argv`

The `_entry` prologue must initialize source locals corresponding to `$argc` and
`$argv` from `__rt_argc` and `__rt_argv`, using the typed-transfer layer when
their slots have widened. The harness passes the same logical argument vector
to every host: `$argc` is its element count, and `$argv[0]` is the exact program
name supplied by the harness rather than a host-selected path.

WASI arguments are byte strings. The package API must document how JavaScript
strings are encoded, reject values that the Preview 1 boundary cannot represent
(including embedded NUL), and preserve all other bytes without host-specific
rewriting.

Acceptance:

- `echo $argc` observes the host argument count including `argv[0]`;
- `$argv[0]`, numeric indexing, `foreach`, empty arguments, non-ASCII arguments,
  rejected embedded NUL, and repeated Node `run()` calls match the declared
  PHP/WASI boundary;
- tests execute on all three hosts with bounded output and timeouts.

#### WASM-ABI-002: injective symbol encoding

Replace lossy punctuation-to-underscore mangling with a deterministic injective
encoding or stable numeric symbol table. Apply it consistently to definitions,
calls, exports, table entries, dispatch stubs, methods, closures, and generated
runtime names.

Acceptance:

- `A\B()` and `A_B()` coexist;
- tests cover namespace separators, method separators, underscores, ASCII
  punctuation, Unicode identifiers, and stable output across two builds;
- definitions, references, exports, table entries, locals, labels, data
  segments, and generated runtime symbols are checked in every relevant WAT
  index space;
- duplicate identifier and duplicate export rejection remain defensive builder
  invariants.

#### WASM-ABI-003: define the environment boundary

WASI environment entries are byte strings. Direct-host and npm harnesses must
apply one documented mapping for key ordering, duplicate keys,
absence-versus-empty values, non-ASCII bytes, JavaScript string encoding, and
embedded NUL rejection. If the accepted PHP surface exposes `$_ENV`, `getenv`,
or related environment APIs, WASM must populate them with PHP-equivalent shapes
or reject the feature through the reviewed capability catalog before emission.

Acceptance covers empty and missing values, duplicate-key handling, non-ASCII
keys and values, embedded NUL rejection, an empty environment, repeated Node
`run()` calls, and identical observations on all three hosts.

### P0 — PHP numeric and error semantics

#### PHP-WASM-NUM-001: shifts

Do not expose WebAssembly's modulo-width shift-count behavior as PHP behavior.
Implement PHP's results for counts greater than or equal to 64 and emit
`ArithmeticError` for negative counts.

Acceptance includes `<<` and `>>` at `-1`, `0`, `1`, `63`, `64`, `65`, very
large counts, negative operands, and `PHP_INT_MIN/MAX`.

#### PHP-WASM-NUM-002: division and remainder

Guard `/`, `%`, and `intdiv()` before the Core numeric instruction:

- zero divisors produce PHP's `DivisionByZeroError`;
- `intdiv(PHP_INT_MIN, -1)` produces `ArithmeticError`;
- `/` preserves PHP float results without losing an integer before conversion;
- neither a Core trap nor `INF` substitutes for a PHP error.

#### PHP-WASM-NUM-003: numeric conversion matrix

Complete and differentially test integer/float/string/Mixed conversions,
comparisons, `NaN`, infinities, signed zero, integer boundaries, precision
boundaries, numeric strings, non-numeric strings, and subnormal values.

The historical audit observed `5e-324` and `1e-320` matching PHP in local
Wasmer and Node runs. That is not a closed gate until a committed differential
regression reproduces it in the pinned host matrix.

### P0 — WASI I/O and termination

#### WASI-IO-001: errno handling

Check the errno result of `args_sizes_get`, `args_get`, `fd_write`, and every
future errno-bearing WASI import. Define one error path that cannot continue
with partially initialized state. `memory.grow` is a Core instruction, not a
WASI errno-bearing import; its `-1` failure result is covered by
`WASM-MEM-001`.

#### WASI-IO-002: complete writes

Loop on `fd_write` until the entire iovec payload is written. Treat a zero-byte
successful write as an error to prevent an infinite loop. Preserve ordering
across mixed string/scalar output.

#### WASI-EXIT-001: non-returning exit

Emit `unreachable` after `proc_exit`. Preserve PHP `exit()`/`die()` integer and
string behavior; frontend restrictions that reject valid PHP arguments must be
fixed in the shared PHP layer and covered on all targets.

The acceptance contract must explicitly define negative and out-of-range PHP
integer statuses at the WASI `proc_exit(i32)` boundary. Host harnesses must
compare the module-level WASI status before any shell-specific 8-bit
normalization, or document and test one common normalization. No successful
local `exit(7)` run alone satisfies this requirement.

The current capability audit rejects `exit`/`die` outside the main function
because caller-owned WASM frames cannot yet be unwound safely. Valid PHP permits
that use, so the rejection is a diagnosed support gap, not compliance. It also
rejects a value-returning main function; the command-mode top-level `return`
value and process-status behavior must be matched to pinned php-src rather than
silently discarded.

### P0 — memory safety and ownership

#### WASM-MEM-001: checked allocator arithmetic

Check alignment rounding, header addition, bump-pointer addition, page
calculation, and i64-to-i32 narrowing before mutation. A failed `memory.grow`
must use a deterministic OOM path, not corrupt state or continue.

#### WASM-MEM-002: unbounded concatenation

Replace the fixed unchecked 64 KiB concatenation area with checked growth or a
heap-backed builder. It must not overlap data, float scratch, heap metadata, or
another frame's live concatenation data.

#### WASM-MEM-003: ownership parity

For strings, arrays, hashes, objects, callables, refcells, `Mixed`, and
iterables, prove balanced ownership across:

- normal return, early return, error, fatal, and process exit;
- overwrite, aliasing, COW, by-reference writeback, and variadic calls;
- closure captures and object destructor re-entry;
- cycles and deliberately malformed runtime cells.

Safety helpers must validate complete block bounds and alignment before reading
headers. Malformed nested `Mixed` chains must terminate with a bounded error or
trap rather than hang.

### P1 — complete EIR and PHP surface

The coverage gate, not this prose, is authoritative. The implementation must
close the following audited families:

| Family | Required work |
|---|---|
| Scalars and strings | string comparisons, strict/loose equality, spaceship, string indexing, persistence, all scalar/string casts, checked mixed numeric operations |
| Arrays and hashes | silent reads, `isset`, mixed keys, append, spread, length, array-to-hash, reference element access, list/unpack behavior, COW |
| Objects and classes | dynamic construction, clone, dynamic properties, initialized checks, static properties, late static binding, reflection paths |
| Calls | named/spread results already normalized by EIR, variadics, by-ref arguments, expression calls, callable descriptors, extern calls, function variants |
| Closures and references | binding, every callable form, refcell binding/loading/release, non-local by-ref storage |
| Exceptions and errors | handler push/pop, catch binding, throw values, fatal terminators, error suppression, cleanup through every exceptional edge |
| Generators and iterators | yield, yield-from, suspension, iterator methods, SPL iterator runtime |
| Globals and statics | writes, `$GLOBALS`, superglobals in the applicable execution mode, static locals and properties |
| Dynamic execution | resolved include/require guards and the supported AOT portion of eval/dynamic dispatch; genuinely impossible AOT behavior requires an explicit public boundary |
| Buffers, pointers, FFI | buffer operations, pointer casts/reads, extern globals/calls, target availability diagnostics |
| Runtime builtins | every variant in the generated current-revision `RuntimeFnId` and unary-string inventories, grouped by their single-source builtin descriptors; the historical 437/15 counts are not fixed requirements |
| Runtime/GC | collection operations, heap metrics, destructors, resource cleanup, callable and object cycles |

Every completed family needs positive PHP-source tests, negative diagnostics,
optimizer-on/off parity where applicable, and ownership tests where values are
refcounted.

### P1 — packaging and host portability

#### WASM-HOST-001: three-host execution matrix

CI must compile real PHP sources once and execute the same validated module on:

- Wasmer;
- Wasmtime;
- Node's WASI API.

The matrix compares stdout, stderr, and module-level exit status. It must pin
the compiler, Wasmer, Wasmtime, Node, external validator, and JavaScript/
TypeScript toolchain versions in its evidence manifest. Tests must have explicit
timeouts and output limits. The job is a required gate; a proposed workflow
patch or a local host run is not evidence until the committed job completes
successfully.

#### WASM-NPM-001: executable package contract

Execute the generated npm package in Node. Test argument, environment, preopen,
exit, thrown-host-error, repeated-run, and concurrent-instance behavior. TypeScript
declarations and JavaScript output must pass their native toolchain checks.

#### WASM-DET-001: deterministic artifacts

Identical inputs, compiler options, and compiler revision must produce identical
WAT, WASM, npm sources, and any package/archive used for distribution after
normalization of explicitly documented metadata.

Acceptance uses separate compiler processes, not two compilations in one
process. The gate records hashes for every output and tests different relevant
map insertion orders. Archive timestamps, entry ordering, permissions, and
compression settings must either be deterministic or be named in the
normalization contract.

## Verification matrix

### Current tested inventory

At the time of this audit, durable repository evidence covers:

- production in-process WAT assembly and WebAssembly Core 3.0 validation with
  `wasmparser`;
- focused artifact-publication tests, including malformed/type-invalid input
  and selected rollback paths;
- Wasmer-backed unit tests in CI, where Wasmer 4.3.7 is installed for the
  previously established relevant jobs;
- compile-time exhaustive enum classification, focused shape checks for the
  audited P0 subset, and target-capability rejection tests;
- focused typed-transfer, `$argc`, void/Mixed result, block-argument, loop, and
  deterministic-ID regressions.

The following observations are useful but are not durable completion evidence:

- local Wasmer 4.3.7 and Node 26.3.0 execution of a hello module;
- local equality of WAT and WASM bytes from two compiler invocations;
- the pending workflow's pinned Wasmer 7.2.1, Wasmtime 47.0.2,
  `wasm-tools` 1.254.0, Node 26.3.0, and TypeScript 6.0.3 matrix until that job
  is committed and observed green.

No current durable gate proves shared-artifact execution on Wasmtime and Node,
external `wasm-tools` validation, npm package execution and toolchain checks, a
complete php-src differential corpus, or the full exit-status boundary.

### Evidence manifest

Every acceptance run must retain a machine-readable manifest containing:

- the Elephc commit, dirty-state status, command line, PHP fixture list, and
  input hashes;
- the pinned WebAssembly/WASI/php-src revisions and the php-src build
  configuration;
- operating system, architecture, locale, time zone, and all relevant
  environment variables;
- exact compiler, Wasmer, Wasmtime, Node, `wasm-tools`, JavaScript, and
  TypeScript tool versions;
- stdout, stderr, module-level exit status, timeouts, output limits, and hashes
  of WAT, WASM, npm sources, and any normalized distribution archives.

Local observations without that manifest may guide implementation but cannot
change a requirement from partial/open to satisfied.

### Core validation

- in-process production validation;
- external `wasm-tools validate`;
- Wasmer validation;
- Wasmtime compilation;
- Node `WebAssembly.Module` construction.

### PHP differential suites

For each supported `--php-version`, compare against the corresponding maintained
php-src behavior for:

- scalar types, casts, operators, warnings, errors, and error text;
- strings including binary/NUL/Unicode bytes;
- arrays, hash key normalization, order, mutation, COW, and references;
- functions, methods, closures, named/spread/variadic/by-ref calls;
- objects, inheritance, properties, clone, destructors, and magic methods;
- control flow, exceptions, generators, includes, and globals;
- every PHP-visible builtin whose descriptor declares WASM support.

The oracle records stdout, stderr, exit status, and, where observable, type/value
shape. A native Elephc result is useful triangulation but is not the PHP oracle.
Each php-src oracle profile must record whether it uses `php -n` or an explicit
INI, loaded extensions and build flags, error-reporting/display settings,
locale, time zone, architecture, and path/line normalization. The harness must
compare the same logical arguments and environment according to
`WASM-ABI-001`; implicit developer-machine configuration is forbidden.

### Corpus gates

1. Every checked-in example applicable to `wasm32-wasi` must compile, validate,
   and match its oracle.
2. Every checked-in codegen PHP fixture must either pass on WASM or carry a
   reviewed, machine-readable execution-mode exclusion.
3. No emitted example may be invalid.
4. Unsupported counts must trend to zero and final compliance requires zero
   reachable unsupported cases within the declared target surface.

### Adversarial gates

Include bounded tests for allocator overflow/OOM, concat growth, multi-megabyte
partial output, zero-length strings, invalid UTF-8 bytes, malformed heap
pointers, double-free defenses, nested/cyclic Mixed cells, deep recursion,
destructor resurrection, closure capture cycles, COW mutation during borrow,
and repeated host instantiation.

## Documentation requirements

Until all gates pass, public documentation must call the target experimental
and enumerate its tested surface. Claims such as "runs under any WASI host" or
"complete target" are forbidden.

When compliance is reached, update:

- the target matrix and CLI reference;
- npm package and host instructions;
- limitations and execution-mode boundaries;
- roadmap and changelog wording;
- generated coverage reports and test commands.

## Completion rule

The work is complete only when:

1. every area in the status table is **Satisfied** with durable evidence;
2. the exhaustive shape-aware coverage gates report no reachable unsupported
   surface;
3. the full artifact, PHP differential, corpus, ownership, npm, and three-host
   matrices pass;
4. the native first-class target CI passes and a dedicated WASM portability job
   validates and runs one shared artifact on Wasmer, Wasmtime, and Node with the
   external validator and npm gates;
5. three independent available reviewers inspect the exact final source
   revision and the same evidence manifest/log bundle, and each records an
   explicit approval without an unresolved blocker.

An API count, a focused test pass, a valid hello-world module, or agreement
between Elephc's native and WASM backends is progress, not completion.
