//! Purpose:
//! Regression tests for the correctness and performance gaps found by running the Manticore
//! compiler benchmark corpus through elephc: overflow-free integer arithmetic proofs, `Never`
//! array element unions, `isset` on scalar slots of `mixed` hashes, `int`/`bool` local merges,
//! `Mixed` float arithmetic, multi-argument `array_merge`, large `json_encode` output, and
//! dynamic string keys in function loops.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use super::*;

/// Counts `ichecked_add`/`iadd` instructions in `main` for a PHP source compiled with `--emit-ir`.
fn count_main_adds(source: &str, name: &str) -> (usize, usize) {
    use std::process::Command;

    let dir = std::env::temp_dir().join(format!(
        "elephc_int_range_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let php_path = dir.join(name);
    std::fs::write(&php_path, source).unwrap();
    let out = Command::new(elephc_cli_bin())
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("emit-ir");
    assert!(out.status.success(), "emit-ir failed");
    let text = String::from_utf8_lossy(&out.stdout);
    let main_body = text.split("function main(").nth(1).expect("main present");
    let main_body = main_body.split("\n  function ").next().unwrap_or(main_body);
    (
        main_body.matches("= iadd ").count(),
        main_body.matches("= ichecked_add ").count(),
    )
}

/// A loop counter bounded by its condition and a masked accumulator are proven overflow-free,
/// so they compile to native `iadd` instead of the boxing `ichecked_add`.
#[test]
fn test_bounded_loop_counter_and_masked_accumulator_use_native_adds() {
    let source = "<?php\n$s = $argc;\nfor ($i = 0; $i < 1000; $i++) {\n  $s = ($s * 3 + $i) & 0xFFFF;\n}\necho $s;\n";
    let (native, checked) = count_main_adds(source, "bounded.php");
    assert_eq!(checked, 0, "no checked add may remain for proven sites");
    assert!(native >= 2, "loop counter and accumulator adds are native");
    assert_eq!(compile_and_run(source), "28661");
}

/// Proofs never change PHP semantics: the increment that reaches `PHP_INT_MAX + 1` still
/// promotes to float, while a loop bounded by `PHP_INT_MAX` ends on the exact int.
#[test]
fn test_overflow_promotion_is_preserved_next_to_proven_sites() {
    let out = compile_and_run(
        "<?php $i = PHP_INT_MAX - 1; $i++; echo gettype($i), ' '; $i++; echo gettype($i), ' ';\n\
         $k = PHP_INT_MAX; $k = $k + 1; echo gettype($k), ' ';\n\
         for ($j = PHP_INT_MAX - 2; $j < PHP_INT_MAX; $j++) {} var_dump($j);",
    );
    assert_eq!(out, "integer double double int(9223372036854775807)\n");
}

/// Recursive fibonacci: the guarded `$n - 1`/`$n - 2` are proven, the unbounded sum is not.
#[test]
fn test_fib_with_proven_argument_arithmetic() {
    let out = compile_and_run(
        "<?php function fib(int $n): int { return $n < 2 ? $n : fib($n - 1) + fib($n - 2); } echo fib(25);",
    );
    assert_eq!(out, "75025");
}

/// `[]` reset inside an outer loop and filled in an inner loop must not type its elements as
/// `Never|int` (which rejected the later arithmetic as non-numeric).
#[test]
fn test_empty_array_refilled_in_nested_loop_keeps_numeric_elements() {
    let out = compile_and_run(
        "<?php for ($r = 0; $r < 2; $r++) { $a = []; for ($i = 0; $i < 2; $i++) { $a[] = 5; } $s = 1 + $a[0]; echo $s; }",
    );
    assert_eq!(out, "66");
}

/// `isset()` on a mixed-valued hash whose slot holds a raw int must read the slot tag instead
/// of unboxing the int as a pointer (this segfaulted the classic word-count loop).
#[test]
fn test_isset_counter_on_mixed_hash_slots() {
    let out = compile_and_run(
        r#"<?php
$w = ["alpha", "beta", "alpha", "gamma", "alpha"];
$c = [];
for ($r = 0; $r < 3; $r++) {
    foreach ($w as $x) {
        if (isset($c[$x])) { $c[$x] = $c[$x] + 1; } else { $c[$x] = 1; }
    }
}
foreach ($c as $k => $v) { echo $k, "=", $v, " "; }
"#,
    );
    assert_eq!(out, "alpha=9 beta=3 gamma=3 ");
}

/// A local that is `false` before a loop and receives an int before `break` must keep the int.
#[test]
fn test_int_assigned_before_break_into_false_local_is_not_read_as_bool() {
    let out = compile_and_run(
        "<?php $found = false; $x = 5; for ($p = 0; $p < 3; $p++) { if ($p == 1) { $found = $x; break; } } var_dump($found);",
    );
    assert_eq!(out, "int(5)\n");
}

/// `int|float` values flowing through `Mixed` arithmetic return the float result instead of
/// falling into the array-union error path.
#[test]
fn test_mixed_float_arithmetic_returns_float() {
    let out = compile_and_run("<?php $d = $argc > 0 ? 5.5 : 3; var_dump($d * 2, $d + 1, $d - 1);");
    assert_eq!(out, "float(11)\nfloat(6.5)\nfloat(4.5)\n");
}

/// `array_merge()` with more than two arguments folds into binary merges.
#[test]
fn test_array_merge_with_three_arguments() {
    let out = compile_and_run("<?php echo implode(',', array_merge([1, 2], [3], [4, 5]));");
    assert_eq!(out, "1,2,3,4,5");
}

/// Encoded JSON larger than the old 64 KiB string scratch area stays intact.
#[test]
fn test_large_json_encode_output_is_not_truncated() {
    let out = compile_and_run(
        r#"<?php
$rows = [];
for ($i = 0; $i < 2000; $i++) {
    $rows[] = ["id" => $i, "name" => "item" . $i, "tags" => ["a", "b"]];
}
echo strlen(json_encode($rows, JSON_PRETTY_PRINT)), " ", strlen(json_encode($rows));
"#,
    );
    assert_eq!(out, "241782 91781");
}

/// `crc32()` matches PHP on empty, short, and long inputs (table-driven helper).
#[test]
fn test_crc32_matches_php() {
    let out = compile_and_run(
        r#"<?php echo crc32("The quick brown fox jumped over the lazy dog."), " ", crc32(""), " ", crc32(str_repeat("abc", 1000));"#,
    );
    assert_eq!(out, "2191738434 0 733562123");
}

/// A function filling `[]` with dynamic string keys in a loop returns a usable hash.
#[test]
fn test_function_loop_with_dynamic_string_keys_returns_hash() {
    let out = compile_and_run(
        r#"<?php
function build_map(int $n): array {
    $a = [];
    for ($i = 0; $i < $n; $i++) {
        $a["k" . $i] = $i;
    }
    return $a;
}
$a = build_map(5);
echo count($a) . ":" . $a["k0"] . ":" . $a["k4"];
"#,
    );
    assert_eq!(out, "5:0:4");
}
