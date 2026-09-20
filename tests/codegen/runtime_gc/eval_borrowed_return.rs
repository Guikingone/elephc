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
