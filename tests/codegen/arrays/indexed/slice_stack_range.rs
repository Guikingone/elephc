//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of indexed array array slicing, stack, and range builtins, including slice, shift, and shift empty.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests `array_slice($a, $offset, $length)` with a 5-element array, offset 1, length 3.
/// Verifies correct sub-sequence extraction (20 30 40) and that indices map correctly.
#[test]
fn test_array_slice() {
    let out = compile_and_run(
        r#"<?php
$a = [10, 20, 30, 40, 50];
$b = array_slice($a, 1, 3);
echo $b[0] . " " . $b[1] . " " . $b[2];
"#,
    );
    assert_eq!(out, "20 30 40");
}

/// Tests `array_shift` removes and returns the first element from a 3-element array.
/// Verifies the popped value (10) and that remaining array length is reduced to 2.
#[test]
fn test_array_shift() {
    let out = compile_and_run(
        r#"<?php
$a = [10, 20, 30];
$first = array_shift($a);
echo $first . " " . count($a);
"#,
    );
    assert_eq!(out, "10 2");
}

/// Tests `array_shift` on a single-element array, then on an already-empty array.
/// Verifies that calling shift on empty returns empty string (no output).
#[test]
fn test_array_shift_empty() {
    let out = compile_and_run("<?php $a = [1]; array_shift($a); echo array_shift($a);");
    assert_eq!(out, "");
}

/// Tests `array_unshift` prepends a value to an array and returns the new count.
/// Verifies new length is returned (3) and that the prepended element is at index 0.
#[test]
fn test_array_unshift() {
    let out = compile_and_run(
        r#"<?php
$a = [2, 3];
$n = array_unshift($a, 1);
echo $n . " " . $a[0];
"#,
    );
    assert_eq!(out, "3 1");
}

/// H5: php-verified multi-value `array_unshift($a, 1, 2)` order — the first-listed
/// value ends up first: `[1, 2, ...old]`, not reversed.
#[test]
fn test_array_unshift_multi_value_order() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 4];
$n = array_unshift($a, 1, 2);
echo $n, "|", implode(",", $a);
"#,
    );
    assert_eq!(out, "4|1,2,3,4");
}

/// H5: `array_unshift()` grows capacity correctly beyond the initial 8-slot
/// default (each call is capacity-checked, matching `__rt_array_push_int`'s
/// grow-then-append idiom mirrored for prepend) — regression for the prior
/// memory-unsafe fixed-buffer shift.
///
/// Starts from a 1-element (not empty-literal) array: `$a = []` followed only
/// by `array_unshift()` writes hits a PRE-EXISTING, separate ir_lower gap
/// (there is no `lower_static_array_unshift` empty-placeholder-widening path
/// mirroring `lower_static_array_push`'s, so a local that starts as `Array(Void)`
/// and is populated only via `array_unshift()` keeps that stale placeholder
/// type for later `$a[i]`-style direct-index reads even though the runtime
/// array itself holds the correct ints — `print_r()`/`count()`/`implode()` are
/// unaffected since they read the array dynamically rather than through the
/// stale static element type). Fixing that is out of scope for H5 (real new
/// plumbing, not a capacity/COW/alias concern); this test instead exercises
/// growth from a realistic non-empty starting array, which is unaffected.
#[test]
fn test_array_unshift_growth_beyond_initial_capacity() {
    let out = compile_and_run(
        r#"<?php
$a = [-1];
for ($i = 0; $i < 30; $i++) {
    array_unshift($a, $i);
}
echo count($a), "|", $a[0], "|", $a[29], "|", $a[30];
"#,
    );
    assert_eq!(out, "31|29|0|-1");
}

/// H5 JURY ADDENDUM item 5: a reference alias (`$b =& $a`) must observe
/// `array_unshift($a, $oneValue)`'s mutation across separate statements —
/// php-verified working (each statement is its own EIR lowering with exactly
/// one by-ref write-back).
///
/// TWO SEPARATE, narrower cases are PRE-EXISTING, latent bugs in the shared
/// codegen_ir by-ref array write-back path
/// (`FunctionContext::store_value_to_local()`'s ref-cell branch,
/// `src/codegen_ir/context.rs`) that this task's real capacity-growth support
/// was the first thing to exercise against an aliased array — NOT a defect in
/// `array_unshift()`'s own shift/insert/capacity logic (php-verified: both
/// cases are byte-for-byte correct without an alias present):
/// 1. A SINGLE `array_unshift($a, $v1, $v2, ...)` call with 2+ values against
///    an ALIASED array — internally this repeats
///    `store_result_value()`/`store_value_to_local()` more than once within
///    ONE lowering (once per value). A second same-call write-back through
///    the SAME ref-cell corrupts the aliased array down to a zeroed/empty
///    state, even when no `__rt_array_grow()` reallocation occurs.
/// 2. Any `array_unshift($a, ...)` call against an ALIASED array that forces
///    `__rt_array_grow()` to reallocate.
///
/// The exact SAME `source_load_local_slot()` + `store_result_value()` +
/// `store_value_to_local()` sequence is ALSO used verbatim by `array_push()`'s
/// own codegen_ir fallback (`crate::codegen_ir::lower_inst::arrays::lower_array_push`,
/// `src/codegen_ir/lower_inst/arrays.rs:346-363`) — but that fallback is
/// normally unreachable for a simple local variable, since
/// `array_push($simpleLocal, ...)` is intercepted earlier by
/// `lower_static_array_push` in `src/ir_lower/expr/mod.rs` (a completely
/// different, `Op::ArrayPush`-based mechanism that never calls
/// `store_value_to_local`). So this looks like a bug in a shared primitive
/// that was very likely NEVER exercised by a real alias+multi-write-back
/// combination on any by-ref array builtin before this task. Flagged as a
/// residual for a follow-up fix rather than silently accepted; not covered by
/// a test here since a test would need to assert the currently-broken behavior.
#[test]
fn test_array_unshift_reference_alias_sees_mutation() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 4];
$b = &$a;
array_unshift($a, 1);
array_unshift($a, 0);
echo count($b), "|", implode(",", $b);
"#,
    );
    assert_eq!(out, "4|0,1,3,4");
}

/// H5 JURY ADDENDUM item 5: COW — a shared (non-aliased, plain-assigned) array
/// must NOT observe `array_unshift()` on the other copy.
#[test]
fn test_array_unshift_cow_shared_array_unaffected() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
$b = $a;
array_unshift($a, 0);
echo implode(",", $a), "|", implode(",", $b);
"#,
    );
    assert_eq!(out, "0,1,2,3|1,2,3");
}

/// Tests `range($start, $end)` with ascending values (1 to 5).
/// Verifies correct count (5) and iteration order (12345).
#[test]
fn test_range() {
    let out = compile_and_run(
        r#"<?php
$a = range(1, 5);
echo count($a) . ":";
foreach ($a as $v) { echo $v; }
"#,
    );
    assert_eq!(out, "5:12345");
}

/// Tests `range` with start greater than end (5 down to 1), verifying descending order.
/// Verifies correct count (5) and iteration order (54321).
#[test]
fn test_range_descending() {
    let out = compile_and_run(
        r#"<?php
$a = range(5, 1);
echo count($a) . ":";
foreach ($a as $v) { echo $v; }
"#,
    );
    assert_eq!(out, "5:54321");
}

/// Tests `range($start, $end)` when start equals end (3 to 3).
/// Verifies a single-element array is produced with count 1 and value 3.
#[test]
fn test_range_single_element() {
    let out = compile_and_run(
        r#"<?php
$a = range(3, 3);
echo count($a) . ":";
foreach ($a as $v) { echo $v; }
"#,
    );
    assert_eq!(out, "1:3");
}

/// Regression: `range()` and `array_slice()` must unbox a `Mixed`/`Union` integer argument
/// (range end, slice offset) instead of using the boxed heap pointer as a raw int. The int args
/// here are read from a heterogeneous (Mixed-valued) associative array. Before the fix these
/// produced empty results or "heap memory exhausted" (a pointer used as a count).
#[test]
fn test_range_and_slice_unbox_mixed_int_args() {
    let out = compile_and_run(
        r#"<?php
$m = ["n" => 2, "t" => "x"];
echo implode(",", range(1, $m["n"])), "|", implode(",", array_slice([10, 20, 30, 40], $m["n"]));
"#,
    );
    assert_eq!(out, "1,2|30,40");
}

/// Regression: the shared slice/splice/range argument marshaling must unbox a `Mixed` length and a
/// `Mixed` offset on `array_splice` (which mutates its source), and unbox both endpoints of
/// `range()`. The integers are read from a heterogeneous (`Mixed`-valued) associative array, so the
/// boxed-pointer-as-int bug would corrupt the offset, length, removed slice, and remaining array.
#[test]
fn test_slice_splice_range_unbox_mixed_offset_and_length() {
    let out = compile_and_run(
        r#"<?php
$m = ["off" => 1, "len" => 2, "t" => "x"];
echo implode(",", array_slice([10, 20, 30, 40, 50], $m["off"], $m["len"])), "|";
$a = [1, 2, 3, 4, 5];
$removed = array_splice($a, $m["off"], $m["len"]);
echo implode(",", $removed), "|", implode(",", $a), "|";
echo implode(",", range($m["off"], $m["len"]));
"#,
    );
    assert_eq!(out, "20,30|2,3|1,4,5|1,2");
}

/// Regression: when the *array itself* is a boxed `Mixed` cell (read from a heterogeneous associative
/// array), `array_slice`/`array_splice` must still unbox a `Mixed` offset and a `Mixed` length instead
/// of passing the boxed heap pointer as a raw integer. A `Mixed` length previously hard-errored at
/// codegen ("array_slice length PHP type Mixed") and a `Mixed` offset silently corrupted the result.
/// Covers offset+length both Mixed (slice), offset Mixed with length absent (slice), and offset+length
/// both Mixed with source mutation (splice).
#[test]
fn test_mixed_array_slice_splice_unbox_mixed_offset_and_length() {
    let out = compile_and_run(
        r#"<?php
$d = ["arr" => [10, 20, 30, 40, 50], "off" => 1, "len" => 2];
$a = $d["arr"];
echo implode(",", array_slice($a, $d["off"], $d["len"])), "|";
$d2 = ["arr" => [10, 20, 30, 40, 50], "off" => 2];
$a2 = $d2["arr"];
echo implode(",", array_slice($a2, $d2["off"])), "|";
$d3 = ["arr" => [1, 2, 3, 4, 5], "off" => 1, "len" => 2];
$a3 = $d3["arr"];
$removed = array_splice($a3, $d3["off"], $d3["len"]);
echo implode(",", $removed), "|", implode(",", $a3);
"#,
    );
    assert_eq!(out, "20,30|30,40,50|2,3|1,4,5");
}
