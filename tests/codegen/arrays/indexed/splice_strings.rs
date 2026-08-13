//! Purpose:
//! PHP-differential regression tests for `array_splice()` on an indexed `array<string>`.
//! The three-argument form used to corrupt memory: the splice helpers built the removed-elements
//! array with `__rt_array_new(n, 8)` and moved 8-byte slots, while indexed string arrays store
//! 16-byte `{pointer, length}` pairs. `array_splice(["a","b","c","d"], 1, 2)` therefore answered
//! `[1, 4362860248]` — a raw heap pointer surfacing as a PHP integer — and left the receiver
//! half-shifted.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected string in this file is verbatim `LC_ALL=C php` 8.4 output for the same fixture.
//! - `__rt_array_splice_str` MOVES the removed string payloads into the result: an indexed string
//!   array owns its persisted bytes exclusively, so the heap-debug fixtures below would report a
//!   double free (a crash) if it retained them and a leak if it copied them.
//! - `__rt_array_splice_insert_str` DUPLICATES each inserted replacement string, because the
//!   replacement array keeps owning its own payloads and the caller releases it afterwards.
//! - The long-replacement fixtures force `__rt_array_grow`, which relocates the receiver, so they
//!   also cover the write-back of the new pointer.

use super::*;
use crate::support::compile_and_run_with_heap_debug;

/// Verifies the three-argument removal on a string receiver: the removed elements come back as
/// strings and the receiver keeps exactly the surviving ones, in order.
///
/// This is the fixture whose second element used to print a raw pointer.
#[test]
fn test_array_splice_string_removal_matches_php() {
    let out = compile_and_run(
        r#"<?php
$a = ["a","b","c","d"]; $r = array_splice($a, 1, 2); echo implode(",",$r), "|", implode(",",$a), "\n";
$b = ["a","b","c","d"]; $s = array_splice($b, -2); echo implode(",",$s), "|", implode(",",$b), "\n";
$c = ["a","b","c","d"]; $t = array_splice($c, 1, -1); echo implode(",",$t), "|", implode(",",$c), "\n";
$d = ["a","b","c"]; $u = array_splice($d, 10, 5); echo count($u), "|", implode(",",$d), "\n";
$e = ["a","b","c"]; $v = array_splice($e, -10, 5); echo implode(",",$v), "|", count($e), "\n";
$f = ["alpha","beta","gamma","delta","epsilon"]; $w = array_splice($f, 1, 3);
echo implode(",",$w), "|", implode(",",$f), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"b,c|a,d
c,d|a,b
b,c|a,d
0|a,b,c
a,b,c|0
beta,gamma,delta|alpha,epsilon
"#
    );
}

/// Verifies the four-argument form on a string receiver across every replacement length:
/// equal to the removed window, longer than it (which grows and relocates the receiver),
/// shorter, a pure insertion, a bare string scalar, `null`, and `[]`.
#[test]
fn test_array_splice_string_replacement_matches_php() {
    let out = compile_and_run(
        r#"<?php
$a = ["a","b","c"]; $r1 = array_splice($a, 1, 1, ["z"]); echo implode(",",$a), "|", implode(",",$r1), "\n";
$b = ["a","b","c"]; $r2 = array_splice($b, 1, 1, ["x","y","z","w"]); echo implode(",",$b), "|", implode(",",$r2), "\n";
$c = ["alpha","beta","gamma","delta","epsilon"]; $r3 = array_splice($c, 1, 3, ["ONE","TWO"]);
echo implode(",",$c), "|", implode(",",$r3), "\n";
$d = ["a","b","c"]; $r4 = array_splice($d, 1, 0, ["p","q"]); echo implode(",",$d), "|", count($r4), "\n";
$e = ["a","b","c"]; $r5 = array_splice($e, 1, 2, "solo"); echo implode(",",$e), "|", implode(",",$r5), "\n";
$f = ["a","b","c"]; $r6 = array_splice($f, 1, 1, null); echo implode(",",$f), "|", implode(",",$r6), "\n";
$g = ["a","b","c"]; $r7 = array_splice($g, 0, 3, []); echo count($g), "|", implode(",",$r7), "\n";
$h = ["a","b"]; $r8 = array_splice($h, 1, 0, ["c","d","e","f","g","h","i","j"]);
echo implode(",",$h), "|", count($r8), "\n";
$i = ["a","b","c"]; $r9 = array_splice($i, 1, 1, replacement: ["N","M"]); echo implode(",",$i), "|", implode(",",$r9), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"a,z,c|b
a,x,y,z,w,c|b
alpha,ONE,TWO,epsilon|beta,gamma,delta
a,p,q,b,c|0
a,solo|b,c
a,c|b
0|a,b,c
a,c,d,e,f,g,h,i,j,b|0
a,N,M,c|b
"#
    );
}

/// Verifies the string receiver's copy-on-write contract and its five receiver forms.
///
/// An alias taken before the call must keep the original elements, and a by-reference parameter,
/// an instance property, a static property, and an array element must all observe the mutation.
#[test]
fn test_array_splice_string_receiver_forms_match_php() {
    let out = compile_and_run(
        r#"<?php
class S { public array $items = ["a","b","c"]; public static array $shared = ["a","b","c"]; }
function bump(array &$a): void { array_splice($a, 1, 1, ["Z"]); }

$local = ["a","b","c"]; $alias = $local; array_splice($local, 1, 1, ["Z"]);
echo implode(",",$local), "|", implode(",",$alias), "\n";

$p = ["a","b","c"]; bump($p); echo implode(",",$p), "\n";

$s = new S(); array_splice($s->items, 1, 1, ["Z"]); echo implode(",",$s->items), "\n";
array_splice(S::$shared, 1, 1, ["Z"]); echo implode(",",S::$shared), "\n";

$nested = [["a","b","c"]]; array_splice($nested[0], 1, 1, ["Z"]); echo implode(",",$nested[0]), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"a,Z,c|a,b,c
a,Z,c
a,Z,c
a,Z,c
a,Z,c
"#
    );
}

/// Verifies the string splice keeps the heap balanced in a loop that both removes and inserts.
///
/// The removed payloads are handed to the result array (which is released at the end of each
/// iteration) and the inserted ones are freshly persisted, so a retained-instead-of-moved
/// removal would double free and a copied-instead-of-moved removal would leak one block per
/// element.
#[test]
fn test_array_splice_string_insertion_leaves_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a","b","c","d","e","f"];
for ($i = 0; $i < 4; $i++) {
    $r = array_splice($a, 1, 2, ["p" . $i, "q" . $i, "r" . $i]);
    echo implode(",", $a), "|", implode(",", $r), "\n";
}
"#,
    );
    assert_eq!(
        out.stdout,
        r#"a,p0,q0,r0,d,e,f|b,c
a,p1,q1,r1,r0,d,e,f|p0,q0
a,p2,q2,r2,r1,r0,d,e,f|p1,q1
a,p3,q3,r3,r2,r1,r0,d,e,f|p2,q2
"#,
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Verifies the removed-elements array outlives the receiver's later mutation.
///
/// The removal moves owned payloads into a fresh array, so growing the receiver afterwards (a
/// reallocation that frees the old backing store) must not disturb the strings the result holds.
#[test]
fn test_array_splice_string_removed_array_survives_receiver_growth() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["one","two","three","four"];
$removed = array_splice($a, 1, 2);
for ($i = 0; $i < 6; $i++) { $a[] = "x" . $i; }
echo implode(",", $removed), "|", implode(",", $a), "\n";
"#,
    );
    assert_eq!(
        out.stdout,
        "two,three|one,four,x0,x1,x2,x3,x4,x5\n",
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}
