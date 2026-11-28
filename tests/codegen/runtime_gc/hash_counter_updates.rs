//! Purpose:
//! Verifies the guarded in-place integer add for `$hash[$key] = $hash[$key] + N` on a local
//! `array<..., mixed>` keeps PHP results, warnings and copy-on-write isolation.
//!
//! Called from:
//! - The runtime/GC codegen integration suite.
//!
//! Key details:
//! - Covers string, numeric-string and integer keys, a missing key (warning + fallback), integer
//!   overflow to float at both ends, float and numeric-string elements, a copy that must not see
//!   the update, a reference into the element, a value read before the update, `+=`/`-=`, and a
//!   top-level counter, a slot widened to boxed storage by a later by-reference call, with a clean
//!   heap in both optimizer modes.

use crate::support::*;

/// Counter updates take the in-place add only when indistinguishable from the ordinary statement.
#[test]
fn test_hash_counter_updates_match_php_on_fast_and_fallback_paths() {
    let source = r#"<?php
function run(): void {
    $c = [];
    $words = ["a", "10", "b", "a", "10", "a"];
    foreach ($words as $w) {
        if (isset($c[$w])) { $c[$w] = $c[$w] + 1; } else { $c[$w] = 1; }
    }
    var_dump($c);
    $k = "missing";
    $c[$k] = $c[$k] + 1;
    $m = "max";
    $c[$m] = PHP_INT_MAX;
    $c[$m] = $c[$m] + 1;
    $c["f"] = 1.5;
    $c["f"] = $c["f"] + 2;
    $c["s"] = "7";
    $c["s"] = $c["s"] + 3;
    $copy = $c;
    $c["a"] = $c["a"] + 100;
    $c["a"] -= 1;
    $c["a"] += 5;
    $i = 10;
    $c[$i] = $c[$i] - 2;
    $c["neg"] = PHP_INT_MIN;
    $c["neg"] = $c["neg"] - 1;
    $r = &$c["b"];
    $c["b"] = $c["b"] + 1;
    echo "r=$r\n";
    unset($r);
    $x = $c["a"];
    $c["a"] = $c["a"] + 1;
    echo "x=$x\n";
    var_dump($c, $copy["a"]);
}
run();
function byr(array &$a): void { $a["z"] = 1; }
function widened(): void {
    $h = ["a" => 1, "s" => "s"];
    for ($i = 0; $i < 1000; $i++) { $h["a"] = $h["a"] + 1; }
    byr($h);
    echo $h["a"], " ", $h["z"], "\n";
}
widened();
$g = ["x" => 1];
for ($n = 0; $n < 5; $n++) { $g["x"] = $g["x"] + 2; }
echo $g["x"], "\n";
"#;
    let expected = r#"array(3) {
  ["a"]=>
  int(3)
  [10]=>
  int(2)
  ["b"]=>
  int(1)
}
r=2
x=107
array(8) {
  ["a"]=>
  int(108)
  [10]=>
  int(0)
  ["b"]=>
  int(2)
  ["missing"]=>
  int(1)
  ["max"]=>
  float(9.223372036854776E+18)
  ["f"]=>
  float(3.5)
  ["s"]=>
  int(10)
  ["neg"]=>
  float(-9.223372036854776E+18)
}
int(3)
1001 1
11
"#;
    for off in [false, true] {
        let run = || {
            let out = compile_and_run_with_heap_debug(source);
            assert!(out.success, "{}", out.stderr);
            assert_eq!(out.stdout, expected, "{}", out.stderr);
            assert_eq!(out.stderr.matches("Undefined array key \"missing\"").count(), 1, "{}", out.stderr);
            assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
        };
        if off { without_ir_opt(run); } else { run(); }
    }
}
