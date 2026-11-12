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

/// Asserts a heap-debug run printed `expected` and reported a clean heap.
fn assert_heap_clean(out: ProgramOutput, expected: &str) {
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, expected);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `return c ? a : b` and `return a + b` in an `int` function return native ints on their
/// common path (no boxed `int|float` temporary) and stay heap-clean.
#[test]
fn test_int_function_return_split_is_exact_and_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function fib(int $n): int { return $n < 2 ? $n : fib($n - 1) + fib($n - 2); }
function f(int $a, int $b): int { return $a - $b; }
function g(int $a): int { return $a > 0 ? $a - 1 : ($a < -5 ? $a + 2 : $a + 1); }
echo fib(20), " ", f(3, 10), " ", g(5), " ", g(-3), " ", g(-9), "\n";
"#,
    );
    assert_heap_clean(out, "6765 -7 4 -2 -7\n");
}

/// Decoding JSON in a loop releases every decoded container, key, and the source copy.
#[test]
fn test_json_decode_loop_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$doc = json_encode([["id" => 1, "name" => "a", "tags" => ["x", "y"], "o" => []], ["id" => 2, "name" => "b"]]);
$sum = 0;
for ($r = 0; $r < 50; $r++) {
    $d = json_decode($doc, true);
    $sum += count($d) + $d[1]["id"] + count($d[0]["tags"]);
}
$o = json_decode("1");
echo $sum, " ", $o, "\n";
"#,
    );
    assert_heap_clean(out, "300 1\n");
}

/// `isset($h[$k])` on a hash unboxed from a `mixed` local releases its temporary reference, so
/// the following write does not copy-on-write split (and leak) the table on every iteration.
#[test]
fn test_isset_then_write_on_mixed_local_hash_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$words = ["alpha", "beta", "alpha"];
$counts = [];
for ($i = 0; $i < 50; $i++) {
    foreach ($words as $w) {
        if (isset($counts[$w])) { $counts[$w] = $counts[$w] + 1; } else { $counts[$w] = 1; }
    }
}
foreach ($counts as $word => $c) { echo $word, "=", $c, " "; }
echo "\n";
"#,
    );
    assert_heap_clean(out, "alpha=100 beta=50 \n");
}

/// Re-inserting keys after `unset()` never duplicates a key that lives past a tombstone, and
/// tombstone-heavy tables are rebuilt instead of degrading into full scans.
#[test]
fn test_hash_unset_reinsert_churn_keeps_unique_keys() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$m = [];
for ($i = 0; $i < 64; $i++) { $m["k$i"] = $i; }
for ($r = 0; $r < 50; $r++) {
    for ($i = 0; $i < 64; $i += 3) { unset($m["k$i"]); }
    for ($i = 0; $i < 64; $i++) { $m["k$i"] = $i + $r; }
}
echo count($m), " ", array_sum($m), " ", implode(",", array_slice(array_keys($m), 0, 5)), "\n";
"#,
    );
    assert_heap_clean(out, "64 5152 k1,k2,k4,k5,k7\n");
}

/// `.=` on a string local grows a uniquely owned buffer in place but keeps copy-on-write
/// semantics for copies, parameters, and array elements holding the same string.
#[test]
fn test_string_append_in_place_keeps_value_semantics() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f(string $p) { $p .= "!"; $p .= $p; return $p; }
$s = "";
for ($i = 0; $i < 1000; $i++) { $s .= "x"; }
$t = $s; $t .= "y"; $s .= "z";
echo strlen($s), substr($s, -2), " ", strlen($t), substr($t, -2), "\n";
$a = "ab"; $b = f($a); $a .= "c"; echo $a, " ", $b, "\n";
$n = 5; $q = "n="; $q .= $n; $q .= "/" . $a . $n; echo $q, "\n";
$arr = [$q]; $q .= "+"; echo $arr[0], " ", $q, "\n";
$h = "lit"; $h .= ""; $h .= $h; echo $h, "\n";
"#,
    );
    assert_heap_clean(
        out,
        "1001xz 1001xy\nabc ab!ab!\nn=5/abc5\nn=5/abc5 n=5/abc5+\nlitlit\n",
    );
}

/// `fgets()` reads whole lines from seekable files without consuming past the newline, so
/// interleaved `fread()`/`fgetc()`/`ftell()` see the same stream position as PHP.
#[test]
fn test_fgets_chunked_reads_keep_stream_position() {
    let dir = std::env::temp_dir().join(format!("elephc_fgets_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("data.txt");
    let source = format!(
        r#"<?php
$p = "{}";
file_put_contents($p, "a\nbb\n\nccc\nlast-no-nl");
$f = fopen($p, "r");
while (($l = fgets($f)) !== false) {{ echo strlen($l), feof($f) ? "E" : "-", " "; }}
fclose($f);
$f = fopen($p, "r");
echo fgets($f), fread($f, 3), "|", fgetc($f), "|", ftell($f), "\n";
"#,
        path.display()
    );
    let out = compile_and_run(&source);
    assert_eq!(out, "2- 3- 1- 4- 10E a\nbb\n|\n|6\n");
}

/// `ksort`/`krsort`/`asort`/`arsort` reorder associative arrays with PHP 8 `SORT_REGULAR`
/// rules (mixed int/string keys, numeric strings, stable ties) and leave copies untouched.
#[test]
fn test_hash_sorts_match_php_and_keep_copies() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$m = ["b" => 3, "a" => 1, "10" => 2, "9" => 2, "x1" => 1, "1.5" => 7, "z" => 3];
$c = $m;
ksort($m); echo json_encode($m), "\n";
krsort($m); echo json_encode($m), "\n";
asort($m); echo json_encode($m), "\n";
arsort($m); echo json_encode($m), "\n";
echo json_encode($c), "\n";
$s = ["pear" => "b", "apple" => "a", "fig" => "B", "kiwi" => "10", "lime" => "9", "plum" => "a"];
asort($s); echo json_encode($s), "\n";
ksort($s, SORT_STRING); echo implode(",", array_keys($s)), "\n";
"#,
    );
    assert_heap_clean(
        out,
        concat!(
            "{\"1.5\":7,\"9\":2,\"10\":2,\"a\":1,\"b\":3,\"x1\":1,\"z\":3}\n",
            "{\"z\":3,\"x1\":1,\"b\":3,\"a\":1,\"10\":2,\"9\":2,\"1.5\":7}\n",
            "{\"x1\":1,\"a\":1,\"10\":2,\"9\":2,\"z\":3,\"b\":3,\"1.5\":7}\n",
            "{\"1.5\":7,\"z\":3,\"b\":3,\"10\":2,\"9\":2,\"x1\":1,\"a\":1}\n",
            "{\"b\":3,\"a\":1,\"10\":2,\"9\":2,\"x1\":1,\"1.5\":7,\"z\":3}\n",
            "{\"lime\":\"9\",\"kiwi\":\"10\",\"fig\":\"B\",\"apple\":\"a\",\"plum\":\"a\",\"pear\":\"b\"}\n",
            "apple,fig,kiwi,lime,pear,plum\n",
        ),
    );
}

/// PHP 8 `<=>` on non-numeric strings releases the stringified operands it compares.
#[test]
fn test_mixed_string_spaceship_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["x" . $argc, "b", 10, "10", 9.5, "abc"];
for ($i = 0; $i < 6; $i++) { for ($j = 0; $j < 6; $j++) { echo $a[$i] <=> $a[$j]; } }
echo "\n";
"#,
    );
    assert_heap_clean(out, "011111-101111-1-1001-1-1-1001-1-1-1-1-10-1-1-11110\n");
}

/// A loop calling a string builtin that writes the concat scratch keeps its per-statement
/// `concat_reset` (runtime calls do not always declare `alloc_concat`), so long loops do not
/// overrun the scratch area.
#[test]
fn test_concat_reset_kept_next_to_scratch_writing_builtins() {
    use std::process::Command;

    let dir = std::env::temp_dir().join(format!("elephc_concat_reset_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let php_path = dir.join("upper.php");
    std::fs::write(
        &php_path,
        "<?php\nfunction f() { $n = 0; $s = 'hello world';\n  for ($i = 0; $i < 3; $i++) { $n += strlen(strtoupper($s)); }\n  return $n; }\necho f();\n",
    )
    .unwrap();
    let out = Command::new(elephc_cli_bin())
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("emit-ir");
    assert!(out.status.success(), "emit-ir failed");
    let text = String::from_utf8_lossy(&out.stdout);
    let body = text.split("function f(").nth(1).expect("f present");
    let body = body.split("\n  function ").next().unwrap_or(body);
    assert!(body.contains("concat_reset"), "loop concat_reset was removed:\n{body}");
    assert_eq!(compile_and_run("<?php function f() { $n = 0; $s = 'hello world'; for ($i = 0; $i < 3; $i++) { $n += strlen(strtoupper($s)); } return $n; } echo f();"), "33");
}
