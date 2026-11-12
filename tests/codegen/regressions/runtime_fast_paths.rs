//! Purpose:
//! Regression tests for runtime fast paths: integer heapsort, shortest float formatting,
//! json_encode string escaping, temporary release in the array_replace family, and the
//! size-class heap bins.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use super::*;

/// `sort()`/`rsort()`/`asort()`/`natsort()` on integer lists (heapsort) agree with PHP for empty, tiny and larger inputs, including the integer extremes.
#[test]
fn test_sort_and_rsort_int_lists_of_every_small_size() {
    let out = compile_and_run(r#"<?php
$seed = 5;
foreach ([0, 1, 2, 3, 7, 64, 1000] as $n) {
  $a = []; for ($i = 0; $i < $n; $i++) { $seed = ($seed * 1103515245 + 12345) % 2147483648; $a[] = ($seed % 101) - 50; }
  $b = $a; $c = $a; $d = $a;
  sort($b); rsort($c); asort($d);
  echo $n, ":", implode(",", array_slice($b, 0, 12)), "|", implode(",", array_slice($c, 0, 12)), "|", implode(",", $b) === implode(",", array_reverse($c)) ? "y" : "n", "\n";
}
$e = [PHP_INT_MAX, PHP_INT_MIN, 0, -1, 1]; sort($e); echo implode(",", $e), "\n";
$f = [3, 1, 2]; natsort($f); echo implode(",", $f), "\n";
$g = [5, 3, 9, 1]; rsort($g); echo implode(",", $g), "\n";
"#);
    assert_eq!(out, "0:||y\n1:-37|-37|y\n2:7,26|26,7|y\n3:-47,-9,-3|-3,-9,-47|y\n7:-48,-45,-42,-32,-13,3,43|43,3,-13,-32,-42,-45,-48|y\n64:-50,-47,-47,-45,-43,-43,-42,-42,-42,-40,-39,-39|50,50,48,46,46,45,45,45,43,41,40,39|y\n1000:-50,-50,-50,-50,-50,-49,-49,-49,-49,-49,-49,-49|50,50,50,50,50,50,50,49,49,49,49,49|y\n-9223372036854775808,-1,0,1,9223372036854775807\n1,2,3\n9,5,3,1\n");
}

/// Floats with a short decimal form take the integer fast path, others (tiny, huge, 17-digit) the general probe; all match PHP's shortest round-trip formatting in json_encode, serialize and var_dump.
#[test]
fn test_json_float_shortest_decimals_and_fallbacks() {
    let out = compile_and_run(r#"<?php
$vals = [0.1, 0.25, 123.25, -7.5, 1.0, 100.0, 1200.0, 0.0001, 0.00012, 0.00009, 1e14, 9.99e14, 1e15, 1e16, 1e-5, 0.3, 0.1+0.2, 1/3, 2/3, -0.0, 0.0, 3.14159, 2.718281828, 1234567.891, 0.000123456, 99999999.99, 5e-324, 1.7976931348623157e308, 123456789012345.6, 0.5, 33.333333, 1e10+0.5, -123.456, 4.35, 0.07, 1.005, 2.675];
echo json_encode($vals), "\n";
echo serialize([1.5, 0.1, -2.25, 1e20]), "\n";
var_dump(0.1, 7.0, -1.25);
foreach ($vals as $v) { echo json_encode($v), " "; } echo "\n";
echo json_encode([1.0, 2.50], JSON_PRESERVE_ZERO_FRACTION), "\n";
"#);
    assert_eq!(out, "[0.1,0.25,123.25,-7.5,1,100,1200,0.0001,0.00012,9.0e-5,100000000000000,999000000000000,1000000000000000,10000000000000000,1.0e-5,0.3,0.30000000000000004,0.3333333333333333,0.6666666666666666,-0,0,3.14159,2.718281828,1234567.891,0.000123456,99999999.99,5.0e-324,1.7976931348623157e+308,123456789012345.6,0.5,33.333333,10000000000.5,-123.456,4.35,0.07,1.005,2.675]\na:4:{i:0;d:1.5;i:1;d:0.1;i:2;d:-2.25;i:3;d:1.0E+20;}\nfloat(0.1)\nfloat(7)\nfloat(-1.25)\n0.1 0.25 123.25 -7.5 1 100 1200 0.0001 0.00012 9.0e-5 100000000000000 999000000000000 1000000000000000 10000000000000000 1.0e-5 0.3 0.30000000000000004 0.3333333333333333 0.6666666666666666 -0 0 3.14159 2.718281828 1234567.891 0.000123456 99999999.99 5.0e-324 1.7976931348623157e+308 123456789012345.6 0.5 33.333333 10000000000.5 -123.456 4.35 0.07 1.005 2.675 \n[1.0,2.5]\n");
}

/// Plain runs, escapes, HEX_* flags and non-ASCII code points interleave correctly in json_encode; code-point escapes use lowercase hex like PHP.
#[test]
fn test_json_string_escaping_mixes_fast_runs_and_escapes() {
    let out = compile_and_run(r#"<?php
$s = ["plain ascii text", "quote\" and \\ backslash", "slash / tab\t nl\n", "<tag> & 'apos'", "caf\u{e9} \u{1F600}", str_repeat("abcdefgh", 40) . "\"end"];
echo json_encode($s), "\n";
echo json_encode($s, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), "\n";
echo json_encode($s, JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT), "\n";
echo json_encode(["k\"ey" => "v\x01al"]), "\n";
"#);
    assert_eq!(out, "[\"plain ascii text\",\"quote\\\" and \\\\ backslash\",\"slash \\/ tab\\t nl\\n\",\"<tag> & 'apos'\",\"caf\\u00e9 \\ud83d\\ude00\",\"abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh\\\"end\"]\n[\"plain ascii text\",\"quote\\\" and \\\\ backslash\",\"slash / tab\\t nl\\n\",\"<tag> & 'apos'\",\"café 😀\",\"abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh\\\"end\"]\n[\"plain ascii text\",\"quote\\u0022 and \\\\ backslash\",\"slash \\/ tab\\t nl\\n\",\"\\u003Ctag\\u003E \\u0026 \\u0027apos\\u0027\",\"caf\\u00e9 \\ud83d\\ude00\",\"abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh\\u0022end\"]\n{\"k\\\"ey\":\"v\\u0001al\"}\n");
}

/// array_replace(_recursive)/array_merge_recursive on list inputs release the converted temporary, not their result (which used to be freed and then overwritten).
#[test]
fn test_array_replace_family_releases_the_temporary_not_the_result() {
    let out = compile_and_run(r#"<?php
$r = array_replace([10, 20, 30], [1 => 99]);
foreach ($r as $k => $v) { echo "$k:$v "; }
$x = [];
for ($i = 0; $i < 50; $i++) { $x[] = str_repeat("z", $i); }
echo "\n";
$r = array_replace_recursive([10, 20, 30], [1 => 99]);
foreach ($r as $k => $v) { echo "$k:$v "; }
echo "\n";
$r = array_merge_recursive([10, 20], [30]);
foreach ($r as $k => $v) { echo "$k:$v "; }
echo "\n";
print_r(array_reverse([1, "a", 2.5]));
"#);
    assert_eq!(out, "0:10 1:99 2:30 \n0:10 1:99 2:30 \n0:10 1:20 2:30 \nArray\n(\n    [0] => 2.5\n    [1] => a\n    [2] => 1\n)\n");
}

/// Strings and hashes of many sizes allocated and freed in rounds keep their contents intact while the size-class bins recycle blocks.
#[test]
fn test_mixed_size_allocation_churn_reuses_blocks() {
    let out = compile_and_run(r#"<?php
// Mixed-size allocations and frees: strings of many lengths, hashes of several capacities.
$keep = [];
for ($r = 0; $r < 50; $r++) {
    $tmp = [];
    for ($i = 0; $i < 200; $i++) {
        $len = ($i * 37 + $r * 11) % 1500;
        $tmp[] = str_repeat(chr(65 + $i % 26), $len);
        if ($i % 17 === 0) { $h = []; for ($k = 0; $k < ($i % 40); $k++) { $h["k$k"] = $k * $r; } $keep[$r % 7] = $h; }
    }
    $total = 0; foreach ($tmp as $t) { $total += strlen($t); }
    echo $total, " ";
}
echo "\n", count($keep), " ", count($keep[3]), "\n";
"#);
    assert_eq!(out, "148300 147500 148200 148900 148100 150300 149500 150200 150900 150100 152300 151500 152200 151400 150600 152800 150500 152700 151900 151100 151800 151000 151700 150900 151600 152300 150000 152200 151400 150600 151300 150500 152700 150400 151100 151800 149500 151700 150900 151600 150800 150000 152200 149900 150600 151300 150500 151200 150400 151100 \n7 27\n");
}
