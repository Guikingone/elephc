---
title: "Mixed/String Ownership Debts"
description: "Findings on the heap blocks stranded by string materialization: what was fixed, what remains, and why the remaining fix is not uniform."
sidebar:
  order: 13
---

This document records an investigation into heap blocks that are allocated while
materializing PHP strings and then never reclaimed. Two of the paths are fixed; the
largest one is diagnosed but deliberately left open, because the obvious fix is a
use-after-free rather than a leak fix.

Every claim below is tagged **[measured]** when it was reproduced by running a compiled
program, or **[read]** when it comes only from reading the source. That distinction is
load-bearing: several confident-sounding conclusions in this area turned out to be wrong
when executed, and they are called out in [Pitfalls](#pitfalls-for-whoever-picks-this-up).

All measurements were taken on macOS aarch64 with `--heap-debug`, against
`php -d xdebug.mode=off` (PHP 8.5.6) as the oracle.

## Background: two allocators, one pointer

Several runtime helpers return *either* a borrowed slice of the 64 KiB concat arena
(`_concat_buf`) *or* an owned `__rt_heap_alloc` block, decided at run time by whether the
data outgrew the arena. The caller cannot know at compile time which it received.

This is safe to reclaim unconditionally because `__rt_heap_free_safe` separates the two
shapes *exactly* rather than heuristically: `_concat_buf` and `_heap_buf` are distinct
`.comm` objects (`src/codegen_support/runtime/data/fixed.rs`), so an arena pointer can
never satisfy the managed-heap window test and passes through untouched.

Consequently every fix in this area needs an **arena-sized negative control** alongside
the oversized fixture: freeing a borrowed arena slice would be a wild free, which no
leak-only assertion would catch.

## Fixed

### `stream_get_contents()` boxed into a Mixed cell

`stream_get_contents()` is typed `string|false`, so its result is boxed into a Mixed cell
inside the lowering itself. The boxing **copies rather than adopts**:
`__rt_mixed_from_value` tag 1 goes through `__rt_str_persist`, which always allocates and
byte-copies. The slurped block was therefore left with no owner — and, because no EIR
value ever names that pointer, with no `release` either.

**[measured]** A 500 KB stream stranded one 524288-byte block per read: `live_blocks=4
live_bytes=2097216` over four reads, versus `clean` after the fix.

Fixed by `box_read_all_result_and_release_source` in
`src/codegen/lower_inst/builtins/io.rs`, called at both read-all lowering sites.

The release is correct here **only because no EIR value names the pre-box pointer**. It
must not be copied to a builtin typed plain `string`: `fread()` is exactly that shape, and
there the EIR release already reclaims the block, so a second free would be a double free.

### The six `file_get_contents()` URL branches

Same shape one level down. Each branch (http / https / ftp, on both architectures) slurps
the response with `__rt_stream_get_contents`, persists it with `__rt_str_persist` for
ownership, and then drops the slurped pointer.

**[measured]** A 200 KB body cost one block of roughly the doubled accumulation capacity
per read (`live_blocks=1 live_bytes=262076` for a single read, linear in call count).

Fixed by `emit_release_slurped_source` in
`src/codegen_support/runtime/io/file_get_contents_url.rs`, applied at all six sites. The
persisted copy is parked on the temporary stack rather than in a frame slot, so the helper
stays independent of each branch's differing frame layout.

## Open: the `Mixed` → `Str` cast leak

This is the large one. It affects a family of builtins, not a single function.

**[measured]** Passing a Mixed/union-typed value where a builtin declares a `string`
parameter strands **one heap block per call, sized to the whole source string** — not to
the result. A 4095-byte source leaks a 4112-byte block per call, strictly linear in call
count. Program output stays correct throughout, so only heap accounting reveals it.

### Correcting the obvious wrong turn first

**[measured]** `Op::MixedCastString` is **dead code**: it has zero construction sites in the
tree. Grepping the EIR for it finds nothing and invites the conclusion that this path does
not exist. The instruction actually emitted is `Op::Cast` with
`Immediate::CastTarget(IrType::Str)`, produced at `src/ir_lower/expr/mod.rs:14742`, and the
EIR prints it as `cast vN Str`. Several ownership allow-lists name the dead variant
instead of the live one, including `src/codegen/lower_inst/ownership.rs`; they are inert.

**[read]** The `alloc_concat` effect on `Op::Cast` is not a blanket lie either. Only the
tag-1 (already-a-string) arm heap-allocates, via `__rt_str_persist`
(`src/codegen_support/runtime/arrays/mixed_cast_string.rs`, both architectures). Tags 0/2/3
go through `__rt_itoa` / `__rt_ftoa`, which do write the arena. **[read]** The helper has
no object arm at all — `__toString` is dispatched at the call site
(`src/codegen/lower_inst/conversions.rs`) before the helper is reached.

**[read]** The `ALLOC_CONCAT` / `ALLOC_HEAP` effect bits are read by nothing. The only
predicate that inspects them puts both in the same set and is never called. Correcting the
annotation alone would change no behaviour.

### The decision site

There is **no liveness or dataflow pass**. Releases are emitted eagerly at lowering time by
a syntactic producer classifier (`LoweringContext::value_is_owning_temporary`,
`src/ir_lower/context.rs`). That classifier *does* recognise the cast value as owning. One
specific consumer then decides to skip its release:

`src/ir_lower/expr/mod.rs:13413-13431` — the `continue` at **line 13427** drops the release
when both of:

1. `return_alias.may_alias_parameter(idx)` — true whenever `result_ownership` is
   `MayAliasArguments`, which is the *default* (`src/ir/runtime_fn.rs`), mapping to
   `ReturnArgAlias::Unknown`, which answers true for every index; and
2. `call_result_may_alias_arg(...)` — true when the resolved result type is
   `Str`/`Mixed`/`Union` (`src/ir_lower/context.rs`).

**[measured]** The same producer leaks or not depending purely on the *consumer*. In
`$s = (string)$m;` the cast value is an assignment source, so it goes through `store_local`
(`src/ir_lower/context.rs`), which has no alias guard at all — there the release *is*
emitted and nothing leaks. In `substr($m, -3)` it is a call argument, so it hits the guard.

### Why a uniform fix is a use-after-free

The guard is **legitimate** for part of the family. Those builtins return an *interior
pointer into the cast buffer*:

- **[read]** `substr` computes `add x1, x1, x0` (`src/codegen/lower_inst/builtins/strings.rs`,
  x86 counterpart nearby) — the result points inside the source.
- **[read]** `trim` returns "the adjusted borrowed string slice … No heap allocation"
  (`src/codegen_support/runtime/strings/trim.rs`).
- **[read]** `strstr` is the same shape.

Releasing the cast copy right after the call would destroy the storage the result points
at. For these, closing the leak requires **extending the lifetime** — releasing after the
result has been consumed or persisted — not adding a release.

The rest are **false positives** of the guard: their result is freshly built, so releasing
the source is safe. They are classified `MayAliasArguments` only because they were never
added to the `Fresh` list. **[read]** `ucfirst` builds through `__rt_strcopy` into the
arena; `md5`, `str_replace`, `str_pad` likewise produce fresh output.

**[measured]** Builtins that do *not* leak, and why: `strtoupper` / `strrev` route through
`UnaryString` ⇒ `Fresh`; `strpos` / `explode` are already in the `Fresh` list; `strlen` /
`str_contains` have non-heap results so the second conjunct is false.

**[measured]** A three-way discriminator isolates both conjuncts cleanly:
`htmlspecialchars($m)` (`Independent` ⇒ no alias) is clean; `str_pad($m, 4096)`
(`MayAliasArguments`, Str result) leaks; `strcasecmp($m, "zz")` (`MayAliasArguments`, Int
result) is clean.

### Suggested approach

Correcting the aliasing classification per builtin is the tractable half: builtins whose
result provably does not alias their string argument can be moved out of the default
`MayAliasArguments`, which makes the existing release fire with no lifetime change. Two
cautions: `result_ownership` is read by other passes, so the change is not purely local;
and each entry needs its result-aliasing proven from the lowering, not assumed.

The genuinely-aliasing half (`substr`, `trim`, `strstr`, …) needs a real lifetime
extension and should be treated as separate work.

## Other defects found during this investigation

None of these are fixed. All were found while auditing the paths above.

### `phar://` dynamic reads corrupt data

**[measured]** This is data corruption, not just a leak. `__rt_phar_read_entry`
(`src/codegen_support/runtime/io/phar_read.rs`, both architectures) returns the
`__rt_stream_get_contents` pointer **verbatim, without `__rt_str_persist`**. When the entry
fits the arena, that is a `_concat_buf` slice, and `box_owned_string_or_false_result`
(`src/codegen/lower_inst/builtins/io.rs`) writes it into the Mixed cell as if it were
owned. Subsequent concat traffic overwrites it in place.

Reading a 100-byte phar entry, running 200 concatenations, then printing it yields
`xxxxxxxxxx` instead of the content. Only the **dynamic** `phar://` route is affected; the
literal route goes through `lower_literal_phar_file_get_contents` and is correct. This is
the only branch of `__rt_file_get_contents_maybe_url` that does not honour the "owned heap
string" contract every other branch honours.

### `hash_file()` on a phar entry leaks, and can exhaust the heap

**[measured]** `lower_hash_file_*` passes the bytes to `__rt_hash` and never releases them:
two blocks per call. With a 2 MB entry the second call dies with `Fatal error: heap memory
exhausted`, where PHP completes.

**[read]** One of those two blocks comes from a second defect: the assembly fallback in
`__rt_phar_read_entry` reads the entire archive and its epilogue returns without releasing
it. That fallback is only taken because `publish_dynamic_phar_function_pointers` is called
solely from `lower_file_get_contents`, never from `lower_hash_file_*`.

### Bounded `stream_get_contents($f, N)` still overruns the arena

**[measured]** The `lseek`-based bound that fixed `fread()` was never applied to the bounded
arm. It still copies into `_concat_buf + start + total` with `total` bounded only by the
caller's requested length, against a fixed 64 KiB `.comm`. Reading with a large `N`
overruns the arena and clobbers the runtime's stream table, surfacing as
`Fatal error: Uncaught TypeError: fclose(): Argument #1 ($stream) must be of type resource,
unknown given`. The read-all arm has both a fix and a regression test; the bounded arm has
neither.

### Indexing a `stream_get_contents()` result returns empty

**[measured]** `$body[0]` yields an empty string while `$body === $expected` and
`strlen($body)` are both correct on the same value. Reproduced on the compiler *before* the
fixes in this document, so it is pre-existing and unrelated to them.

### Deep `===` on a Mixed plus a later `unset()` leaks per iteration

**[measured]** Isolated with a four-way matrix: a deep `===` comparison alone is clean, an
`unset()` alone is clean, and the two together leak one block per loop iteration. Program
output stays correct in all four cells.

### Large top-level strings are never reclaimed at exit

**[measured]** A top-level `Str` local holding a heap block (e.g. `str_repeat("Q", 500000)`)
survives to process exit, and `unset()` does not release it. This is mostly a nuisance for
test authoring — it pollutes `--heap-debug` totals — but it is worth knowing.

## Pitfalls for whoever picks this up

These cost real time during this investigation.

- **Byte totals cannot discriminate which block leaked.** `__rt_str_persist` rounds to the
  next power of two, and the accumulation buffer doubles from 65536. For any leaking case
  both are the *same size*, so "the arithmetic proves it was buffer X" is never a valid
  argument. Count blocks and vary one factor at a time instead.
- **Discard mode gives false zeroes.** Measuring a builtin whose result is unused can
  report no leak purely because the call was dead-code-eliminated. Consume the result.
- **A length-only assertion cannot detect free-before-copy.** Releasing the source before
  the copy completes still yields the right length. Fixtures must read the bytes; the ones
  added here use `str_starts_with` / `str_ends_with`.
- **Do not build fixtures out of leaky helpers.** `substr` and `md5` on a Mixed leak (this
  document's open debt), and a large expected-value local never gets reclaimed. Either will
  make a heap-clean fixture impossible to satisfy for reasons unrelated to what it tests.
- **Treat any builtin sweep as a lower bound.** An automated sweep performed during this
  work missed at least one leaking position (`wordwrap`'s third parameter) and produced
  bucket counts that could not be reproduced from its own artifacts. Re-derive from the
  registry rather than trusting a list.
- **macOS absorbs double frees.** A clean heap-debug run on macOS aarch64 does not prove the
  absence of a double free; only the linux-x86_64 heap-debug path escalates it to a fatal.

## Verification status

The two fixes were proven failing-before and passing-after with exact figures
(`4 blocks / 2097216 B` and `1 block / 262076 B`, both to `clean`), each with an
arena-sized negative control that passes on both sides.

Suites run green on the changed paths: 662 tests across `runtime_gc`, `io::streams`,
`io::filesystem` and `io::files`, plus 1110 library, 1181 binary, 8 parity and 4 cdylib
tests. The full `codegen_tests` binary was not run to completion on the development host —
its `eval` module exhausted local disk twice — but reported zero failures across the 4533
and 1338 tests it did execute before being killed.

**Outstanding: the x86_64 arm has never been executed.** Emission was verified (the correct
frame slots appear at all six URL sites), but no linux-x86_64 `--heap-debug` run was
performed, and per the pitfall above that is exactly the platform that would catch a double
free. That run should be treated as required before considering this work complete.
