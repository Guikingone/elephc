//! Purpose:
//! Ownership coverage for issue #982: retaining a returned value that was read out of a
//! BORROWED eval scope cell must not turn the premature release into a leak.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `$this` and by-value parameters are bound `ScopeCellOwnership::Borrowed`, so
//!   `return $this;` handed the caller a reference it did not own and discarding the
//!   result destroyed a live object. The fix retains on that return; these fixtures pin
//!   that the matching release still happens when the caller drops the result.
//! - Measured as a DIFFERENCE, not against a clean heap. An eval fragment does not end
//!   clean today: the interpreter's per-call machinery leaves roughly a dozen blocks per
//!   loop iteration whatever the callee returns (measured 12.0 blocks/iteration for a
//!   fragment with no borrowed return at all, against 11.0 for these). That residue is a
//!   pre-existing gap in eval teardown, not something a borrowed return causes, and a
//!   fixture asserting `clean` here would be asserting a fix nobody has made. What the
//!   retain must not do is add to it.
//! - One leaked receiver would be visible: a `GcBag` is an object plus its backing array
//!   plus the pushed string, so a per-iteration leak moves these counts by hundreds.

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

/// Builds the issue #982 loop: `$discard` chooses between dropping the borrowed
/// returns and binding them, which must not change what the heap holds at exit.
fn borrowed_return_loop(discard: bool) -> String {
    let calls = if discard {
        "    $o->add(\"a\");\n    gcPassthru($o);\n"
    } else {
        "    $kept1 = $o->add(\"a\");\n    $kept2 = gcPassthru($o);\n"
    };

    format!(
        r#"<?php
eval('
class GcBag {{
    private array $items = [];
    public function add(string $v): self {{ $this->items[] = $v; return $this; }}
    public function count(): int {{ return count($this->items); }}
}}
function gcPassthru(GcBag $b): GcBag {{ return $b; }}

$total = 0;
for ($i = 0; $i < 200; $i++) {{
    $o = new GcBag();
{calls}    $total = $total + $o->count();
}}
echo $total;
');
"#
    )
}

/// Verifies discarding 400 borrowed returns holds exactly as much heap at exit as
/// binding all 400 does — the retain on a borrowed return is released when the caller
/// drops the result.
#[test]
fn test_eval_discarded_borrowed_return_abandons_nothing_extra() {
    let discarded = compile_and_run_with_heap_debug(&borrowed_return_loop(true));
    let bound = compile_and_run_with_heap_debug(&borrowed_return_loop(false));

    assert!(discarded.success, "the discarding program failed: {}", discarded.stderr);
    assert!(bound.success, "the binding program failed: {}", bound.stderr);
    assert_eq!(discarded.stdout, "200", "the receiver did not survive the discards");
    assert_eq!(bound.stdout, "200");
    assert_eq!(
        live_blocks(&discarded.stderr),
        live_blocks(&bound.stderr),
        "discarding a borrowed return abandoned storage that binding it releases"
    );
}

/// Verifies the borrowed-return loop holds no more at exit than the same loop written
/// with no borrowed return at all: the retain adds no per-call residue of its own.
#[test]
fn test_eval_borrowed_return_adds_no_residue_over_a_returnless_control() {
    let borrowed = compile_and_run_with_heap_debug(&borrowed_return_loop(true));
    // Same object count, same call count, same arity — but both callees return a
    // freshly computed int, so nothing crosses the return as a borrowed scope cell.
    let control = compile_and_run_with_heap_debug(
        r#"<?php
eval('
class GcBag {
    private array $items = [];
    public function add(string $v): int { $this->items[] = $v; return count($this->items); }
    public function count(): int { return count($this->items); }
}
function gcPassthru(GcBag $b): int { return $b->count(); }

$total = 0;
for ($i = 0; $i < 200; $i++) {
    $o = new GcBag();
    $o->add("a");
    gcPassthru($o);
    $total = $total + $o->count();
}
echo $total;
');
"#,
    );

    assert!(borrowed.success, "the borrowed-return program failed: {}", borrowed.stderr);
    assert!(control.success, "the control program failed: {}", control.stderr);
    assert_eq!(borrowed.stdout, "200");
    assert_eq!(control.stdout, "200");
    assert!(
        live_blocks(&borrowed.stderr) <= live_blocks(&control.stderr),
        "returning a borrowed scope cell held more than returning a fresh int: {} vs {}",
        live_blocks(&borrowed.stderr),
        live_blocks(&control.stderr)
    );
}

/// Verifies the retain is NOT taken for the conditional branch that did not run. Only the
/// borrowed arm of `$c ? $b : new NArm()` may be retained; retaining because the other arm
/// names a borrowed cell would leak one object per call.
#[test]
fn test_eval_conditional_return_retains_only_the_branch_that_ran() {
    /// `take_borrowed` chooses which arm of the ternary the loop exercises.
    fn conditional_return_loop(take_borrowed: bool) -> String {
        let arg = if take_borrowed { "true" } else { "false" };

        format!(
            r#"<?php
eval('
class NArm {{ public int $k = 1; }}
function npick(NArm $b, bool $c): NArm {{ return $c ? $b : new NArm(); }}
$t = 0;
for ($i = 0; $i < 200; $i++) {{ $o = new NArm(); npick($o, {arg}); $t = $t + $o->k; }}
echo $t;
');
"#
        )
    }

    let fresh_arm = compile_and_run_with_heap_debug(&conditional_return_loop(false));
    let borrowed_arm = compile_and_run_with_heap_debug(&conditional_return_loop(true));
    // No ternary at all: the floor both arms must meet.
    let control = compile_and_run_with_heap_debug(
        r#"<?php
eval('
class NArm { public int $k = 1; }
function npick(NArm $b, bool $c): NArm { return new NArm(); }
$t = 0;
for ($i = 0; $i < 200; $i++) { $o = new NArm(); npick($o, false); $t = $t + $o->k; }
echo $t;
');
"#,
    );

    assert!(fresh_arm.success, "the fresh-arm program failed: {}", fresh_arm.stderr);
    assert!(borrowed_arm.success, "the borrowed-arm program failed: {}", borrowed_arm.stderr);
    assert!(control.success, "the control program failed: {}", control.stderr);
    assert_eq!(fresh_arm.stdout, "200");
    assert_eq!(borrowed_arm.stdout, "200");
    assert_eq!(control.stdout, "200");
    assert_eq!(
        live_blocks(&fresh_arm.stderr),
        live_blocks(&control.stderr),
        "the unexecuted borrowed arm was retained anyway"
    );
    assert_eq!(
        live_blocks(&borrowed_arm.stderr),
        live_blocks(&control.stderr),
        "returning the borrowed arm held more than returning a fresh object"
    );
}

/// Verifies aliasing a borrowed parameter into a local holds no more than the same function
/// without the alias, in the shape that has nothing to balance it: the alias is stored once
/// and never replaced, so the frame unwinds still holding it.
///
/// This is the shape that decided the implementation. Making the store retain — so that its
/// `ScopeCellOwnership::Owned` claim became true — passed the replaced-alias fixture below and
/// leaked three blocks per call here, because a method scope is dropped without
/// `drain_owned_cells` and nothing ever gave that reference back (2208 blocks against 1608 for
/// the control, over 200 iterations). Recording `Borrowed` instead costs nothing.
#[test]
fn test_eval_aliased_borrowed_cell_that_is_never_replaced_adds_no_residue() {
    /// `alias` chooses between aliasing the borrowed parameter and touching only an int.
    fn alias_loop(alias: bool) -> String {
        let body = if alias {
            "function gcAlias(GcBag $b): int { $a = $b; return 7; }"
        } else {
            "function gcAlias(GcBag $b): int { $a = 1; return 7; }"
        };

        format!(
            r#"<?php
eval('
class GcBag {{
    private array $items = [];
    public function count(): int {{ return count($this->items); }}
}}
{body}

$total = 0;
for ($i = 0; $i < 200; $i++) {{
    $o = new GcBag();
    gcAlias($o);
    $total = $total + $o->count();
}}
echo $total;
');
"#
        )
    }

    let aliased = compile_and_run_with_heap_debug(&alias_loop(true));
    let plain = compile_and_run_with_heap_debug(&alias_loop(false));

    assert!(aliased.success, "the aliasing program failed: {}", aliased.stderr);
    assert!(plain.success, "the control program failed: {}", plain.stderr);
    assert_eq!(aliased.stdout, "0");
    assert_eq!(plain.stdout, "0");
    assert!(
        live_blocks(&aliased.stderr) <= live_blocks(&plain.stderr),
        "an alias the frame never replaced held storage the control releases: {} vs {}",
        live_blocks(&aliased.stderr),
        live_blocks(&plain.stderr)
    );
}

/// Verifies the same for the shape that DOES have something to balance it: the alias is
/// replaced, so whatever the store recorded is acted on before the frame unwinds.
#[test]
fn test_eval_aliased_borrowed_cell_that_is_replaced_adds_no_residue() {
    /// `alias` chooses between aliasing the borrowed parameter and touching only an int.
    fn alias_loop(alias: bool) -> String {
        let body = if alias {
            "function gcAlias(GcBag $b): int { $a = $b; $a = 1; return 7; }"
        } else {
            "function gcAlias(GcBag $b): int { $a = 1; $a = 1; return 7; }"
        };

        format!(
            r#"<?php
eval('
class GcBag {{
    private array $items = [];
    public function count(): int {{ return count($this->items); }}
}}
{body}

$total = 0;
for ($i = 0; $i < 200; $i++) {{
    $o = new GcBag();
    gcAlias($o);
    $total = $total + $o->count();
}}
echo $total;
');
"#
        )
    }

    let aliased = compile_and_run_with_heap_debug(&alias_loop(true));
    let plain = compile_and_run_with_heap_debug(&alias_loop(false));

    assert!(aliased.success, "the aliasing program failed: {}", aliased.stderr);
    assert!(plain.success, "the control program failed: {}", plain.stderr);
    assert_eq!(aliased.stdout, "0");
    assert_eq!(plain.stdout, "0");
    assert_eq!(
        live_blocks(&aliased.stderr),
        live_blocks(&plain.stderr),
        "replacing an aliased borrowed cell held storage the control releases"
    );
}

/// Verifies the bare-statement retain is a no-op on the heap: the release that immediately
/// follows it is the one the statement always performed.
#[test]
fn test_eval_bare_borrowed_expression_statement_adds_no_residue() {
    /// `mention` chooses between naming the borrowed parameter and naming an int.
    fn statement_loop(mention: bool) -> String {
        let body = if mention {
            "function gcDrop(GcBag $b): int { $b; return 7; }"
        } else {
            "function gcDrop(GcBag $b): int { 1; return 7; }"
        };

        format!(
            r#"<?php
eval('
class GcBag {{
    private array $items = [];
    public function count(): int {{ return count($this->items); }}
}}
{body}

$total = 0;
for ($i = 0; $i < 200; $i++) {{
    $o = new GcBag();
    gcDrop($o);
    $total = $total + $o->count();
}}
echo $total;
');
"#
        )
    }

    let mentioned = compile_and_run_with_heap_debug(&statement_loop(true));
    let plain = compile_and_run_with_heap_debug(&statement_loop(false));

    assert!(mentioned.success, "the mentioning program failed: {}", mentioned.stderr);
    assert!(plain.success, "the control program failed: {}", plain.stderr);
    assert_eq!(mentioned.stdout, "0");
    assert_eq!(plain.stdout, "0");
    assert_eq!(
        live_blocks(&mentioned.stderr),
        live_blocks(&plain.stderr),
        "naming a borrowed parameter as a statement changed what the heap holds"
    );
}
