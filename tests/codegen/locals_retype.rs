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

/// A `string`-typed READ above the kill must stay a plain string load.
///
/// The abandoned slot is nulled through the ordinary overwrite path, and that path widens the
/// slot to whatever type it stores. A slot's storage type is a whole-FRAME property, so nulling
/// at `Void` re-typed this slot `Str | Void` = `Mixed` and every load lowered ABOVE the kill —
/// `strlen($a)` here — turned into a detach out of a boxed cell that nothing releases. Measured
/// before the fix: `HEAP DEBUG: live_blocks=1 live_bytes=48`, one leaked block per executed read,
/// with the right answer still printed. The kill now nulls at the slot's own storage type.
#[test]
fn test_string_read_above_a_kill_stays_unboxed() {
    let out = compile_and_run_with_heap_debug(
        "<?php $a = \"n\" . $argc; $b = strlen($a); unset($a); $a = 5; echo $b, $a;",
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "25");
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

/// The RHS reads the old heap binding and the retype releases it exactly once.
#[test]
fn test_retype_rhs_reads_the_old_heap_value_and_releases_it_once() {
    let out = compile_and_run_with_heap_debug("<?php $a = \"n\" . $argc; $a = strlen($a); echo $a;");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The retype's NEW value still holds the OLD value: the release must not free storage the
/// fresh binding now owns.
///
/// `$a = [$a]` retypes `string` to `array<string>` while the array element is the very string
/// the old slot holds. A release ordered before the array literal acquires its element would be
/// a use-after-free; one that never runs would leak.
#[test]
fn test_retype_whose_new_value_contains_the_old_one() {
    let out = compile_and_run_with_heap_debug("<?php $a = \"s\" . $argc; $a = [$a]; echo $a[0];");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "s1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
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
