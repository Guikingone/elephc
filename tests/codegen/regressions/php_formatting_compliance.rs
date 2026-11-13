//! Purpose:
//! PHP-compliance regressions for formatted output and numeric conversions: the
//! `sprintf` family (native fast path and the elephc-PHP prelude), `number_format()`,
//! `(int)` of out-of-range floats, the base-conversion helpers, and the
//! `DivisionByZeroError` / `ArgumentCountError` classes.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use super::*;

/// Dynamic `sprintf` formats run through the elephc-PHP formatter (`printf_prelude`): every conversion, flag, width and precision combination below matches PHP, including `%b`/`%u`/`%e`/`%g`, custom padding, PHP's `INF`/`NaN` spellings, and `(int)`/`(float)`/`(string)` argument conversion.
#[test]
fn test_sprintf_dynamic_formats_match_php() {
    let out = compile_and_run(r#"<?php
$fmts=['%d','%05d','%+d','%-5d|','%x','%X','%o','%b','%u','%s','%10s|','%-10s|','%.2s','%\'*10s','%f','%.2f','%10.3f|','%-10.2f|','%010.2f','%+.1f','%e','%.2e','%E','%g','%G','%.3g','%F'];
$vals=[0,-1,42,255,3.14159,-2.5,1e10,-1e-5,"abc","12abc",true,null,PHP_INT_MIN,1.5e300,NAN,-INF];
foreach ($fmts as $f) { echo $f, ':'; foreach ($vals as $v) { echo ' [', sprintf($f, $v), ']'; } echo "\n"; }
"#);
    assert_eq!(out, "%d: [0] [-1] [42] [255] [3] [-2] [10000000000] [0] [0] [12] [1] [0] [-9223372036854775808] [0] [0] [0]\n%05d: [00000] [-0001] [00042] [00255] [00003] [-0002] [10000000000] [00000] [00000] [00012] [00001] [00000] [-9223372036854775808] [00000] [00000] [00000]\n%+d: [+0] [-1] [+42] [+255] [+3] [-2] [+10000000000] [+0] [+0] [+12] [+1] [+0] [-9223372036854775808] [+0] [+0] [+0]\n%-5d|: [0    |] [-1   |] [42   |] [255  |] [3    |] [-2   |] [10000000000|] [0    |] [0    |] [12   |] [1    |] [0    |] [-9223372036854775808|] [0    |] [0    |] [0    |]\n%x: [0] [ffffffffffffffff] [2a] [ff] [3] [fffffffffffffffe] [2540be400] [0] [0] [c] [1] [0] [8000000000000000] [0] [0] [0]\n%X: [0] [FFFFFFFFFFFFFFFF] [2A] [FF] [3] [FFFFFFFFFFFFFFFE] [2540BE400] [0] [0] [C] [1] [0] [8000000000000000] [0] [0] [0]\n%o: [0] [1777777777777777777777] [52] [377] [3] [1777777777777777777776] [112402762000] [0] [0] [14] [1] [0] [1000000000000000000000] [0] [0] [0]\n%b: [0] [1111111111111111111111111111111111111111111111111111111111111111] [101010] [11111111] [11] [1111111111111111111111111111111111111111111111111111111111111110] [1001010100000010111110010000000000] [0] [0] [1100] [1] [0] [1000000000000000000000000000000000000000000000000000000000000000] [0] [0] [0]\n%u: [0] [18446744073709551615] [42] [255] [3] [18446744073709551614] [10000000000] [0] [0] [12] [1] [0] [9223372036854775808] [0] [0] [0]\n%s: [0] [-1] [42] [255] [3.14159] [-2.5] [10000000000] [-1.0E-5] [abc] [12abc] [1] [] [-9223372036854775808] [1.5E+300] [NAN] [-INF]\n%10s|: [         0|] [        -1|] [        42|] [       255|] [   3.14159|] [      -2.5|] [10000000000|] [   -1.0E-5|] [       abc|] [     12abc|] [         1|] [          |] [-9223372036854775808|] [  1.5E+300|] [       NAN|] [      -INF|]\n%-10s|: [0         |] [-1        |] [42        |] [255       |] [3.14159   |] [-2.5      |] [10000000000|] [-1.0E-5   |] [abc       |] [12abc     |] [1         |] [          |] [-9223372036854775808|] [1.5E+300  |] [NAN       |] [-INF      |]\n%.2s: [0] [-1] [42] [25] [3.] [-2] [10] [-1] [ab] [12] [1] [] [-9] [1.] [NA] [-I]\n%'*10s: [*********0] [********-1] [********42] [*******255] [***3.14159] [******-2.5] [10000000000] [***-1.0E-5] [*******abc] [*****12abc] [*********1] [**********] [-9223372036854775808] [**1.5E+300] [*******NAN] [******-INF]\n%f: [0.000000] [-1.000000] [42.000000] [255.000000] [3.141590] [-2.500000] [10000000000.000000] [-0.000010] [0.000000] [12.000000] [1.000000] [0.000000] [-9223372036854775808.000000] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000000] [NaN] [INF]\n%.2f: [0.00] [-1.00] [42.00] [255.00] [3.14] [-2.50] [10000000000.00] [-0.00] [0.00] [12.00] [1.00] [0.00] [-9223372036854775808.00] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00] [NaN] [INF]\n%10.3f|: [     0.000|] [    -1.000|] [    42.000|] [   255.000|] [     3.142|] [    -2.500|] [10000000000.000|] [    -0.000|] [     0.000|] [    12.000|] [     1.000|] [     0.000|] [-9223372036854775808.000|] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000|] [NaN|] [INF|]\n%-10.2f|: [0.00      |] [-1.00     |] [42.00     |] [255.00    |] [3.14      |] [-2.50     |] [10000000000.00|] [-0.00     |] [0.00      |] [12.00     |] [1.00      |] [0.00      |] [-9223372036854775808.00|] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00|] [NaN|] [INF|]\n%010.2f: [0000000.00] [-000001.00] [0000042.00] [0000255.00] [0000003.14] [-000002.50] [10000000000.00] [-000000.00] [0000000.00] [0000012.00] [0000001.00] [0000000.00] [-9223372036854775808.00] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00] [NaN] [-NF]\n%+.1f: [+0.0] [-1.0] [+42.0] [+255.0] [+3.1] [-2.5] [+10000000000.0] [-0.0] [+0.0] [+12.0] [+1.0] [+0.0] [-9223372036854775808.0] [+1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.0] [NaN] [INF]\n%e: [0.000000e+0] [-1.000000e+0] [4.200000e+1] [2.550000e+2] [3.141590e+0] [-2.500000e+0] [1.000000e+10] [-1.000000e-5] [0.000000e+0] [1.200000e+1] [1.000000e+0] [0.000000e+0] [-9.223372e+18] [1.500000e+300] [NaN] [INF]\n%.2e: [0.00e+0] [-1.00e+0] [4.20e+1] [2.55e+2] [3.14e+0] [-2.50e+0] [1.00e+10] [-1.00e-5] [0.00e+0] [1.20e+1] [1.00e+0] [0.00e+0] [-9.22e+18] [1.50e+300] [NaN] [INF]\n%E: [0.000000E+0] [-1.000000E+0] [4.200000E+1] [2.550000E+2] [3.141590E+0] [-2.500000E+0] [1.000000E+10] [-1.000000E-5] [0.000000E+0] [1.200000E+1] [1.000000E+0] [0.000000E+0] [-9.223372E+18] [1.500000E+300] [NaN] [INF]\n%g: [0] [-1] [42] [255] [3.14159] [-2.5] [1.0e+10] [-1.0e-5] [0] [12] [1] [0] [-9.22337e+18] [1.5e+300] [NaN] [INF]\n%G: [0] [-1] [42] [255] [3.14159] [-2.5] [1.0E+10] [-1.0E-5] [0] [12] [1] [0] [-9.22337E+18] [1.5E+300] [NaN] [INF]\n%.3g: [0] [-1] [42] [255] [3.14] [-2.5] [1.0e+10] [-1.0e-5] [0] [12] [1] [0] [-9.22e+18] [1.5e+300] [NaN] [INF]\n%F: [0.000000] [-1.000000] [42.000000] [255.000000] [3.141590] [-2.500000] [10000000000.000000] [-0.000010] [0.000000] [12.000000] [1.000000] [0.000000] [-9223372036854775808.000000] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000000] [NaN] [INF]\n");
}

/// Literal `sprintf` formats: the native fast path (`%d %s %f %x %X %o` with `-`/`+`/`0`, width, precision; 64-bit `ll` conversions) and the prelude route produce PHP's exact output.
#[test]
fn test_sprintf_literal_formats_match_php() {
    let out = compile_and_run(r#"<?php
echo "%d:", " [", sprintf("%d", 0), "]", " [", sprintf("%d", -1), "]", " [", sprintf("%d", 42), "]", " [", sprintf("%d", 255), "]", " [", sprintf("%d", 3.14159), "]", " [", sprintf("%d", -2.5), "]", " [", sprintf("%d", 1e10), "]", " [", sprintf("%d", -1e-5), "]", " [", sprintf("%d", "abc"), "]", " [", sprintf("%d", "12abc"), "]", " [", sprintf("%d", true), "]", " [", sprintf("%d", null), "]", " [", sprintf("%d", PHP_INT_MIN), "]", " [", sprintf("%d", 1.5e300), "]", " [", sprintf("%d", NAN), "]", " [", sprintf("%d", -INF), "]", "\n";
echo "%05d:", " [", sprintf("%05d", 0), "]", " [", sprintf("%05d", -1), "]", " [", sprintf("%05d", 42), "]", " [", sprintf("%05d", 255), "]", " [", sprintf("%05d", 3.14159), "]", " [", sprintf("%05d", -2.5), "]", " [", sprintf("%05d", 1e10), "]", " [", sprintf("%05d", -1e-5), "]", " [", sprintf("%05d", "abc"), "]", " [", sprintf("%05d", "12abc"), "]", " [", sprintf("%05d", true), "]", " [", sprintf("%05d", null), "]", " [", sprintf("%05d", PHP_INT_MIN), "]", " [", sprintf("%05d", 1.5e300), "]", " [", sprintf("%05d", NAN), "]", " [", sprintf("%05d", -INF), "]", "\n";
echo "%+d:", " [", sprintf("%+d", 0), "]", " [", sprintf("%+d", -1), "]", " [", sprintf("%+d", 42), "]", " [", sprintf("%+d", 255), "]", " [", sprintf("%+d", 3.14159), "]", " [", sprintf("%+d", -2.5), "]", " [", sprintf("%+d", 1e10), "]", " [", sprintf("%+d", -1e-5), "]", " [", sprintf("%+d", "abc"), "]", " [", sprintf("%+d", "12abc"), "]", " [", sprintf("%+d", true), "]", " [", sprintf("%+d", null), "]", " [", sprintf("%+d", PHP_INT_MIN), "]", " [", sprintf("%+d", 1.5e300), "]", " [", sprintf("%+d", NAN), "]", " [", sprintf("%+d", -INF), "]", "\n";
echo "%-5d|:", " [", sprintf("%-5d|", 0), "]", " [", sprintf("%-5d|", -1), "]", " [", sprintf("%-5d|", 42), "]", " [", sprintf("%-5d|", 255), "]", " [", sprintf("%-5d|", 3.14159), "]", " [", sprintf("%-5d|", -2.5), "]", " [", sprintf("%-5d|", 1e10), "]", " [", sprintf("%-5d|", -1e-5), "]", " [", sprintf("%-5d|", "abc"), "]", " [", sprintf("%-5d|", "12abc"), "]", " [", sprintf("%-5d|", true), "]", " [", sprintf("%-5d|", null), "]", " [", sprintf("%-5d|", PHP_INT_MIN), "]", " [", sprintf("%-5d|", 1.5e300), "]", " [", sprintf("%-5d|", NAN), "]", " [", sprintf("%-5d|", -INF), "]", "\n";
echo "%x:", " [", sprintf("%x", 0), "]", " [", sprintf("%x", -1), "]", " [", sprintf("%x", 42), "]", " [", sprintf("%x", 255), "]", " [", sprintf("%x", 3.14159), "]", " [", sprintf("%x", -2.5), "]", " [", sprintf("%x", 1e10), "]", " [", sprintf("%x", -1e-5), "]", " [", sprintf("%x", "abc"), "]", " [", sprintf("%x", "12abc"), "]", " [", sprintf("%x", true), "]", " [", sprintf("%x", null), "]", " [", sprintf("%x", PHP_INT_MIN), "]", " [", sprintf("%x", 1.5e300), "]", " [", sprintf("%x", NAN), "]", " [", sprintf("%x", -INF), "]", "\n";
echo "%X:", " [", sprintf("%X", 0), "]", " [", sprintf("%X", -1), "]", " [", sprintf("%X", 42), "]", " [", sprintf("%X", 255), "]", " [", sprintf("%X", 3.14159), "]", " [", sprintf("%X", -2.5), "]", " [", sprintf("%X", 1e10), "]", " [", sprintf("%X", -1e-5), "]", " [", sprintf("%X", "abc"), "]", " [", sprintf("%X", "12abc"), "]", " [", sprintf("%X", true), "]", " [", sprintf("%X", null), "]", " [", sprintf("%X", PHP_INT_MIN), "]", " [", sprintf("%X", 1.5e300), "]", " [", sprintf("%X", NAN), "]", " [", sprintf("%X", -INF), "]", "\n";
echo "%o:", " [", sprintf("%o", 0), "]", " [", sprintf("%o", -1), "]", " [", sprintf("%o", 42), "]", " [", sprintf("%o", 255), "]", " [", sprintf("%o", 3.14159), "]", " [", sprintf("%o", -2.5), "]", " [", sprintf("%o", 1e10), "]", " [", sprintf("%o", -1e-5), "]", " [", sprintf("%o", "abc"), "]", " [", sprintf("%o", "12abc"), "]", " [", sprintf("%o", true), "]", " [", sprintf("%o", null), "]", " [", sprintf("%o", PHP_INT_MIN), "]", " [", sprintf("%o", 1.5e300), "]", " [", sprintf("%o", NAN), "]", " [", sprintf("%o", -INF), "]", "\n";
echo "%b:", " [", sprintf("%b", 0), "]", " [", sprintf("%b", -1), "]", " [", sprintf("%b", 42), "]", " [", sprintf("%b", 255), "]", " [", sprintf("%b", 3.14159), "]", " [", sprintf("%b", -2.5), "]", " [", sprintf("%b", 1e10), "]", " [", sprintf("%b", -1e-5), "]", " [", sprintf("%b", "abc"), "]", " [", sprintf("%b", "12abc"), "]", " [", sprintf("%b", true), "]", " [", sprintf("%b", null), "]", " [", sprintf("%b", PHP_INT_MIN), "]", " [", sprintf("%b", 1.5e300), "]", " [", sprintf("%b", NAN), "]", " [", sprintf("%b", -INF), "]", "\n";
echo "%u:", " [", sprintf("%u", 0), "]", " [", sprintf("%u", -1), "]", " [", sprintf("%u", 42), "]", " [", sprintf("%u", 255), "]", " [", sprintf("%u", 3.14159), "]", " [", sprintf("%u", -2.5), "]", " [", sprintf("%u", 1e10), "]", " [", sprintf("%u", -1e-5), "]", " [", sprintf("%u", "abc"), "]", " [", sprintf("%u", "12abc"), "]", " [", sprintf("%u", true), "]", " [", sprintf("%u", null), "]", " [", sprintf("%u", PHP_INT_MIN), "]", " [", sprintf("%u", 1.5e300), "]", " [", sprintf("%u", NAN), "]", " [", sprintf("%u", -INF), "]", "\n";
echo "%s:", " [", sprintf("%s", 0), "]", " [", sprintf("%s", -1), "]", " [", sprintf("%s", 42), "]", " [", sprintf("%s", 255), "]", " [", sprintf("%s", 3.14159), "]", " [", sprintf("%s", -2.5), "]", " [", sprintf("%s", 1e10), "]", " [", sprintf("%s", -1e-5), "]", " [", sprintf("%s", "abc"), "]", " [", sprintf("%s", "12abc"), "]", " [", sprintf("%s", true), "]", " [", sprintf("%s", null), "]", " [", sprintf("%s", PHP_INT_MIN), "]", " [", sprintf("%s", 1.5e300), "]", " [", sprintf("%s", NAN), "]", " [", sprintf("%s", -INF), "]", "\n";
echo "%10s|:", " [", sprintf("%10s|", 0), "]", " [", sprintf("%10s|", -1), "]", " [", sprintf("%10s|", 42), "]", " [", sprintf("%10s|", 255), "]", " [", sprintf("%10s|", 3.14159), "]", " [", sprintf("%10s|", -2.5), "]", " [", sprintf("%10s|", 1e10), "]", " [", sprintf("%10s|", -1e-5), "]", " [", sprintf("%10s|", "abc"), "]", " [", sprintf("%10s|", "12abc"), "]", " [", sprintf("%10s|", true), "]", " [", sprintf("%10s|", null), "]", " [", sprintf("%10s|", PHP_INT_MIN), "]", " [", sprintf("%10s|", 1.5e300), "]", " [", sprintf("%10s|", NAN), "]", " [", sprintf("%10s|", -INF), "]", "\n";
echo "%-10s|:", " [", sprintf("%-10s|", 0), "]", " [", sprintf("%-10s|", -1), "]", " [", sprintf("%-10s|", 42), "]", " [", sprintf("%-10s|", 255), "]", " [", sprintf("%-10s|", 3.14159), "]", " [", sprintf("%-10s|", -2.5), "]", " [", sprintf("%-10s|", 1e10), "]", " [", sprintf("%-10s|", -1e-5), "]", " [", sprintf("%-10s|", "abc"), "]", " [", sprintf("%-10s|", "12abc"), "]", " [", sprintf("%-10s|", true), "]", " [", sprintf("%-10s|", null), "]", " [", sprintf("%-10s|", PHP_INT_MIN), "]", " [", sprintf("%-10s|", 1.5e300), "]", " [", sprintf("%-10s|", NAN), "]", " [", sprintf("%-10s|", -INF), "]", "\n";
echo "%.2s:", " [", sprintf("%.2s", 0), "]", " [", sprintf("%.2s", -1), "]", " [", sprintf("%.2s", 42), "]", " [", sprintf("%.2s", 255), "]", " [", sprintf("%.2s", 3.14159), "]", " [", sprintf("%.2s", -2.5), "]", " [", sprintf("%.2s", 1e10), "]", " [", sprintf("%.2s", -1e-5), "]", " [", sprintf("%.2s", "abc"), "]", " [", sprintf("%.2s", "12abc"), "]", " [", sprintf("%.2s", true), "]", " [", sprintf("%.2s", null), "]", " [", sprintf("%.2s", PHP_INT_MIN), "]", " [", sprintf("%.2s", 1.5e300), "]", " [", sprintf("%.2s", NAN), "]", " [", sprintf("%.2s", -INF), "]", "\n";
echo "%'*10s:", " [", sprintf("%'*10s", 0), "]", " [", sprintf("%'*10s", -1), "]", " [", sprintf("%'*10s", 42), "]", " [", sprintf("%'*10s", 255), "]", " [", sprintf("%'*10s", 3.14159), "]", " [", sprintf("%'*10s", -2.5), "]", " [", sprintf("%'*10s", 1e10), "]", " [", sprintf("%'*10s", -1e-5), "]", " [", sprintf("%'*10s", "abc"), "]", " [", sprintf("%'*10s", "12abc"), "]", " [", sprintf("%'*10s", true), "]", " [", sprintf("%'*10s", null), "]", " [", sprintf("%'*10s", PHP_INT_MIN), "]", " [", sprintf("%'*10s", 1.5e300), "]", " [", sprintf("%'*10s", NAN), "]", " [", sprintf("%'*10s", -INF), "]", "\n";
echo "%f:", " [", sprintf("%f", 0), "]", " [", sprintf("%f", -1), "]", " [", sprintf("%f", 42), "]", " [", sprintf("%f", 255), "]", " [", sprintf("%f", 3.14159), "]", " [", sprintf("%f", -2.5), "]", " [", sprintf("%f", 1e10), "]", " [", sprintf("%f", -1e-5), "]", " [", sprintf("%f", "abc"), "]", " [", sprintf("%f", "12abc"), "]", " [", sprintf("%f", true), "]", " [", sprintf("%f", null), "]", " [", sprintf("%f", PHP_INT_MIN), "]", " [", sprintf("%f", 1.5e300), "]", " [", sprintf("%f", NAN), "]", " [", sprintf("%f", -INF), "]", "\n";
echo "%.2f:", " [", sprintf("%.2f", 0), "]", " [", sprintf("%.2f", -1), "]", " [", sprintf("%.2f", 42), "]", " [", sprintf("%.2f", 255), "]", " [", sprintf("%.2f", 3.14159), "]", " [", sprintf("%.2f", -2.5), "]", " [", sprintf("%.2f", 1e10), "]", " [", sprintf("%.2f", -1e-5), "]", " [", sprintf("%.2f", "abc"), "]", " [", sprintf("%.2f", "12abc"), "]", " [", sprintf("%.2f", true), "]", " [", sprintf("%.2f", null), "]", " [", sprintf("%.2f", PHP_INT_MIN), "]", " [", sprintf("%.2f", 1.5e300), "]", " [", sprintf("%.2f", NAN), "]", " [", sprintf("%.2f", -INF), "]", "\n";
echo "%10.3f|:", " [", sprintf("%10.3f|", 0), "]", " [", sprintf("%10.3f|", -1), "]", " [", sprintf("%10.3f|", 42), "]", " [", sprintf("%10.3f|", 255), "]", " [", sprintf("%10.3f|", 3.14159), "]", " [", sprintf("%10.3f|", -2.5), "]", " [", sprintf("%10.3f|", 1e10), "]", " [", sprintf("%10.3f|", -1e-5), "]", " [", sprintf("%10.3f|", "abc"), "]", " [", sprintf("%10.3f|", "12abc"), "]", " [", sprintf("%10.3f|", true), "]", " [", sprintf("%10.3f|", null), "]", " [", sprintf("%10.3f|", PHP_INT_MIN), "]", " [", sprintf("%10.3f|", 1.5e300), "]", " [", sprintf("%10.3f|", NAN), "]", " [", sprintf("%10.3f|", -INF), "]", "\n";
echo "%-10.2f|:", " [", sprintf("%-10.2f|", 0), "]", " [", sprintf("%-10.2f|", -1), "]", " [", sprintf("%-10.2f|", 42), "]", " [", sprintf("%-10.2f|", 255), "]", " [", sprintf("%-10.2f|", 3.14159), "]", " [", sprintf("%-10.2f|", -2.5), "]", " [", sprintf("%-10.2f|", 1e10), "]", " [", sprintf("%-10.2f|", -1e-5), "]", " [", sprintf("%-10.2f|", "abc"), "]", " [", sprintf("%-10.2f|", "12abc"), "]", " [", sprintf("%-10.2f|", true), "]", " [", sprintf("%-10.2f|", null), "]", " [", sprintf("%-10.2f|", PHP_INT_MIN), "]", " [", sprintf("%-10.2f|", 1.5e300), "]", " [", sprintf("%-10.2f|", NAN), "]", " [", sprintf("%-10.2f|", -INF), "]", "\n";
echo "%010.2f:", " [", sprintf("%010.2f", 0), "]", " [", sprintf("%010.2f", -1), "]", " [", sprintf("%010.2f", 42), "]", " [", sprintf("%010.2f", 255), "]", " [", sprintf("%010.2f", 3.14159), "]", " [", sprintf("%010.2f", -2.5), "]", " [", sprintf("%010.2f", 1e10), "]", " [", sprintf("%010.2f", -1e-5), "]", " [", sprintf("%010.2f", "abc"), "]", " [", sprintf("%010.2f", "12abc"), "]", " [", sprintf("%010.2f", true), "]", " [", sprintf("%010.2f", null), "]", " [", sprintf("%010.2f", PHP_INT_MIN), "]", " [", sprintf("%010.2f", 1.5e300), "]", " [", sprintf("%010.2f", NAN), "]", " [", sprintf("%010.2f", -INF), "]", "\n";
echo "%+.1f:", " [", sprintf("%+.1f", 0), "]", " [", sprintf("%+.1f", -1), "]", " [", sprintf("%+.1f", 42), "]", " [", sprintf("%+.1f", 255), "]", " [", sprintf("%+.1f", 3.14159), "]", " [", sprintf("%+.1f", -2.5), "]", " [", sprintf("%+.1f", 1e10), "]", " [", sprintf("%+.1f", -1e-5), "]", " [", sprintf("%+.1f", "abc"), "]", " [", sprintf("%+.1f", "12abc"), "]", " [", sprintf("%+.1f", true), "]", " [", sprintf("%+.1f", null), "]", " [", sprintf("%+.1f", PHP_INT_MIN), "]", " [", sprintf("%+.1f", 1.5e300), "]", " [", sprintf("%+.1f", NAN), "]", " [", sprintf("%+.1f", -INF), "]", "\n";
echo "%e:", " [", sprintf("%e", 0), "]", " [", sprintf("%e", -1), "]", " [", sprintf("%e", 42), "]", " [", sprintf("%e", 255), "]", " [", sprintf("%e", 3.14159), "]", " [", sprintf("%e", -2.5), "]", " [", sprintf("%e", 1e10), "]", " [", sprintf("%e", -1e-5), "]", " [", sprintf("%e", "abc"), "]", " [", sprintf("%e", "12abc"), "]", " [", sprintf("%e", true), "]", " [", sprintf("%e", null), "]", " [", sprintf("%e", PHP_INT_MIN), "]", " [", sprintf("%e", 1.5e300), "]", " [", sprintf("%e", NAN), "]", " [", sprintf("%e", -INF), "]", "\n";
echo "%.2e:", " [", sprintf("%.2e", 0), "]", " [", sprintf("%.2e", -1), "]", " [", sprintf("%.2e", 42), "]", " [", sprintf("%.2e", 255), "]", " [", sprintf("%.2e", 3.14159), "]", " [", sprintf("%.2e", -2.5), "]", " [", sprintf("%.2e", 1e10), "]", " [", sprintf("%.2e", -1e-5), "]", " [", sprintf("%.2e", "abc"), "]", " [", sprintf("%.2e", "12abc"), "]", " [", sprintf("%.2e", true), "]", " [", sprintf("%.2e", null), "]", " [", sprintf("%.2e", PHP_INT_MIN), "]", " [", sprintf("%.2e", 1.5e300), "]", " [", sprintf("%.2e", NAN), "]", " [", sprintf("%.2e", -INF), "]", "\n";
echo "%E:", " [", sprintf("%E", 0), "]", " [", sprintf("%E", -1), "]", " [", sprintf("%E", 42), "]", " [", sprintf("%E", 255), "]", " [", sprintf("%E", 3.14159), "]", " [", sprintf("%E", -2.5), "]", " [", sprintf("%E", 1e10), "]", " [", sprintf("%E", -1e-5), "]", " [", sprintf("%E", "abc"), "]", " [", sprintf("%E", "12abc"), "]", " [", sprintf("%E", true), "]", " [", sprintf("%E", null), "]", " [", sprintf("%E", PHP_INT_MIN), "]", " [", sprintf("%E", 1.5e300), "]", " [", sprintf("%E", NAN), "]", " [", sprintf("%E", -INF), "]", "\n";
echo "%g:", " [", sprintf("%g", 0), "]", " [", sprintf("%g", -1), "]", " [", sprintf("%g", 42), "]", " [", sprintf("%g", 255), "]", " [", sprintf("%g", 3.14159), "]", " [", sprintf("%g", -2.5), "]", " [", sprintf("%g", 1e10), "]", " [", sprintf("%g", -1e-5), "]", " [", sprintf("%g", "abc"), "]", " [", sprintf("%g", "12abc"), "]", " [", sprintf("%g", true), "]", " [", sprintf("%g", null), "]", " [", sprintf("%g", PHP_INT_MIN), "]", " [", sprintf("%g", 1.5e300), "]", " [", sprintf("%g", NAN), "]", " [", sprintf("%g", -INF), "]", "\n";
echo "%G:", " [", sprintf("%G", 0), "]", " [", sprintf("%G", -1), "]", " [", sprintf("%G", 42), "]", " [", sprintf("%G", 255), "]", " [", sprintf("%G", 3.14159), "]", " [", sprintf("%G", -2.5), "]", " [", sprintf("%G", 1e10), "]", " [", sprintf("%G", -1e-5), "]", " [", sprintf("%G", "abc"), "]", " [", sprintf("%G", "12abc"), "]", " [", sprintf("%G", true), "]", " [", sprintf("%G", null), "]", " [", sprintf("%G", PHP_INT_MIN), "]", " [", sprintf("%G", 1.5e300), "]", " [", sprintf("%G", NAN), "]", " [", sprintf("%G", -INF), "]", "\n";
echo "%.3g:", " [", sprintf("%.3g", 0), "]", " [", sprintf("%.3g", -1), "]", " [", sprintf("%.3g", 42), "]", " [", sprintf("%.3g", 255), "]", " [", sprintf("%.3g", 3.14159), "]", " [", sprintf("%.3g", -2.5), "]", " [", sprintf("%.3g", 1e10), "]", " [", sprintf("%.3g", -1e-5), "]", " [", sprintf("%.3g", "abc"), "]", " [", sprintf("%.3g", "12abc"), "]", " [", sprintf("%.3g", true), "]", " [", sprintf("%.3g", null), "]", " [", sprintf("%.3g", PHP_INT_MIN), "]", " [", sprintf("%.3g", 1.5e300), "]", " [", sprintf("%.3g", NAN), "]", " [", sprintf("%.3g", -INF), "]", "\n";
echo "%F:", " [", sprintf("%F", 0), "]", " [", sprintf("%F", -1), "]", " [", sprintf("%F", 42), "]", " [", sprintf("%F", 255), "]", " [", sprintf("%F", 3.14159), "]", " [", sprintf("%F", -2.5), "]", " [", sprintf("%F", 1e10), "]", " [", sprintf("%F", -1e-5), "]", " [", sprintf("%F", "abc"), "]", " [", sprintf("%F", "12abc"), "]", " [", sprintf("%F", true), "]", " [", sprintf("%F", null), "]", " [", sprintf("%F", PHP_INT_MIN), "]", " [", sprintf("%F", 1.5e300), "]", " [", sprintf("%F", NAN), "]", " [", sprintf("%F", -INF), "]", "\n";
"#);
    assert_eq!(out, "%d: [0] [-1] [42] [255] [3] [-2] [10000000000] [0] [0] [12] [1] [0] [-9223372036854775808] [0] [0] [0]\n%05d: [00000] [-0001] [00042] [00255] [00003] [-0002] [10000000000] [00000] [00000] [00012] [00001] [00000] [-9223372036854775808] [00000] [00000] [00000]\n%+d: [+0] [-1] [+42] [+255] [+3] [-2] [+10000000000] [+0] [+0] [+12] [+1] [+0] [-9223372036854775808] [+0] [+0] [+0]\n%-5d|: [0    |] [-1   |] [42   |] [255  |] [3    |] [-2   |] [10000000000|] [0    |] [0    |] [12   |] [1    |] [0    |] [-9223372036854775808|] [0    |] [0    |] [0    |]\n%x: [0] [ffffffffffffffff] [2a] [ff] [3] [fffffffffffffffe] [2540be400] [0] [0] [c] [1] [0] [8000000000000000] [0] [0] [0]\n%X: [0] [FFFFFFFFFFFFFFFF] [2A] [FF] [3] [FFFFFFFFFFFFFFFE] [2540BE400] [0] [0] [C] [1] [0] [8000000000000000] [0] [0] [0]\n%o: [0] [1777777777777777777777] [52] [377] [3] [1777777777777777777776] [112402762000] [0] [0] [14] [1] [0] [1000000000000000000000] [0] [0] [0]\n%b: [0] [1111111111111111111111111111111111111111111111111111111111111111] [101010] [11111111] [11] [1111111111111111111111111111111111111111111111111111111111111110] [1001010100000010111110010000000000] [0] [0] [1100] [1] [0] [1000000000000000000000000000000000000000000000000000000000000000] [0] [0] [0]\n%u: [0] [18446744073709551615] [42] [255] [3] [18446744073709551614] [10000000000] [0] [0] [12] [1] [0] [9223372036854775808] [0] [0] [0]\n%s: [0] [-1] [42] [255] [3.14159] [-2.5] [10000000000] [-1.0E-5] [abc] [12abc] [1] [] [-9223372036854775808] [1.5E+300] [NAN] [-INF]\n%10s|: [         0|] [        -1|] [        42|] [       255|] [   3.14159|] [      -2.5|] [10000000000|] [   -1.0E-5|] [       abc|] [     12abc|] [         1|] [          |] [-9223372036854775808|] [  1.5E+300|] [       NAN|] [      -INF|]\n%-10s|: [0         |] [-1        |] [42        |] [255       |] [3.14159   |] [-2.5      |] [10000000000|] [-1.0E-5   |] [abc       |] [12abc     |] [1         |] [          |] [-9223372036854775808|] [1.5E+300  |] [NAN       |] [-INF      |]\n%.2s: [0] [-1] [42] [25] [3.] [-2] [10] [-1] [ab] [12] [1] [] [-9] [1.] [NA] [-I]\n%'*10s: [*********0] [********-1] [********42] [*******255] [***3.14159] [******-2.5] [10000000000] [***-1.0E-5] [*******abc] [*****12abc] [*********1] [**********] [-9223372036854775808] [**1.5E+300] [*******NAN] [******-INF]\n%f: [0.000000] [-1.000000] [42.000000] [255.000000] [3.141590] [-2.500000] [10000000000.000000] [-0.000010] [0.000000] [12.000000] [1.000000] [0.000000] [-9223372036854775808.000000] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000000] [NaN] [INF]\n%.2f: [0.00] [-1.00] [42.00] [255.00] [3.14] [-2.50] [10000000000.00] [-0.00] [0.00] [12.00] [1.00] [0.00] [-9223372036854775808.00] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00] [NaN] [INF]\n%10.3f|: [     0.000|] [    -1.000|] [    42.000|] [   255.000|] [     3.142|] [    -2.500|] [10000000000.000|] [    -0.000|] [     0.000|] [    12.000|] [     1.000|] [     0.000|] [-9223372036854775808.000|] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000|] [NaN|] [INF|]\n%-10.2f|: [0.00      |] [-1.00     |] [42.00     |] [255.00    |] [3.14      |] [-2.50     |] [10000000000.00|] [-0.00     |] [0.00      |] [12.00     |] [1.00      |] [0.00      |] [-9223372036854775808.00|] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00|] [NaN|] [INF|]\n%010.2f: [0000000.00] [-000001.00] [0000042.00] [0000255.00] [0000003.14] [-000002.50] [10000000000.00] [-000000.00] [0000000.00] [0000012.00] [0000001.00] [0000000.00] [-9223372036854775808.00] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.00] [NaN] [-NF]\n%+.1f: [+0.0] [-1.0] [+42.0] [+255.0] [+3.1] [-2.5] [+10000000000.0] [-0.0] [+0.0] [+12.0] [+1.0] [+0.0] [-9223372036854775808.0] [+1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.0] [NaN] [INF]\n%e: [0.000000e+0] [-1.000000e+0] [4.200000e+1] [2.550000e+2] [3.141590e+0] [-2.500000e+0] [1.000000e+10] [-1.000000e-5] [0.000000e+0] [1.200000e+1] [1.000000e+0] [0.000000e+0] [-9.223372e+18] [1.500000e+300] [NaN] [INF]\n%.2e: [0.00e+0] [-1.00e+0] [4.20e+1] [2.55e+2] [3.14e+0] [-2.50e+0] [1.00e+10] [-1.00e-5] [0.00e+0] [1.20e+1] [1.00e+0] [0.00e+0] [-9.22e+18] [1.50e+300] [NaN] [INF]\n%E: [0.000000E+0] [-1.000000E+0] [4.200000E+1] [2.550000E+2] [3.141590E+0] [-2.500000E+0] [1.000000E+10] [-1.000000E-5] [0.000000E+0] [1.200000E+1] [1.000000E+0] [0.000000E+0] [-9.223372E+18] [1.500000E+300] [NaN] [INF]\n%g: [0] [-1] [42] [255] [3.14159] [-2.5] [1.0e+10] [-1.0e-5] [0] [12] [1] [0] [-9.22337e+18] [1.5e+300] [NaN] [INF]\n%G: [0] [-1] [42] [255] [3.14159] [-2.5] [1.0E+10] [-1.0E-5] [0] [12] [1] [0] [-9.22337E+18] [1.5E+300] [NaN] [INF]\n%.3g: [0] [-1] [42] [255] [3.14] [-2.5] [1.0e+10] [-1.0e-5] [0] [12] [1] [0] [-9.22e+18] [1.5e+300] [NaN] [INF]\n%F: [0.000000] [-1.000000] [42.000000] [255.000000] [3.141590] [-2.500000] [10000000000.000000] [-0.000010] [0.000000] [12.000000] [1.000000] [0.000000] [-9223372036854775808.000000] [1500000000000000078757140382806630373056702871662238732373781173267703686983362293679557062620671796065556665749325817265413784853040645863467188277180060474272580801389863705606745350692182135089053429852456199917621678558451461320111979114170213741868888183230085264257173504208294580298189100810240.000000] [NaN] [INF]\n");
}

/// `printf`/`vprintf`/`fprintf` return the byte count, `vsprintf` takes an array, `*` widths and precisions read arguments, and malformed formats throw PHP's `ValueError` / `ArgumentCountError` messages.
#[test]
fn test_printf_family_variants_and_errors() {
    let out = compile_and_run(r#"<?php
$n = printf("%s-%d\n", "a", 5); var_dump($n);
$n = vprintf("%2\$s %1\$s\n", ["x", "y"]); var_dump($n);
echo vsprintf("%05.1f|%'#10s|%-6d|", [3.14159, "hi", 42]), "\n";
$n = fprintf(STDOUT, "%s=%x\n", "k", 255); var_dump($n);
echo sprintf("%*d|%-*s|%.*f", 5, 42, 4, "ab", 2, 3.14159), "\n";
$f = "%d %d";
foreach (['%d %d', '%', '%y', '%0$s', "%'"] as $f) {
  try { echo sprintf($f, 1), "\n"; } catch (\ValueError $e) { echo "ValueError: ", $e->getMessage(), "\n"; } catch (\ArgumentCountError $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
}
try { vsprintf("%d %d %d", [1]); } catch (\ValueError $e) { echo "ValueError: ", $e->getMessage(), "\n"; }
try { sprintf("%d %d %d", 1); } catch (\TypeError $e) { echo get_class($e), " (TypeError): ", $e->getMessage(), "\n"; }
"#);
    assert_eq!(out, "a-5\nint(4)\ny x\nint(4)\n003.1|########hi|42    |\nk=ff\nint(5)\n   42|ab  |3.14\nArgumentCountError: 3 arguments are required, 2 given\nValueError: Missing format specifier at end of string\nValueError: Unknown format specifier \"y\"\nValueError: Argument number specifier must be greater than zero and less than 2147483647\nValueError: Missing padding character\nValueError: The arguments array must contain 3 items, 1 given\nArgumentCountError (TypeError): 4 arguments are required, 2 given\n");
}

/// Non-finite floats print as `INF` / `NaN` with a fixed three-column width on both the native and prelude paths.
#[test]
fn test_sprintf_non_finite_floats() {
    let out = compile_and_run(r#"<?php
$a=-INF;$b=INF;$c=NAN;
printf("[%f][%5.1f][%e][%g][%+f][%010f][%-6e][%f]\n", $a, $a, $b, $c, $b, $a, $a, $c);
echo sprintf("[%f][%5.1f][%-8f]", $a, $b, $c), "\n";
"#);
    assert_eq!(out, "[INF][INF][INF][NaN][INF][-NF][INF][NaN]\n[INF][INF][NaN]\n");
}

/// `number_format()` pre-rounds half away from zero, drops the sign of a value rounding to zero, honours negative and large decimal counts exactly, and `bindec`/`octdec`/`hexdec` on empty or prefix-only strings return 0 (the x86_64 helpers used to read their input from the wrong registers).
#[test]
fn test_number_format_rounds_like_php_and_bindec_octdec_empty() {
    let out = compile_and_run(r#"<?php
var_dump(number_format(-2000, 30), strlen(number_format(-2000, 2768)), number_format(0.1, 30), number_format(1.5e300, 20));
var_dump(number_format(1234567.891), number_format(1234567.891, 2), number_format(1234567.891, 2, ',', '.'), number_format(-0.5), number_format(0.5), number_format(1234.5678, 3, '.', ' '));
var_dump(bindec(""), bindec("0b"), bindec("101"), octdec("17"), hexdec("ff"), octdec(""));
"#);
    assert_eq!(out, "string(37) \"-2,000.000000000000000000000000000000\"\nint(2775)\nstring(32) \"0.100000000000000005551115123126\"\nstring(422) \"1,500,000,000,000,000,078,757,140,382,806,630,373,056,702,871,662,238,732,373,781,173,267,703,686,983,362,293,679,557,062,620,671,796,065,556,665,749,325,817,265,413,784,853,040,645,863,467,188,277,180,060,474,272,580,801,389,863,705,606,745,350,692,182,135,089,053,429,852,456,199,917,621,678,558,451,461,320,111,979,114,170,213,741,868,888,183,230,085,264,257,173,504,208,294,580,298,189,100,810,240.00000000000000000000\"\nstring(9) \"1,234,568\"\nstring(12) \"1,234,567.89\"\nstring(12) \"1.234.567,89\"\nstring(2) \"-1\"\nstring(1) \"1\"\nstring(9) \"1 234.568\"\nint(0)\nint(0)\nint(5)\nint(15)\nint(255)\nint(0)\n");
}

/// `(int)` of a float follows `zend_dval_to_lval`: NaN and infinities give 0 and out-of-range values wrap modulo 2^64; numeric strings saturate.
#[test]
fn test_float_to_int_cast_wraps_like_php() {
    let out = compile_and_run(r#"<?php $a=[1e30, INF, NAN, -1e19, 1e19, 9.2233720368547758E+18]; foreach ($a as $f) { var_dump((int)$f); } var_dump((int)"1e30", intval(1e30));
"#);
    assert_eq!(out, "int(5076964154930102272)\nint(0)\nint(0)\nint(8446744073709551616)\nint(-8446744073709551616)\nint(-9223372036854775808)\nint(9223372036854775807)\nint(5076964154930102272)\n");
}

/// `DivisionByZeroError` extends `ArithmeticError` and `ArgumentCountError` extends `TypeError`.
#[test]
fn test_division_by_zero_and_argument_count_error_classes() {
    let out = compile_and_run(r#"<?php
$e1 = new DivisionByZeroError("d"); $e2 = new ArgumentCountError("a");
var_dump($e1 instanceof ArithmeticError, $e1 instanceof Error, $e2 instanceof TypeError, $e2->getMessage(), get_class($e1));
try { throw new ArgumentCountError("x"); } catch (TypeError $t) { echo "caught ", get_class($t), "\n"; }
"#);
    assert_eq!(out, "bool(true)\nbool(true)\nbool(true)\nstring(1) \"a\"\nstring(19) \"DivisionByZeroError\"\ncaught ArgumentCountError\n");
}

/// Returning a string parameter hands the caller its own reference: the caller's variable keeps
/// its value after the result is released (it used to be freed twice).
#[test]
fn test_returning_a_string_parameter_does_not_double_free() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function pad(string $s, int $w): string { if ($w <= strlen($s)) { return $s; } return str_repeat(' ', $w - strlen($s)) . $s; }
function mk(int $i): string { return "v" . $i; }
function run(): string {
    $out = '';
    for ($i = 0; $i < 3; $i++) {
        $s = mk($i);
        $first = $s[0];
        $out .= pad($s, 0) . $first;
    }
    return $out;
}
echo run(), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "v0vv1vv2v\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Fixes from an independent review: a mixed local returned as a string is copied (it was
/// borrowed and then freed by the slot cleanup), foreach keys read only through `++`/`--` are
/// kept, a typed array passed to a widened `array` parameter is converted on a copy, inlined
/// callees do not leak their string locals, dynamic `sprintf` does not leak, `(int)` folding and
/// numeric strings follow PHP for out-of-range/non-finite values, and the native `sprintf` path
/// only takes conversions libc reproduces exactly (`%s` of any length or with NUL bytes, `%c`,
/// integer precisions, `-0.0`). Runs under heap debug, which must stay clean.
#[test]
fn test_review_fixes_ownership_casts_and_sprintf() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function v1(bool $c, string $a): string { $x = 5; if ($c) { $x = $a . "?"; } return $x; }
function v2(bool $c, string $a): string { $x = 5; if ($c) { $x = $a . "?"; } $y = $x; return $y; }
echo v1(true, str_repeat("ab", 2)), v1(false, "q"), v2(true, str_repeat("cd", 2)), v2(false, "q"), "\n";
function f() { foreach ([10 => "x", 20 => "y"] as $k => $v) { echo $k++, ","; } foreach ([10 => "x"] as $k => $v) { echo ++$k; } }
f(); echo "\n";
function d2(array $a) { return count($a); }
function m() { $fl = [1.5, 2.5]; $n = d2($fl); $other = [9.5, 8.5, 7.5]; return $fl[0] + $fl[1] + $n + count($fl) + count($other); }
echo m(), "\n"; echo d2(["a"]), "\n";
$fl=[1.5]; $mx=[1,"a"]; echo d2($fl), d2($mx), "\n";
function miss(array $g, int $u) { return @$g[$u][0] ?? "none"; }
$g = [[1,2,3],[4,5,6]]; echo miss($g, 1), miss($g, 5), miss([[]], 0), count($g), "\n";
function fill2(int $w): string { $fill = ''; if ($w > 1) { $fill = "yy"; } return $fill . "x"; }
function run2() { $n = 0; for ($i = 0; $i < 100; $i++) { $n += strlen(fill2(0)); } return $n; }
echo run2(), "\n";
for ($i = 0; $i < 50; $i++) { $f = "%d-%s-%f-%x-%e-%g"; $o = sprintf($f, $i, "s", 1.5, 255, 2.5, 0.1); } echo $o, "\n";
var_dump((int)(PHP_INT_MAX + 1), (int)9.2233720368547758e18, (int)"NAN", (int)"1e400", (int)"-1e400", (int)"0x1A", (int)" 12abc", (int)"1.9e3");
echo strlen(sprintf("%s", str_repeat("a", 5000))), "|", strlen(sprintf("%s", "a\0b")), "|", sprintf("%5c|%-5c|%05c|", 65, 66, 67), sprintf("%.3d|%.2x|%.2o|", 5, 255, 255), sprintf("%f|%5.1f", -0.0, -0.0), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "abab?5cdcd?5\n10,20,11\n11\n1\n12\n4nonenone2\n100\n49-s-1.500000-ff-2.500000e+0-0.1\nint(-9223372036854775808)\nint(-9223372036854775808)\nint(0)\nint(0)\nint(0)\nint(0)\nint(12)\nint(1900)\n5000|3|A|B|C|5|||0.000000|  0.0\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Prelude `sprintf` errors follow PHP: a lone `%` without arguments is an argument-count error,
/// `%.f`/`%.e` keep the default precision 6, and precision -1 is only accepted for `%g`/`%G`
/// (where it means the shortest round-trip representation).
#[test]
fn test_sprintf_error_precedence_and_minus_one_precision() {
    let out = compile_and_run(r#"<?php
foreach ([["%"],["%",1],["%.f",1.5],["%.e",1.5],["%.*g",-1,0.1],["%.*G",-1,1e20],["%.*f",-1,3.14],["%5"],["%5", 1]] as $a) { try { var_dump(sprintf(...$a)); } catch (\Throwable $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; } }
"#);
    assert_eq!(out, "ArgumentCountError: 2 arguments are required, 1 given\nValueError: Missing format specifier at end of string\nstring(8) \"1.500000\"\nstring(11) \"1.500000e+0\"\nstring(3) \"0.1\"\nstring(7) \"1.0E+20\"\nValueError: Precision -1 is only supported for %g, %G, %h and %H\nArgumentCountError: 2 arguments are required, 1 given\nValueError: Missing format specifier at end of string\n");
}

/// `$a[$i][0]` copies the selected byte (the source string used to be read after release), by-value `foreach` walks a snapshot of an array its body writes, returning a by-reference string parameter leaves the caller's variable intact, and `implode()` prints `false` as an empty string.
#[test]
fn test_string_offsets_foreach_snapshot_by_ref_return_and_bool_implode() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$arr = ["hello", "world", "abc"];
$t = 0; for ($i = 0; $i < 3; $i++) { $t += ord($arr[$i][0]); } echo $t, "\n";
$h = ["a" => 1.5, "b" => 2.5];
foreach ($h as $k => $v) { $h["c" . $k] = $v * 2; echo $k, "=", $v, " "; } echo count($h), "\n";
function r(string &$a): string { return $a; }
$x = str_repeat("q", 3); $y = r($x); echo $x, $y, "\n";
$l = ["abcdefghijklmnop", "x"]; for ($i = 0; $i < 2; $i++) { $l[$i] = substr($l[$i], 0, 10); } echo implode(",", $l), "\n";
echo implode(",", [true, false]), "\n";
$l2 = []; for ($i = 0; $i < 3; $i++) { if ($i > 0) { $l2[1] .= "C"; } else { $l2[1] = "s"; } } echo $l2[1], "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "320\na=1.5 b=2.5 4\nqqqqqq\nabcdefghij,x\n1,\nsCC\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Writes to the iterated array inside a by-value `foreach` (hash or list, with `break`/`return` exits) do not change the iteration, like PHP.
#[test]
fn test_foreach_by_value_iterates_a_snapshot() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$h = ["a" => 1, "b" => 2];
foreach ($h as $k => $v) { $h["c" . $k] = $v * 2; echo $k, " "; } echo count($h), "\n";
$a = [1, 2, 3];
foreach ($a as $v) { $a[] = $v; if ($v == 2) { break; } } echo count($a), "\n";
function f(array $x) { foreach ($x as $i => $v) { $x[] = $v; if ($i > 5) { return count($x); } } return count($x); }
echo f([1, 2]), "\n";
$h2 = ["a" => 1.5, "b" => 2.5];
foreach ($h2 as $k => $v) { $h2["c" . $k] = $v * 2; echo $k, "=", $v, " "; } echo count($h2), "\n";
$s = ["x", "y"]; foreach ($s as $v) { echo $v, count($s); } echo "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "a b 4\n5\n4\na=1.5 b=2.5 4\nx2y2\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// `implode()` over bool lists, statically typed or through a generic `array` parameter.
#[test]
fn test_implode_bool_lists() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$b = [true, false, true]; echo implode(",", $b), "|", implode("-", [false]), "\n";
function g(array $a) { return implode(",", $a); } echo g([true, false]), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1,,1|\n1,\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// PHP 8 ordering and loose equality over boxed values: null/bool operands compare as bools,
/// null against a string compares with "", arrays sort after scalars and by size, NAN is
/// uncomparable, two ints compare exactly (also when folded at compile time).
#[test]
fn test_mixed_php8_comparison_rules() {
    let out = compile_and_run(r#"<?php
function cmp_all(array $m) {
  $r = [];
  foreach ($m as $x) { foreach ($m as $y) { $r[] = ($x <=> $y) . (($x == $y) ? 'T' : 'F') . (($x < $y) ? 'L' : '-'); } }
  return implode(",", $r);
}
echo cmp_all([null, "", "0", "a", [], [1], 0.0, NAN, -1, 1, true, false, "1e3", 1000]), "\n";
var_dump(PHP_INT_MAX > PHP_INT_MAX - 1, PHP_INT_MAX <=> PHP_INT_MAX - 1);
$big = [PHP_INT_MAX, PHP_INT_MAX - 1, null];
var_dump($big[0] > $big[1], $big[0] == $big[1], $big[2] == 0, $big[2] < -1);
"#);
    assert_eq!(out, "0T-,0T-,-1FL,-1FL,0T-,-1FL,0T-,-1FL,-1FL,-1FL,-1FL,0T-,-1FL,-1FL,0T-,0T-,-1FL,-1FL,-1FL,-1FL,-1FL,1F-,-1FL,-1FL,-1FL,0T-,-1FL,-1FL,1F-,1F-,0T-,-1FL,-1FL,-1FL,0T-,1F-,1F-,-1FL,-1FL,0T-,-1FL,-1FL,1F-,1F-,1F-,0T-,-1FL,-1FL,1F-,1F-,1F-,1F-,0T-,1F-,1F-,1F-,0T-,1F-,1F-,1F-,0T-,-1FL,1F-,1F-,1F-,1F-,-1FL,0T-,1F-,1F-,1F-,1F-,1F-,1F-,1F-,0T-,1F-,1F-,1F-,1F-,0T-,1F-,1F-,1F-,0T-,1F-,0T-,-1FL,-1FL,-1FL,0T-,1F-,1F-,-1FL,-1FL,0T-,-1FL,-1FL,1F-,1F-,1F-,1F-,-1FL,-1FL,1F-,1F-,1F-,1F-,0T-,1F-,1F-,1F-,1F-,1F-,-1FL,-1FL,-1FL,-1FL,-1FL,1F-,0T-,-1FL,0T-,1F-,-1FL,-1FL,1F-,1F-,1F-,-1FL,-1FL,-1FL,1F-,1F-,1F-,0T-,0T-,1F-,-1FL,-1FL,1F-,1F-,1F-,0T-,1F-,0T-,1F-,0T-,0T-,0T-,0T-,1F-,0T-,0T-,0T-,0T-,0T-,-1FL,0T-,-1FL,0T-,-1FL,-1FL,-1FL,-1FL,0T-,-1FL,-1FL,1F-,1F-,1F-,-1FL,-1FL,-1FL,1F-,1F-,1F-,1F-,0T-,1F-,0T-,0T-,1F-,1F-,1F-,-1FL,-1FL,-1FL,1F-,1F-,1F-,1F-,0T-,1F-,0T-,0T-\nbool(true)\nint(1)\nbool(true)\nbool(false)\nbool(true)\nbool(true)\n");
}

/// Assigning the integer 0 to an untyped property stores `int(0)` (it stored null), and a
/// `foreach` over a generator, an Iterator or an IteratorAggregate releases the object it
/// drives on every exit (normal end, `break`, `return`), so destructors run and no
/// generator leaks per loop.
#[test]
fn test_untyped_property_zero_and_foreach_object_release() {
    let out = compile_and_run(r#"<?php
class B { private $i = 0; function r(): void { $this->i = 0; } function s(): void { $this->i = 3; } function g() { return $this->i; } function inc(): void { $this->i++; } }
$b = new B(); var_dump($b->g()); $b->r(); var_dump($b->g()); $b->s(); var_dump($b->g()); $b->r(); var_dump($b->g()); $b->inc(); var_dump($b->g());
function g() { for ($i = 0; $i < 5; $i++) yield $i => $i * 10; }
class It implements Iterator { private $i = 0; function current(): mixed { return $this->i * 2; } function key(): mixed { return $this->i; } function next(): void { $this->i++; } function rewind(): void { $this->i = 0; } function valid(): bool { return $this->i < 3; } function __destruct() { echo "it-gone\n"; } }
class Ag implements IteratorAggregate { function getIterator(): Iterator { return new It(); } }
function f() { foreach (g() as $k => $v) { if ($v > 20) return $k; } return -1; }
echo f(), "\n";
for ($j = 0; $j < 2; $j++) { foreach (g() as $v) { if ($v == 10) break; } }
foreach (new It() as $k => $v) { echo "$k=$v "; } echo "\n";
foreach (new Ag() as $k => $v) { echo "$k=$v "; if ($k == 1) break; } echo "\n";
$m = memory_get_usage(); for ($j = 0; $j < 50; $j++) { foreach (g() as $v) { } } echo memory_get_usage() === $m ? "no-leak" : "leak", "\n";
"#);
    assert_eq!(out, "int(0)\nint(0)\nint(3)\nint(0)\nint(1)\n3\n0=0 1=2 2=4 it-gone\n\n0=0 1=2 it-gone\n\nno-leak\n");
}

/// `return $a + $b` from an `: int` function raises PHP's TypeError when the sum overflows
/// ("Return value must be of type int, float returned"), except a negative overflow that
/// rounds to exactly -2^63; the call is kept even when its result is discarded.
#[test]
fn test_int_return_overflow_raises_type_error() {
    let out = compile_and_run(r#"<?php
function add(int $a, int $b): int { return $a + $b; }
function sub(int $a, int $b): int { return $a - $b; }
class K { function m(int $a): int { return $a + 1; } }
try { echo add(PHP_INT_MAX, 1), "\n"; } catch (TypeError $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
try { echo sub(PHP_INT_MIN, 1), "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { echo (new K)->m(PHP_INT_MAX), "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
echo add(2, 3), sub(2, 3), (new K)->m(4), "\n";
function fib(int $n): int { return $n < 2 ? $n : fib($n - 1) + fib($n - 2); } echo fib(25), "\n";
function sub2(int $a, int $b): int { return $a - $b; }
try { echo sub2(PHP_INT_MIN, 1000), "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { echo sub2(PHP_INT_MIN, 5000), "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { echo add(PHP_INT_MIN, -1), "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
for ($i = 0; $i < 3; $i++) { try { add(PHP_INT_MAX, $i + 1); } catch (TypeError $e) { echo "c"; } } echo "\n";
"#);
    assert_eq!(out, "TypeError: add(): Return value must be of type int, float returned\n-9223372036854775808\nK::m(): Return value must be of type int, float returned\n5-15\n75025\n-9223372036854775808\nsub2(): Return value must be of type int, float returned\n-9223372036854775808\nccc\n");
}

/// `explode()` with a limit (positive, zero, negative) follows PHP through the elephc-PHP
/// prelude, one-byte delimiters use the memchr splitter, list destructuring of a temporary
/// array releases it, and returning a local array widened to `array<mixed>` neither leaks nor
/// mutates a caller's array.
#[test]
fn test_explode_limit_destructuring_and_returned_conversions_are_heap_clean() {
    let out = compile_and_run_with_heap_debug(r#"<?php
foreach ([2, 1, 0, -1, -2, 5, -5, 3] as $l) { echo $l, ":", json_encode(explode(",", "a,b,c,d", $l)), json_encode(explode("||", "x||y||z", $l)), json_encode(explode(",", "", $l)), "\n"; }
[$k, $v] = explode("=", "key=val=ue", 2); echo $k, "|", $v, "\n";
echo json_encode(explode("-", "1-2-3", 2)), "\n";
[$k, $v] = explode("=", "key=val"); echo $k, $v, "\n";
function f() { return ["a", [1, 2], 3.5]; } [$a, $b, $c] = f(); echo $a, json_encode($b), $c, "\n";
[$p, $q] = ["x", "y"]; echo $p, $q, "\n";
$src = ["m", "n"]; [$m, $n] = $src; $src[0] = "z"; echo $m, $n, json_encode($src), "\n";
foreach ([["a", 1], ["b", 2]] as [$s, $i]) { echo $s, $i; } echo "\n";
[$k2, $v2] = explode("=", "k=v=w", 2); echo $k2, "|", $v2, "\n";
function myx2(string $s): array { $parts = explode(",", $s); if (strlen($s) > 100) { $o = []; $o[] = 1.5; return $o; } return $parts; }
function pass(array $a): array { if (count($a) > 100) { return [1.5]; } return $a; }
$x = myx2("a,b,c,d"); echo count($x), "\n";
$y = ["p", "q"]; $z = pass($y); $y[] = "r"; echo json_encode($y), json_encode($z), "\n";
echo json_encode(explode(",", "a,b,c,d", 2)), json_encode(explode(",", "a,b,c,d", -1)), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2:[\"a\",\"b,c,d\"][\"x\",\"y||z\"][\"\"]\n1:[\"a,b,c,d\"][\"x||y||z\"][\"\"]\n0:[\"a,b,c,d\"][\"x||y||z\"][\"\"]\n-1:[\"a\",\"b\",\"c\"][\"x\",\"y\"][]\n-2:[\"a\",\"b\"][\"x\"][]\n5:[\"a\",\"b\",\"c\",\"d\"][\"x\",\"y\",\"z\"][\"\"]\n-5:[][][]\n3:[\"a\",\"b\",\"c,d\"][\"x\",\"y\",\"z\"][\"\"]\nkey|val=ue\n[\"1\",\"2-3\"]\nkeyval\na[1,2]3.5\nxy\nmn[\"z\",\"n\"]\na1b2\nk|v=w\n4\n[\"p\",\"q\",\"r\"][\"p\",\"q\"]\n[\"a\",\"b,c,d\"][\"a\",\"b\",\"c\"]\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// An array write keyed by data instead of a position (`foreach` value, array element, call
/// or property result) builds a hash, like PHP: a packed list renumbered such keys or
/// zero-filled the gaps (`$m[$id] = ...` with ids 100, 5, 42 printed 101 entries).
#[test]
fn test_data_keyed_array_writes_build_hashes() {
    let out = compile_and_run(r#"<?php
class U { public function __construct(public int $id, public string $n) {} public function key(): int { return $this->id * 10; } }
$us = [new U(7, "x"), new U(3, "y"), new U(7, "z")];
$byId = []; foreach ($us as $u) { $byId[$u->id] = $u->n; } var_dump($byId);
$byKey = []; foreach ($us as $u) { $byKey[$u->key()] = $u->id; } echo json_encode($byKey), "\n";
$cnt = []; foreach ([5, 3, 5, 9, 3, 5] as $v) { $cnt[$v] = ($cnt[$v] ?? 0) + 1; } print_r($cnt); ksort($cnt); echo json_encode($cnt), "\n";
$sq = []; foreach ([4, 2] as $v) { $sq[intval($v * $v)] = true; } var_dump($sq, isset($sq[16]), isset($sq[0]));
$list = []; for ($i = 0; $i < 5; $i++) { $list[$i] = $i * $i; } echo json_encode($list), "\n";
$app = []; foreach (["a", "b"] as $v) { $app[count($app)] = $v; } echo json_encode($app), "\n";
$inv = []; $src = ["p" => 2, "q" => 0]; foreach ($src as $k => $v) { $inv[$v] = $k; } print_r($inv);
$rows = [[1, "one"], [0, "zero"]]; $m = []; foreach ($rows as $r) { $m[$r[0]] = $r[1]; } echo json_encode($m), " ", count($m), "\n";
$total = 0; foreach ($cnt as $k => $c) { $total += $k * $c; } echo $total, "\n";
$rows = [["id" => 100, "n" => "a"], ["id" => 5, "n" => "b"], ["id" => 42, "n" => "c"]];
$byId = [];
foreach ($rows as $r) { $byId[$r["id"]] = $r["n"]; }
print_r($byId); echo count($byId), "\n";
$ids = [100, 5, 42]; $m = [];
foreach ($ids as $id) { $m[$id] = $id * 2; }
echo count($m), " ", json_encode($m), "\n";
"#);
    assert_eq!(out, "array(2) {\n  [7]=>\n  string(1) \"z\"\n  [3]=>\n  string(1) \"y\"\n}\n{\"70\":7,\"30\":3}\nArray\n(\n    [5] => 3\n    [3] => 2\n    [9] => 1\n)\n{\"3\":2,\"5\":3,\"9\":1}\narray(2) {\n  [16]=>\n  bool(true)\n  [4]=>\n  bool(true)\n}\nbool(true)\nbool(false)\n[0,1,4,9,16]\n[\"a\",\"b\"]\nArray\n(\n    [2] => p\n    [0] => q\n)\n{\"1\":\"one\",\"0\":\"zero\"} 2\n30\nArray\n(\n    [100] => a\n    [5] => b\n    [42] => c\n)\n3\n3 {\"100\":200,\"5\":10,\"42\":84}\n");
}

/// A write keyed by a `foreach` key of a list may skip indices (filtering), so the
/// destination is a hash: the packed list filled the gaps with nulls. `implode()`/`join()`
/// and `array_product()` accept associative arrays (through `array_values()`).
#[test]
fn test_foreach_key_filter_writes_keep_keys() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$src = [3, 1, 4, 1, 5, 9];
$r = []; foreach ($src as $k => $v) { if ($v % 2) { $r[$k] = $v; } }
echo count($r), " ", json_encode($r), " ", implode(",", $r), " ", array_product($r), " ", array_sum($r), "\n";
print_r($r); var_dump(isset($r[2]), $r[5]);
foreach ($r as $k => $v) { echo "$k=$v "; } echo "\n";
$all = []; foreach ($src as $k => $v) { $all[$k] = $v * 2; } echo json_encode($all), " ", implode("-", $all), "\n";
echo implode(",", ["x" => "a", "y" => "b"]), " ", join("/", ["p" => 1.5, "q" => 2]), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5 {\"0\":3,\"1\":1,\"3\":1,\"4\":5,\"5\":9} 3,1,1,5,9 135 19\nArray\n(\n    [0] => 3\n    [1] => 1\n    [3] => 1\n    [4] => 5\n    [5] => 9\n)\nbool(false)\nint(9)\n0=3 1=1 3=1 4=5 5=9 \n[6,2,8,2,10,18] 6-2-8-2-10-18\na,b 1.5/2\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// A list passed twice to an `array` parameter widened to a hash is converted per call
/// without stealing the caller's reference (it was freed after the first call); `max()` /
/// `min()` of an array use PHP comparison (first of equals, `ValueError` when empty);
/// rebuilding a mixed-key hash through `foreach` keeps its keys; an owned `mixed` key is
/// released after the write.
#[test]
fn test_array_param_conversion_max_min_and_mixed_key_rebuilds() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function f(array $v): int { $n = 0; foreach ($v as $k => $x) { echo "[$k=$x]"; $n++; } return count($v) * 100 + $n; }
$a = [3, 1, 2]; echo f($a), "\n"; echo f($a), "\n"; echo f(["p" => 5]), "\n";
echo max([3, 1, 2]), min([3, 1, 2]), " ";
$b = [3, 1, 2]; echo max($b) + min($b), " ";
$s = ["b", "a", "c"]; echo max($s), min($s), "\n";
try { max([]); } catch (ValueError $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
$h = ["a" => 1, "b" => 2, 5 => 3]; $r2 = []; foreach ($h as $k => $v) { if ($v != 2) { $r2[$k] = $v * 10; } } echo json_encode($r2), "\n";
$rows = [["id" => 100, "n" => "a"], ["id" => 5, "n" => "b"]]; $byId = []; foreach ($rows as $r) { $byId[$r["id"]] = $r["n"]; } echo json_encode($byId), " ", max($byId), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "[0=3][1=1][2=2]303\n[0=3][1=1][2=2]303\n[p=5]101\n31 4 ca\nValueError: max(): Argument #1 ($value) must contain at least one element\n{\"a\":10,\"5\":30}\n{\"100\":\"a\",\"5\":\"b\"} b\n");
}

/// Review regressions: a string slice used as a hash key is copied (never taken for an
/// interned literal key, which corrupted the source); a directly-called closure that builds a
/// capturing closure keeps the runtime's callee-saved state; data-keyed hash promotion leaves
/// by-reference, filled and in-place-rewritten arrays as lists; `int|null` keys; boxed rows
/// stored in a promoted hash; `max([])` throws.
#[test]
fn test_review_regressions_interned_keys_closures_and_promotion() {
    let out = compile_and_run(r#"<?php
$s2 = "the quick brown fox jumps over the lazy dog again and again";
$w = [];
$w[strstr($s2, "lazy")] = 9; $w[strrchr($s2, "a")] = 8; $w[substr($s2, 20)] = 7;
echo $s2, " ", count($w), "\n";
$src = str_repeat("zzzz", 2) . "HPLE" . "thekey";
$h = []; $h[substr($src, 12)] = 1; echo $src, " ", implode(",", array_keys($h)), "\n";
$a = [1, 2, 3];
print_r(array_map(fn(int $x): int => (fn(int $y): int => $y * $x)(2), $a));
function f(array &$out, array $src): void { foreach ($src as $k => $v) { $out[$k] = $v; } }
$o = [9, 9, 9, 9]; f($o, [1, 2]); echo implode(",", $o), "\n";
function g(array &$out, array $src): void { foreach ($src as $v) { $out[$v] = "x"; } }
$o2 = ["a", "b", "c"]; g($o2, [1]); echo implode(",", $o2), "\n";
$b = [3, 1, 2];
foreach ($b as $i => $v) { $t = $b[$i]; $b[$i] = $t; }
echo implode(",", $b), "\n";
function run(array $p) { $c = [0, 0, 0]; $c[$p[0]] = 5; return $c; }
echo implode(",", run([1])), "\n";
$out = []; foreach ([4, 5, 6] as $i => $v) { $out[$i] = $v * 2; } echo implode(",", array_slice($out, 1)), " ", implode(",", array_reverse($out)), "\n";
$idx = [2, 0, 1]; $vals = ["a", "b", "c"]; $perm2 = [];
for ($j = 0; $j < 3; $j++) { $perm2[$idx[$j]] = $vals[$j]; } echo json_encode($perm2), "\n";
$rows = [["id" => 3, "n" => "c"], ["id" => 1, "n" => "a"]]; $by2 = [];
foreach ($rows as $row) { $by2[$row["id"]] = $row; } echo json_encode($by2), "\n";
try { max([]); } catch (ValueError $e) { echo $e->getMessage(), "\n"; }
"#);
    assert_eq!(out, "the quick brown fox jumps over the lazy dog again and again 3\nzzzzzzzzHPLEthekey thekey\nArray\n(\n    [0] => 2\n    [1] => 4\n    [2] => 6\n)\n1,2,9,9\na,x,c\n3,1,2\n0,5,0\n10,12 12,10,8\n{\"2\":\"a\",\"0\":\"b\",\"1\":\"c\"}\n{\"3\":{\"id\":3,\"n\":\"c\"},\"1\":{\"id\":1,\"n\":\"a\"}}\nmax(): Argument #1 ($value) must contain at least one element\n");
}
