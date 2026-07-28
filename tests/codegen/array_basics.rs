//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of array literals, indexing, and string offsets, including literal and count, access, and access variable index.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

// --- Arrays ---

/// Compiles `[1, 2, 3]` and verifies `count()` returns the array length.
#[test]
fn test_array_literal_and_count() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; echo count($a);");
    assert_eq!(out, "3");
}

/// Compiles `[10, 20, 30]` and accesses elements at literal indices 0, 1, 2.
#[test]
fn test_array_access() {
    let out =
        compile_and_run("<?php $a = [10, 20, 30]; echo $a[0] . \" \" . $a[1] . \" \" . $a[2];");
    assert_eq!(out, "10 20 30");
}

/// Verifies array access variable index.
#[test]
fn test_array_access_variable_index() {
    let out = compile_and_run("<?php $a = [10, 20, 30]; $i = 2; echo $a[$i];");
    assert_eq!(out, "30");
}

/// Verifies string indexing returns single character.
#[test]
fn test_string_indexing_returns_single_character() {
    let out = compile_and_run(r#"<?php $s = "hello"; echo $s[1];"#);
    assert_eq!(out, "e");
}

/// Verifies string indexing out of bounds returns empty string.
#[test]
fn test_string_indexing_out_of_bounds_returns_empty_string() {
    let out = compile_and_run(r#"<?php $s = "hello"; echo "[" . $s[99] . "]";"#);
    assert_eq!(out, "[]");
}

/// Verifies string indexing negative offset counts from end.
#[test]
fn test_string_indexing_negative_offset_counts_from_end() {
    let out = compile_and_run(r#"<?php $s = "hello"; echo $s[-1];"#);
    assert_eq!(out, "o");
}

/// Verifies string indexing with variable offset.
#[test]
fn test_string_indexing_with_variable_offset() {
    let out = compile_and_run(r#"<?php $s = "hello"; $i = 3; echo $s[$i];"#);
    assert_eq!(out, "l");
}

/// Verifies string indexing accepts numeric string offsets.
#[test]
fn test_string_indexing_accepts_numeric_string_offsets() {
    let out = compile_and_run(
        r#"<?php $s = "abcd"; echo $s["0"]; echo $s["01"]; echo $s["+2"]; echo $s[" -1 "]; echo "\n"; echo isset($s["3"]) ? "y" : "n"; echo isset($s["4"]) ? "y\n" : "n\n";"#,
    );
    assert_eq!(out, "abcd\nyn\n");
}

/// Verifies string indexing empty string returns empty string.
#[test]
fn test_string_indexing_empty_string_returns_empty_string() {
    let out = compile_and_run(r#"<?php $s = ""; $i = 0; echo "[" . $s[$i] . "]";"#);
    assert_eq!(out, "[]");
}

/// Verifies string indexing negative beyond length returns empty.
#[test]
fn test_string_indexing_negative_beyond_length_returns_empty() {
    let out = compile_and_run(r#"<?php $s = "hi"; echo "[" . $s[-10] . "]";"#);
    assert_eq!(out, "[]");
}

/// Verifies string indexing exactly negative length returns first.
#[test]
fn test_string_indexing_exactly_negative_length_returns_first() {
    let out = compile_and_run(r#"<?php $s = "abc"; echo $s[-3];"#);
    assert_eq!(out, "a");
}

/// Verifies string indexing at length returns empty.
#[test]
fn test_string_indexing_at_length_returns_empty() {
    let out = compile_and_run(r#"<?php $s = "ab"; echo "[" . $s[2] . "]";"#);
    assert_eq!(out, "[]");
}

/// Verifies string indexing last valid index.
#[test]
fn test_string_indexing_last_valid_index() {
    let out = compile_and_run(r#"<?php $s = "abc"; echo $s[2];"#);
    assert_eq!(out, "c");
}

/// Verifies array assign.
#[test]
fn test_array_assign() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; $a[1] = 99; echo $a[1];");
    assert_eq!(out, "99");
}

/// Verifies array compound assign.
#[test]
fn test_array_compound_assign() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; $a[1] += 40; $a[2] *= 10; echo $a[1] . \"|\" . $a[2];");
    assert_eq!(out, "42|30");
}

/// Verifies array compound assign evaluates index once.
#[test]
fn test_array_compound_assign_evaluates_index_once() {
    let out = compile_and_run(
        r#"<?php
function idx() {
    echo "i";
    return 1;
}

$a = [10, 20, 30];
$a[idx()] += 5;
echo ":" . $a[1];
"#,
    );
    assert_eq!(out, "i:25");
}

/// Verifies array compound assign effectful index all operator families.
#[test]
fn test_array_compound_assign_effectful_index_all_operator_families() {
    let out = compile_and_run(
        r#"<?php
function idx() {
    echo ".";
    return 0;
}

$num = [2];
$num[idx()] **= 3;
echo ":" . $num[0];

$bits = [8];
$bits[idx()] >>= 1;
echo ":" . $bits[0];

$text = ["a"];
$text[idx()] .= "b";
echo ":" . $text[0];

$fallback = [null];
$fallback[idx()] ??= 7;
echo ":" . $fallback[0];
"#,
    );
    assert_eq!(out, ".:8.:4.:ab.:7");
}

/// Verifies array assign into empty array updates length.
#[test]
fn test_array_assign_into_empty_array_updates_length() {
    let out = compile_and_run(r#"<?php $a = []; $a[0] = 7; echo count($a) . "|" . $a[0];"#);
    assert_eq!(out, "1|7");
}

/// Verifies array push.
#[test]
fn test_array_push() {
    let out = compile_and_run("<?php $a = [1, 2]; $a[] = 3; echo count($a) . \" \" . $a[2];");
    assert_eq!(out, "3 3");
}

/// Verifies array push builtin.
#[test]
fn test_array_push_builtin() {
    let out =
        compile_and_run("<?php $a = [10]; array_push($a, 20); echo count($a) . \" \" . $a[1];");
    assert_eq!(out, "2 20");
}

/// Regression: appending strings into an empty `[]` literal past its initial capacity must
/// not corrupt the first element. An empty literal is typed `array<never>`; `array_new`
/// previously sized its slots at 8 bytes, but the first string append specializes the header
/// to 16-byte `{ptr,len}` slots in place without reallocating, overflowing the undersized
/// backing store. Growth then copied the overflowed bytes and the first element came out
/// garbled. Pushing 8 strings forces at least one grow from the initial 4-element capacity.
#[test]
fn test_empty_array_string_append_grows() {
    let out = compile_and_run(
        r#"<?php
$a = [];
for ($i = 0; $i < 8; $i++) { $a[] = "x"; }
echo implode(",", $a);
"#,
    );
    assert_eq!(out, "x,x,x,x,x,x,x,x");
}

/// Regression: same empty-array grow corruption, exercised with distinct interpolated strings
/// so a mis-sized first slot is caught by value (not just by a repeated character). Verifies
/// the first element survives the grow and every appended string round-trips.
#[test]
fn test_empty_array_interpolated_string_append_grows() {
    let out = compile_and_run(
        r#"<?php
$a = [];
for ($i = 0; $i < 10; $i++) { $a[] = "item$i"; }
echo implode("|", $a);
"#,
    );
    assert_eq!(out, "item0|item1|item2|item3|item4|item5|item6|item7|item8|item9");
}

/// Regression guard: an empty `[]` literal that first receives refcounted (object) elements must
/// also survive growth. Object slots are 8-byte pointers, so the empty `array<never>` buffer is
/// already the right size and only the string-append path needs the capacity rescale — this test
/// confirms the fix left the pointer-slot grow path untouched. Pushing 6 objects forces a grow
/// from the initial 4-element capacity.
#[test]
fn test_empty_array_object_append_grows() {
    let out = compile_and_run(
        r#"<?php
class P { public function __construct(public string $n) {} }
$a = [];
for ($i = 0; $i < 6; $i++) { $a[] = new P("p$i"); }
echo $a[0]->n . "|" . $a[5]->n . "|" . count($a);
"#,
    );
    assert_eq!(out, "p0|p5|6");
}

/// Regression for #452: pushing heterogeneous scalars (int then float) into an empty array
/// inside a loop promotes the array to mixed-element storage on iteration 1, but the earlier
/// push site was lowered against the stale pre-promotion type and wrote an unboxed scalar
/// into the mixed array on iteration 2. Reading that element then dereferenced the raw
/// scalar as a boxed cell pointer and crashed. The loop body must see the fixed-point
/// (back-edge) element type so every push site boxes correctly.
#[test]
fn test_loop_grown_mixed_array_element_read() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) { $vals[] = 1; $vals[] = 2.0; }
echo $vals[2];
"#,
    );
    assert_eq!(out, "1");
}

/// Regression for #452: consuming every element of a loop-grown mixed array (foreach with
/// arithmetic) must see each value correctly instead of crashing on the element written by
/// the stale-typed push site.
#[test]
fn test_loop_grown_mixed_array_foreach_sum() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) { $vals[] = 1; $vals[] = 2.0; }
$sum = 0;
foreach ($vals as $v) { $sum += intval($v); }
echo $sum;
"#,
    );
    assert_eq!(out, "6");
}

/// Regression for #452: the int + string flavour of the same loop-grown promotion. Before
/// the fix the raw int push into the mixed array happened to survive reads (the payload was
/// misread as a pointer that pointed at a valid persistent string) but corrupted values and
/// leaked; the fixed-point loop typing must produce the correct elements.
#[test]
fn test_loop_grown_mixed_array_int_string_elements() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) { $vals[] = 1; $vals[] = "x"; }
echo $vals[0], $vals[1], $vals[2], $vals[3];
"#,
    );
    assert_eq!(out, "1x1x");
}

/// Regression for #452: float-first ordering inside the loop (the promotion happens at the
/// first site, the second site pushes the int raw) must also produce a sound mixed array.
#[test]
fn test_loop_grown_mixed_array_float_first() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) { $vals[] = 2.0; $vals[] = 1; }
$sum = 0;
foreach ($vals as $v) { $sum += intval($v); }
echo $sum;
"#,
    );
    assert_eq!(out, "6");
}

/// Regression for #452: the widening prescan must find push sites nested inside an `if`
/// within the loop body, so a conditionally-executed heterogeneous push still fixes the
/// array's element type before the loop.
#[test]
fn test_loop_grown_mixed_array_push_under_if() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 4; $i++) {
    if ($i % 2 == 0) {
        $vals[] = 1;
    } else {
        $vals[] = 2.0;
    }
}
$sum = 0;
foreach ($vals as $v) { $sum += intval($v); }
echo $sum, "|", intval($vals[2]);
"#,
    );
    assert_eq!(out, "6|1");
}

/// Regression for #452: the same loop-grown promotion through a `while` loop (the widening
/// runs on every loop kind, not just `for`).
#[test]
fn test_loop_grown_mixed_array_while_loop() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
$i = 0;
while ($i < 2) {
    $vals[] = 1;
    $vals[] = 2.0;
    $i++;
}
$sum = 0;
foreach ($vals as $v) { $sum += intval($v); }
echo $sum;
"#,
    );
    assert_eq!(out, "6");
}

/// Regression for #594: reassigning an array local to a literal built from its own current
/// element (`$r = [$r[0] - 1, 0]`) inside a `for` loop. The rebuilt literal is `array<mixed>`
/// (the `$r[0] - 1` overflow-checked subtraction boxes), but a read of `$r[0]` at the top of the
/// single-pass loop body was still typed against the entry `array<int>`, so from iteration 2 on
/// it read the boxed cell as a raw int and printed heap addresses. The loop-entry widening must
/// promote `$r` to `array<mixed>` so every read uses the boxed element representation.
#[test]
fn test_loop_reassigned_self_ref_literal_decrement_for() {
    let out = compile_and_run(
        r#"<?php
$r = [3, 0];
$out = "";
for ($k = 0; $k < 6; $k++) {
    $out = $out . $r[0] . ",";
    $r = [$r[0] - 1, 0];
}
echo $out;
"#,
    );
    assert_eq!(out, "3,2,1,0,-1,-2,");
}

/// Regression for #594: the same self-referential rebind in a `while` condition. Before the fix
/// the corrupted `$r[0]` read was a heap address (always truthy), so the loop never terminated;
/// the guard here caps it at 10 to keep the test finite. With the fix the loop terminates after
/// exactly 3 iterations, matching PHP.
#[test]
fn test_loop_reassigned_self_ref_literal_while_terminates() {
    let out = compile_and_run(
        r#"<?php
$count = 0;
$r = [3, 0];
$guard = 0;
while ($r[0] && $guard < 10) {
    $count++;
    $guard++;
    $r = [$r[0] - 1, 0];
}
echo $count;
"#,
    );
    assert_eq!(out, "3");
}

/// Regression for #594: a pure self-referential swap `$r = [$r[1], $r[0]]`. Each array read is
/// typed `int|null` (the inline tagged-scalar representation), so the rebuilt literal changes
/// element shape versus the entry `array<int>` even though no arithmetic is involved. The same
/// loop-entry widening must promote `$r` so both elements read back correctly each iteration.
#[test]
fn test_loop_reassigned_self_ref_literal_swap() {
    let out = compile_and_run(
        r#"<?php
$r = [1, 2];
$out = "";
for ($k = 0; $k < 4; $k++) {
    $out = $out . $r[0] . "-" . $r[1] . ",";
    $r = [$r[1], $r[0]];
}
echo $out;
"#,
    );
    assert_eq!(out, "1-2,2-1,1-2,2-1,");
}

/// Regression for #594: the float flavour of the self-referential rebind, confirming the
/// promotion is representation-general (float element boxed into the mixed cell), not int-only.
#[test]
fn test_loop_reassigned_self_ref_literal_float() {
    let out = compile_and_run(
        r#"<?php
$r = [1.5, 0.0];
$out = "";
for ($k = 0; $k < 3; $k++) {
    $out = $out . $r[0] . ",";
    $r = [$r[0] - 1, 0.0];
}
echo $out;
"#,
    );
    assert_eq!(out, "1.5,0.5,-0.5,");
}

/// Regression guard for #594: seeding the initializer with a runtime value (`[$argc + 2, 0]`)
/// already produced correct output before the fix, because the entry array is `array<mixed>`
/// from the start. This must stay correct so the widening does not depend on a constant-literal
/// initializer.
#[test]
fn test_loop_reassigned_self_ref_literal_runtime_seeded_control() {
    let out = compile_and_run(
        r#"<?php
$r = [$argc + 2, 0];
$out = "";
for ($k = 0; $k < 6; $k++) {
    $out = $out . $r[0] . ",";
    $r = [$r[0] - 1, 0];
}
echo $out;
"#,
    );
    assert_eq!(out, "3,2,1,0,-1,-2,");
}

/// Regression guard for #594: a self-referential rebind whose element stays a raw `int`
/// (`$r = [count($r), 0]`, still `array<int>`) must NOT be widened — the entry representation is
/// unchanged, so promoting it would waste a conversion and change nothing. Verifies the widening
/// triggers on the element-representation change, not on self-reference alone.
#[test]
fn test_loop_reassigned_self_ref_same_repr_not_widened() {
    let out = compile_and_run(
        r#"<?php
$r = [3, 0];
$out = "";
for ($k = 0; $k < 3; $k++) {
    $out = $out . $r[0] . ",";
    $r = [count($r), 0];
}
echo $out;
"#,
    );
    assert_eq!(out, "3,2,2,");
}

/// Regression for #594: the promotion must not leak or double-free the previous array. The
/// widening converts the entry array in place and each rebind releases the prior array before
/// storing the new one; a `for` loop over the self-referential rebind must end with a clean heap
/// (allocations == deallocations).
#[test]
fn test_loop_reassigned_self_ref_literal_heap_clean() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$r = [50, 0];
for ($k = 0; $k < 50; $k++) {
    $r = [$r[0] - 1, 0];
}
echo $r[0];
"#,
    );
    assert_eq!(out.stdout, "0");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Regression for #607: loop storage inference must iterate when an external scalar widens
/// before feeding an array rebind. A one-shot scan sees `$v` as the entry `int` and misses that
/// checked subtraction makes it `mixed`, leaving the next iteration's `$r[0]` read raw.
#[test]
fn test_loop_reassigned_array_cascading_fixed_point() {
    let out = compile_and_run(
        r#"<?php
$r = [3, 0];
$out = "";
$v = 3;
for ($k = 0; $k < 4; $k++) {
    $out = $out . $r[0] . ",";
    $v = $v - 1;
    $r = [$v, 0];
}
echo $out;
"#,
    );
    assert_eq!(out, "3,2,1,0,");
}

/// Regression for #608: two different raw array element layouts must join to boxed `mixed`
/// storage at the loop header. Otherwise the single lowered string read interprets later raw
/// integers as `{pointer, length}` string descriptors and silently emits empty output.
#[test]
fn test_loop_reassigned_array_raw_to_raw_representation_join() {
    let out = compile_and_run(
        r#"<?php
$r = ["x", "y"];
$out = "";
for ($k = 0; $k < 3; $k++) {
    $out = $out . $r[0] . ",";
    $r = [$k, $k];
}
echo $out;
"#,
    );
    assert_eq!(out, "x,0,1,");
}

/// Verifies loop storage inference follows a non-literal RHS through another local instead of
/// relying on array-literal-only inference in EIR lowering.
#[test]
fn test_loop_reassigned_array_non_literal_rhs_fixed_point() {
    let out = compile_and_run(
        r#"<?php
$r = [3, 0];
$next = $r;
$out = "";
for ($k = 0; $k < 4; $k++) {
    $out = $out . $r[0] . ",";
    $next = [$r[0] - 1, 0];
    $r = $next;
}
echo $out;
"#,
    );
    assert_eq!(out, "3,2,1,0,");
}

/// Verifies a call-produced array rebind consumes the same checker storage contract as literals
/// and local aliases; EIR must not need to recognize the RHS expression shape independently.
#[test]
fn test_loop_reassigned_array_call_rhs_fixed_point() {
    let out = compile_and_run(
        r#"<?php
function makeRow(int $value): array {
    return [$value, $value];
}
$r = ["x", "y"];
$out = "";
for ($k = 0; $k < 3; $k++) {
    $out = $out . $r[0] . ",";
    $r = makeRow($k);
}
echo $out;
"#,
    );
    assert_eq!(out, "x,0,1,");
}

/// Verifies the checker-recorded fixed-point contract reaches function EIR lowering rather than
/// depending on main's final top-level environment.
#[test]
fn test_loop_reassigned_array_fixed_point_inside_function() {
    let out = compile_and_run(
        r#"<?php
function countdown(array $row): string {
    $out = "";
    for ($k = 0; $k < 4; $k++) {
        $out = $out . $row[0] . ",";
        $row = [$row[0] - 1, 0];
    }
    return $out;
}
echo countdown([3, 0]);
"#,
    );
    assert_eq!(out, "3,2,1,0,");
}

/// Verifies nested closure scope keys carry the checker contract into the generated closure EIR
/// function without colliding with loops at the same line/column in other function-like bodies.
#[test]
fn test_loop_reassigned_array_fixed_point_inside_closure() {
    let out = compile_and_run(
        r#"<?php
$countdown = function(array $row): string {
    $out = "";
    for ($k = 0; $k < 4; $k++) {
        $out = $out . $row[0] . ",";
        $row = [$row[0] - 1, 0];
    }
    return $out;
};
echo $countdown([3, 0]);
"#,
    );
    assert_eq!(out, "3,2,1,0,");
}

/// Verifies the fixed-point storage join covers associative-array values and materializes
/// `HashToMixed` before a raw string payload is rebound to raw integers.
#[test]
fn test_loop_reassigned_assoc_array_raw_to_raw_representation_join() {
    let out = compile_and_run_with_gc_stats(
        r#"<?php
$row = ["value" => "x"];
$out = "";
for ($k = 0; $k < 3; $k++) {
    $out = $out . $row["value"] . ",";
    $row = ["value" => $k];
}
echo $out;
"#,
    );
    assert_eq!(out.stdout, "x,0,1,");
    let (allocs, frees) = parse_gc_stats(&out.stderr);
    assert_eq!(allocs, frees, "expected clean heap, got: {}", out.stderr);
}

/// Verifies array access on function call result.
#[test]
fn test_array_access_on_function_call_result() {
    let out = compile_and_run(
        r#"<?php
function getColor() {
    return [255, 128, 0];
}
echo getColor()[1];
"#,
    );
    assert_eq!(out, "128");
}

/// Verifies foreach int.
#[test]
fn test_foreach_int() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; foreach ($a as $v) { echo $v; }");
    assert_eq!(out, "123");
}

/// Verifies foreach value by reference mutates indexed array.
#[test]
fn test_foreach_value_by_reference_mutates_indexed_array() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as &$v) {
    $v *= 2;
}
foreach ($a as $x) {
    echo $x;
}
"#,
    );
    assert_eq!(out, "246");
}

/// Verifies foreach value by reference reuse value name in next loop.
#[test]
fn test_foreach_value_by_reference_reuse_value_name_in_next_loop() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as $k => &$v) {
    $v *= 2;
}
foreach ($a as $k => $v) {
    echo $k . "=" . $v . ";";
}
"#,
    );
    assert_eq!(out, "0=2;1=4;2=4;");
}

/// Verifies foreach value by reference post assignment mutates last element.
#[test]
fn test_foreach_value_by_reference_post_assignment_mutates_last_element() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as &$v) {
    $v += 10;
}
$v = 99;
foreach ($a as $x) {
    echo $x;
}
echo "|" . $v;
"#,
    );
    assert_eq!(out, "111299|99");
}

/// Verifies foreach value by reference empty loop preserves existing value.
#[test]
fn test_foreach_value_by_reference_empty_loop_preserves_existing_value() {
    let out = compile_and_run(
        r#"<?php
$v = 7;
$a = [1];
array_pop($a);
foreach ($a as &$v) {
    $v = 9;
}
echo $v;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies foreach value by reference rebinds existing reference param.
#[test]
fn test_foreach_value_by_reference_rebinds_existing_reference_param() {
    let out = compile_and_run(
        r#"<?php
function update(&$v) {
    $a = [1];
    foreach ($a as &$v) {
        $v = 2;
    }
    $v = 9;
    echo $a[0] . "|" . $v;
}

$x = 5;
update($x);
echo "|" . $x;
"#,
    );
    assert_eq!(out, "9|9|5");
}

/// Verifies foreach value by reference splits COW indexed array.
#[test]
fn test_foreach_value_by_reference_splits_cow_indexed_array() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2];
$b = $a;
foreach ($b as &$v) {
    $v *= 3;
}
foreach ($a as $x) {
    echo $x;
}
echo "|";
foreach ($b as $x) {
    echo $x;
}
"#,
    );
    assert_eq!(out, "12|36");
}

/// Verifies foreach string.
#[test]
fn test_foreach_string() {
    let out = compile_and_run(r#"<?php $a = ["a", "b", "c"]; foreach ($a as $v) { echo $v; }"#);
    assert_eq!(out, "abc");
}

/// Verifies foreach break.
#[test]
fn test_foreach_break() {
    let out = compile_and_run(
        "<?php $a = [1, 2, 3, 4, 5]; foreach ($a as $v) { if ($v == 3) { break; } echo $v; }",
    );
    assert_eq!(out, "12");
}

/// Verifies array in function.
#[test]
fn test_array_in_function() {
    let out = compile_and_run(
        r#"<?php
function sum($arr) {
    $total = 0;
    foreach ($arr as $v) {
        $total += $v;
    }
    return $total;
}
echo sum([1, 2, 3, 4, 5]);
"#,
    );
    assert_eq!(out, "15");
}

/// Verifies string array.
#[test]
fn test_string_array() {
    let out = compile_and_run(
        r#"<?php
$names = ["Alice", "Bob"];
$names[] = "Charlie";
echo count($names) . ": ";
foreach ($names as $n) { echo $n . " "; }
"#,
    );
    assert_eq!(out, "3: Alice Bob Charlie ");
}

// --- Array functions ---

/// Verifies array pop.
#[test]
fn test_array_pop() {
    let out =
        compile_and_run("<?php $a = [1, 2, 3]; $v = array_pop($a); echo $v . \" \" . count($a);");
    assert_eq!(out, "3 2");
}

/// Verifies array pop empty.
#[test]
fn test_array_pop_empty() {
    let out = compile_and_run("<?php $a = [1]; array_pop($a); echo array_pop($a);");
    assert_eq!(out, "");
}

/// Verifies `array_pop()` on a Mixed receiver mutates the caller's INDEXED array in place and
/// returns the removed element (gradual-typing dynamic path). Regression for the runtime-tag
/// dispatched `lower_array_pop_dynamic` (tag 4 = indexed).
#[test]
fn test_array_pop_mixed_indexed() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_pop($a); return $x . \"|\" . implode(\",\", $a); } echo f([1, 2, 3]);",
    );
    assert_eq!(out, "3|1,2");
}

/// Verifies `array_pop()` on a Mixed receiver whose runtime value is an ASSOCIATIVE hash removes
/// the insertion-order tail entry in place and returns its value (tag 5 = hash path).
#[test]
fn test_array_pop_mixed_assoc() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_pop($a); return $x . \"|\" . json_encode($a); } echo f(['a' => 1, 'b' => 2, 'c' => 3]);",
    );
    assert_eq!(out, "3|{\"a\":1,\"b\":2}");
}

/// Verifies copy-on-write: popping a Mixed receiver that aliases a sibling variable (`$b = $a;`)
/// must not corrupt the alias — the sibling keeps the original array.
#[test]
fn test_array_pop_mixed_cow_alias() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $b = $a; $x = array_pop($a); return $x . \"|\" . json_encode($a) . \"|\" . json_encode($b); } echo f([1, 2, 3]);",
    );
    assert_eq!(out, "3|[1,2]|[1,2,3]");
}

/// Verifies a Mixed receiver holding a heterogeneous (boxed-Mixed element) array pops the last
/// element with the correct runtime type and leaves the rest intact.
#[test]
fn test_array_pop_mixed_heterogeneous() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_pop($a); return json_encode($x) . \"|\" . json_encode($a); } echo f([1, 'two', 3.0]);",
    );
    assert_eq!(out, "3|[1,\"two\"]");
}

/// Verifies `array_pop()` on an empty Mixed array returns null and leaves the array empty.
#[test]
fn test_array_pop_mixed_empty() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_pop($a); return var_export($x, true) . \"|\" . count($a); } echo f([]);",
    );
    assert_eq!(out, "NULL|0");
}

/// Verifies by-value semantics: `array_pop()` on a Mixed parameter must not mutate the caller's
/// original array variable (the parameter is a copy).
#[test]
fn test_array_pop_mixed_by_value_caller_unchanged() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ array_pop($a); } $arr = [1, 2, 3]; f($arr); echo count($arr);",
    );
    assert_eq!(out, "3");
}

/// Verifies a non-array runtime value behind a checker-accepted Mixed receiver throws PHP's exact
/// catchable `\TypeError` from `array_pop()`.
#[test]
fn test_array_pop_mixed_non_array_throws_type_error() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ try { array_pop($a); } catch (\\TypeError $e) { echo $e->getMessage(); } } f(42);",
    );
    assert_eq!(
        out,
        "array_pop(): Argument #1 ($array) must be of type array, int given"
    );
}

/// Verifies `array_shift()` on a Mixed receiver removes the FIRST element of the caller's INDEXED
/// array in place and reindexes the remaining integer keys from zero (runtime-tag dynamic path,
/// tag 4 = indexed).
#[test]
fn test_array_shift_mixed_indexed() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_shift($a); return $x . \"|\" . implode(\",\", $a); } echo f([1, 2, 3]);",
    );
    assert_eq!(out, "1|2,3");
}

/// Verifies `array_shift()` on a Mixed receiver whose runtime value is an ASSOCIATIVE hash removes
/// the insertion-order head entry and renumbers surviving integer keys `0,1,…` while preserving
/// string keys (tag 5 = hash rebuild path).
#[test]
fn test_array_shift_mixed_assoc_renumbers() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_shift($a); return $x . \"|\" . json_encode($a); } echo f(['x' => 1, 5 => 2, 'y' => 3, 10 => 4]);",
    );
    assert_eq!(out, "1|{\"0\":2,\"y\":3,\"1\":4}");
}

/// Verifies copy-on-write: shifting a Mixed receiver that aliases a sibling variable (`$b = $a;`)
/// must not corrupt the alias — the sibling keeps the original array.
#[test]
fn test_array_shift_mixed_cow_alias() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $b = $a; $x = array_shift($a); return $x . \"|\" . json_encode($a) . \"|\" . json_encode($b); } echo f([10, 20, 30]);",
    );
    assert_eq!(out, "10|[20,30]|[10,20,30]");
}

/// Verifies a Mixed receiver holding a string-valued indexed array shifts the first string and
/// slides the 16-byte string slots one position toward the front.
#[test]
fn test_array_shift_mixed_string_indexed() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_shift($a); return $x . \"|\" . implode(\",\", $a); } echo f(['aa', 'bb', 'cc']);",
    );
    assert_eq!(out, "aa|bb,cc");
}

/// Verifies `array_shift()` on an empty Mixed array returns null and leaves the array empty.
#[test]
fn test_array_shift_mixed_empty() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ $x = array_shift($a); return var_export($x, true) . \"|\" . count($a); } echo f([]);",
    );
    assert_eq!(out, "NULL|0");
}

/// Verifies a non-array runtime value behind a checker-accepted Mixed receiver throws PHP's exact
/// catchable `\TypeError` from `array_shift()`.
#[test]
fn test_array_shift_mixed_non_array_throws_type_error() {
    let out = compile_and_run(
        "<?php function f(mixed $a){ try { array_shift($a); } catch (\\TypeError $e) { echo $e->getMessage(); } } f(42);",
    );
    assert_eq!(
        out,
        "array_shift(): Argument #1 ($array) must be of type array, int given"
    );
}

/// Verifies `array_combine()` pairs two checker-accepted Mixed operands positionally into an
/// associative array (gradual `__rt_array_combine_mixed` path).
#[test]
fn test_array_combine_mixed_operands() {
    let out = compile_and_run(
        "<?php function f(mixed $k, mixed $v){ return json_encode(array_combine($k, $v)); } echo f(['a', 'b'], [1, 2]);",
    );
    assert_eq!(out, "{\"a\":1,\"b\":2}");
}

/// Verifies `array_combine()` coerces keys exactly like PHP: integers stay integer keys, numeric
/// strings normalize to integers, and non-integer scalars are `(string)`-cast (float `1.9`→`"1.9"`,
/// `null`→`""`).
#[test]
fn test_array_combine_mixed_key_coercion() {
    let out = compile_and_run(
        "<?php function f(mixed $k, mixed $v){ return json_encode(array_combine($k, $v)); } echo f([1.9, 5.0, true, null], ['a', 'b', 'c', 'd']);",
    );
    assert_eq!(out, "{\"1.9\":\"a\",\"5\":\"b\",\"1\":\"c\",\"\":\"d\"}");
}

/// Verifies `array_combine()` accepts a concrete `Array` operand (a raw container pointer boxed via
/// `__rt_mixed_from_array_kind`) alongside a Mixed operand, and preserves heterogeneous values.
#[test]
fn test_array_combine_mixed_array_operand() {
    let out = compile_and_run(
        "<?php function f(array $a){ return json_encode(array_combine(array_keys($a), array_values($a))); } echo f(['x' => 1, 'y' => 'two', 'z' => 3.5]);",
    );
    assert_eq!(out, "{\"x\":1,\"y\":\"two\",\"z\":3.5}");
}

/// Verifies `array_combine()` throws PHP's exact catchable `\ValueError` when the operands have
/// different element counts.
#[test]
fn test_array_combine_mixed_count_mismatch_throws() {
    let out = compile_and_run(
        "<?php function f(mixed $k, mixed $v){ try { $r = array_combine($k, $v); echo json_encode($r); } catch (\\ValueError $e) { echo $e->getMessage(); } } f([1, 2], [1]);",
    );
    assert_eq!(
        out,
        "array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements"
    );
}

/// Verifies in array found.
#[test]
fn test_in_array_found() {
    let out = compile_and_run("<?php $a = [10, 20, 30]; echo in_array(20, $a);");
    assert_eq!(out, "1");
}

/// Verifies in array not found. `in_array` returns bool, so `echo false` is the empty
/// string (not "0").
#[test]
fn test_in_array_not_found() {
    let out = compile_and_run("<?php $a = [10, 20, 30]; echo in_array(99, $a);");
    assert_eq!(out, "");
}

/// Verifies in array string found.
#[test]
fn test_in_array_string_found() {
    let out = compile_and_run(r#"<?php $a = ["a", "b", "c"]; echo in_array("b", $a);"#);
    assert_eq!(out, "1");
}

/// Verifies in array string not found. A false result echoes as the empty string.
#[test]
fn test_in_array_string_not_found() {
    let out = compile_and_run(r#"<?php $a = ["a", "b", "c"]; echo in_array("x", $a);"#);
    assert_eq!(out, "");
}

/// Verifies `in_array` returns a real `bool` (var_dump shows bool, not int), matching PHP.
/// Regression: previously typed `int`, so `var_dump` printed `int(1)`/`int(0)` and a false
/// result echoed as "0" instead of "".
#[test]
fn test_in_array_returns_bool() {
    let out = compile_and_run(
        r#"<?php
$a = [10, 20, 30];
var_dump(in_array(20, $a));
var_dump(in_array(99, $a));
var_dump(in_array(99, $a) === false);
"#,
    );
    assert_eq!(out, "bool(true)\nbool(false)\nbool(true)\n");
}

/// Verifies strict `in_array` matches a same-type integer needle (`===` over an int array).
#[test]
fn test_in_array_strict_int_match() {
    let out = compile_and_run("<?php var_dump(in_array(2, [1, 2, 3], true));");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies strict `in_array` rejects a string needle against an int array because `===`
/// requires identical types, so `"2"` is never identical to `2`.
#[test]
fn test_in_array_strict_string_needle_int_array_false() {
    let out = compile_and_run(r#"<?php var_dump(in_array("2", [1, 2, 3], true));"#);
    assert_eq!(out, "bool(false)\n");
}

/// Verifies strict `in_array` over a same-type string array still matches by exact value.
#[test]
fn test_in_array_strict_string_match() {
    let out = compile_and_run(r#"<?php var_dump(in_array("b", ["a", "b", "c"], true));"#);
    assert_eq!(out, "bool(true)\n");
}

/// Verifies the strict flag is honored through named arguments (`strict: true`).
#[test]
fn test_in_array_strict_named_argument() {
    let out =
        compile_and_run("<?php var_dump(in_array(needle: 2, haystack: [1, 2, 3], strict: true));");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies sort.
#[test]
fn test_sort() {
    let out =
        compile_and_run(r#"<?php $a = [5, 3, 1, 4, 2]; sort($a); foreach ($a as $v) { echo $v; }"#);
    assert_eq!(out, "12345");
}

/// Verifies rsort.
#[test]
fn test_rsort() {
    let out =
        compile_and_run(r#"<?php $a = [1, 3, 2]; rsort($a); foreach ($a as $v) { echo $v; }"#);
    assert_eq!(out, "321");
}

/// Verifies array keys.
#[test]
fn test_array_keys() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; $k = array_keys($a); foreach ($k as $v) { echo $v; }"#,
    );
    assert_eq!(out, "012");
}

/// Verifies isset.
#[test]
fn test_isset() {
    let out = compile_and_run("<?php $x = 42; echo isset($x);");
    assert_eq!(out, "1");
}

/// Verifies isset multiple arguments requires all non null.
#[test]
fn test_isset_multiple_arguments_requires_all_non_null() {
    let out = compile_and_run(
        r#"<?php
$a = 1;
$b = null;
echo isset($a, $b) ? "yes\n" : "no\n";
"#,
    );
    assert_eq!(out, "no\n");
}

/// Verifies isset multiple arguments short circuits.
#[test]
fn test_isset_multiple_arguments_short_circuits() {
    let out = compile_and_run(
        r#"<?php
function mark(): int {
    echo "bad";
    return 0;
}
$a = null;
$items = [1];
echo isset($a, $items[mark()]) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "no");
}

/// Verifies isset array element empty string and missing key.
#[test]
fn test_isset_array_element_empty_string_and_missing_key() {
    let out = compile_and_run(
        r#"<?php
$items = [""];
echo isset($items[0]);
echo isset($items[1]);
$mixed = [null, 0];
echo isset($mixed[0]);
echo isset($mixed[1]);
$map = ["name" => ""];
echo isset($map["name"]);
echo isset($map["missing"]);
"#,
    );
    // `isset` is a bool: echoing `false` yields "" (not "0"), matching PHP.
    // Set/false sequence T,F,F,T,T,F renders as "1","","","1","1","" = "111".
    assert_eq!(out, "111");
}

/// Verifies unset multiple variables.
#[test]
fn test_unset_multiple_variables() {
    let out = compile_and_run(
        r#"<?php
$a = 1;
$b = 2;
unset($a, $b);
echo isset($a) ? "a\n" : "na\n";
echo isset($b) ? "b\n" : "nb\n";
"#,
    );
    assert_eq!(out, "na\nnb\n");
}

/// Verifies `unset($hash[$key])` removes a string-keyed associative-array entry, leaving the
/// remaining entries, their iteration order, and `count()`/`isset()` consistent with PHP.
#[test]
fn test_unset_assoc_string_key() {
    let out = compile_and_run(
        r#"<?php
$m = ['a' => 1, 'b' => 2, 'c' => 3];
unset($m['b']);
echo count($m), "\n";
foreach ($m as $k => $v) { echo "$k=$v\n"; }
echo isset($m['b']) ? "has-b\n" : "no-b\n";
echo isset($m['a']) ? "has-a\n" : "no-a\n";
"#,
    );
    assert_eq!(out, "2\na=1\nc=3\nno-b\nhas-a\n");
}

/// Verifies a removed entry leaves a tombstone that keeps probe chains intact: after removing
/// a key, later inserts still resolve, and re-adding the removed key appends it at the end in
/// PHP insertion order.
#[test]
fn test_unset_assoc_then_reinsert_preserves_order() {
    let out = compile_and_run(
        r#"<?php
$m = ['a' => 1, 'b' => 2, 'c' => 3];
unset($m['a']);
$m['d'] = 4;
$m['a'] = 99;
foreach ($m as $k => $v) { echo "$k=$v "; }
echo "\n", count($m), "\n";
echo $m['c'], "\n";
"#,
    );
    assert_eq!(out, "b=2 c=3 d=4 a=99 \n4\n3\n");
}

/// Verifies `unset()` on an integer-keyed associative array removes the matching entry.
#[test]
fn test_unset_assoc_int_key() {
    let out = compile_and_run(
        r#"<?php
$m = [0 => 'x', 1 => 'y', 2 => 'z'];
unset($m[1]);
foreach ($m as $k => $v) { echo "$k=$v "; }
echo "\n", count($m), "\n";
"#,
    );
    assert_eq!(out, "0=x 2=z \n2\n");
}

/// Verifies copy-on-write: removing a key from a copy of an associative array does not mutate
/// the shared original.
#[test]
fn test_unset_assoc_copy_on_write() {
    let out = compile_and_run(
        r#"<?php
$a = ['x' => 1, 'y' => 2, 'z' => 3];
$b = $a;
unset($b['x']);
echo "a:"; foreach ($a as $k => $v) { echo " $k=$v"; }
echo "\nb:"; foreach ($b as $k => $v) { echo " $k=$v"; }
echo "\n";
"#,
    );
    assert_eq!(out, "a: x=1 y=2 z=3\nb: y=2 z=3\n");
}

/// Verifies removing entries that own heap payloads (a string and a nested array) releases them
/// without corrupting the surviving entries.
#[test]
fn test_unset_assoc_releases_heap_values() {
    let out = compile_and_run(
        r#"<?php
$m = ['s' => 'hello world', 'arr' => [1, 2, 3], 'n' => 5];
unset($m['s']);
unset($m['arr']);
foreach ($m as $k => $v) { echo "$k=$v "; }
echo "\n", count($m), "\n";
"#,
    );
    assert_eq!(out, "n=5 \n1\n");
}

/// Verifies repeatedly setting and unsetting an associative-array key in a bounded heap does not
/// leak storage (the loop would exhaust the heap if the removed values were not released).
#[test]
fn test_unset_assoc_no_leak_under_churn() {
    let out = compile_and_run(
        r#"<?php
$m = [];
for ($i = 0; $i < 5000; $i++) {
    $m['key'] = "value-" . $i;
    unset($m['key']);
}
echo count($m), "\n";
echo "done\n";
"#,
    );
    assert_eq!(out, "0\ndone\n");
}

/// Verifies unsetting a key that is absent from an associative array is a no-op.
#[test]
fn test_unset_assoc_missing_key_is_noop() {
    let out = compile_and_run(
        r#"<?php
$m = ['a' => 1, 'b' => 2];
unset($m['zzz']);
echo count($m), "\n";
foreach ($m as $k => $v) { echo "$k=$v "; }
echo "\n";
"#,
    );
    assert_eq!(out, "2\na=1 b=2 \n");
}

/// Verifies `unset($arr[$key])` on a packed indexed array removes the element without renumbering
/// the survivors: PHP keeps the original keys (a hole), so the array becomes sparse/associative.
#[test]
fn test_unset_indexed_creates_hole() {
    let out = compile_and_run(
        r#"<?php
$arr = [1, 2, 3];
unset($arr[1]);
foreach ($arr as $k => $v) { echo "$k=$v "; }
echo "\n", count($arr), "\n";
echo isset($arr[1]) ? "has1\n" : "no1\n";
echo isset($arr[2]) ? "has2\n" : "no2\n";
"#,
    );
    assert_eq!(out, "0=1 2=3 \n2\nno1\nhas2\n");
}

/// Verifies that appending after an indexed unset continues at `max_key + 1`, matching PHP.
#[test]
fn test_unset_indexed_then_append_continues_max_key() {
    let out = compile_and_run(
        r#"<?php
$arr = [1, 2, 3];
unset($arr[1]);
$arr[] = 9;
foreach ($arr as $k => $v) { echo "$k=$v "; }
echo "\n";
"#,
    );
    assert_eq!(out, "0=1 2=3 3=9 \n");
}

/// Verifies indexed-array element unset inside a function local (the array is converted to a hash
/// at the unset site).
#[test]
fn test_unset_indexed_in_function_local() {
    let out = compile_and_run(
        r#"<?php
function dump(): void {
    $arr = [10, 20, 30, 40];
    unset($arr[1]);
    foreach ($arr as $k => $v) { echo "$k=$v "; }
    echo "\n";
}
dump();
"#,
    );
    assert_eq!(out, "0=10 2=30 3=40 \n");
}

/// Verifies indexed-array element unset on a by-value array parameter.
#[test]
fn test_unset_indexed_by_value_param() {
    let out = compile_and_run(
        r#"<?php
function strip(array $a): int {
    unset($a[1]);
    return count($a);
}
echo strip([1, 2, 3]), "\n";
"#,
    );
    assert_eq!(out, "2\n");
}

/// Verifies copy-on-write for the indexed-unset conversion path: removing an element from a copy
/// does not mutate the shared original packed array.
#[test]
fn test_unset_indexed_copy_on_write() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3, 4];
$b = $a;
unset($b[1]);
echo "a:"; foreach ($a as $k => $v) { echo " $k=$v"; }
echo "\nb:"; foreach ($b as $k => $v) { echo " $k=$v"; }
echo "\n";
"#,
    );
    assert_eq!(out, "a: 0=1 1=2 2=3 3=4\nb: 0=1 2=3 3=4\n");
}

/// Verifies unsetting an element of an empty array is a no-op (the array stays empty and can still
/// be appended to afterwards).
#[test]
fn test_unset_indexed_empty_array_noop() {
    let out = compile_and_run(
        r#"<?php
$arr = [];
unset($arr[0]);
echo count($arr), "\n";
$arr[] = 5;
echo $arr[0], "\n";
"#,
    );
    assert_eq!(out, "0\n5\n");
}

/// Verifies isset string offset respects bounds.
#[test]
fn test_isset_string_offset_respects_bounds() {
    let out = compile_and_run(
        r#"<?php
$s = "abc";
echo isset($s[0]) ? "y\n" : "n\n";
echo isset($s[3]) ? "y\n" : "n\n";
echo isset($s[-1]) ? "y\n" : "n\n";
echo isset($s[-4]) ? "y\n" : "n\n";
"#,
    );
    assert_eq!(out, "y\nn\ny\nn\n");
}

/// Verifies isset array offset respects bounds for non scalar elements.
#[test]
fn test_isset_array_offset_respects_bounds_for_non_scalar_elements() {
    let out = compile_and_run(
        r#"<?php
$a = ["x"];
echo isset($a[0]) ? "y\n" : "n\n";
echo isset($a[1]) ? "y\n" : "n\n";
"#,
    );
    assert_eq!(out, "y\nn\n");
}

/// Verifies isset null variable is false.
#[test]
fn test_isset_null_variable_is_false() {
    // `isset` is a bool: `isset($x)` on null is `false`, which echoes as "" (not
    // "0") in PHP; `isset($y)` on 0 is `true`, echoing "1". So the result is "1".
    let out = compile_and_run("<?php $x = null; $y = 0; echo isset($x); echo isset($y);");
    assert_eq!(out, "1");
}

/// An assignment inside an `isset()` array-index operand defines the assigned variable for code
/// that runs after the `isset()` call, mirroring PHP's always-evaluated index-expression
/// semantics (php-verified: `isset($a[$h = f()])` defines `$h` even when the outer index does
/// not exist in `$a`). Matches the RedisTrait `!isset($connections[$h = $redis->_target($id)])`
/// shape.
#[test]
fn test_isset_array_index_assignment_defines_variable_after_call() {
    let out = compile_and_run(
        r#"<?php
function target($id) { return "h_" . $id; }
$connections = [];
if (!isset($connections[$h = target(5)])) {
    $connections[$h] = "conn";
}
echo $h, "\n";
echo $connections[$h], "\n";
"#,
    );
    assert_eq!(out, "h_5\nconn\n");
}

/// The same always-evaluated-index rule applies to `unset()`'s operand: an assignment inside the
/// index expression defines the variable for code after the `unset()` call.
#[test]
fn test_unset_array_index_assignment_defines_variable_after_call() {
    let out = compile_and_run(
        r#"<?php
$a = [10, 20, 30];
unset($a[$k = 1]);
echo $k, "\n";
echo count($a), "\n";
"#,
    );
    assert_eq!(out, "1\n2\n");
}

/// A by-reference out-parameter call nested inside an `isset()` index expression still defines
/// its output variable after the call, exactly like the same call outside `isset()` (JURY
/// ADDENDUM #3's "nested by-reference in isset" regression probe).
#[test]
fn test_isset_array_index_nested_by_ref_output_defines_variable() {
    let out = compile_and_run(
        r#"<?php
$arr = [1, 2, 3];
if (isset($arr[preg_match('/\d/', 'x', $matches) ? 0 : 1])) {
    echo "matched\n";
}
echo count($matches), "\n";
"#,
    );
    assert_eq!(out, "matched\n0\n");
}

/// Regression: `isset()` on an undeclared property still routes through `__isset` instead of
/// being rejected as a bare property access — the property-magic skip the isset/unset lazy-
/// construct path exists for must survive walking always-evaluated index sub-expressions.
#[test]
fn test_isset_undeclared_property_still_routes_through_magic_isset() {
    let out = compile_and_run(
        r#"<?php
class Bar {
    private array $data = [];
    public function __isset($name) { return isset($this->data[$name]); }
}
$b = new Bar();
echo isset($b->undeclaredProp) ? "y" : "n", "\n";
"#,
    );
    assert_eq!(out, "n\n");
}

/// Verifies array values.
#[test]
fn test_array_values() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; $v = array_values($a); foreach ($v as $x) { echo $x; }"#,
    );
    assert_eq!(out, "102030");
}

/// Verifies die.
#[test]
fn test_die() {
    let out = compile_and_run("<?php echo \"before\"; die(); echo \"after\";");
    assert_eq!(out, "before");
}

// --- Nested control flow ---

/// Verifies nested if.
#[test]
fn test_nested_if() {
    let out = compile_and_run(
        "<?php $x = 5; if ($x > 0) { if ($x > 3) { echo \"big\"; } else { echo \"small\"; } }",
    );
    assert_eq!(out, "big");
}

/// Verifies nested loops.
#[test]
fn test_nested_loops() {
    let out = compile_and_run(
        "<?php for ($i = 0; $i < 3; $i++) { for ($j = 0; $j < 2; $j++) { echo $i . $j . \" \"; } }",
    );
    assert_eq!(out, "00 01 10 11 20 21 ");
}

/// Verifies for continue.
#[test]
fn test_for_continue() {
    let out =
        compile_and_run("<?php for ($i = 0; $i < 5; $i++) { if ($i == 2) { continue; } echo $i; }");
    assert_eq!(out, "0134");
}

/// Verifies while with function.
#[test]
fn test_while_with_function() {
    let out = compile_and_run(
        r#"<?php
function sum_to($n) {
    $s = 0;
    $i = 1;
    while ($i <= $n) {
        $s = $s + $i;
        $i++;
    }
    return $s;
}
echo sum_to(10);
"#,
    );
    assert_eq!(out, "55");
}

/// Verifies function with if return.
#[test]
fn test_function_with_if_return() {
    let out = compile_and_run(
        r#"<?php
function abs_val($x) {
    if ($x < 0) {
        return -$x;
    }
    return $x;
}
echo abs_val(-5) . " " . abs_val(3);
"#,
    );
    assert_eq!(out, "5 3");
}

/// Verifies function calling function.
#[test]
fn test_function_calling_function() {
    let out = compile_and_run(
        r#"<?php
function square($x) { return $x * $x; }
function sum_of_squares($a, $b) { return square($a) + square($b); }
echo sum_of_squares(3, 4);
"#,
    );
    assert_eq!(out, "25");
}

/// Verifies multiple elseif.
#[test]
fn test_multiple_elseif() {
    let out = compile_and_run(
        r#"<?php
$x = 4;
if ($x == 1) { echo "one"; }
elseif ($x == 2) { echo "two"; }
elseif ($x == 3) { echo "three"; }
elseif ($x == 4) { echo "four"; }
else { echo "other"; }
"#,
    );
    assert_eq!(out, "four");
}

/// Regression: `in_array()` with a string needle must work over an indexed `array<Mixed>`. A
/// function whose container return is built from an untyped parameter is lowered to `array<Mixed>`
/// (each element a boxed Mixed cell), as is a `foreach`-value collected into a fresh array. Before
/// the fix the backend rejected `in_array(Str, array<Mixed>)` with an "unsupported" error; the
/// scan now unboxes each cell and string-compares the string-tagged ones.
#[test]
fn test_in_array_string_needle_over_mixed_array() {
    let out = compile_and_run(
        r#"<?php
function collect($x) { $r = []; $r[] = $x; return $r; }
$a = collect("hello");
$names = [];
foreach (["alpha", "beta", "gamma"] as $n) { $names[] = $n; }
echo (in_array("hello", $a) ? "y" : "n"),
     (in_array("beta", $names) ? "y" : "n"),
     (in_array("missing", $names) ? "y" : "n");
"#,
    );
    assert_eq!(out, "yyn");
}

/// Regression: `in_array()` with a `Mixed` needle must work over a concrete indexed `array<Str>`
/// (the inverse of the string-needle / Mixed-array case). An untyped function parameter is a boxed
/// `Mixed` value; searching it against a literal string array surfaced in symfony/yaml. Each string
/// element is boxed into a temporary Mixed cell and compared with the boxed needle: loose mode uses
/// the PHP 8 three-way comparison helper (so a numeric needle can match a numeric string element,
/// e.g. `in_array(1, ["1"])` is true), while strict mode uses runtime tag identity (so `1 !== "1"`).
#[test]
fn test_in_array_mixed_needle_over_string_array() {
    let out = compile_and_run(
        r#"<?php
function check($needle) {
    $arr = ["a", "b", "c"];
    return in_array($needle, $arr) ? "y" : "n";
}
function loose_num($needle) {
    return in_array($needle, ["1", "x"]) ? "y" : "n";
}
function strict($needle) {
    return in_array($needle, ["1", "b"], true) ? "y" : "n";
}
echo check("b"), check("x"), check(2), "|",
     loose_num(1), loose_num(9), "|",
     strict("b"), strict(1);
"#,
    );
    // check: string hit, string miss, non-numeric int miss (2 cast to "2" != "a"/"b"/"c").
    // loose_num: int 1 loose-matches string "1"; int 9 misses.
    // strict: string "b" identity-matches; int 1 never identity-matches string "1".
    assert_eq!(out, "ynn|yn|yn");
}

// --- Long-form `array(...)` literal ---

/// Verifies that the long-form `array(...)` produces an indexed array equivalent to `[...]`.
#[test]
fn test_long_array_indexed() {
    let out = compile_and_run("<?php $a = array(10, 20, 30); echo count($a) . \":\" . $a[0] . \":\" . $a[2];");
    assert_eq!(out, "3:10:30");
}

/// Verifies that an empty long-form `array()` is an empty array.
#[test]
fn test_long_array_empty() {
    let out = compile_and_run("<?php $a = array(); echo count($a);");
    assert_eq!(out, "0");
}

/// Verifies that long-form `array("k" => v)` produces an associative array with the given keys.
#[test]
fn test_long_array_assoc() {
    let out = compile_and_run(
        "<?php $m = array(\"a\" => 1, \"b\" => 2); echo $m[\"a\"] + $m[\"b\"];",
    );
    assert_eq!(out, "3");
}

/// Verifies that a runtime-valued key works in a long-form `array($k => v)` literal.
#[test]
fn test_long_array_dynamic_key() {
    let out = compile_and_run("<?php $k = \"dyn\"; $kv = array($k => 42); echo $kv[\"dyn\"];");
    assert_eq!(out, "42");
}

/// Verifies that long-form arrays nest like the short form.
#[test]
fn test_long_array_nested() {
    let out = compile_and_run(
        "<?php $n = array(\"x\" => array(1, 2), \"y\" => 3); echo count($n[\"x\"]) . \":\" . $n[\"y\"];",
    );
    assert_eq!(out, "2:3");
}

/// Verifies mixed positional and keyed entries in a long-form array (positional elements keep
/// their auto-incremented integer keys around the explicit string key, as in PHP).
#[test]
fn test_long_array_mixed_positional_and_keyed() {
    let out = compile_and_run(
        "<?php $m = array(10, \"k\" => 20, 30); echo $m[0] . \":\" . $m[\"k\"] . \":\" . $m[1];",
    );
    assert_eq!(out, "10:20:30");
}

/// Verifies that spread (`...`) works inside a long-form array literal.
#[test]
fn test_long_array_spread() {
    let out = compile_and_run("<?php $s = array(...array(1, 2), 3); echo count($s);");
    assert_eq!(out, "3");
}

/// Verifies that the long-form keyword is case-insensitive (`ARRAY(...)`), matching PHP.
#[test]
fn test_long_array_case_insensitive() {
    let out = compile_and_run("<?php $a = ARRAY(1, 2); echo count($a);");
    assert_eq!(out, "2");
}

/// Verifies that the short `[...]` and long `array(...)` forms interoperate: a long-form array
/// passed to a builtin (`array_merge`) combines with a short-form array as expected.
#[test]
fn test_long_array_interops_with_short_form() {
    let out = compile_and_run(
        "<?php $a = array(1, 2); $b = [3, 4]; $c = array_merge($a, $b); echo count($c) . \":\" . $c[0] . \":\" . $c[3];",
    );
    assert_eq!(out, "4:1:4");
}

/// Verifies `end()` returns the last element of a non-empty array, for both a literal argument
/// and a variable. elephc models only the last-element read (it has no internal array pointer),
/// boxing the element through the `__rt_end_boxed` runtime helper.
#[test]
fn test_end_returns_last_element() {
    let out = compile_and_run("<?php echo end([1, 2, 3]);");
    assert_eq!(out, "3");
    let out = compile_and_run("<?php $a = [10, 20, 30]; echo end($a);");
    assert_eq!(out, "30");
}

/// Verifies the canonical builtin catalog exposes `end()` to `function_exists()` and lets
/// case-insensitive unqualified calls inside a namespace fall back to the global builtin.
#[test]
fn test_end_catalog_supports_case_insensitive_namespace_fallback() {
    let out = compile_and_run(
        r#"<?php
namespace App\Catalog;
$values = [10, 20, 30];
echo \function_exists("EnD") ? "Y:" : "N:";
echo EnD($values);
"#,
    );
    assert_eq!(out, "Y:30");
}

/// Verifies `end()` on an empty array returns `false`, matching PHP's empty-array behavior.
/// `var_dump` renders the boxed `false` result so the bool type is observable.
#[test]
fn test_end_on_empty_array_returns_false() {
    let out = compile_and_run("<?php $a = []; var_dump(end($a));");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies `end()` accepts a `Mixed`/union-containing-array argument under the gradual-typing
/// boundary: an array read from a heterogeneous associative array (boxed as `Mixed`) is unboxed
/// and its last element returned.
#[test]
fn test_end_on_mixed_array_argument() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"a\"] = [10, 20, 30];
        $h[\"b\"] = \"s\";
        $arr = $h[\"a\"];
        echo end($arr);
        ",
    );
    assert_eq!(out, "30");
}

/// Verifies count() accepts a genuinely `Mixed`-typed argument (a `mixed` function
/// parameter) under the gradual-typing boundary and returns the element count at
/// runtime. Regression for the checker wrongly rejecting `count($mixed)`.
#[test]
fn test_count_mixed_argument() {
    let out = compile_and_run(
        r#"<?php
function n(mixed $x): int { return count($x); }
echo n([1, 2, 3, 4]);
"#,
    );
    assert_eq!(out, "4");
}

/// Verifies appending a statically PHP-null (`Void`) value into an `array<never>` indexed
/// array: the null is stored as an 8-byte sentinel slot, the length grows, and the element
/// reads back as null. Regression for the symfony/yaml `array_push for PHP type Void` gap.
#[test]
fn test_array_push_null_into_empty_array() {
    let out = compile_and_run(
        r#"<?php
$a = [];
$x = null;
$a[] = $x;
echo count($a);
echo "|";
echo is_null($a[0]) ? "null" : "set";
"#,
    );
    assert_eq!(out, "1|null");
}

/// Regression: the loop-widening prescan must not treat a variable defined only inside
/// the loop from a non-literal source as `mixed` evidence. The compiler-synthesized
/// `MultipleIterator::detachIterator` body rebuilds an `array<Iterator>` through such a
/// variable; a spurious widen made its typed-property storeback fail to compile.
#[test]
fn test_loop_rebuild_of_typed_array_not_spuriously_widened() {
    let out = compile_and_run(
        r#"<?php
$multi = new MultipleIterator();
$it = new ArrayIterator([1]);
$multi->attachIterator($it);
$multi->detachIterator($it);
echo $multi->countIterators();
"#,
    );
    assert_eq!(out, "0");
}

/// Regression companion for #452: heterogeneous scalars carried through loop-defined
/// variables (literal assignments inside the body) must still widen the pushed array —
/// the literal-assignment scan supplies the evidence the loop-entry lookup lacks.
#[test]
fn test_loop_grown_mixed_array_via_local_literals() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) {
    $a = 1;
    $vals[] = $a;
    $b = 2.0;
    $vals[] = $b;
}
$sum = 0;
foreach ($vals as $v) { $sum += intval($v); }
echo $sum;
"#,
    );
    assert_eq!(out, "6");
}

/// Regression for #452: a typed call assigned inside the loop must contribute its return
/// type before the pushed array's fixed-point element type is selected.
#[test]
fn test_loop_grown_mixed_array_via_typed_local() {
    let out = compile_and_run(
        r#"<?php
function get_int(): int { return 1; }
$vals = [];
for ($i = 0; $i < 2; $i++) {
    $x = get_int();
    $vals[] = $x;
    $vals[] = 2.0;
}
echo $vals[2];
"#,
    );
    assert_eq!(out, "1");
}

/// Regression for #452: two differently typed call results must widen the array before
/// either append is lowered; treating both calls as opaque left the first append raw and
/// corrupted mixed storage on the loop back edge.
#[test]
fn test_loop_grown_mixed_array_via_typed_calls() {
    let out = compile_and_run(
        r#"<?php
function get_int(): int { return 1; }
function get_float(): float { return 2.0; }
$vals = [];
for ($i = 0; $i < 2; $i++) {
    $vals[] = get_int();
    $vals[] = get_float();
}
echo $vals[2];
"#,
    );
    assert_eq!(out, "1");
}

/// Regression for #452: declared member and array-element types must reach the EIR
/// loop prescan, keeping its fixed-point decision aligned with the type checker.
#[test]
fn test_loop_grown_mixed_array_via_typed_member_and_element_reads() {
    let out = compile_and_run(
        r#"<?php
class Values {
    public int $intValue = 1;
    public float $floatValue = 2.0;
    public function getInt(): int { return 1; }
    public function getFloat(): float { return 2.0; }
    public static function staticInt(): int { return 1; }
    public static function staticFloat(): float { return 2.0; }
}

$source = new Values();
$methods = [];
$statics = [];
$properties = [];
$elements = [];
$ints = [1];
$floats = [2.0];
for ($i = 0; $i < 2; $i++) {
    $methods[] = $source->getInt();
    $methods[] = $source->getFloat();
    $statics[] = Values::staticInt();
    $statics[] = Values::staticFloat();
    $properties[] = $source->intValue;
    $properties[] = $source->floatValue;
    $elements[] = $ints[0];
    $elements[] = $floats[0];
}
echo $methods[2], $statics[2], $properties[2], $elements[2];
"#,
    );
    assert_eq!(out, "1111");
}

/// Regression for #452: an in-loop typed reassignment must override stale loop-entry
/// evidence when the assigned variable is appended to an array that later becomes mixed.
#[test]
fn test_loop_grown_mixed_array_via_reassigned_entry_local() {
    let out = compile_and_run(
        r#"<?php
function get_float(): float { return 2.0; }
$x = 0;
$vals = [];
for ($i = 0; $i < 2; $i++) {
    $x = get_float();
    $vals[] = $x;
    $vals[] = 1;
}
echo intval($vals[2]);
"#,
    );
    assert_eq!(out, "2");
}

/// Regression for #452: `array_push` is a growth site equivalent to `$a[] =` for the
/// loop-widening prescan; omitting it left the same raw-into-mixed corruption.
#[test]
fn test_loop_grown_mixed_array_via_array_push() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) {
    array_push($vals, 1);
    array_push($vals, 2.0);
}
echo $vals[2];
"#,
    );
    assert_eq!(out, "1");
}

/// Regression for #452: indexed writes that grow the array (`$a[count($a)] =`) share the
/// same single-pass / back-edge representation bug as `$a[] =`.
#[test]
fn test_loop_grown_mixed_array_via_index_assign() {
    let out = compile_and_run(
        r#"<?php
$vals = [];
for ($i = 0; $i < 2; $i++) {
    $vals[count($vals)] = 1;
    $vals[count($vals)] = 2.0;
}
echo $vals[2];
"#,
    );
    assert_eq!(out, "1");
}

/// EC-2 (#485): verifies `in_array()` accepts the optional 3rd `strict` argument
/// and preserves exact membership for same-typed string and int arrays.
#[test]
fn test_in_array_strict_flag() {
    let out = compile_and_run(
        "<?php echo in_array('b', ['a','b','c'], true) ? '1' : '0'; echo in_array('x', ['a','b'], true) ? '1' : '0'; echo in_array(2, [1,2,3], true) ? '1' : '0'; echo in_array(9, [1,2], true) ? '1' : '0';",
    );
    assert_eq!(out, "1010");
}

/// Verifies loose `in_array()` parses a string needle as a PHP numeric string when
/// searching an integer array, while `strict=true` rejects the cross-type match.
#[test]
fn test_in_array_loose_numeric_string_needle_matches_int_array() {
    let out = compile_and_run(
        "<?php echo in_array('2', [1,2,3]) ? '1' : '0'; echo in_array('2', [1,2,3], true) ? '1' : '0'; echo in_array('2.0', [1,2,3]) ? '1' : '0'; echo in_array('2.5', [1,2,3]) ? '1' : '0'; echo in_array('foo', [0]) ? '1' : '0';",
    );
    assert_eq!(out, "10100");
}

/// Verifies loose `in_array()` parses string array elements as PHP numeric strings
/// when the needle is an integer, and keeps `strict=true` type-identical.
#[test]
fn test_in_array_loose_int_needle_matches_numeric_string_array() {
    let out = compile_and_run(
        "<?php echo in_array(2, ['1','02','3']) ? '1' : '0'; echo in_array(2, ['2.0']) ? '1' : '0'; echo in_array(2, ['2.5']) ? '1' : '0'; echo in_array(0, ['foo']) ? '1' : '0'; echo in_array(2, ['02'], true) ? '1' : '0';",
    );
    assert_eq!(out, "11000");
}

/// Verifies loose string-array membership uses PHP string `==` semantics, where
/// numeric strings compare by numeric value and non-numeric strings compare by bytes.
#[test]
fn test_in_array_loose_string_array_uses_string_loose_equality() {
    let out = compile_and_run(
        "<?php echo in_array('2', ['02']) ? '1' : '0'; echo in_array('2', ['02'], true) ? '1' : '0'; echo in_array('2a', ['2']) ? '1' : '0'; echo in_array('foo', ['foo']) ? '1' : '0';",
    );
    assert_eq!(out, "1001");
}

/// Verifies loose bool/int membership compares PHP truthiness, while `strict=true`
/// distinguishes booleans from integers.
#[test]
fn test_in_array_strict_distinguishes_bool_int_membership() {
    let out = compile_and_run(
        "<?php echo in_array(true, [2]) ? '1' : '0'; echo in_array(true, [2], true) ? '1' : '0'; echo in_array(false, [0]) ? '1' : '0'; echo in_array(false, [0], true) ? '1' : '0'; echo in_array(2, [true]) ? '1' : '0'; echo in_array(2, [true], true) ? '1' : '0';",
    );
    assert_eq!(out, "101010");
}
