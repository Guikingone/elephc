//! Purpose:
//! End-to-end codegen tests for the `parse_ini_file()` prelude (see
//! `crate::parse_ini_prelude`): flat `key = value` parsing across the NORMAL /
//! RAW / TYPED scanner modes, plus the unreadable-file `false` return. The
//! sectioned / `key[] =` nested forms are a documented residual (a pre-existing
//! elephc Mixed-array-element-access gap) and are deliberately NOT asserted here.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each test writes its INI file with `file_put_contents` into the binary's
//!   (unique, per-test) working directory and reads it back, so the fixtures are
//!   self-contained and parallel-safe.
//! - Every expected value is php-verified (PHP 8.5.6 local, `php -n`). Values are
//!   iterated (never `var_export`-ed as a whole array) so the assertions do not
//!   depend on the separate `var_export`-of-array gap on this branch.

use crate::support::*;

/// The INI_SCANNER_* constants resolve to PHP's ext/standard values.
#[test]
fn test_ini_scanner_constant_values() {
    let out = compile_and_run(
        "<?php echo INI_SCANNER_NORMAL, \",\", INI_SCANNER_RAW, \",\", INI_SCANNER_TYPED;",
    );
    assert_eq!(out, "0,1,2");
}

/// NORMAL mode: values are strings; `on/true/yes` -> "1", `off/false/no` -> "";
/// quoted values keep their literal contents; comments and blank lines are skipped.
#[test]
fn test_parse_ini_file_normal_mode() {
    let out = compile_and_run(
        r#"<?php
file_put_contents('t.ini', "a=1\nb=hello\n; a comment\nq = \"x y\"\nd=on\ne=off\nnum=42\n");
$r = parse_ini_file('t.ini');
foreach ($r as $k => $v) { echo "$k=$v;"; }
"#,
    );
    assert_eq!(out, "a=1;b=hello;q=x y;d=1;e=;num=42;");
}

/// RAW mode: values are the raw token (no boolean coercion), quotes stripped,
/// inline `;` comments trimmed from unquoted values.
#[test]
fn test_parse_ini_file_raw_mode() {
    let out = compile_and_run(
        r#"<?php
file_put_contents('t.ini', "a=1\nb=hello\nq = \"x y\"\nd=on\ne=off\ninline=5 ; trailing\n");
$r = parse_ini_file('t.ini', false, INI_SCANNER_RAW);
foreach ($r as $k => $v) { echo "$k=$v;"; }
"#,
    );
    assert_eq!(out, "a=1;b=hello;q=x y;d=on;e=off;inline=5;");
}

/// TYPED mode: booleans become real `bool`, integers `int`, floats `float`;
/// non-numeric and quoted values stay `string`.
#[test]
fn test_parse_ini_file_typed_mode() {
    let out = compile_and_run(
        r#"<?php
file_put_contents('t.ini', "a=1\nb=hello\nq = \"123\"\nd=on\ne=off\nflt=3.14\nneg=-7\n");
$r = parse_ini_file('t.ini', false, INI_SCANNER_TYPED);
foreach ($r as $k => $v) { echo "$k=" . gettype($v) . "=" . var_export($v, true) . ";"; }
"#,
    );
    assert_eq!(
        out,
        "a=integer=1;b=string='hello';q=string='123';d=boolean=true;e=boolean=false;flt=double=3.14;neg=integer=-7;"
    );
}

/// An unreadable / missing file returns `false` (the warning is emitted to
/// stderr and is not part of stdout).
#[test]
fn test_parse_ini_file_missing_returns_false() {
    let out = compile_and_run(
        r#"<?php
$r = parse_ini_file('definitely_missing_file.ini');
var_dump($r);
"#,
    );
    assert_eq!(out, "bool(false)\n");
}
