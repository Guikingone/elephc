//! Purpose:
//! php-src-derived regression coverage for the array, sort, string and JSON paths optimized
//! by the native-speed branch.
//!
//! Called from:
//! - `cargo test` through the codegen test harness.
//!
//! Key details:
//! - Cases are ported from PHP 8.5 php-src `.phpt` files, adapted only where elephc rejects a
//!   pre-existing unsupported form (reference elements in array literals, scalar coercion in
//!   `str_repeat`). Those omissions are called out at the case that needs them.
//! - Each fixture runs with heap debug so ownership regressions fail with a leak report.

use crate::support::*;

/// Ports `ext/standard/tests/array/array_sum*.phpt` and the fused `array_map` path.
///
/// Covers integer/float/mixed arrays, empty arrays, numeric strings, octal/hex literals,
/// duplicates, associative keys, booleans, nulls, a 64-bit-value int array, and positive/negative/
/// overflow-promoted fused map sums. `array_sum_on_reference.phpt` is represented only by
/// its numeric-value behavior: elephc rejects a reference element inside an array literal.
#[test]
fn php_src_array_sum_and_fused_map_match_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$int = [1, 2, 3, 4, 5];
$float = [1.0, 2.2, 3.4, 4.6];
$mixed = [1, 2.3, 4, 0.6, 10];
echo array_sum($int), '|', array_sum($float), '|', array_sum($mixed), '|', array_sum([]), '|';
echo array_sum([3, 2, 100, 150, 25, 350, 0, -3, -1200]), '|';
echo array_sum([056, 023, 00, 015, -045, 01, -07]), '|';
echo array_sum([0xAE, 0x2B, 0X10, -0xCF, 0X12, -0XF2]), '|';
echo array_sum([2.3, 1.9, -4.1, 0.5, 1.9, -4.1, 3.6, 0.5]), '|';
echo array_sum(['a' => 20, "b" => 50, 'c' => 0, 'd' => -30, "e" => 100]), '|';
echo array_sum([true, true, false, true, false]), '|';
echo array_sum([null, null]), '|';
$ints = [];
$strings = [];
for ($i = 1; $i <= 1000; $i++) { $ints[] = $i; $strings[] = (string) $i; }
echo array_sum($ints), '|', array_sum($strings), '|';
echo array_sum([3000000000, 2000000000, 123456789]), '|';
$src = range(1, 5);
echo array_sum(array_map(fn($x) => $x + 1, $src)), '|';
$neg = [1, 2, 3];
echo array_sum(array_map(fn($x) => $x - 10, $neg)), '|';
$ov = range(PHP_INT_MAX, PHP_INT_MAX);
$z = 0;
$z ^= array_sum(array_map(fn($x) => $x + 1, $ov));
echo $z, "|\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "15|11.2|17.9|0|-573|35|-198|2.5|140|3|0|500500|500500|5123456789|20|-24|-9223372036854775808|\n"
    );
    assert!(
        out.stderr
            .contains("The float 9.223372036854776E+18 is not representable as an int"),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Ports numeric `sort_basic`/`rsort_basic` cases and the case-conversion/str_repeat variants.
///
/// `strtolower.phpt` and `strtoupper1.phpt` exercise every byte 0..255 plus the documented
/// basic strings; `str_repeat.phpt` and `str_repeat_variation1.phpt` cover zero/positive
/// counts, binary payloads, the negative-count `ValueError`, and the constant length fold.
/// Scalar `str_repeat` coercion is a pre-existing unsupported form and is not asserted here.
#[test]
fn php_src_sort_case_and_repeat_match_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$u = [100, 33, 555, 22];
sort($u);
echo implode(',', $u), '|';
rsort($u);
echo implode(',', $u), '|';
$descending = range(100, 1);
sort($descending);
echo $descending[0], ',', $descending[99], '|';
$ascending = range(1, 100);
rsort($ascending);
echo $ascending[0], ',', $ascending[99], '|';
$duplicates = [5, 5, 3, 3, 1];
sort($duplicates);
echo implode(',', $duplicates), '|';
$bad = 0;
for ($i = 0; $i <= 255; $i++) {
    $char = chr($i);
    $lower = strtolower($char);
    $upper = strtoupper($char);
    if (strlen($lower) !== 1 || strlen($upper) !== 1) { $bad++; }
    if ($i >= 65 && $i <= 90) {
        if ($lower !== chr($i + 32)) { $bad++; }
    } elseif ($i >= 97 && $i <= 122) {
        if ($upper !== chr($i - 32)) { $bad++; }
    } elseif ($lower !== $char || $upper !== $char) {
        $bad++;
    }
}
echo $bad, '|';
echo strtolower("Mary Had A liTTle LAmb and ShE loveD IT So"), '|';
echo bin2hex(strtolower("ABCD\0abcdABCD")), '|';
echo bin2hex(strtoupper("abcd\0ABCD")), '|';
echo strlen(strtolower("Hello Wörld")), '|', strlen(strtoupper("Hello Wörld")), '|';
echo bin2hex(str_repeat("a", 3)), '|';
echo str_repeat("foo", 3), '|';
echo bin2hex(str_repeat("\x00", 3)), '|';
echo bin2hex(str_repeat(chr(0) . chr(128) . chr(129) . chr(234) . chr(235) . chr(254) . chr(255), 3)), '|';
$s = "ab";
echo strlen(str_repeat($s, 32)), '|';
try { str_repeat("x", -1); } catch (ValueError $e) { echo $e->getMessage(), '|'; }
echo "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "22,33,100,555|555,100,33,22|1,100|100,1|1,3,3,5,5|0|\
         mary had a little lamb and she loved it so|\
         61626364006162636461626364|414243440041424344|12|12|\
         616161|foofoofoo|000000|008081eaebfeff008081eaebfeff008081eaebfeff|64|\
         str_repeat(): Argument #2 ($times) must be greater than or equal to 0|\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Ports the count-only `json_decode` observations from php-src JSON tests.
///
/// Covers a normal object root, the default depth-512 rejection, invalid UTF-8, escaped
/// and duplicate keys, numeric-looking keys, and the >64-key decoder fallback.
#[test]
fn php_src_json_decode_count_matches_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$obj = '{"a":1,"b":[1,2,3],"c":"text"}';
echo count(json_decode($obj, true)), '|';
$deep = str_repeat('[', 513) . str_repeat(']', 513);
try { count(json_decode($deep, true)); } catch (TypeError $e) { echo 'deep:', $e->getMessage(), '|'; }
$badUtf8 = "{\"a\":\"\xC3\x28\"}";
try { count(json_decode($badUtf8, true)); } catch (TypeError $e) { echo 'utf8:', $e->getMessage(), '|'; }
$escaped = '{"a\u0062":1,"x":2}';
echo count(json_decode($escaped, true)), '|';
$duplicate = '{"a":1,"a":2,"b":3}';
echo count(json_decode($duplicate, true)), '|';
$numeric = '{"1":1,"01":2}';
echo count(json_decode($numeric, true)), '|';
$large = '{' . implode(',', array_map(fn($i) => '"k' . $i . '":1', range(1, 100))) . '}';
echo count(json_decode($large, true)), '|';
echo "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "3|deep:count(): Argument #1 ($value) must be of type Countable|array, null given|\
         utf8:count(): Argument #1 ($value) must be of type Countable|array, null given|\
         2|2|2|100|\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Ports the scalar/search/JSON-encode and fused filter/reduce cases from php-src.
///
/// `strpos.phpt` supplies the offset/NUL/empty-needle/false cases, `sqrt.phpt` the
/// integer truncation and checked-add promotion cases, `json_encode_*` the scalar/array/
/// escaping/flags cases, `array_filter` the modulo-count cases (including a negative
/// divisor and DivisionByZeroError), and `array_reduce` the empty/overflow cases.
#[test]
fn php_src_strpos_sqrt_json_encode_filter_reduce_match_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
echo (int)strpos("Hello World", "o"), '|';
echo (int)strpos("Hello World", "o", 5), '|';
echo (int)strpos("Hello World", "x"), '|';
echo var_export(strpos("Hello World", "x"), true), '|';
echo (int)strpos("a\0b", "b"), '|';
echo (int)strpos("abc", ""), '|';
echo (int)strpos("abc", "c", -1), '|';
echo (int)sqrt(9), '|', (int)sqrt(2), '|', (int)sqrt(PHP_INT_MAX), '|';
$a = PHP_INT_MAX;
$b = 1;
echo (int)sqrt($a + $b), '|';
echo json_encode([1, 2, 3]), '|';
echo json_encode(["a" => "x", "b" => true, "c" => null]), '|';
echo json_encode("é"), '|';
echo json_encode("a/b"), '|';
echo json_encode(1.5), '|';
echo json_encode([]), '|';
echo json_encode(["a" => "x"], JSON_UNESCAPED_SLASHES), '|';
echo strlen(json_encode(["a" => 1, "b" => [1, 2, 3], "c" => "text"])), '|';
$ints = range(1, 10);
echo count(array_filter($ints, fn($x) => $x % 2 === 0)), '|';
$empty = [];
echo count(array_filter($empty, fn($x) => $x % 2 === 0)), '|';
$threes = range(1, 10);
echo count(array_filter($threes, fn($x) => $x % 3 === 0)), '|';
$negs = range(1, 10);
echo count(array_filter($negs, fn($x) => $x % -3 === 0)), '|';
try { count(array_filter($ints, fn($x) => $x % 0 === 0)); } catch (DivisionByZeroError $e) { echo 'div0|'; }
$r = range(1, 100);
echo array_reduce($r, fn($c, $x) => $c + $x, 0), '|';
echo array_reduce($empty, fn($c, $x) => $c + $x, 7), '|';
$ovr = [PHP_INT_MAX, PHP_INT_MAX];
$v = array_reduce($ovr, fn($c, $x) => $c + $x, 0);
echo is_float($v) ? 'float|' : 'int|';
echo md5(""), '|';
echo md5("abc"), '|';
echo strlen(md5("abc")), '|';
echo bin2hex(md5("abc", true)), '|';
echo base64_encode(""), '|';
echo base64_encode("abc"), '|';
echo strlen(base64_encode("abcdef")), '|';
echo strlen(base64_encode("abc")), "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "4|7|0|false|2|0|2|3|1|3037000499|3037000499|[1,2,3]|\
         {\"a\":\"x\",\"b\":true,\"c\":null}|\"\\u00e9\"|\"a\\/b\"|1.5|[]|\
         {\"a\":\"x\"}|30|5|0|3|3|div0|5050|7|float|\
         d41d8cd98f00b204e9800998ecf8427e|900150983cd24fb0d6963f7d28e17f72|32|\
         900150983cd24fb0d6963f7d28e17f72||YWJj|8|4\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}
