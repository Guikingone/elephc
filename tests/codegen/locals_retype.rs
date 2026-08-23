//! Purpose:
//! End-to-end coverage for local binding kill (`unset`) and retype-to-fresh-slot lowering.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every fixture derives its value from `$argc` so constant folding cannot erase the
//!   construct under test before lowering sees it.

use crate::support::*;

/// unset kills the int binding; the string reassignment gets a fresh heap-typed slot.
#[test]
fn test_unset_then_retype_int_to_string() {
    let out = compile_and_run("<?php $a = $argc; unset($a); $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// unset releases the owned heap string before the int rebind (no leak, no UAF).
#[test]
fn test_unset_then_retype_string_to_int() {
    let out = compile_and_run("<?php $a = \"ciao\" . $argc; unset($a); $a = 7; echo $a;");
    assert_eq!(out, "7");
}

/// unset without a following reassignment still compiles and runs.
#[test]
fn test_unset_without_reassignment() {
    let out = compile_and_run("<?php $a = $argc; unset($a); echo \"ok\";");
    assert_eq!(out, "ok");
}

/// PHP still allows `isset`/`empty`/`??` on the unbound name, and they must answer "not set"
/// instead of reading the abandoned slot.
///
/// Those probes lower as an ordinary variable load. When the kill dropped the name's type fact
/// outright, that load minted a `Mixed` slot and read uninitialized frame storage as a pointer:
/// this fixture (and `array_basics::test_unset_multiple_variables`) segfaulted. The kill now
/// leaves the name typed `Void`, which is the state a never-assigned name already has.
#[test]
fn test_probes_after_unset_see_an_unbound_name() {
    let out = compile_and_run(
        r#"<?php
$a = "s" . $argc;
$b = $argc;
unset($a, $b);
echo isset($a) ? "set" : "unset";
echo empty($b) ? "|empty" : "|full";
echo "|", $a ?? "dflt";"#,
    );
    assert_eq!(out, "unset|empty|dflt");
}

/// Two kills of the same name in one body each abandon their own slot.
#[test]
fn test_repeated_kill_and_rebind_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = \"x\" . $argc; unset($a); $a = \"y\" . $argc; unset($a); $a = 3; echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "3");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The heap value the retyped binding allocates is owned by the FRESH slot alone.
///
/// Before the kill was lowered, `unset` null-stored into the existing `Int` slot and the
/// following string store widened that slot to `Mixed`. Every later store/release then went
/// through boxed storage the entry-typed slot never released, leaking three blocks per run
/// (`HEAP DEBUG: live_blocks=3`) while still printing the right answer — a silent leak, not a
/// wrong result, which is why the plain fixtures above pass either way.
#[test]
fn test_unset_then_retype_int_to_string_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = $argc; unset($a); $a = \"ciao\" . $argc; echo strlen($a), \"|\", $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5|ciao1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The owned heap string is released AT the kill, not leaked past the int rebind.
#[test]
fn test_unset_then_retype_string_to_int_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = \"ciao\" . $argc; unset($a); $a = 7; echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "7");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// An array binding killed by `unset` releases its table before the string rebind.
#[test]
fn test_unset_then_retype_array_to_string_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = [1, $argc]; unset($a); $a = \"str\" . $argc; echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "str1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A `string`-typed READ above the kill still answers correctly.
///
/// Ending a binding leaves the abandoned slot at its OWN storage type and zeroes it in place
/// (`Op::ZeroLocalSlot`), so a `Str` load lowered ABOVE the kill still reads `Str` storage. When
/// the kill nulled through the ordinary overwrite path at `PhpType::Void` instead, the slot
/// widened to `Mixed` frame-wide and every such read became an unreleased
/// `__rt_mixed_cast_string` detach — one leaked block per EXECUTED read.
#[test]
fn test_string_read_above_a_kill_still_answers_correctly() {
    let out = compile_and_run("<?php $a = \"n\" . $argc; $b = strlen($a); unset($a); $a = 5; echo $b, $a;");
    assert_eq!(out, "25");
}

/// Every `string` read ABOVE a retype is leak-free, however many of them execute.
///
/// The leak this pins was LINEAR in executed reads: three reads left three detached copies of
/// the string behind (`live_blocks=3`), because the abandon widened the slot to `Mixed` while
/// each read kept its `Str` IR type.
#[test]
fn test_string_reads_above_a_retype_leave_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): int {
    $a = "x" . $n;
    $t = strlen($a) + strlen($a) + strlen($a);
    $a = 5;
    return $t + $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "11");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same for the `unset` kill, which is where the widening arrived.
#[test]
fn test_string_reads_above_a_kill_leave_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): int {
    $a = "x" . $n;
    $t = strlen($a) + strlen($a) + strlen($a);
    unset($a);
    $a = 5;
    return $t + $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "11");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A LOOP of reads above a retype finishes instead of exhausting the heap.
///
/// One leaked copy per executed read makes the leak unbounded in a loop: this shape printed
/// `Fatal error: heap memory exhausted` where PHP prints the sum.
#[test]
fn test_a_loop_of_string_reads_above_a_retype_finishes() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): int {
    $a = "x" . $n;
    $t = 0;
    for ($i = 0; $i < 200000; $i++) { $t += strlen($a); }
    $a = 5;
    return $t + $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "400005");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same loop shape at TOP LEVEL, where the frame is `main`'s.
#[test]
fn test_a_loop_of_string_reads_above_a_top_level_retype_finishes() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = "x" . $argc;
$t = 0;
for ($i = 0; $i < 200000; $i++) { $t += strlen($a); }
$a = 5;
echo $t + $a;"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "400005");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A 100KB string read above a retype leaks no COPY of it.
///
/// The leaked block was a full detached duplicate, not a 48-byte box: this shape leaked
/// 131,088 bytes on top of the string itself.
#[test]
fn test_a_large_string_read_above_a_retype_leaks_no_copy() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): int {
    $a = str_repeat("x", 40000 + $n);
    $t = strlen($a);
    $a = 5;
    return $t + $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "40006");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The copy taken out of a local BEFORE it is retyped stays valid afterwards.
///
/// Direct pin for the use-after-free above: `$r` holds the same buffer as `$q`, and the retype's
/// release must not be an unconditional free.
#[test]
fn test_copy_taken_before_a_retype_survives_it() {
    let out = compile_and_run(
        r#"<?php
function probe(int $n): string {
    $q = "a" . $n;
    $r = $q;
    $q = 1;
    return $r . "|" . $q;
}
echo probe($argc);"#,
    );
    assert_eq!(out, "a1|1");
}

/// The same for the `unset` kill.
#[test]
fn test_copy_taken_before_a_kill_survives_it() {
    let out = compile_and_run(
        r#"<?php
function probe(int $n): string {
    $q = "a" . $n;
    $r = $q;
    unset($q);
    $q = 1;
    return $r . "|" . $q;
}
echo probe($argc);"#,
    );
    assert_eq!(out, "a1|1");
}

/// Killing a by-value PARAMETER binding does not over-release the caller's value.
///
/// The abandon releases the slot's occupant, so the frame has to OWN that occupant — which for a
/// parameter means the prologue's retain, and the prologue emits it only for a parameter slot the
/// frame writes. When the abandon stopped writing the slot through `StoreLocal`, that retain
/// disappeared while the release stayed: the caller's box was freed early, the allocator handed
/// the block straight back for the returned string, and the caller's own free poisoned it
/// (`grown3` came back as `\xa5` bytes under heap debug).
#[test]
fn test_kill_of_a_by_value_parameter_does_not_over_release_the_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe($a, int $n): string {
    unset($a);
    $a = "grown" . $n;
    return $a;
}
echo probe(1, $argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "grown1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same for the retype form, whose abandon is the same operation.
#[test]
fn test_retype_of_a_by_value_parameter_does_not_over_release_the_argument() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe($a, int $n): string {
    $a = "grown" . $n;
    return $a;
}
echo probe([1, 2], $argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "grown1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same kill/rebind inside a function body, where the frame is torn down on return.
#[test]
fn test_unset_then_retype_in_function_body_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): string {
    $a = $n;
    unset($a);
    $a = "ciao" . $n;
    return $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ciao1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

// ---------------------------------------------------------------------------
// Implicit retype: an incompatible depth-0 reassignment re-binds the name to a
// FRESH slot, with no intervening `unset`.
// ---------------------------------------------------------------------------

/// Implicit retype: int local re-binds to a fresh string slot.
#[test]
fn test_implicit_retype_int_to_string() {
    let out = compile_and_run("<?php $a = $argc; $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// Implicit retype: heap string local re-binds to a fresh int slot (old value released).
#[test]
fn test_implicit_retype_string_to_int() {
    let out = compile_and_run("<?php $a = \"ciao\" . $argc; $a = 7; echo $a;");
    assert_eq!(out, "7");
}

/// The RHS of a retype assignment reads the OLD binding.
#[test]
fn test_retype_rhs_reads_old_value() {
    let out = compile_and_run("<?php $a = $argc; $a = \"n=\" . $a; echo $a;");
    assert_eq!(out, "n=1");
}

/// Retype after a loop that used the old binding.
#[test]
fn test_retype_after_loop() {
    let out = compile_and_run(
        "<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a += $i; } $a = \"done\"; echo $a;",
    );
    assert_eq!(out, "done");
}

/// A by-value closure capture keeps the old value across a later retype.
#[test]
fn test_closure_capture_before_retype() {
    let out = compile_and_run(
        "<?php $a = $argc; $f = function() use ($a) { return $a; }; $a = \"x\"; echo $f() . $a;",
    );
    assert_eq!(out, "1x");
}

/// Fully-constant retype (AST folding may pre-resolve it — output must match either way).
#[test]
fn test_constant_retype() {
    let out = compile_and_run("<?php $a = 3; $a = \"ciao\"; echo $a;");
    assert_eq!(out, "ciao");
}

/// Conditional unset then depth-0 retype: the fresh slot is written on both paths and
/// the old heap value is released exactly once whichever path ran (release must be
/// null-tolerant — the unset path may have nulled the old slot already).
#[test]
fn test_retype_after_conditional_unset_of_heap_local() {
    let out = compile_and_run("<?php $a = \"s\" . $argc; if ($argc > 1) { unset($a); } $a = 7; echo $a;");
    assert_eq!(out, "7");
}

/// The compound-assignment shape (`$x .= "a"`), which the parser lowers into
/// `StmtKind::Assign { value: BinaryOp }` and which is therefore retype-eligible.
///
/// This is the shape where "lower the RHS BEFORE abandoning the old slot" is not an
/// optimization but the whole meaning of the statement: the concatenation's left operand IS
/// the old binding.
#[test]
fn test_compound_assign_retype_reads_the_old_binding() {
    let out = compile_and_run("<?php $x = $argc; $x .= \"a\"; echo $x;");
    assert_eq!(out, "1a");
}

/// The same compound shape with heap accounting: the concatenation's result is owned by the
/// fresh string slot, and the abandoned int slot holds nothing to free.
#[test]
fn test_compound_assign_retype_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug("<?php $x = $argc; $x .= \"a\"; echo $x;");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1a");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A `string` local that is a `++` target, retyped to `int`, then incremented again.
///
/// `CheckResult::string_incdec_locals` is keyed by (scope, name) with no binding identity, so
/// the entry the FIRST binding needs is still present at the retyped one. This pins that the
/// int binding increments as an int (`5` -> `6`) while the string binding still increments as
/// PHP's string increment (`"a1"` -> `"a2"`).
#[test]
fn test_string_incdec_local_retyped_to_int_still_increments_as_int() {
    let out = compile_and_run("<?php $s = \"a\" . $argc; $s++; echo $s; $s = 5; $s++; echo $s;");
    assert_eq!(out, "a26");
}

/// The same shape with heap accounting: the boxed string-incdec storage of the first binding is
/// released at the retype, not carried into (or leaked past) the fresh int slot.
#[test]
fn test_string_incdec_local_retyped_to_int_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $s = \"a\" . $argc; $s++; echo $s; $s = 5; $s++; echo $s;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "a26");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The retyped-away heap string is released at the retype, not leaked past the int rebind.
#[test]
fn test_implicit_retype_string_to_int_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug("<?php $a = \"ciao\" . $argc; $a = 7; echo $a;");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "7");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The heap string the retyped binding allocates is owned by the FRESH slot alone.
#[test]
fn test_implicit_retype_int_to_string_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = $argc; $a = \"ciao\" . $argc; echo strlen($a), \"|\", $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5|ciao1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A retyped ARRAY binding releases its table before the string rebind.
#[test]
fn test_implicit_retype_array_to_string_leaves_a_clean_heap() {
    let out =
        compile_and_run_with_heap_debug("<?php $a = [1, $argc]; $a = \"str\" . $argc; echo $a;");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "str1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The RHS reads the old heap binding and the retype still answers correctly.
///
/// No heap assertion: the `strlen($a)` read sits ABOVE the abandon, so it pays the boxed-detach
/// leak described on `test_string_read_above_a_kill_still_answers_correctly`.
#[test]
fn test_retype_rhs_reads_the_old_heap_value() {
    let out = compile_and_run("<?php $a = \"n\" . $argc; $a = strlen($a); echo $a;");
    assert_eq!(out, "2");
}

/// The retype's NEW value still holds the OLD value: the release must not free storage the
/// fresh binding now owns.
///
/// `$a = [$a]` retypes `string` to `array<string>` while the array element is the very string the
/// old slot holds. A release ordered before the array literal takes its own reference would be a
/// use-after-free. The read of `$a` above the abandon means this shape leaks (same boxed-detach
/// cause as above), so only the ANSWER is asserted.
#[test]
fn test_retype_whose_new_value_contains_the_old_one() {
    let out = compile_and_run("<?php $a = \"s\" . $argc; $a = [$a]; echo $a[0];");
    assert_eq!(out, "s1");
}

/// Retyping an OBJECT-typed binding releases the instance (and runs its destructor) instead of
/// stranding it in the abandoned slot.
#[test]
fn test_implicit_retype_object_to_string_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Box {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function __destruct() { echo "bye|"; }
}
$o = new Box($argc);
echo $o->v, "|";
$o = "gone" . $argc;
echo $o;"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1|bye|gone1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same retype inside a function body, where the frame is torn down on return.
#[test]
fn test_implicit_retype_in_function_body_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): string {
    $a = $n;
    $a = "ciao" . $n;
    return $a;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ciao1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Abandoned OBJECT and ARRAY slots inside a function body, where the frame epilogue — not
/// program exit — is what cleans the frame up.
///
/// The nulled slot keeps its concrete `Box` / `array<int>` storage type, so the epilogue emits
/// refcounted cleanup for it and reads back whatever the retype stored. Both retypes also have to
/// release exactly once: the destructor prints, and it prints BEFORE the new value is echoed.
#[test]
fn test_retyped_object_and_array_slots_survive_the_frame_epilogue() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Box {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function __destruct() { echo "bye|"; }
}
function probe(int $n): string {
    $o = new Box($n);
    $arr = [1, $n];
    echo $o->v, "|", $arr[1], "|";
    $o = "s" . $n;
    $arr = "t" . $n;
    return $o . $arr;
}
echo probe($argc);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1|1|bye|s1t1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

// ---------------------------------------------------------------------------
// Tail-sinking duplicates the straight-line tail of an `if`/`switch`/`try` into
// EVERY branch (`optimize::control::dce`, which runs AFTER type checking). The
// copies share their spans, so one checker decision is consumed once per copy.
// The decision must stay attached to ONE syntactic site.
// ---------------------------------------------------------------------------

/// A retype below an `if` survives the tail being sunk into both branches.
///
/// Measured before the fix: `EIR backend error: unsupported EIR backend feature: local load from
/// PHP type Int as Str`. The `if.then` copy of the tail re-bound `$q` to a fresh slot, and because
/// the abandon mutates the binding maps permanently, the `if.else` copy lowered its `echo $q` —
/// which is ABOVE the retype in source — against that fresh slot.
#[test]
fn test_retype_below_an_if_survives_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; if ($argc > 5) { echo \"x\"; } echo $q; $q = 1; echo \"|\", $q;",
    );
    assert_eq!(out, "a1|1");
}

/// The same shape with TWO retypes, which compiled but printed the wrong answer.
///
/// Measured before the fix: `|s` instead of `a1|s` — the `echo $q` above the retypes read a slot
/// no copy had written yet, so the old binding's value vanished silently.
#[test]
fn test_two_retypes_below_an_if_survive_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; if ($argc > 5) { echo \"x\"; } echo $q; $q = 1; $q = \"s\"; echo \"|\", $q;",
    );
    assert_eq!(out, "a1|s");
}

/// The `unset` kill has the same shape, so the hazard predates the retype hook.
#[test]
fn test_unset_kill_below_an_if_survives_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; if ($argc > 5) { echo \"x\"; } echo $q; unset($q); $q = 5; echo \"|\", $q;",
    );
    assert_eq!(out, "a1|5");
}

/// An `if`/`else` sinks the tail into both written arms.
#[test]
fn test_retype_below_an_if_else_survives_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; if ($argc > 5) { echo \"x\"; } else { echo \"y\"; } echo $q; $q = 1; echo \"|\", $q;",
    );
    assert_eq!(out, "ya1|1");
}

/// `switch` is tail-sunk too, into every case body.
#[test]
fn test_retype_below_a_switch_survives_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; switch ($argc) { case 9: echo \"x\"; break; default: echo \"y\"; } echo $q; $q = 1; echo \"|\", $q;",
    );
    assert_eq!(out, "ya1|1");
}

/// `try` is in the same tail-sinking set.
#[test]
fn test_retype_below_a_try_survives_tail_sinking() {
    let out = compile_and_run(
        "<?php $q = \"a\" . $argc; try { echo \"t\"; } catch (Exception $e) { echo \"c\"; } echo $q; $q = 1; echo \"|\", $q;",
    );
    assert_eq!(out, "ta1|1");
}

/// The heap shape: the string the RE-BOUND binding allocates is owned once, however many copies
/// of the retype the optimizer made.
///
/// The local read above the retype is an `int` here on purpose. The shape with a STRING read
/// above it (the fixtures directly above) is the one that pays the pre-existing boxed-detach
/// leak, which would mask what this fixture is for.
#[test]
fn test_retype_below_an_if_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $n = $argc; if ($argc > 5) { echo \"x\"; } echo $n; $n = \"s\" . $argc; echo \"|\", $n;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1|s1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The `unset` kill variant, likewise allocated and freed exactly once.
#[test]
fn test_unset_kill_below_an_if_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $n = $argc; if ($argc > 5) { echo \"x\"; } echo $n; unset($n); $n = \"s\" . $argc; echo \"|\", $n;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1|s1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A heap binding retyped below an `if` with NO read above it: nothing masks the accounting, so
/// the abandoned string is released exactly once whatever the optimizer duplicated.
#[test]
fn test_retype_of_a_heap_local_below_an_if_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $q = \"a\" . $argc; if ($argc > 5) { echo \"x\"; } $q = 1; echo \"|\", $q;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "|1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same shape inside a function body, where the frame epilogue also has to agree, and where
/// a COPY of the string outlives the retype.
#[test]
fn test_retype_below_an_if_in_a_function_answers_correctly() {
    let out = compile_and_run(
        r#"<?php
function probe(int $n): string {
    $q = "a" . $n;
    if ($n > 5) { echo "x"; }
    $r = $q;
    $q = 1;
    return $r . "|" . $q;
}
echo probe($argc);"#,
    );
    assert_eq!(out, "a1|1");
}

/// A retype in one file must not re-bind an unrelated assignment at the same line and column of
/// another file.
///
/// `Span` carries line/col and nothing about which FILE they name, and include resolution splices
/// every included file's statements into one program without rebasing line numbers. Line 4
/// column 1 of `main.php` and of `lib.php` are therefore the SAME `Span`, and lowering consults
/// the retype decisions at EVERY `StmtKind::Assign`. Here `$x = "s";` (main, line 4) is a real
/// retype and `$w = "b" . $argc;` (lib, line 4) is not: measured before the decisions were keyed
/// by span AND local name, the library's conditional assignment abandoned `$w`'s binding, so the
/// `echo` after the `if` read a fresh slot the untaken branch never wrote and the program printed
/// `|s` where PHP prints `a1|s`.
#[test]
fn test_retype_does_not_reach_a_same_position_assignment_in_another_file() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$x = 1;\n$x = \"s\";\necho \"|\", $x;\n",
            ),
            (
                "lib.php",
                "<?php\n$w = \"a\" . $argc;\nif ($argc > 5) {\n$w = \"b\" . $argc;\n}\necho $w;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "a1|s");
}

/// The residual same-NAME same-position collision is a hard compile error, never a wrong answer.
///
/// `main.php` line 4 column 1 retypes `$q`; `lib.php` line 4 column 1 assigns `$q` inside an `if`.
/// The two spans are equal and the names are equal, so the `(span, name)` key cannot tell them
/// apart and lowering would re-bind the library's conditional assignment as well: measured as
/// printing `|s` where PHP prints `a1|5`, in a program that was a plain compile error before this
/// feature existed. Giving `Span` file identity is the real fix and is out of scope; until then
/// the checker refuses the ambiguous decision outright, so the checker and lowering agree on which
/// programs are accepted and no program silently changes meaning.
#[test]
fn test_same_name_same_position_collision_is_a_compile_error() {
    let error = compile_files_error_message(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho \"|\";\n$q = 5;\necho $q;\n",
            ),
            (
                "lib.php",
                "<?php\n$q = \"a\" . $argc;\nif ($argc > 5) {\n$q = \"b\" . $argc;\n}\necho $q;\n",
            ),
        ],
        "main.php",
    )
    .expect("an ambiguous (span, name) local-binding decision must not compile");
    assert!(
        error.contains("Cannot re-bind $q here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
    assert!(
        error.contains("line 4 column 1"),
        "the diagnostic must name the shared position, got: {error}"
    );
}

/// A collision that STRIPS another body's mixed-storage decisions is caught too, not just one
/// that leaves two live keys behind.
///
/// `lib.php`'s function marks `$a` as branch-divergent and records its two store sites at lines
/// 4 and 6, column 1. `main.php` has two ordinary top-level `$a = …;` statements at the very same
/// positions, and the final top-level scan RE-DECIDES every assignment it sees — dropping any
/// decision already filed under that `(span, name)`. Measured before retired keys were checked:
/// both store sites were removed, `mixed_storage_local_names()` lost `$a` (so constant
/// propagation was no longer blocked for it), the checker still typed the local `Mixed`, and the
/// compiler PANICKED with `strlen cannot lower checked operand type Int` on valid PHP.
#[test]
fn test_stripped_mixed_storage_decisions_are_a_compile_error() {
    let error = compile_files_error_message(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n\n$a = 42;\necho \"|\";\n$a = 99;\necho $a;\n",
            ),
            (
                "lib.php",
                "<?php\nfunction f($n) {\n    if ($n > 1) {\n$a = 42;\n    } else {\n$a = \"hello\";\n    }\n    echo strlen($a), \"|\";\n}\nf($argc);\n",
            ),
        ],
        "main.php",
    )
    .expect("a collision that strips a mixed-storage decision must not compile");
    assert!(
        error.contains("Cannot re-bind $a here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
}

/// The PARTIAL-strip variant of the fixture above: only the FIRST store site collides.
///
/// It compiled before, and printed the right answer — but only by widening luck: lowering saw one
/// of the two recorded store sites, so the local's slot was never pre-declared boxed at its first
/// store. The surviving site kept the NAME in `mixed_storage_local_names()`, which is why this one
/// did not panic like the fixture above. One matched key naming two nodes is the hazard R5 rejects
/// whether or not this particular program survived it.
#[test]
fn test_partially_stripped_mixed_storage_decisions_are_a_compile_error() {
    let error = compile_files_error_message(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\n\n$a = 5;\necho $a, \"|\";\n"),
            (
                "lib.php",
                "<?php\nfunction f($n) {\n    $q = 2;\n$a = 123456789;\n    for ($i = 1; $i < $n; $i++) {\n$a = \"s\";\n    }\n    var_dump($a);\n    return $q;\n}\nf($argc);\n",
            ),
        ],
        "main.php",
    )
    .expect("a collision that strips one of two mixed-storage decisions must not compile");
    assert!(
        error.contains("Cannot re-bind $a here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
    assert!(
        error.contains("line 4 column 1"),
        "the diagnostic must name the shared position, got: {error}"
    );
}

/// The KILL and RETYPE maps keep their existing behaviour: a stripped decision there degrades to
/// the pre-feature lowering path, which is correct, so it is not promoted to an error.
///
/// Here `lib.php` line 3 column 1 is a real retype of `$q` and `main.php` line 3 column 1 is an
/// ordinary compatible assignment to `$q`; the top-level walk re-decides the shared key and drops
/// the library's retype. Lowering then falls back to widening the old slot instead of minting a
/// fresh one — exactly what the program did before retype sites were lowered at all — and prints
/// PHP's answer. Rejecting it for uniformity would turn a correct program into a compile error,
/// which is why the retired-key check covers the mixed-storage map alone: losing a MIXED decision
/// changes what the CHECKER typed (the name leaves `mixed_storage_local_names()` while the local
/// stays `Mixed`), and that disagreement is what panics the compiler.
#[test]
fn test_stripped_retype_decision_still_compiles_and_runs() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\n$q = 2;\necho \"|\", $q;\n"),
            ("lib.php", "<?php\n$q = \"a\" . $argc;\n$q = 5;\necho $q;\n"),
        ],
        "main.php",
    );
    assert_eq!(out, "5|2");
}

/// The counter has to walk TRAIT bodies: a trait's methods are checked and lowered exactly like a
/// class's, so a decision recorded inside one is consulted inside one.
///
/// Measured before the declaration arms were made exhaustive: the trait body was not counted, the
/// collision went undetected, and the program compiled with one warning and printed `|5` instead
/// of `a1|5`. Replacing `trait T` with `class C` in the identical pair DID error, which is what
/// isolated the cause to the uncounted region.
#[test]
fn test_collision_inside_a_trait_body_is_a_compile_error() {
    let error = compile_files_error_message(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$q = \"a\" . $argc;\n$q = 5;\necho \"|\", $q;\n",
            ),
            (
                "lib.php",
                "<?php\ntrait T {\npublic function go(int $n): string {\n$q = \"b\" . $n;\nreturn $q;\n}\n}\n",
            ),
        ],
        "main.php",
    )
    .expect("a collision inside a trait body must not compile");
    assert!(
        error.contains("Cannot re-bind $q here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
}

/// The same for ENUM method bodies, which the declaration arm also used to skip.
#[test]
fn test_collision_inside_an_enum_method_is_a_compile_error() {
    let error = compile_files_error_message(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$q = \"a\" . $argc;\n$q = 5;\necho \"|\", $q;\n",
            ),
            (
                "lib.php",
                "<?php\nenum E: int {\npublic function go(int $n): string {\n$q = \"b\" . $n;\nreturn $q;\n}\ncase A = 1;\n}\n",
            ),
        ],
        "main.php",
    )
    .expect("a collision inside an enum method must not compile");
    assert!(
        error.contains("Cannot re-bind $q here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
}

/// Including one file TWICE with `require` splices its statements in twice, so a retype at its top
/// level genuinely has two sites and is rejected.
///
/// The rejection is conservative rather than necessary — PHP runs this, printing `|5|5` — but
/// allowing it would mean firing a decision at splices the checker never approved (only the LAST
/// splice's binding state produced it). What the diagnostic must not do is blame "two files"
/// alone: it names duplicate inclusion as a cause too. It deliberately stops there and does NOT
/// offer `require_once` as the fix, because `_once` does not rescue this program — see
/// `test_single_require_of_a_retyping_file_still_compiles` for why.
#[test]
fn test_double_require_of_a_retyping_file_reports_duplicate_inclusion() {
    let error = compile_files_error_message(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\nrequire 'lib.php';\n"),
            (
                "lib.php",
                "<?php\n$q = \"a\" . $argc;\n$q = 5;\necho \"|\", $q;\n",
            ),
        ],
        "main.php",
    )
    .expect("a file spliced twice gives its retype two sites, which must not compile");
    assert!(
        error.contains("Cannot re-bind $q here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
    assert!(
        error.contains("included more than once"),
        "the diagnostic must name duplicate inclusion as a cause, got: {error}"
    );
}

/// Control: including the SAME file once compiles and runs, so the rejection above is about the
/// double splice and not about the file's contents.
///
/// `require_once` is deliberately not the control. It splices once, but wraps the body in an
/// include-once GUARD, which puts the retype at conditional depth and makes it the pre-existing
/// hard `cannot reassign` error instead — so `_once` does not rescue this program, and the
/// diagnostic does not claim it does.
#[test]
fn test_single_require_of_a_retyping_file_still_compiles() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\n"),
            (
                "lib.php",
                "<?php\n$q = \"a\" . $argc;\n$q = 5;\necho \"|\", $q;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "|5");
}

/// Control: the SAME two files with the library's assignment moved one column right compile and
/// run correctly, so the test above pins the ambiguity and not the two-file shape itself.
#[test]
fn test_same_name_different_column_across_files_still_compiles() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho \"|\";\n$q = 5;\necho $q;\n",
            ),
            (
                "lib.php",
                "<?php\n$q = \"a\" . $argc;\nif ($argc > 5) {\n    $q = \"b\" . $argc;\n}\necho $q;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "a1|5");
}

/// Control: the same position but a DIFFERENT local name is unambiguous and compiles.
#[test]
fn test_different_name_same_position_across_files_still_compiles() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho \"|\";\n$q = 5;\necho $q;\n",
            ),
            (
                "lib.php",
                "<?php\n$w = \"a\" . $argc;\nif ($argc > 5) {\n$w = \"b\" . $argc;\n}\necho $w;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "a1|5");
}

/// `unset` must not end a TOP-LEVEL binding whose storage another body reaches with `global`.
///
/// PHP 8.4 prints `5`: the `unset` drops `$GLOBALS['a']`, `w()`'s `global $a` recreates it and
/// stores 5, and the top-level `echo` reads it back. `Checker::active_globals` is per-body and
/// EMPTY at top level, so the kill was accepted and the `echo` became `Undefined variable: $a` —
/// a program that compiled before this feature existed. The checker now consults the same
/// program-wide `global` set lowering already used to refuse abandoning the slot.
#[test]
fn test_unset_of_a_program_wide_global_name_keeps_the_binding() {
    let out = compile_and_run(
        "<?php function w() { global $a; $a = 5; } $a = 1; unset($a); w(); echo $a;",
    );
    assert_eq!(out, "5");
}

/// Control: the RETYPE of the same program-global name keeps working, printing PHP's `5`. Lowering
/// refuses to abandon the slot, so the site falls back to the widening path it used before retypes
/// were lowered — which is why the veto covers the kill alone.
#[test]
fn test_retype_of_a_program_wide_global_name_still_runs() {
    let out = compile_and_run(
        "<?php function w() { global $a; $a = 5; } $a = \"x\"; $a = 2; w(); echo $a;",
    );
    assert_eq!(out, "5");
}

/// A marked local assigned inside a branch a type guard narrowed still lowers through its one boxed
/// slot, and prints what PHP prints.
///
/// The mark is what makes the local's frame storage boxed `Mixed`; flow narrowing used to pull the
/// name back out of `Mixed` in the guarded branch, and the assignment there became a hard
/// `cannot reassign $a from int to string` in BOTH modes. With the mark authoritative on every
/// assignment path, the guarded store is boxed like every other store of the name.
/// PHP 8.4 prints `s` (argc is 1, so the else arm binds `"s"` and `is_int` is false).
#[test]
fn test_marked_local_assigned_inside_a_type_guarded_branch() {
    let out = compile_and_run(
        "<?php if ($argc > 1) { $a = 1; } else { $a = \"s\"; } if (is_int($a)) { $a = \"z\"; } echo $a;",
    );
    assert_eq!(out, "s");
}

/// READING a marked local inside a type-guarded branch answers correctly on both arms.
///
/// The checker narrows the marked name to `Str`/`Int` inside the guard while its slot is boxed
/// `Mixed`. Nothing misloads, and the reason is structural: `load_local` types every read from the
/// LOWERING's own `local_types` — which every marked store set to `Mixed` — and no lowering path
/// narrows on a type guard, so the guard's narrowing becomes an unbox APPLIED TO THE LOADED VALUE
/// rather than a differently-shaped load. This fixture pins the answers on both sides of the guard
/// and through a checked builtin that would reject a mis-typed operand.
#[test]
fn test_narrowed_reads_of_a_marked_local_answer_on_both_arms() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function probe(int $n): void {
    if ($n > 1) { $a = "hi" . $n; } else { $a = 42; }
    if (is_string($a)) { echo "s:", strlen($a), ":", strtoupper($a), "|"; }
    if (is_int($a)) { echo "i:", $a + 1, "|"; }
    var_dump($a);
}
probe($argc);
probe($argc + 1);"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "i:43|int(42)\ns:3:HI2|string(3) \"hi2\"\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same fixture must not leak: the guarded store goes through the boxed slot the mark declared,
/// not through a fresh one.
#[test]
fn test_marked_local_assigned_inside_a_type_guarded_branch_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php if ($argc > 1) { $a = 1; } else { $a = \"s\" . $argc; } if (is_int($a)) { $a = \"z\"; } echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "s1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The three by-VALUE capture shapes the replay used to get wrong, each printing what PHP prints.
///
/// A pre-bound name is replayed from the type it ARRIVES with and never through the depth-0 retype
/// arm, because a capture has no binding depth this body recorded and `local_binding_is_killable`
/// refuses it. `$m = 1; $m = "s";` (two depth-0 stores) and `$m = null;` first (an unseeded replay
/// starts at `Void`, which absorbs the later `string`) both went unmarked and hard-errored in
/// PERMISSIVE mode. The guarded one is the opposite error: a region over a pre-bound name's FIRST
/// in-body store IS transparent — the guard has a binding to narrow — so the seeded replay clears
/// it and no WARNING is emitted. It is still MARKED: the unseeded replay finds its own conflict and
/// the capture slot gets its boxed storage (the closure body carries the marked pattern, a
/// `Heap(Mixed)` load and release before each store). Silent, not unmarked — the two are different
/// answers, and only the diagnostic is withheld.
#[test]
fn test_capture_replay_shapes_run_php_identically() {
    assert_eq!(
        compile_and_run(
            "<?php\n$m = 1;\n$f = function (int $n) use ($m) { $m = 1; $m = \"s\"; return $m; };\nvar_dump($f(2));"
        ),
        "string(1) \"s\"\n"
    );
    assert_eq!(
        compile_and_run(
            "<?php\n$m = 1;\n$f = function (int $n) use ($m) { $m = null; if ($n > 1) { $m = \"s\"; } return $m; };\nvar_dump($f(2));"
        ),
        "string(1) \"s\"\n"
    );
    assert_eq!(
        compile_and_run(
            "<?php\n$m = 1;\n$f = function (int $n) use ($m) { if (is_string($m)) { $m = \"x\"; } if ($n > 1) { $m = 2; } return $m; };\nvar_dump($f(2));"
        ),
        "int(2)\n"
    );
}

/// The seeded replay's marks box real storage, so the capture slot still releases cleanly.
///
/// The capture arrives holding an owned heap string and the body stores `null` and then an `int`
/// through it — a conflict only the SEEDED replay sees, since an unseeded one starts at `Void` and
/// absorbs both. PHP 8.4 prints `int(7)`.
#[test]
fn test_a_seeded_capture_mark_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php\n$m = \"k\" . $argc;\n$f = function (int $n) use ($m) { $m = null; if ($n > 1) { $m = 7; } return $m; };\nvar_dump($f(2));",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(7)\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A capture whose INCOMING type rejects the body's stores is marked OUT LOUD, and the boxed slot
/// it gets still releases cleanly.
///
/// The enclosing `$m` is `int` here, so `--strict-locals` really does reject the closure body and
/// the warning's advice is true. What must not change is the storage: the mark and its store sites
/// are recorded whether or not the warning is emitted, so the capture slot is boxed and its
/// previous occupant released exactly as in the silent case. PHP 8.4 prints `string(1) "s"`.
#[test]
fn test_a_warned_capture_mark_still_boxes_and_releases() {
    let out = compile_and_run_with_heap_debug(
        "<?php\n$m = \"k\" . $argc;\n$f = function (int $n) use ($m) { $m = \"j\" . $n; if ($n > 1) { $m = 7; } return $m; };\nvar_dump($f(2));",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(7)\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The three guarded-region shapes the marking used to miss, each printing PHP 8.4's `1`.
///
/// All three hard-errored in PERMISSIVE mode before the guard model followed the checker: the
/// innermost guard governs its branch (narrowings compose), and a guarded region is only invisible
/// to the marking when replaying ALL of its assignments from the guard target merges every time
/// (the checker carries an in-branch assignment forward to the next statement of the same branch,
/// including across the arms of a nested non-guard `if`).
#[test]
fn test_guarded_region_shapes_run_php_identically() {
    assert_eq!(
        compile_and_run(
            "<?php $a = 1; if (is_string($a)) { if (is_float($a)) { $a = \"x\"; } } echo $a;"
        ),
        "1"
    );
    assert_eq!(
        compile_and_run("<?php $a = 1; if (is_string($a)) { $a = \"x\"; $a = 2; } echo $a;"),
        "1"
    );
    assert_eq!(
        compile_and_run(
            "<?php $a = 1; if (is_string($a)) { if ($argc > 1) { $a = \"x\"; } else { $a = 2; } } echo $a;"
        ),
        "1"
    );
    assert_eq!(
        compile_and_run(
            "<?php $a = 1; if (is_string($a)) { $a = \"p\"; if (is_float($a)) { $a = \"x\"; } $a = \"q\"; } echo $a;"
        ),
        "1"
    );
}

/// Two DIFFERENT marked names at one shared position keep BOTH of their mixed-storage decisions.
///
/// `main.php` marks `$a` with store sites at lines 5 and 7, column 1; `lib.php`'s function marks
/// `$b` at the very same positions. The decision map used to be keyed by `Span` ALONE, so the
/// second body's inserts silently EVICTED the first body's — with no retirement, because the
/// retire loop only matches a recorded decision that carries the same NAME. The evicted name then
/// left `mixed_storage_local_names()` while the checker still typed the local `Mixed`, and
/// lowering never boxed its slot: the compiler PANICKED with
/// `strlen cannot lower checked operand type Int` on valid PHP (an unconditional `panic!`, so
/// release builds died too). Keying the map by `(span, name)` lets both decisions coexist.
#[test]
fn test_two_marked_names_at_one_shared_position_both_compile() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n\nif ($argc > 1) {\n$a = 42;\n} else {\n$a = \"hello\";\n}\necho strlen($a), \"|\";\n",
            ),
            (
                "lib.php",
                "<?php\nfunction f($n) {\n\nif ($n > 1) {\n$b = 42;\n} else {\n$b = \"world\";\n}\necho strlen($b), \"|\";\n}\nf($argc);\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "5|5|");
}

/// A retype inside an array-representation fixed point region: the statement that retypes is
/// also the statement that could convert an array local's storage.
///
/// The fixed-point pass discovers conversions by lowering the statement speculatively and then
/// hoisting them to the region ENTRY. A conversion recorded for a name the region RE-BINDS
/// belongs to the fresh slot, not the entry one, so hoisting it would convert the old binding's
/// storage — which at region entry is a plain `int`.
#[test]
fn test_retype_inside_a_conversion_hiding_region() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, $argc];
$b = $argc;
$b = $argc > 0 ? "yes" : "no";
$a[0] = "s";
echo $b, "|", $a[0], "|", $a[1];"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "yes|s|1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

// ---------------------------------------------------------------------------
// Branch-divergent locals compiled as boxed `mixed` frame storage.
//
// The checker's pre-scan (`types::checker::mixed_storage_scan`) marks the name and records
// every one of its assignment spans; lowering declares the slot `Mixed` before the first of
// them and stores every recorded assignment AT `Mixed`, so both dynamic outcomes live in one
// boxed slot and every read of the name is a boxed read. Every fixture below is verified to
// actually GO THROUGH that path by a sibling assertion in `error_tests::type_system` that the
// program warns "compiled as boxed mixed storage" — a fixture that silently fell back to the
// error path would otherwise look like a pass here.
// ---------------------------------------------------------------------------

/// Branch-divergent local: the else arm runs (argc == 1) and prints the string.
#[test]
fn test_branch_divergent_local_runs_else_arm() {
    let out = compile_and_run("<?php if ($argc > 1) { $a = 0; } else { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "ciao");
}

/// Single-branch retype, branch taken: the Mixed slot holds the string.
#[test]
fn test_single_branch_retype_taken() {
    let out = compile_and_run("<?php $a = 41; if ($argc > 0) { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "ciao");
}

/// Single-branch retype, branch NOT taken: the Mixed slot still holds the boxed int.
/// This is the load-bearing test — both dynamic outcomes flow through one slot.
#[test]
fn test_single_branch_retype_not_taken() {
    let out = compile_and_run("<?php $a = 41; if ($argc > 5) { $a = \"ciao\"; } echo $a;");
    assert_eq!(out, "41");
}

/// Heterogeneous loop-carried local: each iteration re-boxes; previous value released.
///
/// The RHS is a CONCATENATION rather than a literal on purpose: it allocates a fresh heap
/// string per iteration, which is the release-pressure shape the boxed store path has to get
/// right (see the heap-debug fixture below).
#[test]
fn test_loop_carried_heterogeneous_local() {
    let out = compile_and_run("<?php $a = 0; for ($i = 0; $i < $argc; $i++) { $a = \"s\" . $i; } echo $a;");
    assert_eq!(out, "s0");
}

/// A marked local reaching a CHECKED builtin, whose lowering demands one concrete operand
/// representation.
///
/// Measured before the hook: the compiler PANICKED outright — "checked builtin strlen failed
/// backend-neutral EIR lowering at 1:63: strlen cannot lower checked operand type Int". DCE
/// tail-sinking copies the `echo` into BOTH arms, and the copy in the `int` arm was lowered
/// while the local's logical type was still `Int`, which `strlen`'s checked fast path has no
/// lowering for. Storing every recorded assignment at `Mixed` — not just declaring the slot
/// that way — is what hands the builtin an operand it can lower on either path.
#[test]
fn test_branch_divergent_local_reaches_a_checked_builtin() {
    let out = compile_and_run(
        "<?php if ($argc > 1) { $a = 42; } else { $a = \"hello\"; } echo strlen($a);",
    );
    assert_eq!(out, "5");
}

/// The same shape through more builtin/inspection surfaces, so the boxed operand is not a
/// one-builtin accident.
#[test]
fn test_branch_divergent_local_through_several_builtins() {
    let out = compile_and_run(
        r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
echo strlen($a), "|", strtoupper($a), "|", gettype($a), "|";
var_dump(is_string($a));"#,
    );
    assert_eq!(out, "5|HELLO|string|bool(true)\n");
}

/// A loop whose body never runs must leave the entry binding's VALUE AND TYPE intact.
///
/// Measured before the pre-declare hook: `var_dump` printed `string(9) "123456789"` — the
/// entry store wrote a raw `int` into an `Int` slot that the loop body's string store later
/// widened, so the surviving int payload was read back through the widened string view. With
/// the slot declared `Mixed` up front the int is boxed at the entry store and reads back as
/// one.
#[test]
fn test_zero_trip_loop_keeps_the_entry_binding_type() {
    let out = compile_and_run(
        "<?php $a = 123456789; for ($i = 1; $i < $argc; $i++) { $a = \"s\"; } var_dump($a);",
    );
    assert_eq!(out, "int(123456789)\n");
}

/// A marked TOP-LEVEL local that another body writes through `global $a`.
///
/// The per-body pre-scan cannot see `q()`'s `global $a` while scanning the top level, so the
/// name is marked there. The invariant that makes it sound is NOT that lowering sees every
/// `global` — it does not, see `test_marked_and_unmarked_agree_on_a_closure_declared_global`
/// below — it is that MARKING CHANGES NOTHING for a `global`-aliased name. Where
/// `collect_global_var_names` does see the declaration (a NAMED function's body, as here),
/// `LoweringContext::uses_global_storage` puts the top-level name in program-global storage that
/// `global_alias_type` already types `Mixed`, so `store_local` overrides the marked `Mixed` with
/// the identical type and writes the shared symbol; where it does not, the marked and unmarked
/// programs are wrong in exactly the same pre-existing way. Either way the marked program behaves
/// like the unmarked one.
#[test]
fn test_marked_top_level_local_written_through_a_global_alias() {
    let out = compile_and_run(
        r#"<?php
function q() { global $a; $a = 42; }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
q();
echo $a, "|";
var_dump($a);"#,
    );
    assert_eq!(out, "hello|42|int(42)\n");
}

/// The same cross-body write from a METHOD, which `collect_global_var_names` also walks.
#[test]
fn test_marked_top_level_local_written_through_a_method_global_alias() {
    let out = compile_and_run(
        r#"<?php
class W { public function w() { global $a; $a = 42; } }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
(new W())->w();
echo $a, "|";
var_dump($a);"#,
    );
    assert_eq!(out, "hello|42|int(42)\n");
}

/// The other direction: MAIN writes the marked `global`-aliased name and the callee READS it,
/// both before and after a re-store that changes the runtime type.
///
/// The callee's `global $a` reads the shared symbol, so it has to observe the boxed value main
/// put there — the string on the first call and the int on the second. Reading through a
/// concrete view on either side would print the other type's payload.
///
/// Heap: this shape leaks 48 bytes, and that has nothing to do with the marking — the boxed
/// occupant of `_eir_global_a` is not released at program exit. An unmarked program leaks the same
/// block (`test_marked_and_unmarked_global_alias_reads_leak_alike`), and so does this one built
/// from the parent commit. Output correctness is what this fixture pins.
#[test]
fn test_marked_top_level_local_read_back_through_a_global_alias() {
    let out = compile_and_run(
        r#"<?php
function q() { global $a; var_dump($a); }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
q();
$a = 42;
q();"#,
    );
    assert_eq!(out, "string(5) \"hello\"\nint(42)\n");
}

/// Control for the leak note above: a `global`-aliased read with NO marking anywhere leaks the
/// same way, so the leak belongs to program-global storage rather than to boxed locals.
///
/// Asserted as an EQUIVALENCE rather than a byte count. Both programs put one boxed string in
/// `_eir_global_a` and neither releases it at exit (measured: `live_blocks=1 live_bytes=48` for
/// each, on this commit and on the parent alike). Whoever fixes that fixes both, and this test
/// keeps passing; what it refuses is one of them leaking while the other does not.
#[test]
fn test_marked_and_unmarked_global_alias_reads_leak_alike() {
    let marked = compile_and_run_with_heap_debug(
        r#"<?php
function q() { global $a; var_dump($a); }
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
q();"#,
    );
    let unmarked = compile_and_run_with_heap_debug(
        r#"<?php
function q() { global $a; var_dump($a); }
$a = "hello" . $argc;
q();"#,
    );
    assert!(marked.success, "marked program failed: {}", marked.stderr);
    assert!(unmarked.success, "unmarked program failed: {}", unmarked.stderr);
    assert_eq!(marked.stdout, "string(5) \"hello\"\n");
    assert_eq!(unmarked.stdout, "string(6) \"hello1\"\n");

    /// Extracts the `leak summary: …` line so the two runs can be compared directly.
    fn leak_summary(stderr: &str) -> &str {
        stderr
            .lines()
            .find(|line| line.contains("HEAP DEBUG: leak summary:"))
            .unwrap_or("<no leak summary>")
    }
    assert_eq!(
        leak_summary(&marked.stderr),
        leak_summary(&unmarked.stderr),
        "marking must not change what a global-aliased local leaks"
    );
}

/// A `global $a` written inside a CLOSURE literal does NOT reach main's binding, and marking
/// changes nothing about that.
///
/// The walk EIR lowering consumes covers statement bodies only, so this declaration is invisible
/// to it: main keeps `$a` in a frame slot instead of the shared symbol and the closure's write
/// never reaches it — both programs print `hello|hello|` where PHP prints `hello|42|`. That is the
/// PRE-EXISTING global-in-closure write loss (tracked upstream), deliberately preserved: the one
/// change that closed it moved such names into `_eir_global_*` program storage, which types them
/// `Mixed` and drove the array builtins into their pre-existing `Mixed`-array backend gaps
/// (`test_closure_declared_global_leaves_a_top_level_array_alone` pins the repro). The checker's
/// `unset`-kill veto reads a WIDER scope that does see this declaration, which is safe because it
/// only ever withholds a kill.
///
/// What this fixture asserts either way is the EQUIVALENCE: whatever the hole does, marking must
/// not change it. If the hole is ever closed, both sides move together or this test says so.
#[test]
fn test_marked_and_unmarked_agree_on_a_closure_declared_global() {
    let marked = compile_and_run(
        r#"<?php
$w = function () { global $a; $a = 42; };
if ($argc > 1) { $a = 0; } else { $a = "hello"; }
echo $a, "|";
$w();
echo $a, "|";"#,
    );
    let unmarked = compile_and_run(
        r#"<?php
$w = function () { global $a; $a = 42; };
$a = "hello";
echo $a, "|";
$w();
echo $a, "|";"#,
    );
    assert_eq!(
        marked, unmarked,
        "marking must not change what a closure-declared `global` does to a top-level local"
    );
    assert_eq!(marked, "hello|hello|");
}

/// A loop-carried marked local whose every iteration allocates a fresh heap string: the
/// previous boxed occupant must be released before the slot is overwritten, and the last one
/// at frame teardown.
#[test]
fn test_loop_carried_heterogeneous_local_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = 0; for ($i = 0; $i < $argc + 3; $i++) { $a = \"s\" . $i; } echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "s3");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The branch-divergent shape holds a heap string on one path and a raw int on the other, so
/// the frame teardown has to release exactly one of them.
#[test]
fn test_branch_divergent_local_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        "<?php if ($argc > 1) { $a = 42; } else { $a = \"hello\" . $argc; } echo $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "hello1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A marked local captured BY VALUE by a closure that overwrites it, with the enclosing frame
/// reading the same name through a SECOND capture afterwards.
///
/// This is the one shape where a marked name is already bound when its first recorded store is
/// lowered: the capture arrives as a parameter, so the pre-declare finds a slot and only the
/// `Mixed` store type applies. Both frames' `$m` are marked here (the outer `if`/`else` and the
/// closure's), and the closure's store releases the previous occupant of the capture slot, so the
/// enclosing box has to survive for the second capture to read.
///
/// Measured on the parent commit (built from `cacee00c3` in a side worktree): the same program
/// ran to completion but leaked two blocks, 96 bytes.
#[test]
fn test_marked_local_captured_by_value_and_overwritten_in_a_closure() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
if ($argc > 1) { $m = 1; } else { $m = "z"; }
$f = function (int $n) use ($m) {
    if ($n > 1) { $m = 0; } else { $m = "s"; }
    return $m;
};
var_dump($f($argc));
$g = function () use ($m) { return $m; };
var_dump($g());"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(1) \"s\"\nstring(1) \"z\"\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A marked local re-assigned to a LITERAL after the divergent branch, then read by a checked
/// builtin.
///
/// The literal makes the read a constant-propagation target. `PostTypecheckOptimizer::propagate`
/// runs after the checker and knows nothing about the type it bound, so it substituted `99` for
/// the read of `$a` — handing lowering a concrete `Int` where the checker said `mixed`, and
/// `strlen`'s checked fast path PANICKED the compiler ("strlen cannot lower checked operand type
/// Int"). Measured identically on the parent commit: this shape was already broken by Task 6's
/// marking. The pass is now told which names the checker boxed and records no fact for them.
#[test]
fn test_marked_local_reassigned_to_a_literal_reaches_a_checked_builtin() {
    let out = compile_and_run(
        r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
$a = 99;
echo strlen($a);"#,
    );
    assert_eq!(out, "2");
}

/// The same shape through the builtins whose lowering failed in OTHER ways: `strtoupper` failed
/// EIR validation with an `OperandTypeMismatch { expected: "Str", actual: I64 }` rather than
/// panicking, and `str_repeat` failed in the backend.
#[test]
fn test_marked_local_reassigned_to_a_literal_through_several_builtins() {
    let out = compile_and_run(
        r#"<?php
if ($argc > 1) { $a = 42; } else { $a = "hello"; }
$a = 99;
echo str_repeat($a, 2), "|", strlen($a), "|", strtoupper($a), "|", gettype($a), "|";
var_dump($a);"#,
    );
    assert_eq!(out, "9999|2|99|integer|int(99)\n");
}

/// The same shape inside a FUNCTION body, and read by the non-builtin consumers a propagated
/// literal would also have narrowed: a `switch` subject, a comparison and an arithmetic operand.
#[test]
fn test_marked_local_reassigned_to_a_literal_inside_a_function_body() {
    let out = compile_and_run(
        r#"<?php
function f(int $n) {
    if ($n > 1) { $a = 42; } else { $a = "hello"; }
    $a = 7;
    return strlen($a) . "|" . strtoupper($a) . "|" . gettype($a);
}
echo f($argc), "\n";
if ($argc > 1) { $c = 1; } else { $c = "x"; }
$c = 3;
switch ($c) { case 3: echo "three|"; break; default: echo "other|"; }
echo ($c == 3 ? "eq" : "ne"), "|", $c + 1, "|";
var_dump($c);"#,
    );
    assert_eq!(out, "1|7|integer\nthree|eq|4|int(3)\n");
}

/// The concat-widened form of the same shape.
///
/// This one is on THIS change's account rather than Task 6's: before string concatenation became
/// exact marking evidence the program was a clean `cannot reassign $a from int to string` error,
/// so widening the whitelist is what admitted it into the crash family in the first place.
#[test]
fn test_concat_marked_local_reassigned_to_a_literal_reaches_a_checked_builtin() {
    let out = compile_and_run(
        r#"<?php
$a = 0;
if ($argc > 1) { $a = "s" . $argc; }
$a = 5;
echo strlen($a), "|", strtoupper($a), "|";
var_dump($a);"#,
    );
    assert_eq!(out, "1|5|int(5)\n");
}

/// Only the CLOSURE's `$m` is marked here; the enclosing one is `mixed` by ordinary inference.
/// Measured on the parent commit: one leaked block, 48 bytes.
#[test]
fn test_marked_local_in_a_closure_over_an_unmarked_mixed_capture() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$m = $argc > 1 ? 1 : "z";
$f = function (int $n) use ($m) { if ($n > 1) { $m = 0; } else { $m = "s"; } return $m; };
var_dump($f($argc));
$g = function () use ($m) { return $m; };
var_dump($g());"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(1) \"s\"\nstring(1) \"z\"\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// eval'd code retypes locals dynamically; permissive AOT now matches it.
///
/// Observed BEFORE this plan's permissive retyping shipped for ordinary locals: this exact
/// program already compiled clean (no warning) and printed `ciao`. A literal `eval(...)`
/// fragment is lowered by `src/eval_aot.rs` into its own EIR scope function that reads/writes
/// `$a` through the dynamic `__elephc_eval_scope_set`/`get` scope map rather than through a
/// normal typed local slot, so it was never subject to the old strict "cannot reassign" local
/// check in the first place — the eval scope was always dynamically typed. `--strict-locals`
/// does not gate it either (verified separately; the flag only tightens the AOT local-slot
/// checker, which this construct never goes through). This test pins that parity as a
/// regression: it is not a case Task 1-7's mechanism newly admits, but the two paths (ordinary
/// permissive AOT locals and eval's scope map) now agree on the observable result.
#[test]
fn test_eval_local_retype_matches_aot() {
    let out = compile_and_run("<?php eval('$a = 1; $a = \"ciao\"; echo $a;');");
    assert_eq!(out, "ciao");
}

/// An `unset` in a body that calls `eval()` keeps the binding, so the fragment's write lands
/// where the later read looks.
///
/// The eval scope addresses caller locals BY NAME; the kill drops the name's frame slot and mints
/// a fresh one at the next store. Measured with the kill still firing here, this program printed
/// NOTHING (PHP prints `5`), and it printed nothing SILENTLY — no warning, no error. The gate
/// that fixes it is body-scoped rather than point-in-time, because the `eval` below the `unset`
/// has not raised any barrier yet at the moment the kill is judged.
#[test]
fn test_unset_in_an_eval_body_keeps_the_binding() {
    let out = compile_and_run("<?php $a = 1; unset($a); eval('$a = 5;'); echo $a;");
    assert_eq!(out, "5");
}

/// The eval fragment still sees a local the body assigned before it, with the kill gated out of
/// the way: `unset` degrades to the plain null-store it did before kills were lowered.
#[test]
fn test_unset_in_an_eval_body_still_nulls_the_local() {
    let out = compile_and_run(
        "<?php $a = \"s\" . $argc; unset($a); eval('echo isset($a) ? \"set\" : \"unset\";');",
    );
    assert_eq!(out, "unset");
}

/// Control: the branch-divergent shape is NOT gated by the eval rule and keeps working, because
/// boxed `Mixed` storage is what the eval scope wants in the first place.
#[test]
fn test_branch_divergent_local_survives_an_eval_body() {
    let out = compile_and_run(
        "<?php if ($argc > 1) { $b = 1; } else { $b = \"z\"; } eval('echo $b; $b = \"w\";'); echo \"|\", $b;",
    );
    assert_eq!(out, "z|w");
}

// ---------------------------------------------------------------------------
// Eval-AOT fragments are OUTSIDE the decision maps.
// ---------------------------------------------------------------------------

/// An eval-AOT fragment must never consult the OUTER program's binding decisions.
///
/// A fragment is parsed from a string literal, so its spans live in an independent span space
/// that starts at line 1 of that string and that nothing in the ambiguity tally can see. A match
/// against a decision the checker recorded for the main file is therefore always an ACCIDENT of
/// two unrelated nodes landing on the same line and column — and it is observable: the main file's
/// marked `$b` records a store site at 2:1, the fragment's own unrelated `$b = 9;` sits at 2:1 of
/// the eval string, and the mixed pre-declare fired on the fragment's local, giving it boxed
/// storage the checker never asked for.
///
/// Both eval-AOT lowerers now receive EMPTY maps, so the fragment's `$b` keeps the unboxed `Int`
/// storage its own value implies.
#[test]
fn test_eval_aot_fragment_ignores_a_colliding_outer_decision() {
    let dir = make_cli_test_dir("elephc_eval_aot_maps");
    let php_path = dir.join("main.php");
    std::fs::write(
        &php_path,
        "<?php\n$b = \"s\";\nif ($argc > 1) { $b = 1; }\necho $b;\necho eval(\"\\n\\$b = 9;\\nreturn \\$b;\");\n",
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --emit-ir");
    assert!(
        output.status.success(),
        "elephc --emit-ir failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let fragment = eval_aot_fragment_body(&stdout);
    assert!(
        fragment.contains("const_i64 9"),
        "expected the fragment's own literal, got: {fragment}"
    );
    assert!(
        !fragment.contains("php=mixed own=maybe_owned = load_local"),
        "the fragment local was boxed by the outer program's decision: {fragment}"
    );
    assert!(
        fragment.contains("mixed_box"),
        "an unboxed fragment local must be boxed at the eval return: {fragment}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Returns the body of the single eval-AOT fragment function in an `--emit-ir` dump.
fn eval_aot_fragment_body(module_text: &str) -> String {
    let mut lines = module_text.lines().skip_while(|line| {
        !line.trim_start().starts_with("function __eir@evalaot")
    });
    let header = lines.next().expect("no eval-AOT fragment function in the IR dump");
    let mut body = String::from(header);
    for line in lines {
        body.push('\n');
        body.push_str(line);
        if line == "  }" {
            break;
        }
    }
    body
}

/// The same program still prints PHP's answer.
#[test]
fn test_eval_aot_fragment_with_a_colliding_outer_decision_runs() {
    let out = compile_and_run(
        "<?php\n$b = \"s\";\nif ($argc > 1) { $b = 1; }\necho $b;\necho eval(\"\\n\\$b = 9;\\nreturn \\$b;\");\n",
    );
    assert_eq!(out, "s9");
}

/// Two DIFFERENT locals killed at one (line, column) both keep their decision.
///
/// A `Span` carries line/column and nothing about which file they name, so `unset($w);` at line 4
/// column 1 of `lib.php` and `unset($m);` at line 4 column 1 of `main.php` file their kills under
/// the SAME key. With one name per span the later insert silently evicted the earlier one, and
/// the evicted site fell back to the pre-feature null store — which widens the `Str` slot to
/// boxed `Mixed` while every read already lowered above it keeps its `Str` IR type, leaking one
/// detached copy per executed read. Measured before the maps carried a SET of names: `Fatal
/// error: heap memory exhausted` where PHP prints `400005|s`.
#[test]
fn test_two_different_names_killed_at_one_position_both_take_effect() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$m = 1;\nunset($m);\n$m = \"s\";\necho \"|\", $m;\n",
            ),
            (
                "lib.php",
                "<?php\n$w = \"x\" . $argc; $t = 0;\nfor ($i = 0; $i < 200000; $i++) { $t += strlen($w); }\nunset($w);\n$w = 5;\necho $t + $w;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "400005|s");
}

/// The RETYPE twin of the fixture above: two different locals re-bound at one (line, column).
///
/// `$w = 5;` (lib, line 4 column 1) re-binds a `string` to `int` under a loop of reads, and
/// `$m = "s";` (main, line 4 column 1) re-binds an `int` to `string`. The same eviction left the
/// library's retype on the slot-widening path and exhausted the heap.
#[test]
fn test_two_different_names_retyped_at_one_position_both_take_effect() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\n$m = 1;\n$m = \"s\";\necho \"|\", $m;\n",
            ),
            (
                "lib.php",
                "<?php\n$w = \"x\" . $argc; $t = 0;\nfor ($i = 0; $i < 200000; $i++) { $t += strlen($w); }\n$w = 5;\necho $t + $w;\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "400005|s");
}

/// Control: the same-NAME collision keeps its current outcome — the hard ambiguity error, not a
/// silently merged pair of decisions. A set of names per span must not turn two genuinely
/// indistinguishable sites into two accepted ones.
#[test]
fn test_same_name_collision_stays_ambiguous_with_multi_name_spans() {
    let error = compile_files_error_message(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib.php';\necho \"|\";\n$q = 5;\necho $q;\n",
            ),
            (
                "lib.php",
                "<?php\n$q = \"a\" . $argc;\nif ($argc > 5) {\n$q = \"b\" . $argc;\n}\necho $q;\n",
            ),
        ],
        "main.php",
    )
    .expect("a same-name (span, name) collision must still be rejected");
    assert!(
        error.contains("Cannot re-bind $q here"),
        "expected the ambiguity diagnostic, got: {error}"
    );
}

/// A top-level `unset` must not end a binding whose storage a CLOSURE reaches with `global`.
///
/// The same rule `test_unset_of_a_program_wide_global_name_keeps_the_binding` pins for a named
/// function, where lowering ALSO sees the declaration and the program prints PHP's `5`. Here it
/// does not: the checker's veto reads the wide `collect_global_var_names_in_nested_bodies` scope,
/// lowering reads the narrow one, and the split is deliberate (widening lowering broke working
/// array programs — see `test_closure_declared_global_leaves_a_top_level_array_alone`).
///
/// What the veto delivers is therefore ACCEPTANCE, not the value: before it, the checker approved
/// the kill, `$a` left the environment, and this program was a false `Undefined variable: $a`
/// though PHP runs it. It now compiles and behaves exactly as it did before local-binding kills
/// existed — the `unset` is a plain typing no-op that nulls the slot, and the closure's write goes
/// to program storage main never reads, so the `echo` prints nothing where PHP prints `5`. That
/// residual is the pre-existing global-in-closure write loss (tracked upstream), whose real fix is
/// blocked on the `Mixed`-array backend gaps; this fixture pins that the kill no longer REJECTS
/// the program, and that the write loss is unchanged rather than newly introduced.
#[test]
fn test_unset_of_a_name_a_closure_declares_global_keeps_the_binding() {
    let out = compile_and_run(
        "<?php $a = 1; unset($a); $f = function() { global $a; $a = 5; }; $f(); echo $a;",
    );
    assert_eq!(out, "");
}

/// The same for an ENUM method body, the other declaration the veto's wide scope walks and
/// lowering's narrow scope does not. Compiles instead of being falsely rejected; the value is lost
/// to the same pre-existing write loss.
#[test]
fn test_unset_of_a_name_an_enum_method_declares_global_keeps_the_binding() {
    let out = compile_and_run(
        r#"<?php
enum E: int {
    case A = 1;
    public function go(): int { global $a; $a = 5; return 1; }
}
$a = 1;
unset($a);
E::A->go();
echo $a;"#,
    );
    assert_eq!(out, "");
}

/// The two files a `require_once` fixture shares: a top-level pair of incompatible assignments
/// AND a function whose body does the same thing.
const REQUIRE_ONCE_LIB: &str = "<?php\n$q = \"a\" . $argc;\n$q = 5;\necho \"|\", $q;\nfunction g(int $n): string {\n    $r = \"b\" . $n;\n    $r = 7;\n    return \"|\" . $r;\n}\n";

/// `require_once` puts every TOP-LEVEL statement of the included file at conditional depth ≥ 1,
/// while a FUNCTION declared in the same file starts its own body at depth 0.
///
/// The include-once guard lowers to a runtime branch, so the top-level `$q = "a" . $argc; $q = 5;`
/// pair is not eligible for the straight-line retype (shape 2) — the pre-scan's branch-divergent
/// marking (shape 3, which is depth-gated by nothing) picks it up instead and gives `$q` boxed
/// `Mixed` storage. `g()`'s body is a separate frame whose depth counter
/// `enter_local_binding_scope` resets, so `$r` takes the ordinary retype with its warning. Both
/// print PHP's answer.
#[test]
fn test_require_once_scopes_the_depth_rule_to_top_level_statements() {
    let warnings = check_files_diagnostics(
        &[
            ("main.php", "<?php\nrequire_once 'lib.php';\necho g($argc);\n"),
            ("lib.php", REQUIRE_ONCE_LIB),
        ],
        "main.php",
        false,
    )
    .expect("the require_once fixture must type-check");
    assert_eq!(
        warnings,
        vec![
            "$q is assigned incompatible types (string and int); it is compiled as boxed mixed storage (compile with --strict-locals to make this an error)".to_string(),
            "$r changes type from string to int; the previous value is discarded (compile with --strict-locals to make this an error)".to_string(),
        ],
        "the guarded top level must take the MIXED shape and the function body the RETYPE shape"
    );

    let out = compile_and_run_files(
        &[
            ("main.php", "<?php\nrequire_once 'lib.php';\necho g($argc);\n"),
            ("lib.php", REQUIRE_ONCE_LIB),
        ],
        "main.php",
    );
    assert_eq!(out, "|5|7");
}

/// The plain-`require` contrast: no include guard, so the included file's top-level statements are
/// inline code at depth 0 and the SAME pair takes the straight-line retype instead of the mixed
/// marking. The function body is unaffected either way.
#[test]
fn test_plain_require_leaves_the_included_top_level_retype_eligible() {
    let warnings = check_files_diagnostics(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\necho g($argc);\n"),
            ("lib.php", REQUIRE_ONCE_LIB),
        ],
        "main.php",
        false,
    )
    .expect("the require fixture must type-check");
    assert_eq!(
        warnings,
        vec![
            "$q changes type from string to int; the previous value is discarded (compile with --strict-locals to make this an error)".to_string(),
            "$r changes type from string to int; the previous value is discarded (compile with --strict-locals to make this an error)".to_string(),
        ],
        "an unguarded include leaves its top-level statements on the retype path"
    );

    let out = compile_and_run_files(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\necho g($argc);\n"),
            ("lib.php", REQUIRE_ONCE_LIB),
        ],
        "main.php",
    );
    assert_eq!(out, "|5|7");
}

/// `--strict-locals` turns BOTH shapes back into the hard error, whichever include form was used —
/// the mixed marking and the straight-line retype are the two shapes the flag governs.
#[test]
fn test_strict_locals_rejects_both_include_forms() {
    for main in [
        "<?php\nrequire_once 'lib.php';\necho g($argc);\n",
        "<?php\nrequire 'lib.php';\necho g($argc);\n",
    ] {
        let error = check_files_diagnostics(
            &[("main.php", main), ("lib.php", REQUIRE_ONCE_LIB)],
            "main.php",
            true,
        )
        .expect_err("--strict-locals must reject the retyping include");
        assert!(
            error.contains("cannot reassign $q from string to int"),
            "expected the strict retype error, got: {error}"
        );
    }
}

/// The `unset` KILL is the shape `require_once` genuinely takes away: it is depth-gated like the
/// straight-line retype, and nothing else picks the pair up, so the program that compiles under a
/// plain `require` is a hard `cannot reassign` under `require_once`.
#[test]
fn test_require_once_makes_a_top_level_unset_kill_ineligible() {
    const KILL_LIB: &str = "<?php\n$q = 1;\nunset($q);\n$q = \"s\";\necho \"|\", $q;\n";
    let error = compile_files_error_message(
        &[
            ("main.php", "<?php\nrequire_once 'lib.php';\n"),
            ("lib.php", KILL_LIB),
        ],
        "main.php",
    )
    .expect("a guarded top-level unset must not be kill-eligible");
    assert!(
        error.contains("cannot reassign $q from int to string"),
        "expected the pre-feature error, got: {error}"
    );

    let out = compile_and_run_files(
        &[
            ("main.php", "<?php\nrequire 'lib.php';\n"),
            ("lib.php", KILL_LIB),
        ],
        "main.php",
    );
    assert_eq!(out, "|s");
}

/// A `switch` arm is a conditional branch like an `if` arm, so a local assigned incompatible
/// types across arms is MARKED (shape 3) rather than rejected.
///
/// The pre-scan's divergence rule is about branching, not about which statement introduces it —
/// this pins the `switch` shape alongside the `if`/`else` one the feature is usually shown with.
/// The mark is depth-gated by nothing, which is exactly why it reaches inside the arms.
#[test]
fn test_marked_local_assigned_across_switch_arms() {
    let out = compile_and_run(
        "<?php switch ($argc) { case 1: $a = 0; break; default: $a = \"ciao\"; } echo $a, \"|\"; var_dump($a);",
    );
    assert_eq!(out, "0|int(0)\n");
}

/// The mixed-storage WARNING the `switch` shape produces, and its `--strict-locals` rejection.
#[test]
fn test_marked_local_across_switch_arms_warns_and_is_strict_rejected() {
    let source = "<?php switch ($argc) { case 1: $a = 0; break; default: $a = \"ciao\"; } echo $a;";
    let warnings = check_files_diagnostics(&[("main.php", source)], "main.php", false)
        .expect("the switch fixture must type-check");
    assert_eq!(
        warnings,
        vec![
            "$a is assigned incompatible types (int and string); it is compiled as boxed mixed storage (compile with --strict-locals to make this an error)".to_string(),
        ],
    );
    let error = check_files_diagnostics(&[("main.php", source)], "main.php", true)
        .expect_err("--strict-locals must reject the marked switch local");
    assert!(
        error.contains("cannot reassign $a"),
        "expected the strict rejection, got: {error}"
    );
}

/// `--strict-locals` and eval-AOT compose: a literal `eval` fragment still retypes its own locals
/// and still writes the caller's, because the fragment goes through the dynamic scope map rather
/// than the AOT local-slot checker the flag tightens.
///
/// Structural, not incidental: the eval-AOT lowerers are handed EMPTY decision maps
/// (`ir_lower::function::eval_aot_decision_maps`), so no binding decision — and no strict-mode
/// refusal of one — can reach a fragment. Run through the real CLI so the flag under test is the
/// one users pass.
#[test]
fn test_eval_fragment_composes_with_strict_locals() {
    let own_local = compile_cli_file_and_run_with_flags(
        "<?php\n$a = \"s\";\necho $a, \"|\";\neval('$b = 1; $b = \"ciao\"; echo $b;');\n",
        &["--strict-locals"],
    );
    assert_eq!(own_local, "s|ciao");

    let caller_local = compile_cli_file_and_run_with_flags(
        "<?php\n$a = 1;\neval('$a = \"z\";');\necho $a;\n",
        &["--strict-locals"],
    );
    assert_eq!(caller_local, "z");
}

/// A top-level ARRAY whose name some closure declares `global` must keep working.
///
/// Widening the SHARED `collect_global_var_names` walk to closure bodies moved such a name into
/// `_eir_global_*` program storage, whose element type is `Mixed` — and the array builtins have
/// pre-existing backend gaps for `Mixed` arrays. `implode` went SILENT (this fixture printed
/// nothing where PHP prints `3,1,2`) and `array_sum`/`sort`/`in_array`/`array_map`/`array_keys`/
/// `array_reverse` became hard `unsupported EIR backend feature: … for PHP type Mixed`. The
/// lowering side therefore keeps the statement-only walk; only the CHECKER's kill veto sees the
/// widened one.
#[test]
fn test_closure_declared_global_leaves_a_top_level_array_alone() {
    let out = compile_and_run(
        "<?php $d = function () { global $a; echo \"x\"; }; $a = [3, 1, 2]; echo implode(\",\", $a);",
    );
    assert_eq!(out, "3,1,2");
}

/// The hard-error half of the same blast radius: an array builtin with no `Mixed` backend arm.
///
/// `array_sum` is the representative; `sort`, `usort`, `in_array`, `array_map`, `array_keys` and
/// `array_reverse` fail identically. The program must BUILD, not just run.
#[test]
fn test_closure_declared_global_leaves_array_builtins_lowerable() {
    let out = compile_and_run(
        "<?php $d = function () { global $a; echo \"x\"; }; $a = [3, 1, 2]; echo array_sum($a), \"|\"; sort($a); echo implode(\",\", $a), \"|\", (int) in_array(2, $a);",
    );
    assert_eq!(out, "6|1,2,3|1");
}

/// The same for the ASSIGNMENT-PRELUDE reach the widened walk added: a closure literal nested in
/// an assignment's synthesized prelude must not move a top-level array either.
#[test]
fn test_closure_in_an_assignment_expression_leaves_a_top_level_array_alone() {
    let out = compile_and_run(
        "<?php $a = [3, 1, 2]; $n = ($d = function () { global $a; echo \"x\"; }) ? 1 : 0; echo implode(\",\", $a), \"|\", $n;",
    );
    assert_eq!(out, "3,1,2|1");
}
