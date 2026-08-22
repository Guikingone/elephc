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
/// Nulling at the slot's OWN type removes the leak but is NOT a valid fix: `release` of a raw
/// `Str` is an unconditional `__rt_heap_free_safe` (strings are not refcounted —
/// `codegen::lower_inst::ownership::release_loaded_string`), so it frees a buffer any copy of the
/// local still points at. Measured: `$q = "a" . $n; $r = $q; $q = 1; return $r . "|" . $q;`
/// returned four NUL bytes. The `Mixed` widening is what makes the release a decref instead, so
/// the leak is the price of the safety and belongs to its own fix in the ownership model.
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
    assert!(
        compile_files_fails(
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
        ),
        "an ambiguous (span, name) local-binding decision must not compile"
    );
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
