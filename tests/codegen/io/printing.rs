//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of I/O printing, including print basic, print integer, and print expression returns one.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies that `print` outputs a plain string literal unchanged.
#[test]
fn test_print_basic() {
    let out = compile_and_run("<?php print \"hello\";");
    assert_eq!(out, "hello");
}

/// Verifies that `print` outputs a bare integer literal as its decimal string representation.
#[test]
fn test_print_int() {
    let out = compile_and_run("<?php print 42;");
    assert_eq!(out, "42");
}

/// Verifies that `print` returns `1` when used in an expression context, matching PHP's value-for-side-effect semantics.
#[test]
fn test_print_expression_returns_one() {
    let out = compile_and_run("<?php $ok = print \"hello\"; echo \"\\n\"; echo $ok;");
    assert_eq!(out, "hello\n1");
}

/// Verifies that `print` returning `1` is correctly absorbed by `echo`, producing `"x1"` not `"x"` or a parse error.
#[test]
fn test_print_expression_can_be_nested_in_echo() {
    let out = compile_and_run("<?php echo print \"x\";");
    assert_eq!(out, "x1");
}

/// Verifies that `print` can accept a short-ternary expression as its operand; `print` binds tighter than `?:`, so `false ?: "fallback"` is evaluated first, then printed, and the resulting `1` return is echoed.
#[test]
fn test_print_expression_operand_accepts_short_ternary() {
    let out = compile_and_run("<?php echo print false ?: \"fallback\";");
    assert_eq!(out, "fallback1");
}

/// Verifies precedence: `print "x" and false` parses as `(print "x") and false` — `print` outputs and returns `1`, which is truthy, so `and false` does not suppress output.
#[test]
fn test_print_expression_binds_tighter_than_word_and() {
    let out = compile_and_run("<?php echo print \"x\" and false;");
    assert_eq!(out, "x");
}

/// Verifies that `print __FILE__` emits the source file path at compile time (magic constant lowering).
#[test]
fn test_print_expression_lowers_magic_constants() {
    let out = compile_and_run("<?php print __FILE__;");
    assert!(out.ends_with("test.php"), "unexpected __FILE__ output: {out}");
}

/// Verifies `var_dump` formats a bare integer as `int(N)` with a trailing newline.
#[test]
fn test_var_dump_int() {
    let out = compile_and_run("<?php var_dump(42);");
    assert_eq!(out, "int(42)\n");
}

/// Verifies `var_dump` formats a string as `string(N) "..."` including length, quotes, and a trailing newline.
#[test]
fn test_var_dump_string() {
    let out = compile_and_run(r#"<?php var_dump("hello");"#);
    assert_eq!(out, "string(5) \"hello\"\n");
}

/// Verifies `var_dump` formats boolean `true` as `bool(true)` with a trailing newline.
#[test]
fn test_var_dump_bool_true() {
    let out = compile_and_run("<?php var_dump(true);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies `var_dump` formats boolean `false` as `bool(false)` with a trailing newline.
#[test]
fn test_var_dump_bool_false() {
    let out = compile_and_run("<?php var_dump(false);");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies `var_dump` formats `null` as `NULL` (uppercase, no parentheses) with a trailing newline.
#[test]
fn test_var_dump_null() {
    let out = compile_and_run("<?php var_dump(null);");
    assert_eq!(out, "NULL\n");
}

/// Verifies `var_dump` formats a float as `float(VALUE)` with full precision and a trailing newline.
#[test]
fn test_var_dump_float() {
    let out = compile_and_run("<?php var_dump(3.14);");
    assert_eq!(out, "float(3.14)\n");
}

/// Verifies `var_dump` emits the correct concrete type tag and value for each heterogeneous assoc-array slot: int, string, bool, null, array, and object.
#[test]
fn test_var_dump_mixed_prints_concrete_payload() {
    let out = compile_and_run(
        r#"<?php
class Box {}

$map = [
    "i" => 42,
    "s" => "hello",
    "b" => true,
    "n" => null,
    "a" => [1, 2],
    "o" => new Box(),
];

var_dump($map["i"]);
var_dump($map["s"]);
var_dump($map["b"]);
var_dump($map["n"]);
var_dump($map["a"]);
var_dump($map["o"]);
"#,
    );
    assert_eq!(
        out,
        "int(42)\nstring(5) \"hello\"\nbool(true)\nNULL\narray(2) {\n}\nobject(Box)\n"
    );
}

/// Verifies `print_r` outputs a bare integer as its decimal string representation (no type label), no trailing newline.
#[test]
fn test_print_r_int() {
    let out = compile_and_run("<?php print_r(42);");
    assert_eq!(out, "42");
}

/// Verifies `print_r` outputs a string unchanged, no type label, no trailing newline.
#[test]
fn test_print_r_string() {
    let out = compile_and_run(r#"<?php print_r("hello");"#);
    assert_eq!(out, "hello");
}

/// Verifies `print_r` outputs `1` for boolean `true`, no type label, no trailing newline.
#[test]
fn test_print_r_bool_true() {
    let out = compile_and_run("<?php print_r(true);");
    assert_eq!(out, "1");
}

/// Verifies `print_r` outputs an empty string for boolean `false`.
#[test]
fn test_print_r_bool_false() {
    let out = compile_and_run("<?php print_r(false);");
    assert_eq!(out, "");
}

/// Verifies `print_r` renders an indexed array with PHP's recursive
/// `Array\n(\n    [N] => value\n)\n` body and numeric keys.
#[test]
fn test_print_r_array() {
    let out = compile_and_run("<?php print_r([1, 2, 3]);");
    assert_eq!(out, "Array\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\n");
}

/// Verifies `print_r` renders an indexed string array, with raw (unquoted) values.
#[test]
fn test_print_r_string_array() {
    let out = compile_and_run(r#"<?php print_r(["a", "b", "c"]);"#);
    assert_eq!(out, "Array\n(\n    [0] => a\n    [1] => b\n    [2] => c\n)\n");
}

/// Verifies `print_r` renders a bool array with PHP's `1`/empty rendering for true/false.
#[test]
fn test_print_r_bool_array() {
    let out = compile_and_run("<?php print_r([true, false, true]);");
    assert_eq!(out, "Array\n(\n    [0] => 1\n    [1] => \n    [2] => 1\n)\n");
}

/// Verifies `print_r` renders a float array using PHP's float text.
#[test]
fn test_print_r_float_array() {
    let out = compile_and_run("<?php print_r([1.5, 2.25]);");
    assert_eq!(out, "Array\n(\n    [0] => 1.5\n    [1] => 2.25\n)\n");
}

/// Verifies `print_r` renders an associative array with unquoted string keys.
#[test]
fn test_print_r_assoc_array() {
    let out = compile_and_run(r#"<?php print_r(["name" => "bob", "age" => 30]);"#);
    assert_eq!(out, "Array\n(\n    [name] => bob\n    [age] => 30\n)\n");
}

/// Verifies `print_r` renders an empty array as the bare `Array\n(\n)\n` shell.
#[test]
fn test_print_r_empty_array() {
    let out = compile_and_run("<?php print_r([]);");
    assert_eq!(out, "Array\n(\n)\n");
}

/// Verifies `print_r` renders a hash with a heterogeneous (Mixed) value set,
/// matching PHP's per-type rendering (string raw, bool `1`, null empty).
#[test]
fn test_print_r_mixed_value_hash() {
    let out = compile_and_run(r#"<?php print_r(["s" => "x", "b" => true, "n" => null]);"#);
    assert_eq!(out, "Array\n(\n    [s] => x\n    [b] => 1\n    [n] => \n)\n");
}

/// Verifies `print_r` recurses into a nested array inside a hash, indenting the
/// nested body by 8 spaces per level and emitting the trailing blank line that
/// PHP writes after a nested array's closing paren.
#[test]
fn test_print_r_nested_array_in_hash() {
    let out = compile_and_run(r#"<?php print_r(["x" => [1, 2], "y" => 3]);"#);
    assert_eq!(
        out,
        "Array\n(\n    [x] => Array\n        (\n            [0] => 1\n            [1] => 2\n        )\n\n    [y] => 3\n)\n"
    );
}

/// Verifies `print_r` recurses into an array of arrays (indexed nesting), which
/// relies on the runtime value_type stamp to dispatch the nested element type.
#[test]
fn test_print_r_nested_indexed_arrays() {
    let out = compile_and_run("<?php print_r([[1, 2], [3, 4]]);");
    assert_eq!(
        out,
        "Array\n(\n    [0] => Array\n        (\n            [0] => 1\n            [1] => 2\n        )\n\n    [1] => Array\n        (\n            [0] => 3\n            [1] => 4\n        )\n\n)\n"
    );
}

/// Verifies `print_r` renders a deeply nested structure with the correct
/// cumulative indentation at each level.
#[test]
fn test_print_r_deep_nesting() {
    let out = compile_and_run(r#"<?php print_r([1 => ["a" => ["z" => 9]]]);"#);
    assert_eq!(
        out,
        "Array\n(\n    [1] => Array\n        (\n            [a] => Array\n                (\n                    [z] => 9\n                )\n\n        )\n\n)\n"
    );
}

/// Verifies `print_r` renders a single boxed Mixed scalar (an element read from
/// a heterogeneous array) with no type wrapper, matching PHP.
#[test]
fn test_print_r_mixed_scalar_element() {
    let out = compile_and_run(
        r#"<?php $a = [1, "two", 3.5, true, null]; print_r($a[1]); echo "|"; print_r($a[3]);"#,
    );
    assert_eq!(out, "two|1");
}

/// H5: php-verified — `print_r($v)` (no `$return`, or `$return = false`)
/// writes to stdout AND returns the concrete `true` (never void/nothing).
#[test]
fn test_print_r_no_return_arg_still_returns_true() {
    let out = compile_and_run(r#"<?php $r = print_r("x"); var_dump($r);"#);
    assert_eq!(out, "xbool(true)\n");
}

/// H5: `print_r($v, true)` captures the exact same scalar rendering as the
/// stdout form into a returned string instead of writing it — php-verified
/// scalar formats: int/float/string raw, bool `true`→`"1"`, bool
/// `false`/`null`→`""`.
#[test]
fn test_print_r_return_true_scalars() {
    let out = compile_and_run(
        r#"<?php
echo print_r(42, true), "|";
echo print_r("hello", true), "|";
echo print_r(true, true), "|";
echo print_r(false, true), "|";
echo print_r(null, true), "|";
echo print_r(1.5, true);
"#,
    );
    assert_eq!(out, "42|hello|1|||1.5");
}

/// H5: `print_r($array, true)` renders the exact same recursive
/// `Array\n(\n    [k] => v\n)\n` body as the stdout form, captured into a string.
#[test]
fn test_print_r_return_true_nested_array() {
    let out = compile_and_run(
        r#"<?php
$s = print_r(["a" => 1, "b" => ["c" => 2, "d" => 3]], true);
echo $s;
"#,
    );
    assert_eq!(
        out,
        "Array\n(\n    [a] => 1\n    [b] => Array\n        (\n            [c] => 2\n            [d] => 3\n        )\n\n)\n"
    );
}

/// H5: `print_r($v, true)`'s returned string is independently usable (not an
/// alias into reused scratch state) — two consecutive calls must not corrupt
/// each other's captured output.
#[test]
fn test_print_r_return_true_two_calls_independent() {
    let out = compile_and_run(
        r#"<?php
$a = print_r([1, 2], true);
$b = print_r(["x", "y", "z"], true);
echo $a;
echo $b;
"#,
    );
    assert_eq!(
        out,
        "Array\n(\n    [0] => 1\n    [1] => 2\n)\nArray\n(\n    [0] => x\n    [1] => y\n    [2] => z\n)\n"
    );
}

/// H5: `print_r($v, true)` on a plain (non-nested) value still returns `string`,
/// usable directly with string functions.
#[test]
fn test_print_r_return_true_usable_as_string() {
    let out = compile_and_run(r#"<?php echo strlen(print_r("hello", true));"#);
    assert_eq!(out, "5");
}

/// H5: `print_r($object, true)` stays loud (object dumps need class metadata
/// the capture-buffer walker lacks, matching the pre-existing stdout-form limitation).
#[test]
#[should_panic(expected = "print_r($v, true) for PHP type Object")]
fn test_print_r_return_true_object_stays_loud() {
    compile_and_run(
        r#"<?php
class Foo {}
$f = new Foo();
print_r($f, true);
"#,
    );
}

/// Verifies `print_r($value, true)` returns the rendered int as a string instead
/// of writing to stdout.
#[test]
fn test_print_r_return_int() {
    let out = compile_and_run(r#"<?php echo print_r(42, true);"#);
    assert_eq!(out, "42");
}

/// Verifies `print_r($value, true)` returns the rendered string unchanged.
#[test]
fn test_print_r_return_string() {
    let out = compile_and_run(r#"<?php echo print_r("hello", true);"#);
    assert_eq!(out, "hello");
}

/// Verifies `print_r($value, true)` returns `1` for boolean true (no type label).
#[test]
fn test_print_r_return_bool_true() {
    let out = compile_and_run(r#"<?php echo print_r(true, true);"#);
    assert_eq!(out, "1");
}

/// Verifies `print_r($value, true)` returns the empty string for boolean false.
#[test]
fn test_print_r_return_bool_false() {
    let out = compile_and_run(r#"<?php $s = print_r(false, true); echo strlen($s);"#);
    assert_eq!(out, "0");
}

/// Verifies `print_r($value, true)` returns the full array body as a string.
#[test]
fn test_print_r_return_array() {
    let out = compile_and_run(r#"<?php echo print_r([1, 2, 3], true);"#);
    assert_eq!(out, "Array\n(\n    [0] => 1\n    [1] => 2\n    [2] => 3\n)\n");
}

/// Verifies `print_r($value, true)` returns the associative-array body as a string.
#[test]
fn test_print_r_return_assoc_array() {
    let out = compile_and_run(r#"<?php echo print_r(["a" => 1], true);"#);
    assert_eq!(out, "Array\n(\n    [a] => 1\n)\n");
}

/// Verifies `print_r($value, true)` captures nested array output recursively.
#[test]
fn test_print_r_return_nested_array() {
    let out = compile_and_run(r#"<?php echo print_r([[1, 2], [3, 4]], true);"#);
    assert_eq!(
        out,
        "Array\n(\n    [0] => Array\n        (\n            [0] => 1\n            [1] => 2\n        )\n\n    [1] => Array\n        (\n            [0] => 3\n            [1] => 4\n        )\n\n)\n"
    );
}

/// Verifies `print_r($value)` without `$return` still writes to stdout (backward compat).
#[test]
fn test_print_r_no_return_still_echoes() {
    let out = compile_and_run(r#"<?php $r = print_r(42); echo "|$r";"#);
    assert_eq!(out, "42|1");
}

/// Verifies `print_r($value, false)` keeps echo mode (writes to stdout, returns true).
#[test]
fn test_print_r_return_false_echoes() {
    let out = compile_and_run(r#"<?php $r = print_r(42, false); echo "|$r";"#);
    assert_eq!(out, "42|1");
}

/// Verifies the string returned by `print_r($value, true)` has the correct length.
#[test]
fn test_print_r_return_length() {
    let out = compile_and_run(r#"<?php echo strlen(print_r(["a" => 1], true));"#);
    assert_eq!(out, "23");
}

/// Verifies `print_r($value, $flag)` with a runtime-truthy flag selects return
/// mode at runtime: nothing is echoed by print_r and the rendered string is
/// returned (as a boxed Mixed value, since the static type is `string|bool`).
#[test]
fn test_print_r_return_runtime_flag_true() {
    let out = compile_and_run(
        r#"<?php
$flag = $argc > 0;
$r = print_r("hi", $flag);
echo "|";
echo $r;
"#,
    );
    assert_eq!(out, "|hi");
}

/// Verifies `print_r($value, $flag)` with a runtime-falsy flag keeps echo mode:
/// the value is printed and the call returns boolean true.
#[test]
fn test_print_r_return_runtime_flag_false() {
    let out = compile_and_run(
        r#"<?php
$flag = $argc > 5;
$r = print_r("hi", $flag);
echo "|";
var_dump($r);
"#,
    );
    assert_eq!(out, "hi|bool(true)\n");
}

/// Verifies the runtime-flag return mode captures a full array body through the
/// recursive walker writes, exactly like the literal `true` fast path.
#[test]
fn test_print_r_return_runtime_flag_array() {
    let out = compile_and_run(
        r#"<?php
$flag = $argc > 0;
echo print_r([1, 2], $flag);
"#,
    );
    assert_eq!(out, "Array\n(\n    [0] => 1\n    [1] => 2\n)\n");
}

/// Verifies `print_r($value, true)` clamps the capture at the 64 KiB buffer size
/// instead of overflowing into adjacent runtime data. PHP would return all 70000
/// bytes; elephc truncates at 65536 (documented safety cap of the fixed buffer).
/// The prefix must survive intact — a corrupted capture or clobbered BSS would
/// change the rendered bytes or crash the binary.
#[test]
fn test_print_r_return_clamps_at_buffer_capacity() {
    let out = compile_and_run(
        r#"<?php
$s = str_repeat("x", 70000);
$r = print_r($s, true);
echo strlen($r);
echo "|";
echo substr($r, 0, 3);
echo "|";
echo substr($r, 65533, 3);
"#,
    );
    assert_eq!(out, "65536|xxx|xxx");
}

/// Verifies `var_dump` formats each argument independently with correct type tags and a trailing newline per call, in source order.
#[test]
fn test_var_dump_multiple() {
    let out = compile_and_run(
        r#"<?php
var_dump(1);
var_dump("hi");
var_dump(true);
"#,
    );
    assert_eq!(out, "int(1)\nstring(2) \"hi\"\nbool(true)\n");
}

/// Regression: `var_dump` of a heterogeneous (Mixed) indexed array must emit one typed line per
/// element, not an empty body. The Mixed-array walker previously masked the value_type stamp with
/// `0xff`, leaving the COW bit set so the `== Mixed` check failed and skipped the whole body.
#[test]
fn test_var_dump_mixed_indexed_array() {
    let out = compile_and_run(
        r#"<?php
var_dump([1, "x", 2.5]);
"#,
    );
    assert_eq!(
        out,
        "array(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  string(1) \"x\"\n  [2]=>\n  float(2.5)\n}\n"
    );
}

/// `var_export` renders scalars the way PHP does: bare integers, `'…'`-quoted strings with
/// `\\`/`\'` escaping, `true`/`false`, `NULL`, and an integer-valued float gaining a `.0`.
#[test]
fn test_var_export_scalars() {
    let out = compile_and_run(
        r#"<?php
var_export(42); echo "|";
var_export(-5); echo "|";
var_export(3.5); echo "|";
var_export(1.0); echo "|";
var_export(true); echo "|";
var_export(false); echo "|";
var_export(null); echo "|";
var_export("it's a \\test");
"#,
    );
    assert_eq!(out, r"42|-5|3.5|1.0|true|false|NULL|'it\'s a \\test'");
}

/// `var_export` renders floats with PHP's `serialize_precision = -1` semantics: the
/// shortest decimal that round-trips (so `1/3` keeps 16 significant digits, not 14),
/// scientific notation with a `.0` mantissa and minimal exponent (`1.0E+17`, `1.0E-6`),
/// and `-0.0` preserved. This is distinct from the default `(string)`/`echo` precision.
#[test]
fn test_var_export_float_precision() {
    let out = compile_and_run(
        r#"<?php
var_export(0.1); echo "|";
var_export(1.0 / 3.0); echo "|";
var_export(1.5e300); echo "|";
var_export(1e17); echo "|";
var_export(1e16); echo "|";
var_export(0.000001); echo "|";
var_export(1234567890123456.0); echo "|";
var_export(-0.0); echo "|";
var_export(-123.456);
"#,
    );
    assert_eq!(
        out,
        "0.1|0.3333333333333333|1.5E+300|1.0E+17|10000000000000000.0|1.0E-6|1234567890123456.0|-0.0|-123.456"
    );
}

/// `var_export` renders arrays in PHP's parsable `array ( … )` layout: 2-space-per-level indent,
/// `key => value,` entries, integer keys bare and string keys quoted, and a nested array placed on
/// its own line. Covers the empty array and a nested associative array.
#[test]
fn test_var_export_arrays() {
    let out = compile_and_run(
        "<?php var_export([]); echo \"\\n---\\n\"; \
         var_export([1, 'two', ['a' => 1, 'b' => [10, 20]]]);",
    );
    assert_eq!(
        out,
        "array (\n)\n---\narray (\n  0 => 1,\n  1 => 'two',\n  2 => \n  array (\n    'a' => 1,\n    'b' => \n    array (\n      0 => 10,\n      1 => 20,\n    ),\n  ),\n)"
    );
}

/// `var_export($value, true)` returns the rendered string instead of printing it, and
/// `function_exists('var_export')` sees the injected function. The unused-on-echo return is null.
#[test]
fn test_var_export_return_mode_and_function_exists() {
    let out = compile_and_run(
        r#"<?php
echo function_exists("var_export") ? "Y" : "N";
echo "|";
$s = var_export([1, 2], true);
echo $s;
"#,
    );
    assert_eq!(out, "Y|array (\n  0 => 1,\n  1 => 2,\n)");
}

/// A user-defined `var_export` wins over the injected prelude (the prelude must detect the
/// declaration and skip injection, so there is no redeclaration error).
#[test]
fn test_var_export_user_definition_wins() {
    let out = compile_and_run(
        r#"<?php
function var_export($value, $return = false) { return "custom"; }
echo var_export(123, true);
"#,
    );
    assert_eq!(out, "custom");
}

/// `var_export` usage inside a PSR-4 autoloaded file is detected after the pipeline move
/// (injection now runs after `autoload::run`), the prelude is injected, and the bare
/// namespaced `var_export(...)` call inside `App\Dumper` resolves to the injected global
/// via the name_resolver prelude-global fallback. This is the regression that previously
/// produced "Undefined function: var_export" for autoloaded Symfony files.
///
/// This is a pipeline-level (type-check) assertion rather than a full `compile_and_run_files`
/// end-to-end run: the legacy direct-AST `codegen::generate` path used by the multi-file
/// helper does not lower `is_array` (a builtin the prelude body calls) and so fails at link
/// time independently of this fix. Asserting `types::check_with_target` succeeds proves the
/// prelude was injected and the call resolved at the pipeline stage the move targets, without
/// coupling to the unrelated legacy-codegen `is_array` gap.
#[test]
fn test_var_export_in_autoloaded_file_is_injected() {
    use crate::support::target;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicU64, Ordering};

    static LOCAL_ID: AtomicU64 = AtomicU64::new(0);
    let id = LOCAL_ID.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "elephc_var_export_autoload_{}_{}",
        std::process::id(),
        id,
    ));
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    fs::write(
        dir.join("src/Dumper.php"),
        "<?php\nnamespace App;\nclass Dumper {\n    public static function dump(mixed $v): string { return var_export($v, true); }\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.php"),
        "<?php\necho App\\Dumper::dump([1, 2]);\n",
    )
    .unwrap();

    let php_path = dir.join("main.php");
    let base_dir = php_path.parent().unwrap();
    let source = fs::read_to_string(&php_path).unwrap();
    let tokens = elephc::lexer::tokenize(&source).expect("tokenize failed");
    let ast = elephc::parser::parse(&tokens).expect("parse failed");
    let ast = elephc::magic_constants::substitute_file_and_scope_constants(ast, &php_path);
    let define_set: HashSet<String> = HashSet::new();
    let ast = elephc::conditional::apply(ast, &define_set);
    let (autoload_registry, ast) = elephc::autoload::Registry::build(base_dir, ast);
    let resolved = elephc::resolver::resolve(ast, base_dir).expect("resolve failed");
    let resolved = elephc::autoload::collect_aliases(resolved);
    let resolved = elephc::name_resolver::resolve(resolved).expect("name resolve failed");
    let (resolved, _warnings) =
        elephc::autoload::run(resolved, base_dir, &autoload_registry).expect("autoload failed");
    let resolved = elephc::resolver::hoist_conditional_function_declarations(resolved);
    // The fix under test: inject AFTER autoload::run + hoist so the autoloaded-file
    // usage is detected and the declaration is present before the type checker.
    let resolved = elephc::var_export_prelude::inject_if_used(resolved);
    let resolved = elephc::optimize::fold_constants(resolved);
    let check_result = elephc::types::check_with_target(&resolved, target());
    let _ = fs::remove_dir_all(&dir);
    match check_result {
        Ok(_) => {}
        Err(e) => panic!(
            "type check failed for autoloaded var_export usage: {}",
            e.message
        ),
    }
}

// The "only when used" guard (no injection when the program has no `var_export` usage)
// is verified at the function level in `src/var_export_prelude.rs::tests::no_injection_when_unused`,
// since a runtime `function_exists($non_literal_name)` probe is not supported by the EIR
// backend and a `"var_export"` string literal would itself trigger detection.

// --- File I/O: CSV, timestamps, directory listing, temp files, seek/rewind/eof ---
