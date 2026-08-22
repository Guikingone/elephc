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
