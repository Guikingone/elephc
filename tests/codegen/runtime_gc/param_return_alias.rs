//! Purpose:
//! Ownership regressions for issue #992: a function that passes its parameter to a call and
//! also returns that parameter must not leave the caller releasing a reference nobody acquired.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A call site lends the argument at `+0` and releases the RESULT. It skips that release only
//!   when `ReturnArgAlias` proves the result is the argument, so every loss of precision in that
//!   summary turns into a release of borrowed storage — `bad refcount` under `--heap-debug`, and
//!   silent corruption without it.
//! - The reported repro blamed a *conditional* return of a *mixed* parameter holding an *object*.
//!   None of the three is the discriminator: an `int` payload, a declared class type and a single
//!   trailing return all fail the same way. What is required is that the body hand the parameter
//!   to a call, and that the argument reach the callee through a local rather than nested
//!   directly (a nested call result is an owning temporary, which takes the other path).
//! - Two call sites are also required: one call is inlined, which removes the boundary entirely.
//! - The same shape reached through three other CALL SPELLINGS reproduced issue #992's own
//!   `bad refcount` fatal, because only a positional call to a free function consulted the
//!   parameter modes: a named argument was classified non-positional and unreadable, and a
//!   method or static-method call wiped every tracked provenance outright. All three printed
//!   `Fatal error: heap debug detected bad refcount` where PHP prints `ok`.
//! - A proven parameter return does NOT make a BOXED result borrowed. When the argument is an
//!   unboxed slot and the parameter is `mixed`, the boundary boxes it, so the cell the callee
//!   hands back is a fresh allocation even though the summary is right about the PHP value.
//!   Treating it as borrowed left that box unreleased — 2 heap blocks per call, measured against
//!   a flat line at the merge-base, and invisible to the fixtures above because they carry int
//!   and object payloads reached through a `mixed`-returning producer rather than a bare slot.
//! - These fixtures assert `leak summary: clean`, which is the honest bar here — unlike an
//!   `eval()` fragment, a compiled program does end clean, so both a premature release and a
//!   skipped one are visible.

use crate::support::compile_and_run_with_heap_debug;

/// Reads `live_blocks=` out of a `--heap-debug` exit summary.
fn live_blocks(stderr: &str) -> i64 {
    stderr
        .lines()
        .find_map(|line| line.split("live_blocks=").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|n| n.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().ok())
        .unwrap_or_else(|| panic!("no live_blocks in: {stderr}"))
}

/// Verifies the reported shape: a `mixed` parameter inspected by a type predicate and then
/// returned, reached through a local, called more than once.
#[test]
fn test_parameter_returned_after_a_builtin_call_stays_balanced() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Tag { public string $s = "tag"; }

function pick(int $i): mixed {
    if ($i > 100) { return "never"; }
    return new Tag();
}

function ident(mixed $value): mixed {
    if (is_object($value)) { return $value; }
    return $value;
}

for ($i = 0; $i < 5; $i++) {
    $tmp = pick($i);
    $value = ident($tmp);
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected the lent argument to survive its own call, got: {}",
        out.stderr
    );
}

/// Verifies the discriminator is the call, not the conditional return: the same body with one
/// trailing return, and with the builtin's result discarded entirely, is the same defect.
#[test]
fn test_parameter_returned_after_a_discarded_call_stays_balanced() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function pick(int $i): mixed { return 7; }

function identA(mixed $value): mixed { $x = is_object($value); return $value; }
function identB(mixed $value): mixed { is_object($value); return $value; }
function identC(mixed $value): mixed { if (strlen(gettype($value)) > 0) { return $value; } return $value; }

$n = 0;
for ($i = 0; $i < 5; $i++) {
    $tmp = pick($i);
    $n = $n + (identA($tmp) === 7 ? 1 : 0);
    $n = $n + (identB($tmp) === 7 ? 1 : 0);
    $n = $n + (identC($tmp) === 7 ? 1 : 0);
}
echo $n;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "15");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a discarded inner call to leave the parameter alone, got: {}",
        out.stderr
    );
}

/// Verifies the payload type is irrelevant: a string, an array and a bare int all fail
/// identically before the fix, so none of them may regress.
#[test]
fn test_parameter_returned_after_a_call_stays_balanced_for_every_payload() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function pickStr(int $i): mixed { if ($i > 100) { return 1; } return str_repeat("x", 3); }
function pickArr(int $i): mixed { if ($i > 100) { return 1; } return ["a", "b"]; }
function pickInt(int $i): mixed { if ($i > 100) { return "never"; } return 7; }

function ident(mixed $value): mixed { is_object($value); return $value; }

$n = 0;
for ($i = 0; $i < 5; $i++) {
    $a = ident(pickStr($i));
    $b = pickArr($i);
    $c = pickInt($i);
    $n = $n + strlen($a) + count(ident($b)) + ident($c);
}
echo $n;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "60");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected every payload shape to stay balanced, got: {}",
        out.stderr
    );
}

/// Verifies a USER function in the body is handled like a builtin. Before the fix any call to a
/// source-declared function wiped every tracked provenance, not just the argument's, because an
/// unknown callee and a known one were not distinguished.
#[test]
fn test_parameter_returned_after_a_user_call_stays_balanced() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function pick(int $i): mixed { if ($i > 100) { return "never"; } return 7; }
function peek(mixed $v): bool { return true; }

function ident(mixed $value): mixed {
    if (peek($value)) { return $value; }
    return $value;
}

$n = 0;
for ($i = 0; $i < 5; $i++) { $tmp = pick($i); $n = $n + ident($tmp); }
echo $n;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "35");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a by-value user call to leave the parameter's provenance alone, got: {}",
        out.stderr
    );
}

/// Verifies a BY-REFERENCE argument still invalidates. The precision added for by-value
/// arguments must not reach the one case where the callee really can rebind the variable.
#[test]
fn test_by_reference_argument_still_loses_its_provenance() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function bump(int &$v): void { $v = $v + 1; }

function ident(int $value): int {
    bump($value);
    return $value;
}

$n = 0;
for ($i = 0; $i < 5; $i++) { $n = $n + ident($i); }
echo $n;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "15");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a by-reference rebind to stay correct, got: {}",
        out.stderr
    );
}

/// Verifies a name captured BY REFERENCE stays unknowable for the rest of the body, not only at
/// the moment the closure is created.
///
/// Recording the capture is what makes the by-value call rule above safe. Without it, a later
/// assignment re-establishes a provenance the closure can still write through, and because a
/// source function is no longer treated as able to rebind anything, that provenance survives the
/// call — so the caller suppresses a release it owed.
///
/// Measured as a SLOPE across two iteration counts rather than against a clean heap: a
/// by-reference capture costs a ref cell per call whatever the summary says, so the absolute
/// count is not the signal. Measured 2.0 blocks per iteration with the rule and 4.0 without it.
#[test]
fn test_a_by_reference_capture_is_never_proven_again() {
    // The loop, parameterised by iteration count so the per-call cost can be isolated.
    fn capture_loop(iterations: usize) -> String {
        format!(
            r#"<?php
function apply_one(callable $cb, mixed $v): mixed {{ $cb(); return $v; }}
function ident(mixed $value, mixed $other): mixed {{
    $f = function () use (&$value) {{ $value = str_repeat("x", 3); }};
    $value = $other;
    apply_one($f, 1);
    return $value;
}}
$n = 0;
for ($i = 0; $i < {iterations}; $i++) {{
    $a = str_repeat("y", 4);
    $b = str_repeat("z", 5);
    $n = $n + strlen(ident($a, $b));
}}
echo $n;
"#
        )
    }

    let small = compile_and_run_with_heap_debug(&capture_loop(100));
    let large = compile_and_run_with_heap_debug(&capture_loop(300));

    assert!(small.success, "the 100-iteration program failed: {}", small.stderr);
    assert!(large.success, "the 300-iteration program failed: {}", large.stderr);
    assert_eq!(small.stdout, "300");
    assert_eq!(large.stdout, "900");

    let per_call = (live_blocks(&large.stderr) - live_blocks(&small.stderr)) as f64 / 200.0;
    assert!(
        per_call <= 3.0,
        "a by-reference capture kept a provenance it cannot have: {per_call} blocks per call"
    );
}

/// Verifies a branch returning a CONCATENATION of the parameter does not poison the branch that
/// returns the parameter itself.
///
/// `Parameters({0})` merged with `Unknown` collapses to `Unknown`, so one such branch was enough
/// to make the caller release storage it had only lent. A binary operator always computes a new
/// value — concatenation builds a fresh string, arithmetic and comparison produce numbers, and
/// PHP's `&&`/`||` yield a bool rather than an operand — so it answers `None` and the merge keeps
/// the proven path. Without that, this program dies with `bad refcount`.
#[test]
fn test_a_concatenating_branch_does_not_poison_the_returning_branch() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function pick(int $i): mixed { if ($i > 100) { return 1; } return str_repeat("q", 3); }
function f(mixed $v, bool $c): mixed { if ($c) { return $v; } return $v . "x"; }
$n = 0;
for ($i = 0; $i < 200; $i++) { $tmp = pick($i); $n = $n + strlen(f($tmp, true)); }
$m = 0;
for ($i = 0; $i < 5; $i++) { $tmp = pick($i); $m = $m + strlen(f($tmp, false)); }
echo $n + $m;
"#,
    );
    assert!(
        out.success,
        "the concatenating branch made the caller release a lent reference: {}",
        out.stderr
    );
    assert_eq!(out.stdout, "422");
}

/// Verifies the issue's own shape stays clean when the by-value call is spelled with a NAMED
/// argument, a static method call, or an instance method call.
///
/// Only a positional call to a free function consulted the parameter modes. Every other spelling
/// lost the provenance, and a lost provenance is not a conservative choice here: the call site
/// asks for a proof, so `Unknown` makes the caller release storage it never acquired. All three
/// reproduced issue #992's own `bad refcount` fatal.
#[test]
fn test_every_by_value_call_spelling_keeps_the_provenance() {
    for (label, body) in [
        ("named argument", "if (peek_free(x: $value)) { return $value; }"),
        ("static method", "if (P::peek($value)) { return $value; }"),
        ("instance method", "if ((new P())->look($value)) { return $value; }"),
    ] {
        let out = compile_and_run_with_heap_debug(&format!(
            r#"<?php
class Tag {{ public string $s = "tag"; }}
class P {{
    public static function peek(mixed $x): bool {{ return true; }}
    public function look(mixed $x): bool {{ return true; }}
}}
function peek_free(mixed $x): bool {{ return true; }}
function pick(int $i): mixed {{ if ($i > 100) {{ return "never"; }} return new Tag(); }}

function ident(mixed $value): mixed {{
    {body}
    return $value;
}}

for ($i = 0; $i < 5; $i++) {{ $tmp = pick($i); $v = ident($tmp); }}
echo "ok";
"#
        ));

        assert!(out.success, "{label} failed: {}", out.stderr);
        assert_eq!(out.stdout, "ok", "{label} printed the wrong answer");
        assert!(
            !out.stderr.contains("bad refcount"),
            "{label} released storage it never acquired: {}",
            out.stderr
        );
        assert!(
            out.stderr.contains("HEAP DEBUG: leak summary: clean"),
            "{label} did not end clean: {}",
            out.stderr
        );
    }
}

/// Verifies a proven parameter return does not suppress the release of a BOXED call result.
///
/// `$s` is a bare `Str` slot and the parameter is `mixed`, so the call boundary boxes it: the
/// cell the callee returns is a fresh allocation, not the argument's storage. The summary is
/// right that the PHP value is the parameter, and wrong that nobody owes a release.
///
/// Measured as a SLOPE across two iteration counts: 2.0 blocks per call before the fix, 0.0
/// after, and 0.0 at the merge-base, so this is a regression the precision work introduced by
/// making the proof reachable rather than a pre-existing hole.
#[test]
fn test_a_proven_return_of_a_boxed_argument_is_still_released() {
    // The loop, parameterised by iteration count so the per-call cost can be isolated.
    fn boxed_loop(iterations: usize) -> String {
        format!(
            r#"<?php
function peek_free(mixed $x): int {{ return 1; }}
function ident(mixed $v): mixed {{ peek_free($v); return $v; }}

$kept = 0;
for ($i = 0; $i < {iterations}; $i++) {{
    $s = str_repeat("y", 4);
    $kept = $kept + strlen(ident($s));
}}
echo $kept;
"#
        )
    }

    let small = compile_and_run_with_heap_debug(&boxed_loop(100));
    let large = compile_and_run_with_heap_debug(&boxed_loop(300));

    assert!(small.success, "the 100-iteration program failed: {}", small.stderr);
    assert!(large.success, "the 300-iteration program failed: {}", large.stderr);
    assert_eq!(small.stdout, "400");
    assert_eq!(large.stdout, "1200");

    let per_call = (live_blocks(&large.stderr) - live_blocks(&small.stderr)) as f64 / 200.0;
    assert!(
        per_call < 0.5,
        "a boxed call result was treated as borrowed: {per_call} blocks per call"
    );
}
