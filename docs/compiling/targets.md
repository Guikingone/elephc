---
title: "Targets and cross-compilation"
description: "The supported target matrix, how to select a target with --target, and the accepted target spellings."
sidebar:
  order: 4
---

elephc compiles to native machine code for a fixed set of first-class targets.
All native targets are equal: a feature is not considered done until it works on
every one of them. Elephc can also target experimental platforms outside this
matrix. `wasm32-wasi` is one such target: it compiles to a WebAssembly module
rather than native machine code and is documented separately below.

## Supported target matrix

| Target | Platform | Architecture |
|---|---|---|
| `macos-aarch64` | macOS | ARM64 (Apple Silicon) |
| `linux-aarch64` | Linux | ARM64 |
| `linux-x86_64` | Linux | x86-64 |

By default the compiler targets the **host** it runs on, detected automatically.
The native macOS/Linux targets are the first-class supported matrix.

## Experimental targets

| Target | Platform | Architecture | Status |
|---|---|---|---|
| `wasm32-wasi` | WebAssembly / WASI Preview 1 | wasm32 | Experimental; incomplete PHP/EIR parity and portability gates |

Experimental target availability is not a first-class support guarantee. See
[WebAssembly partial parity](#webassembly-partial-parity) for the currently
tested surface and open gates.

## Selecting a target

```bash
elephc --target linux-aarch64 hello.php
elephc --target linux-x86_64 hello.php
elephc --target=macos-aarch64 hello.php
```

Both the spaced (`--target VALUE`) and inline (`--target=VALUE`) forms work.

## Accepted spellings

Each target accepts several spellings, including the LLVM-style triple, so build
scripts written for other toolchains keep working:

| Canonical | Also accepted |
|---|---|
| `macos-aarch64` | `macos-arm64`, `aarch64-apple-darwin` |
| `linux-aarch64` | `linux-arm64`, `aarch64-unknown-linux-gnu` |
| `linux-x86_64` | `x86_64-unknown-linux-gnu` |
| `wasm32-wasi` | `wasm32-wasip1`, `wasm32-unknown-wasi`, `wasm` |

## WebAssembly partial parity

The `wasm32-wasi` target is a non-native target: instead of emitting native
assembly and invoking the system assembler and linker, it emits a WebAssembly
module (`.wat`/`.wasm`) through the dedicated `src/codegen_wasm` backend, which
consumes the same EIR the native backends use. The target is experimental.
Production artifact generation performs in-process Core validation with
`wasmparser`. The dedicated host-portability job is green on Elephc
`0505b837ad`; it checksum-pins Wasmer 7.2.1, Wasmtime 47.0.2, `wasm-tools`
1.254.0, Node 26.3.0, and TypeScript 6.0.3. `--emit npm` packages the resulting
command module for Node.js 20 or newer.

Unlike the native macOS/Linux targets, `wasm32-wasi` is **not yet at full
parity**. It supports a growing subset of the language, and an EIR operation
that the WebAssembly backend does not yet implement aborts compilation of the
whole module rather than degrading a single function. The pre-emission
capability audit classifies every operation, runtime identity, and terminator,
then checks cross-cutting operand, result, immediate, representation,
ownership, callable, object/property, iterator, and control-flow shapes. Static
acceptance is followed by one exact in-memory planning pass; every fallible
lowering and identifier-consistency check must succeed before an accepted plan
exists. Artifact publication consumes that private plan without re-running
lowering, so a module accepted by the complete gate cannot later fail with a
backend `Unsupported`. Rejected source remains a target-capability gap even
though no artifact is published. The audited acceptance contract and remaining
gaps are tracked in
[WebAssembly and PHP Compliance](../specs/wasm-compliance.md).

The durable tested inventory currently includes:

- production in-process WAT assembly and Core 3.0 validation;
- focused artifact publication, shape-complete capability, and target-capability rejection
  tests;
- focused typed-transfer, `$argc`, void/Mixed result, block-argument, loop, and
  deterministic-identifier regressions;
- one shared artifact validated with `wasmparser`, `wasm-tools`, Wasmer,
  Wasmtime, and Node, then executed under all three hosts with exact output and
  `exit(7)`;
- independent-process WAT/WASM/npm/archive reproducibility, partial `fd_write`,
  repeated Node imports, npm contents, and strict TypeScript NodeNext checks;
- refcount balance measured as wasm page growth over 20000 iterations, each
  against a control loop that deliberately retains its children — a flat page
  count means nothing unless the control grows.

It does not yet include a full php-src differential corpus or exhaustive EIR
shape, ownership, argument/environment/preopen, and process-status coverage.

### Measured parity against the example suite

Parity is tracked against the repository's own examples rather than a prose
claim. Of the 191 examples under `examples/` that carry a `main.php`, **53
compile to `wasm32-wasi`**, and every one of them except `ifdef`, `union-types`
and `enums` reproduces php-src's output byte for byte. Those three have no
php-src output to match rather than a different one: `ifdef` and `union-types`
use Elephc-only syntax php-src cannot parse at all — they are checked against the
NATIVE backend instead, and agree with it — and `enums` reaches the Elephc-only
builtin `SortDirection` enum, which php-src answers with `Class "SortDirection"
not found` while this target prints the rest, including the line php-src never
gets to. So every example this target compiles, it also runs correctly.

When comparing against php-src yourself, pass the script path as the module's
first WASI argument. php-src puts it in `$argv[0]` and counts it in `$argc`; a
host that starts the module with an empty argument vector makes both differ for
reasons that have nothing to do with the backend.

**30 of the 138 remaining examples will never compile here.** `stream_socket_*`,
sockets, FFI/`extern` calls, SDL, PDO drivers and the image extensions have no
WASI Preview 1 equivalent, so the realistic ceiling is about 161, not 191.

Of the rest, the work is a long tail rather than a few large levers. Counting
examples whose blockers are *entirely* contained in one subsystem — as opposed
to examples that merely mention it, which overstates every subsystem several
times over:

| Subsystem completed in full | Examples it alone unblocks |
|---|---|
| The whole file/stream family (38 functions) | 5 |
| Casts (`Mixed`-to-scalar, `IToStr`, truthiness) | 4 |
| Mixed containers (`array_get`/`array_set`/`iter_start`/`strict_eq`) | 2 |
| All three together | 17 |

Those figures are an upper bound, and the first row has since been measured
against reality: the eight core file functions removed `fopen`, `fread`,
`fwrite`, `fclose`, `unlink`, `file_exists`, `file_get_contents` and
`file_put_contents` from between 14 and 29 examples each, and unblocked **none**
of them outright. What it did do is move seven examples down to a single
remaining blocker. Two examples did start compiling and were then deliberately
refused again: both opened a `scheme://` stream wrapper, which this target does
not implement, so compiling them meant answering `false` where PHP answers a
handle.

The blocker-count distribution says the same thing. Counting the distinct
refusals a single `elephc build --target wasm32-wasi` reports per example: of the
138 that do not compile, four never reach this backend at all — `strict-php`,
`web-session`, `web-session-trans-sid` and `web-session-upload` stop in the type
checker on every target — and of the remaining 134, 15 have one distinct blocker,
16 have two, 13 have three, 28 have four, and the tail runs to 23. (Distinct
REASONS, not refusal lines: `fibers` reports 58 lines but only eleven different
gaps, and ranking by lines answers "how often is this hit", which is a different
question from "what would unblock an example".)

The fifteen reachable by a single blocker are `array-access-exception-order`,
`constructor-promotion`, `fsockopen`, `ftp-stream`, `hello-preg`,
`json-jsonserializable`, `logical`, `phar-writer`, `php-wrapper`,
`print_r-return`, `sdl_audio`, `socket-pair`, `stream-copy`,
`stream-get-contents` and `type-ops`. Seven of those are dead ends rather than
work: `sdl_audio`, `fsockopen` and `socket-pair` are among the thirty that never
will; `ftp-stream`, `phar-writer` and `php-wrapper` need stream wrappers this
target refuses on purpose; and php-src itself fatals on
`constructor-promotion`, so compiling it would match nothing.

Read that distribution with one caveat: a refusal counted once can stand for a
whole subsystem. The most frequent one used to be a catch-all — 47 examples
reported `missing typed runtime target` and nothing more, because every
`Op::RuntimeCall` without a typed immediate fell into the same arm. Naming what
those calls actually carry split the bucket, and serving its shapes one at a time
— the generic `$mixed[$key]` read (34 examples with a string key, 25 with an
integer one), then the narrowing of a boxed value into a declared class slot —
took it from 47 examples to **7**. What is left there is the coercion family on a
single boxed operand, and `ArrayAccess` on an object receiver.

Progress is otherwise roughly one example per fix, so the example counter is a
poor guide to correctness work — running a differential corpus against php-src
has been finding more, and more serious, defects than making the counter move.

Note also that the example suite is not a pure PHP corpus, so "matches php-src"
is not a question that can be asked of all of it. Measured with `php -n` over all
190: **10 do not parse under php-src at all** (`ffi`, `ffi-memory`, `hot-path`,
`ifdef`, `pointers`, `union-types` and the four `sdl_*`), because they use
Elephc-only syntax; and 25 more reach a fatal, most of them legitimately — an
Elephc-only builtin (`clamp`, `log2`, `grapheme_strrev`, `zval_pack`,
`class_attribute_names`), an extension this machine lacks (Imagick, Gmagick,
Cairo), or a service that is not running (the PDO driver examples).

### PHP semantics this target reproduces exactly

Behaviour that a naive lowering gets wrong, and that this backend implements
against measured php-src 8.5.6 output rather than by analogy:

- **File I/O, against a preopened directory.** `fopen`, `fread`, `fwrite`,
  `fclose`, `file_get_contents`, `file_put_contents`, `file_exists` and `unlink`
  reach the real filesystem. WASI Preview 1 is capability-based, so a module has
  no filesystem at all unless the host preopens a directory for it — `node`'s
  `preopens`, `wasmer --dir .` — and every path resolves against the first
  preopen. Without one, each answers PHP's failure value, which is also the right
  answer for a host that grants nothing. A stream handle is a boxed value
  carrying the WASI fd, so it lives in locals and arrays like any other. The mode
  string is PHP's: `r`/`w`/`a`/`x`/`c` with an optional `+`, and `b`/`t` read and
  ignored. `STDOUT`, `STDERR`, `STDIN` and the `php://stdout`, `php://stderr`,
  `php://stdin`, `php://output` names reach the process fds directly and need no
  preopen; `fclose` on one of them answers true without closing the fd, which is
  what php-src does for a `php://stdout` handle. Any OTHER `scheme://` path is a
  stream wrapper — `http://`, `ftp://`, `phar://`, `php://memory` — and a literal
  one is refused at compile time rather than opened as a filename, which would
  answer `false` where PHP answers a handle.
- **`$mixed[$key]` on a receiver whose type is only known at runtime.** Reading
  one element out of another (`$deep["db"]["host"]`) makes the inner read answer
  `mixed`, so the outer one dispatches on the cell's tag. An array or hash reads
  normally and warns `Undefined array key` on a miss, quoting a string key and
  not an integer one; a STRING is indexed by byte, counting a negative offset
  from the end and warning `Uninitialized string offset` past either end before
  answering `""`; and an int, float, bool or null warns before answering null.
  That last wording is version-profiled the same way the null receiver already
  was: before 8.3 PHP names the type for all of them, and from 8.3 it names the
  type for int and float but the VALUE for a boolean, so `on true` and
  `on false` are distinct messages there. The string and scalar receivers are
  two places the native backend answers a silent null and this one does not.
  A string indexed by a string key, and an object receiver, still answer null
  here as they do natively, rather than php-src's `TypeError` and `Error`.
- **`(float) $string`.** PHP parses the LEADING numeric prefix and answers
  `0.0` when there is none, silently. `"12abc"` is `12.0`, `"  .25e2xyz"` is
  `25.0`, `"0x1A"` is `0.0` because PHP reads no hex here, `"INF"` and `"NAN"`
  as text are `0.0`, an underscore stops the parse so `"1_000"` is `1.0`, and
  `"1e400"` answers `INF` rather than saturating. This is the same parser
  `(int) $string` routes float-form prefixes through, so the two spellings
  cannot disagree about what a string holds.
- **Coercion at a declared `int`, `float` or `bool` return.** This is not the
  `(int)` cast: returning `null` from a function declared `int` is a
  `TypeError`, and returning `5.7` deprecates before truncating. A non-numeric
  string, an out-of-range or non-finite float, a container, and an object each
  raise, with the object naming its class. `declare(strict_types=1)` performs no
  coercion at all and is refused rather than answered with the weak-mode result.
- **`__toString`.** `echo $obj`, `"x" . $obj` and `(string) $obj` call the
  method when the class is statically known, and raise php-src's
  `Error: Object of class X could not be converted to string` when the class
  provably has none. A subclass that overrides `__toString` leaves the site
  undecidable and is refused.
- **`count()` of a boxed value.** Raises php-src's own `TypeError`, which names
  a boolean by VALUE (`true given`, not `bool given`).
- **Relational comparison against a boxed value.** PHP compares without
  converting, so `"abc" <= 1` is false and `[1] <= 1` is false. A cast-then-
  compare lowering answers both incorrectly.
- **A float reaching a string.** `(string) $f`, `"$f"` and `echo $f` share one
  renderer, so all three agree: `-0` for negative zero, `1.0E+20` and `1.0E-7`
  in exponent form, `100` rather than `100.0` for an integral value.
- **`array_keys()` and `array_values()` of an associative array.** Both project
  in php-src's INSERTION order — `["z" => 26, "a" => 1]` answers `z`, `a` —
  never sorted, and never the bucket table's order.
- **A typed property with no default.** Reading one before it is written is
  `Error: Typed property C::$p must not be accessed before initialization`, and
  this backend has no sentinel for it: the allocator zeroes the slot and zero is
  a legitimate `int`. Such a read is refused unless the constructor writes the
  property unconditionally, which removes the question rather than answering it.
  The same reasoning retires PHP's implicit `= null` on an untyped property: once
  the constructor always overwrites it, no reader can observe the null, which is
  what makes `class Node { public $value; public $next; }` constructible here.
  Inside the constructor the store must be shown to come FIRST, which is a
  question about the individual read rather than about the class:
  `__construct(string $n) { $this->name = $n; echo $this->name; }` is admitted,
  and a read placed before the store is not.
  An ABSTRACT declaration cannot be instantiated, so what decides there is what
  its CONCRETE descendants do: with every one of them initializing the slot, no
  instance exists whose read could raise. One descendant that does not brings the
  refusal back for the whole hierarchy.
- **Reading an untyped property.** The read is an ownership question, not an
  opcode one. `ir_lower` stabilizes a borrowed result with an `Op::Acquire` and
  skips that when the result is already owned — which is what an untyped property
  gives — so borrowing unconditionally frees the cell the object still points at,
  and retaining unconditionally leaks one reference per read of a declared
  `string` or `array` property. The load asks the value which it is.
- **Writing a container after iterating it.** `foreach ($h as ...) {}` followed
  by `$h["c"] = 3;` is accepted: the iterator is dead by then. Only mutations the
  loop itself can reach are refused, since those have no PHP snapshot to write
  against. An iterator's live range ends where its own `IterStart` runs again, so
  a `foreach` nested in a `while` does not make the whole outer body untouchable.
  Refusal covers the mutations that are not written at the loop: every in-place
  builtin (`sort`, `shuffle`, `array_splice`, …), a call that RECEIVES the
  container — whether its parameter is declared `&` is not visible at the call
  site — and any reference bound to it, since an alias has its own slot and would
  otherwise pass every slot-keyed check.
- **Rendering an `array<int>` element.** A read of one answers an int-or-null,
  and PHP renders the null arm as the empty string, so `echo $a[0] . "|" . $a[1]`
  is exact — including the miss, which warns and contributes nothing.
- **`gettype()`.** php-src's historical spellings, not the type names PHP 8
  prints elsewhere: an int is `integer`, a float `double`, a bool `boolean`, and
  null `NULL` in capitals. A settled type answers at compile time; a boxed one
  reads the cell tag. A RESOURCE is refused — php-src distinguishes an open
  handle from `resource (closed)`, and the tag does not carry that. The check
  reads the DECLARED type rather than its codegen representation, since a
  resource represents as an integer and would otherwise answer `integer`.
- **`Foo::class` and `self::class`.** Resolved at compile time to a data-segment
  address, since the EIR already carries the class. `static::class` is refused:
  late static binding resolves it from the CALLED class, which this target does
  not forward, and answering the defining class instead would be wrong for every
  subclass.
- **`round($value, $places)`.** Not `round($value)` with a default argument.
  Scaling is inexact — `0.285 * 1e10` is `2849999999.9999995` — so php-src
  extracts the integral part and then repairs the extraction, which is why
  `round(1.005, 2)` is `1.01` and `round(9.995, 2)` is `10`. Scale-round-unscale
  gets both wrong. Transcribed from php-src 8.5's `_php_math_round` and validated
  at 1420/1420 over the halfway values, the classic traps, the 1e15 and 1e-15
  boundaries, both signed zeroes and 1200 random values across 24 orders of
  magnitude. A precision outside php_intpow10's exact 0..22 table is refused
  rather than answered nearly-right.

Known shared defect, measured and not yet fixed: PHP's `null` is dropped when a
possibly-missing array element crosses a **function return**. A direct read keeps
it — `$v = $a[5]; $v === null` is true — but `function pick($i) { return
$a[$i]; }` is typed `int`, so `pick(5) === null` answers false on BOTH backends
where php-src answers true. The checker infers the read as `int` while the EIR
produces `int|null`, and the signature is what the call site believes. Anything
reading that type inherits the wrong answer, `gettype()` included; patching it in
one consumer would hide it from the others.

**A Mixed value rendered as a string through a `??` merge** is admitted. Such a
cast is accepted when every consumer is a place PHP renders a string — echo,
concat, interpolation, `strlen` — and `echo $x ?? "d"` looked like it had no
consumer at all: the merge parks the value in a hidden slot and reads it back in
the merge block, so the cast's only direct uses were the `acquire`/`release` pair
the check rightly ignores, and "no string consumer" was indistinguishable from
"no consumer". The walk now follows an `acquire` to its result and a `store_local`
to every LOAD of that slot. Following every load is what keeps it sound: one load
reaching a non-string context still refuses the whole cast. Unblocks
`examples/union-types`.

**`foreach ($a as &$x) { $x += n; }`** writes back through the cell. `$x + 5`
types Mixed because the add can overflow into a float, while the cell it writes
through is the array's own `int`. The emitter already handled that narrowing; the
capability gate was the only thing refusing a store the backend could perform.
The native backend answers this shape correctly, so it was a WASM-only gap. What
it inherits is the EIR's widening gap rather than a new one: on a REAL overflow
the value is a float and narrowing it into an `int` cell is wrong, which the
native backend does there too — refusing the whole shape to avoid that would cost
every ordinary by-reference accumulate. Unblocks `examples/foreach-ref`.

**A NULLABLE SCALAR parameter** takes both the `null` literal and a concrete value.
A `?int` is an inline two-word `{payload, tag}` slot, so the literal `null` —
which arrives as `Void`/`I64` — is a conversion rather than a copy, and had no
classification at all. Null is tag 8 with an unread payload; a concrete value
becomes the payload and names its own tag.

What made that correct rather than merely compilable is the second half:
`is_null` required the operand's php type to literally be `TaggedScalar`, but a
`?int` PARAMETER carries its nullability in the declaration instead, so the test
fell through to a `statically non-null` fallback and answered false over a value
whose tag said 8 — `describe(null)` printed `NULL:` where php-src prints
`NULL:null`. The tag is the truth for any two-word scalar whatever the static
type calls it, which is the same correction the pointer arm needed. Unblocks
`examples/functions`.

**`Enum::cases()` and `Enum::tryFrom()`** are open-coded against the case
singletons. The refusal was `missing method body Color::cases`, which was true
and beside the point: PHP synthesizes both for every enum, so there is no
function to find on either backend. They are emitted directly instead — the same
treatment the `Throwable` accessors get — and audited against what the emitter
produces rather than against a body that will never exist.

`cases()` materializes every case in DECLARATION order into a pointer-slot array
under `value_type` 4, which is an ordinary `array<Object>`: `count()`, `foreach`
and an indexed read all reach it with no special case. `tryFrom()` walks the
cases as an equality ladder over the BACKING value and boxes the winner under
Mixed tag 6, a miss boxing null under tag 8 — which is what makes `?? Default`,
`is_null` and `===` answer the way php-src does. For a string-backed enum the
length is compared first and separately, since the byte comparison reads the
case's length from the needle and would otherwise run past a shorter one.

`from()` stays refused: it must raise php-src's `ValueError` naming the enum and
the offending value when nothing matches, and answering it without that raise
would turn a fatal into a wrong value. Unblocks `examples/enums`.

**An element that is itself an ARRAY or an OBJECT** can be read out of an indexed
array. This was refused because of the MISS rather than the hit: every accepted
element type had somewhere to put PHP's missing-key null — an int became a tagged
scalar, a bool/string/mixed became a Mixed cell — and a raw container pointer
appeared to have nowhere. It does: pointer 0, the same null the native backend
produces, and one no live array or object can collide with since both are
allocated.

Three things had to hold together. The getters had to survive a null SOURCE, now
that a read chained onto a miss passes one — they were loading the length straight
off the pointer, which for 0 reads address 0, valid linear memory in wasm, so it
would have answered from whatever was there rather than trapping. PHP's two
diagnostics had to be told apart, which is decided by the source and not the
result, since both give null: a missing key off a real array is `Undefined array
key N`, and any offset off a null is `Trying to access array offset on null`. And
`is_null` had to stop answering from the static type, whose `statically non-null`
fallback the EIR cannot back — a missing element of `array<array<int>>` is typed
`array<int>`, the null having been dropped at that boundary, so a pointer that IS
0 called itself non-null.

The last of them separated reading memory from reading FREED memory. The getter
hands its pointer back BORROWED while the EIR types the result `own=owned` and
emits a matching `release`, so without a reference of its own that release dropped
the PARENT's: `$g[1][0][1]` freed a child its parent still pointed at, where
`$x = $g[1]; count($x)` did not, the chained form having no `acquire` to balance
it. Verified balanced by measurement, not by reading: 20000 build-then-read
iterations hold at 3 wasm pages, exactly like the same loop without the read,
while a loop that deliberately retains every child grows to 32.

WRITING one back works the same way, with one addition the read does not need:
the slot's previous occupant is a refcounted child, so overwriting it has to
RELEASE it. That release happens AFTER the store, which is what makes
`$a[0] = $a[0]` safe — the incoming pointer is already increfed, so a
self-assignment nets to no change instead of freeing the value mid-write.
Measured the same way: 20000 overwrites of one slot hold at 3 pages against the
retaining loop's 32. Together these unblock `examples/nested-arrays` and
`examples/cow`, the latter matching php-src's copy-on-write output byte for byte.

Two CONSUMERS of that null needed the same treatment, and a multi-model audit
(GLM 5.2, Kimi K2.7, Kimi K3) is what surfaced them. Writing through the null was
the worse one, because it answered rather than stopped: php-src AUTOVIVIFIES
silently, building a fresh array, where this backend exhausted memory treating
address 0 as an array header. The setters now build the array php-src builds.
Calling a method on the null terminated either way but named the wrong thing —
`Invalid callable dispatch`, the dispatch ladder's fallthrough trap, instead of
php-src's `Call to a member function hi() on null`. A raw object pointer used to
be non-zero by construction, so nothing checked; the native backend already
answered this correctly from the same EIR, so the guard is parity rather than
caution.

A second audit round found two more, both in the round-one fixes. The
null-receiver check had been fused to the dispatch load, which misses the
open-coded `Throwable` accessor — that path resolves and RETURNS before
dispatching, so `$e = $a[9]; $e->getMessage()` read address 0 plus the property
offset, printed an empty message and carried on. The check is now separate from
the load and runs before every path. And a container write with a NEGATIVE index
leaked: the setters reject one, because php-src stores a negative KEY there and a
dense array has no slot for it, but the caller increfs the child before the call,
so returning without storing stranded it — 24 wasm pages over 20000 such writes,
now 3.

A third and fourth round pushed on the same seam, which is where the null that
the EIR drops meets consumers that trust the type. `count()` of the missed
element answered `4295050542` off address 0 and carried on, where php-src raises
`TypeError: count(): Argument #1 ($value) must be of type Countable|array, null
given` and terminates; it now raises exactly that. And `tryFrom`'s needle width
was read off the case list rather than the enum's declared backing type, so
`enum E: string {}` — a backed enum with no cases, which is legal PHP — took the
int path and left half a string on the operand stack.

Reading a PROPERTY through the missed element was the last of that seam. It used
to answer a bare `1` off address 0 with no diagnostic at all; it now emits
php-src's own `Attempt to read property "age" on null`, in php-src's order, and
substitutes a defined value instead of whatever the data segments start with. The
VALUE is still not php-src's — the read evaluates to null there, and the EIR types
the result a non-nullable `int` — so the substitute is the same null sentinel the
NATIVE backend leaves in that slot, which puts the two backends in exact agreement
on a null neither can type. A string property answers the empty string and a
container property a null pointer, both of which ARE php-src's answers.

The warning helper lives in the command runtime, so a reactor/library module —
which has no `main`, no stderr contract and no `__rt_warn_*` at all — emits the
plain load exactly as before. That is the same line the warning-producing indexed
read already draws.

Writing PAST the end is a known shared gap that neither of these caused. PHP
treats `$a[3]` on a one-element array as a SPARSE key, so `count()` is 2; a dense
representation with no occupancy bit fills the gap with nulls and answers 4.
Measured identically on the scalar setter that predates this work, so the
container setter matches it rather than adding a second wrong answer.

**A method call on an INTERFACE-typed receiver** dispatches on the runtime class.
This was refused as an `unknown receiver class`, which named the wrong problem: an
interface is not an unknown class, it is a class-less one. It declares no storage
and owns no body, so there is nothing to call into — but the object that arrives
carries its real class id in its own header, which is exactly what PHP dispatches
on. The callee is therefore the closed set of concrete implementors, and that set
is enumerable at compile time, so these calls now go through the same class-id
if-ladder that already serves virtual calls, with the implementors as its arms.

Membership is walked in both directions, because PHP hands a class its parents'
interfaces and an interface its parents' methods: `class C extends B` where `B
implements J` and `interface J extends I` makes a `C` a legitimate `I`. Reading
only a class's own `implements` list would miss every implementor that inherits
the obligation. Enum cases and anonymous classes are ordinary implementors here.

Two shapes stay refused, both because the implementors cannot share one stub, and
both by the AUDIT rather than by the stub emitter — the emitter skips an interface
it cannot serve, so an audit that accepted either would leave the module calling a
function that was never defined. An interface with no concrete implementor has no
arm to select. And an interface method with no declared return type lets each
implementor pick its own, so a `void` body and an `int` body can differ in return
ARITY and unbalance the wasm stack; that is caught even when the call discards the
result. Unblocks `examples/intersection-types`, `examples/anonymous-classes` and
`examples/enum-methods`.

**A subclass argument** may bind to a parameter that declares one of its
ancestors. The capability audit compares an argument's representation against the
parameter's, and two object types with different class names read as two
different representations — so `look(Base $x)` called with `new Kid()` was
refused, though PHP's whole inheritance story is that the call is legal. The
refusal made a representation claim that is not true: an object is ONE pointer to
a header naming its own runtime class, which is exactly why `instanceof` and
virtual dispatch both answer off the value rather than the static type. Two object
pointers are therefore copy-compatible at the physical layer, and whether a given
class may stand in for another is decided where the hierarchy is in scope — in
the audit, which walks `parent` links and so admits a descendant and nothing
else. An interface-typed parameter is a different question, answered by the
interface dispatch described above rather than by this rule. Verified byte-identical to
php-src 8.5.6 for inherited field offsets under a subclass that appends fields of
other representations, virtual dispatch of an overridden method through an
ancestor-typed parameter, a write through such a parameter landing on the base
field, three-level hierarchies, and one callee body serving both classes across
repeated calls. Unblocks `examples/instanceof`.

**By-reference container parameters** are supported. A by-ref argument arrives as
a ref-cell pointer, so the callee loads it with `LoadRefCell` rather than
`LoadLocal`; the writeback recognised only the latter, so a callee that MOVED the
array — `$a[] = 41` growing it past its capacity — had nowhere to put the new
pointer and dropped it. That answered `106808` where php-src answers `2`, and the
`$a[0] = 41` form answered `0` for `1`. Writing the pointer THROUGH the cell when
the slot is ref-bound repairs appends, offset writes, associative keys, wholesale
reassignment, several by-ref parameters at once, nested by-ref calls, repeated
calls and by-ref callees that also return a value — all verified byte-identical
to php-src 8.5.6.

One narrower shape stays refused, and it is a shared defect rather than a WASM
one: a callee that **replaces the container's representation**. `$a[] = $i` where
`$i` came from `$i++` appends a `mixed`, because the increment can overflow into
a float; EIR then widens the whole array with `ArrayToMixed` and stores the wider
array back through the cell. The caller receives the new pointer but keeps the
`array<int>` element type it passed in, and reads 24-byte Mixed cells as a dense
i64 buffer. `count()` stays right — the length field IS shared — so nothing
announces the mismatch. The native backend prints the same raw heap addresses
from the same EIR, so the gap is upstream: the callee's post-condition never
reaches the call site's type facts. This target refuses the call rather than
answering garbage.

A shared front-end bug surfaced while measuring this and is fixed: an
`if`/`elseif`/`else` chain whose FIRST condition folded to false propagated the
`else` branch's constants, ignoring the elseifs entirely. Both backends then
answered from the wrong branch — silently, since the branch itself was lowered
correctly and only the propagated fact was wrong.

**A dynamic dispatch is no longer vetoed by an unrelated namesake.** A `mixed`
receiver names no class, so the ladder's arms come from every class declaring the
method — and the audit then demands that every one of them lower, which let a
bystander refuse a whole program. Two filters now narrow that list before the
audit sees it. The receiver's STATIC type is the first: an `int|Money` receiver
admits `Money` and its subtree, so a prelude `DateInterval::format(string)` is not
an arm of `$money->format()`; a narrowing that empties the list is treated as the
type telling us nothing and the unnarrowed list stands. The call's ARITY is the
second, and it is what saves a plain `mixed`, whose type narrows nothing: a class
php-src would not ENTER with this many arguments cannot be the right arm.

That filter runs one way only. Measured on php-src 8.5.6, a user method accepts
SURPLUS arguments silently — `C::m(int $a)` called with two runs, and
`func_num_args()` sees both — so passing too many disqualifies nothing. An upper
bound here was a miscompile, not a missed refusal: it dropped a class php-src
dispatches to, and the ladder answered `Call to undefined method Money::show()`
for a call that should have printed.

Dropping a class does not mean forgetting it. Each one keeps an arm raising PHP's
own `ArgumentCountError`, with php-src's count and wording — `exactly N` when
every declared parameter is required, `at least N` when a default makes the
counts differ, and (measured) `exactly` still when the tail is VARIADIC. Left to
the ladder's fallthrough it would have reported `Call to undefined method` for a
method that plainly exists; the native backend does worse and says `Call to a
member function show() on null`. Only the location tail and stack trace are
missing, the same convention the other composed fatals follow.

Fixed alongside it: a ladder with two or more `void` arms stored its result from
an empty stack. When every candidate returns `void` the checker types the call
expression `I64 php=null` rather than boxing it, and the arm has to supply the
null the callee never pushed — which the direct and interface paths already did.
The module failed WebAssembly validation outright rather than miscompiling.

**`===` and `!==` against a nullable int** are lowered. This target stores a `?int`
as an inline `{payload, tag}` PAIR rather than one word, and the pair was refused
outright — which turned away every `$x === null`, the single most common thing
anyone writes with one and the largest strict-comparison gap in the suite. Its tag
is 0 or 8 and nothing else (`codegen_repr` folds only `int|null` to this
representation), which is what makes each case decidable rather than approximate:
against a string, a bool or a float the answer is a compile-time FALSE, because a
`?int` holds none of those and `===` compares the type first. Against a concrete
int the TAG is checked before the payload, so `$x === 0` does not answer true for a
null whose payload word also happens to be zero.

What the pair may not meet is a runtime-tagged `Mixed` cell — that cell's own tag is
dynamic too, and the existing mixed/concrete path assumes exactly one side is —
so that pair stays refused. Producing the slot needed two more pieces: an untyped
runtime call WIDENING a concrete scalar into it (a widening cannot lose information
or raise, unlike the narrowing in the other direction, which stays refused), and
`const_null` writing both components instead of the one-word sentinel. The latter
was a latent invalid-module bug, reachable only once the widening was admitted.

**A boxed object reaches `__toString`** instead of being refused outright. The
object arm of every string conversion raised `Error: Object of class C could not
be converted to string` for EVERY class, which is php-src's answer only for a
class that does not define the method — measured, `(string)$tag` prints `<em>`
there and fatally raised here, a wrong ANSWER in shipped code rather than a
refusal. The conversion now goes through a runtime class-id dispatch, so a class
defining `__toString` converts and one that does not still raises. All three
ownership shapes are covered: a literal returns a data-segment pointer that must
not be released, a property read returns storage the object still owns, and a
concat returns a fresh heap string.

That dispatch only carries classes whose `__toString` has a BODY in the module,
which is what excludes most of the prelude: `Exception`, `SplFileInfo` and the
`Reflection*` family all declare the method with no body this target can call, so
they still reach the fatal. That is unchanged from before rather than new, and it
is the same set the `Throwable` accessors already approximate.

**A boxed value reaching a builtin's declared `string` parameter** is coerced
PHP's way. This is a THIRD implicit `Str` conversion, distinct from both the
explicit `(string)` cast and the one an echo performs, and the difference is not
cosmetic. Measured on php-src 8.5.6 for `strtoupper($mixed)`: a string, int,
float or bool converts exactly as `(string)` does; `null` converts to `""` but
raises `Deprecated: strtoupper(): Passing null to parameter #1 ($string) of type
string is deprecated`; and an array — which `(string)` would have turned into
`"Array"` with a warning — does not convert at all, it is `TypeError:
strtoupper(): Argument #1 ($string) must be of type string, array given`. An
object contributes its CLASS name to that message, and a closure the word
`Closure`.

Both names in those messages come from the builtin registry, which the EIR
reaches through the call's runtime target. A target several PHP names share is
refused rather than guessed — `count` and `sizeof` reach the same one, and
php-src reports the name AS WRITTEN, which the target cannot recover. Unblocks
`examples/pipe-operator`.

**A boxed value reaching a slot declared as a class** is narrowed back to an
object. A `?Node` property is stored boxed, so `return $this->node;` from a method
declared `: Node` moves a `Heap(Mixed)` into an object slot through a call that
carries no runtime function id at all — the frontend leaves the conversion
implicit, and the audit refused it as `missing typed runtime target`. The lowering
unboxes the cell and takes its payload, with a tag guard the native backend does
without: a cell holding anything but an object yields a null pointer rather than a
scalar reinterpreted as an address. Unblocks `examples/type-narrowing`.

Two shared defects found the same way remain OPEN, both in the checker rather
than in this backend:

- An untyped property narrowed from its null default to a concrete class drops
  the null it still holds, so `$a->next === null` answers false for a slot that
  is null, and `$cur = $head; while ($cur !== null) { $cur = $cur->next; }` does
  not terminate on the native backend. A declared `?N $next = null` is correct in
  both backends, so the narrowing rather than the storage is at fault. Giving the
  slot union storage fixes the answer but stops a self-referential cycle from
  being collected, and a leaked cycle is not an improvement on a wrong answer, so
  the change was reverted rather than shipped half-right. This target refuses the
  shape.
- The checker analyses a loop condition once, with the environment from before
  the loop, so a local reassigned in the body is typed from its pre-loop value in
  the condition. The straight-line form of the same walk is correct.

Known divergence: an uncaught error prints a class-agnostic fatal, and
diagnostics omit php-src's ` in <file> on line <n>` tail. A `TypeError` raised
by a coercion or by `count()` is a deterministic fatal rather than a catchable
throw, because a message composed at runtime cannot be resolved from the static
error table.

To select it:

```bash
elephc --target wasm32-wasi hello.php
elephc --target wasm32-wasi --emit npm hello.php
node hello-npm/index.mjs
```

The NPM form writes `hello-npm/` with `module.wasm`, an ESM loader
(`index.mjs`), TypeScript declarations, package metadata, and a README. The
loader can also be imported:

```js
import { run } from "./hello-npm/index.mjs";

const exitCode = await run({
  args: ["hello", "first-argument"],
  env: process.env,
  preopens: { "/work": process.cwd() },
});
```

WASM output is currently a WASI command (`_start`); `--emit cdylib` reactors are
rejected with a focused diagnostic.

Native-only compiler options are also rejected with focused diagnostics instead
of being ignored: web-server mode, native source maps/DWARF, native heap and
register-allocation controls, native linker/framework flags, and bridge-crate
linking are not yet available on `wasm32-wasi`.

The parser also recognizes `macos-x86_64` / `x86_64-apple-darwin` and
`windows-x86_64` / `x86_64-pc-windows-msvc` /
`x86_64-pc-windows-gnu`. These spellings are groundwork for future backends,
not supported targets: compilation stops with an explicit unsupported-backend
diagnostic.

## Cross-compilation notes

Selecting a native target different from the host produces assembly and an
object file for that target. Producing a final linked binary still depends on
having a linker and any target libraries available for that platform; the
elephc test suite uses the Docker scripts under `scripts/` to build and run the
Linux targets from a macOS host. `wasm32-wasi` instead follows the artifact and
host workflow described above.

For the target-aware ABI and runtime details behind each platform, see
[Architecture](../internals/architecture.md) and
[The Code Generator](../internals/the-codegen.md).
