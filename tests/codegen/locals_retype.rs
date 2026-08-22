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
/// It also LEAKS one 48-byte block per executed read, which this fixture deliberately does not
/// assert on. Ending a binding nulls the slot through the ordinary overwrite path at
/// `PhpType::Void`, which widens the slot to `Mixed`; a slot's storage type is a whole-FRAME
/// property, so every load lowered ABOVE the kill — `strlen($a)` here — keeps its `Str` IR type
/// against boxed storage and becomes a detach nothing releases.
///
/// Nulling at the slot's OWN type removes the leak but is NOT a valid fix: the null is an `I64`
/// in the int result register, while a two-word `Str` slot is written from the STRING register
/// pair, so the store writes stale registers instead of clearing the slot and the frame epilogue
/// then frees the live pointer left behind. Measured:
/// `$q = "a" . $n; $r = $q; $q = 1; return $r . "|" . $q;` returned four NUL bytes. Widening to
/// `Mixed` is what makes the null actually land in the slot, so the leak is the price of that and
/// belongs to its own fix in the ownership model. See `release_and_abandon_local_binding`.
#[test]
fn test_string_read_above_a_kill_still_answers_correctly() {
    let out = compile_and_run("<?php $a = \"n\" . $argc; $b = strlen($a); unset($a); $a = 5; echo $b, $a;");
    assert_eq!(out, "25");
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
/// splice's binding state produced it). What the diagnostic must not do is blame "two files": it
/// names duplicate inclusion as a cause and points at `require_once`.
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
/// `global` — it does not, see `test_marked_and_unmarked_agree_through_the_collect_global_hole`
/// below — it is that MARKING CHANGES NOTHING for a `global`-aliased name. Where
/// `collect_global_var_names` does see the declaration, `LoweringContext::uses_global_storage`
/// puts the top-level name in program-global storage that `global_alias_type` already types
/// `Mixed`, so `store_local` overrides the marked `Mixed` with the identical type and writes the
/// shared symbol; where it does not see it, the marked and unmarked programs are wrong in exactly
/// the same pre-existing way. Either way the marked program behaves like the unmarked one.
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

/// `collect_global_var_names_in_body` never descends into EXPRESSIONS, so a `global $a` inside a
/// closure literal is invisible to it and main keeps `$a` in a frame slot instead of the shared
/// symbol — the closure's write does not reach main. That is a PRE-EXISTING hole, and this pins
/// that marking does not change it: the marked program and the unmarked control print the same
/// shape. If the hole is ever closed, both sides move together or this test says so.
#[test]
fn test_marked_and_unmarked_agree_through_the_collect_global_hole() {
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
