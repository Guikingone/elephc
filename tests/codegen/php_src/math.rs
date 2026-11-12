//! Purpose:
//! php-src `.phpt` conformance tests for math, migrated to Rust: each test compiles the
//! phpt `--FILE--` section and compares stdout with PHP's output.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`codegen::php_src`).
//!
//! Key details:
//! - Generated from php-src (https://github.com/php/php-src); only tests whose output elephc
//!   reproduces byte for byte (with no warnings) are kept. The expected strings are the PHP
//!   interpreter's own output, which matches the phpt `--EXPECT--` section.

use crate::support::*;

/// php-src `ext/standard/tests/math/bug24142.phpt`: Bug #24142 (round() problems)
#[test]
fn phpt_bug24142() {
    let out = compile_and_run(r#"<?php
echo "round(0.005, 2)\n";
var_dump(round(0.005, 2));
echo "\n";

echo "round(0.015, 2)\n";
var_dump(round(0.015, 2));
echo "\n";

echo "round(0.025, 2)\n";
var_dump(round(0.025, 2));
echo "\n";

echo "round(0.035, 2)\n";
var_dump(round(0.035, 2));
echo "\n";

echo "round(0.045, 2)\n";
var_dump(round(0.045, 2));
echo "\n";

echo "round(0.055, 2)\n";
var_dump(round(0.055, 2));
echo "\n";

echo "round(0.065, 2)\n";
var_dump(round(0.065, 2));
echo "\n";

echo "round(0.075, 2)\n";
var_dump(round(0.075, 2));
echo "\n";

echo "round(0.085, 2)\n";
var_dump(round(0.085, 2));
?>
"#);
    assert_eq!(out, "round(0.005, 2)\nfloat(0.01)\n\nround(0.015, 2)\nfloat(0.02)\n\nround(0.025, 2)\nfloat(0.03)\n\nround(0.035, 2)\nfloat(0.04)\n\nround(0.045, 2)\nfloat(0.05)\n\nround(0.055, 2)\nfloat(0.06)\n\nround(0.065, 2)\nfloat(0.07)\n\nround(0.075, 2)\nfloat(0.08)\n\nround(0.085, 2)\nfloat(0.09)\n");
}

/// php-src `ext/standard/tests/math/bug27646.phpt`: Bug #27646 (Cannot serialize/unserialize non-finite numeric values)
#[test]
fn phpt_bug27646() {
    let out = compile_and_run(r#"<?php
set_time_limit(5);

$f=12.3;
var_dump($f);
var_dump(serialize($f));
var_dump(unserialize(serialize($f)));

$f=-12.3;
var_dump($f);
var_dump(serialize($f));
var_dump(unserialize(serialize($f)));

$f=-INF;
var_dump($f);
var_dump(serialize($f));
var_dump(unserialize(serialize($f)));

$f=INF;
var_dump($f);
var_dump(serialize($f));
var_dump(unserialize(serialize($f)));

$f=NAN;
var_dump($f);
var_dump(serialize($f));
var_dump(unserialize(serialize($f)));

?>
"#);
    assert_eq!(out, "float(12.3)\nstring(7) \"d:12.3;\"\nfloat(12.3)\nfloat(-12.3)\nstring(8) \"d:-12.3;\"\nfloat(-12.3)\nfloat(-INF)\nstring(7) \"d:-INF;\"\nfloat(-INF)\nfloat(INF)\nstring(6) \"d:INF;\"\nfloat(INF)\nfloat(NAN)\nstring(6) \"d:NAN;\"\nfloat(NAN)\n");
}

/// php-src `ext/standard/tests/math/bug62112.phpt`: Bug #62112: number_format() is not binary safe
#[test]
fn phpt_bug62112() {
    let out = compile_and_run(r#"<?php
var_dump(number_format(2.5, 2, "\0", "'"));
?>
"#);
    assert_eq!(out, "string(4) \"2\u{0}50\"\n");
}

/// php-src `ext/standard/tests/math/bug74039.phpt`: Bug #74039: is_infinite(-INF) returns false
#[test]
fn phpt_bug74039() {
    let out = compile_and_run(r#"<?php

var_dump(is_finite(INF));
var_dump(is_infinite(INF));
var_dump(is_nan(INF));

var_dump(is_finite(-INF));
var_dump(is_infinite(-INF));
var_dump(is_nan(-INF));

var_dump(is_finite(NAN));
var_dump(is_infinite(NAN));
var_dump(is_nan(NAN));

?>
"#);
    assert_eq!(out, "bool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\n");
}

/// php-src `ext/standard/tests/math/floorceil.phpt`: Tests for floor en ceil
#[test]
fn phpt_floorceil() {
    let out = compile_and_run(r#"<?php
    $a = ceil (-0);   $b = ceil (-1);	$c = ceil (-1.5);
    $d = ceil (-1.8); $e = ceil (-2.7);
    var_dump ($a, $b, $c, $d, $e);

    $a = ceil (0);   $b = ceil (0.5); $c = ceil (1);
    $d = ceil (1.5); $e = ceil (1.8); $f = ceil (2.7);
    var_dump ($a, $b, $c, $d, $e, $f);

    $a = floor (-0);   $b = floor (-0.5); $c = floor (-1);
    $d = floor (-1.5); $e = floor (-1.8); $f = floor (-2.7);
    var_dump ($a, $b, $c, $d, $e, $f);

    $a = floor (0);   $b = floor (0.5); $c = floor (1);
    $d = floor (1.5); $e = floor (1.8); $f = floor (2.7);
    var_dump ($a, $b, $c, $d, $e, $f);
?>
"#);
    assert_eq!(out, "float(0)\nfloat(-1)\nfloat(-1)\nfloat(-1)\nfloat(-2)\nfloat(0)\nfloat(1)\nfloat(1)\nfloat(2)\nfloat(2)\nfloat(3)\nfloat(0)\nfloat(-1)\nfloat(-1)\nfloat(-2)\nfloat(-2)\nfloat(-3)\nfloat(0)\nfloat(0)\nfloat(1)\nfloat(1)\nfloat(1)\nfloat(2)\n");
}

/// php-src `ext/standard/tests/math/gh22410.phpt`: gh-22410: round() preserves integral doubles
#[test]
fn phpt_gh22410() {
    let out = compile_and_run(r#"<?php
var_dump(round(1.0));
var_dump(round(-1.0));
var_dump(round((float) 9003241321073828));
var_dump(round((float) -9003241321073828));
?>
"#);
    assert_eq!(out, "float(1)\nfloat(-1)\nfloat(9003241321073828)\nfloat(-9003241321073828)\n");
}

/// php-src `ext/standard/tests/math/hexdec_variation2.phpt`: Test hexdec() function : strange literals
#[test]
fn phpt_hexdec_variation2() {
    let out = compile_and_run(r#"<?php

var_dump(hexdec('0x'));
var_dump(hexdec('0X'));
var_dump(hexdec(''));

?>
"#);
    assert_eq!(out, "int(0)\nint(0)\nint(0)\n");
}

/// php-src `ext/standard/tests/math/log.phpt`: log() tests
#[test]
fn phpt_log() {
    let out = compile_and_run(r#"<?php
echo "On failure, please mail result to php-dev@lists.php.net\n";
for ($x = 0, $count= 0; $x < 200; $x++) {
    $x2 = (int) exp(log($x));
    // e ^ log(x) should be close in range to x
    if (($x2 < ($x + 2)) && ($x2 > ($x - 2))) {
        $count++;
    } else {
        print "$x : $x2\n";
    }
}
print $count . "\n";

// Now test the base form of log
for ($base = 2; $base < 11; $base++) {
    for ($x = 0, $count= 0; $x < 50; $x++) {
        $x2 = (int) pow($base, log($x, $base));
        // base ^ log(x) should be close in range to x
        if (($x2 < ($x + 2)) && ($x2 > ($x - 2))) {
            $count++;
        } else {
             print "base $base: $x : $x2\n";
        }
    }
    print $count . "\n";
}
?>
"#);
    assert_eq!(out, "On failure, please mail result to php-dev@lists.php.net\n200\n50\n50\n50\n50\n50\n50\n50\n50\n50\n");
}

/// php-src `ext/standard/tests/math/number_format_with_3_args.phpt`: number_format should use default thousands seperator when 3 arguments are used
#[test]
fn phpt_number_format_with_3_args() {
    let out = compile_and_run(r#"<?php

$number = 2020.1415;

var_dump(number_format($number, 2, 'F'));

?>
"#);
    assert_eq!(out, "string(8) \"2,020F14\"\n");
}

/// php-src `ext/standard/tests/math/octdec_variation2.phpt`: Test octdec() function : strange literals
#[test]
fn phpt_octdec_variation2() {
    let out = compile_and_run(r#"<?php

var_dump(octdec('0o'));
var_dump(octdec('0O'));
var_dump(octdec('0'));
var_dump(octdec(''));

?>
"#);
    assert_eq!(out, "int(0)\nint(0)\nint(0)\nint(0)\n");
}

/// php-src `ext/standard/tests/math/round.phpt`: Simple math tests
#[test]
fn phpt_round() {
    let out = compile_and_run(r#"<?php

define('LONG_MAX', is_int(5000000000)? 9223372036854775807 : 0x7FFFFFFF);
define('LONG_MIN', -LONG_MAX - 1);
printf("%d,%d,%d,%d\n",is_int(LONG_MIN  ),is_int(LONG_MAX  ),
                       is_int(LONG_MIN-1),is_int(LONG_MAX+1));

function epsilon_equal($left, $right): bool {
    return abs(($left-$right) / $left) < 1e-12;
}

var_dump(epsilon_equal( -1 , ceil(-1.5) ));
var_dump(epsilon_equal( 2 , ceil( 1.5) ));
var_dump(epsilon_equal( -2 , floor(-1.5) ));
var_dump(epsilon_equal( 1 , floor(1.5) ));
var_dump(epsilon_equal( LONG_MIN   , ceil(LONG_MIN - 0.5) ));
var_dump(epsilon_equal( LONG_MIN+1 , ceil(LONG_MIN + 0.5) ));
var_dump(epsilon_equal( LONG_MIN-1 , round(LONG_MIN - 0.6) ));
var_dump(epsilon_equal( LONG_MIN   , round(LONG_MIN - 0.4) ));
var_dump(epsilon_equal( LONG_MIN   , round(LONG_MIN + 0.4) ));
var_dump(epsilon_equal( LONG_MIN+1 , round(LONG_MIN + 0.6) ));
var_dump(epsilon_equal( LONG_MIN-1 , floor(LONG_MIN - 0.5) ));
var_dump(epsilon_equal( LONG_MIN   , floor(LONG_MIN + 0.5) ));
var_dump(epsilon_equal( LONG_MAX   , ceil(LONG_MAX - 0.5) ));
var_dump(epsilon_equal( LONG_MAX+1 , ceil(LONG_MAX + 0.5) ));
var_dump(epsilon_equal( LONG_MAX-1 , round(LONG_MAX - 0.6) ));
var_dump(epsilon_equal( LONG_MAX   , round(LONG_MAX - 0.4) ));
var_dump(epsilon_equal( LONG_MAX   , round(LONG_MAX + 0.4) ));
var_dump(epsilon_equal( LONG_MAX+1 , round(LONG_MAX + 0.6) ));
var_dump(epsilon_equal( LONG_MAX-1 , floor(LONG_MAX - 0.5) ));
var_dump(epsilon_equal( LONG_MAX   , floor(LONG_MAX + 0.5) ));

?>
"#);
    assert_eq!(out, "1,1,0,0\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n");
}
