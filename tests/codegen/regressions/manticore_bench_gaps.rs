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
        .arg("build")
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
#[ignore = "variadic array_merge() is not ported onto the builtin registry yet"]
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

/// Returns the `--emit-ir` text of a PHP source.
fn emit_ir_text(source: &str, name: &str) -> String {
    use std::process::Command;

    let dir = std::env::temp_dir().join(format!(
        "elephc_packed_counter_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let php_path = dir.join(name);
    std::fs::write(&php_path, source).unwrap();
    let out = Command::new(elephc_cli_bin())
        .arg("build")
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("emit-ir");
    assert!(out.status.success(), "emit-ir failed");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A proven-`int` `for` counter writing into an empty array keeps packed storage: the checker's
/// packed-counter proof reaches the lowering, which no longer promotes the array to a hash (and
/// then re-boxes the whole hash at every `+=` of the matmul kernel).
#[test]
fn test_int_for_counter_keeps_empty_array_packed() {
    let source = "<?php\nfunction build(int $n): array {\n    $row = [];\n    for ($j = 0; $j < $n; $j++) { $row[$j] = $j * 2; }\n    for ($j = 0; $j < $n; $j++) { $row[$j] += 1; }\n    return $row;\n}\necho json_encode(build(4));\n";
    let ir = emit_ir_text(source, "packed_counter.php");
    let body = ir.split("function build(").nth(1).expect("build present");
    let body = body.split("\n  function ").next().unwrap_or(body);
    assert!(!body.contains("array_to_hash"), "counter writes must stay packed:\n{body}");
    assert_eq!(compile_and_run(source), "[1,3,5,7]");
}

/// A counter the body can skip past still leaves the gap PHP leaves.
#[test]
fn test_skipped_for_counter_write_keeps_php_keys() {
    let out = compile_and_run(
        "<?php\nfunction build(int $n): array {\n    $a = [];\n    for ($i = 0; $i < $n; $i++) { if ($i == 1) { continue; } $a[$i] = $i; }\n    return $a;\n}\necho json_encode(build(4));\n",
    );
    assert_eq!(out, "{\"0\":0,\"2\":2,\"3\":3}");
}

/// A range-proven int (or any int/float) stored into a declared `string` property is
/// converted under weak-mode typing, for static and instance properties alike.
#[test]
fn test_int_and_float_stores_into_string_properties_convert() {
    let out = compile_and_run(
        "<?php class S { public static string $t = \"\"; public string $u = \"\"; } \
         $x = ($argc & 255) + 2; S::$t = $x; $o = new S; $o->u = $x; var_dump(S::$t, $o->u); \
         S::$t = 5; $o->u = 2.5; var_dump(S::$t, $o->u);",
    );
    assert_eq!(out, "string(1) \"3\"\nstring(1) \"3\"\nstring(1) \"5\"\nstring(3) \"2.5\"\n");
}

/// Ordering a scalar against `true`, `false` or `null` compares both sides as booleans.
#[test]
fn test_scalar_ordering_against_bool_and_null_uses_truthiness() {
    let out = compile_and_run(
        "<?php $x = $argc & 255; $f = $x + 0.5; var_dump(($x + 2) <=> true, ($x - 2) <=> null, \
         null <=> ($x - 2), ($x - 2) < null, ($x - 1) > null, ($x + 2) >= true, \
         ($x - 1) <=> false, $f <=> null, null < $f, true <=> false);",
    );
    assert_eq!(
        out,
        "int(0)\nint(1)\nint(-1)\nbool(false)\nbool(false)\nbool(true)\nint(0)\nint(1)\nbool(true)\nint(1)\n"
    );
}

/// `declare(strict_types=1)` keeps PHP's `TypeError` for a proven int stored into a `bool`
/// property instead of converting it.
#[test]
#[ignore = "typed property stores do not enforce strict_types yet (a mixed value is stored too)"]
fn test_strict_types_int_into_bool_property_throws() {
    let out = compile_and_run(
        "<?php declare(strict_types=1); class F { public bool $flag = false; } $f = new F; \
         try { $f->flag = ($argc & 255) + 2; echo \"stored\\n\"; } \
         catch (TypeError $e) { echo get_class($e), \"\\n\"; }",
    );
    assert_eq!(out, "TypeError\n");
}

/// A string converted from a proven int by `ReflectionProperty::setValue()` survives later
/// string building.
#[test]
fn test_reflection_static_string_store_persists_converted_int() {
    let out = compile_and_run(
        "<?php class C { public static string $t = \"\"; } \
         (new ReflectionProperty(C::class, \"t\"))->setValue(null, ($argc & 255) + 12344); \
         $x = str_repeat(\"z\", 9) . \"y\"; echo C::$t, \"|\", $x, \"\\n\"; \
         $a = \"ab\" . $argc; (new ReflectionProperty(C::class, \"t\"))->setValue(null, $a . \"cd\"); \
         $x = str_repeat(\"q\", 9) . \"y\"; echo C::$t, \"|\", $x, \"\\n\";",
    );
    assert_eq!(out, "12345|zzzzzzzzzy\nab1cd|qqqqqqqqqy\n");
}

/// A `for` counter passed bare to a by-value user function keeps the filled array packed,
/// while a by-reference callee that advances the counter still yields PHP's sparse keys.
#[test]
fn test_packed_counter_survives_by_value_calls_only() {
    let out = compile_and_run(
        "<?php function v(int $i): float { return $i * 0.5; } \
         function bump(&$i): float { $i++; return 1.0; } \
         function build(int $n): array { $o = []; for ($i = 0; $i < $n; $i++) { $o[$i] = v($i); } return $o; } \
         function skip(int $n): array { $o = []; for ($i = 0; $i < $n; $i++) { $o[$i] = bump($i); } return $o; } \
         echo json_encode(build(4)), json_encode(skip(5)), \"\\n\";",
    );
    assert_eq!(out, "[0,0.5,1,1.5]{\"1\":1,\"3\":1,\"5\":1}\n");
}

/// Integer parameters of functions only called directly take the call sites' ranges, so
/// `$i + $j` inside the callee is native arithmetic; an unbounded call keeps it checked.
#[test]
fn test_direct_call_ranges_prove_callee_arithmetic() {
    let bounded = "<?php function a(int $i, int $j): float { $n = $i + $j; return 1.0 / ($n + 1); }\n\
        $s = 0.0; for ($i = 0; $i < 50; $i++) { for ($j = 0; $j < 50; $j++) { $s += a($i, $j); } }\n\
        printf(\"%.6f\\n\", $s);\n";
    assert_eq!(compile_and_run(bounded), "68.817218\n");
    let unbounded = "<?php function a(int $i, int $j) { return $i + $j; }\n\
        echo a(1, 2), \" \"; var_dump(a(PHP_INT_MAX, 1));\n";
    assert_eq!(compile_and_run(unbounded), "3 float(9.223372036854776E+18)\n");
}

/// A negative key written into a list a proven `for` counter filled still promotes it to a
/// hash like PHP, when the bound comes from a range-proven function argument.
#[test]
fn test_negative_key_after_counter_filled_list_promotes_to_hash() {
    let out = compile_and_run(
        "<?php function s(int $n): string { $arr = []; \
         for ($i = 0; $i < $n; $i++) { $arr[] = $i * 10; } \
         $arr[$n - 10] = 99; $arr[-1 + $n * 0] = 7; return json_encode($arr); } \
         echo s($argc + 3), \"\\n\";",
    );
    assert_eq!(out, "{\"0\":0,\"1\":10,\"2\":20,\"3\":30,\"-6\":99,\"-1\":7}\n");
}
