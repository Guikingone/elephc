//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of casts, constants, and introspection introspection, including gettype integer, gettype float, and gettype string.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests that `gettype(42)` returns "integer".
#[test]
fn test_gettype_int() {
    let out = compile_and_run("<?php echo gettype(42);");
    assert_eq!(out, "integer");
}

/// Tests that `gettype(3.14)` returns "double" (PHP's float type name).
#[test]
fn test_gettype_float() {
    let out = compile_and_run("<?php echo gettype(3.14);");
    assert_eq!(out, "double");
}

/// Tests that `gettype("hi")` returns "string".
#[test]
fn test_gettype_string() {
    let out = compile_and_run("<?php echo gettype(\"hi\");");
    assert_eq!(out, "string");
}

/// Tests that `gettype(true)` returns "boolean".
#[test]
fn test_gettype_bool() {
    let out = compile_and_run("<?php echo gettype(true);");
    assert_eq!(out, "boolean");
}

/// Tests that `gettype(null)` returns "NULL".
#[test]
fn test_gettype_null() {
    let out = compile_and_run("<?php echo gettype(null);");
    assert_eq!(out, "NULL");
}

/// Tests that `gettype` on a mixed value returns the concrete payload type
/// (integer, string, NULL, array, boolean) rather than "mixed".
#[test]
fn test_gettype_mixed_returns_concrete_payload_type() {
    let out = compile_and_run(
        r#"<?php
$map = [
    "i" => 42,
    "s" => "hi",
    "n" => null,
    "a" => [1, 2],
    "b" => true,
];
echo gettype($map["i"]);
echo "|";
echo gettype($map["s"]);
echo "|";
echo gettype($map["n"]);
echo "|";
echo gettype($map["a"]);
echo "|";
echo gettype($map["b"]);
"#,
    );
    assert_eq!(out, "integer|string|NULL|array|boolean");
}

// --- empty ---

/// Tests that `empty(0)` is true (0 is falsy in PHP).
#[test]
fn test_empty_zero() {
    let out = compile_and_run("<?php echo empty(0);");
    assert_eq!(out, "1");
}

/// Tests that `empty(42)` is false (non-zero int is truthy).
#[test]
fn test_empty_nonzero() {
    let out = compile_and_run("<?php echo empty(42);");
    assert_eq!(out, "");
}

/// Tests that `empty("")` is true (empty string is falsy).
#[test]
fn test_empty_empty_string() {
    let out = compile_and_run("<?php echo empty(\"\");");
    assert_eq!(out, "1");
}

/// Tests that `empty("hi")` is false (non-empty string is truthy).
#[test]
fn test_empty_nonempty_string() {
    let out = compile_and_run("<?php echo empty(\"hi\");");
    assert_eq!(out, "");
}

/// Tests that `empty(null)` is true.
#[test]
fn test_empty_null() {
    let out = compile_and_run("<?php echo empty(null);");
    assert_eq!(out, "1");
}

/// Tests that `empty(false)` is true.
#[test]
fn test_empty_false() {
    let out = compile_and_run("<?php echo empty(false);");
    assert_eq!(out, "1");
}

/// Tests that `empty(true)` is false.
#[test]
fn test_empty_true() {
    let out = compile_and_run("<?php echo empty(true);");
    assert_eq!(out, "");
}

/// Tests that `empty` on a mixed-valued associative array uses boxed payload
/// semantics (zeros/blank/null/empty-array are falsy; non-zeros/non-blank are truthy).
#[test]
fn test_empty_mixed_uses_boxed_payload_semantics() {
    let out = compile_and_run(
        r#"<?php
$map = [
    "zero" => 0,
    "blank" => "",
    "null" => null,
    "arr" => [],
    "one" => 1,
    "text" => "hi",
];
echo empty($map["zero"]) ? "1" : "0";
echo empty($map["blank"]) ? "1" : "0";
echo empty($map["null"]) ? "1" : "0";
echo empty($map["arr"]) ? "1" : "0";
echo empty($map["one"]) ? "1" : "0";
echo empty($map["text"]) ? "1" : "0";
"#,
    );
    assert_eq!(out, "111100");
}

// --- unset ---

/// Tests that `unset` marks a variable as undefined so `is_null` returns true.
#[test]
fn test_unset_variable() {
    let out = compile_and_run(
        r#"<?php
$x = 42;
unset($x);
echo is_null($x);
"#,
    );
    assert_eq!(out, "1");
}

// --- settype ---

/// Tests that `settype($x, "string")` converts an integer to a string.
#[test]
fn test_settype_to_string() {
    let out = compile_and_run(
        r#"<?php
$x = 42;
settype($x, "string");
echo $x;
"#,
    );
    assert_eq!(out, "42");
}

/// Tests that `settype($x, "integer")` truncates a float to an integer.
#[test]
fn test_settype_to_int() {
    let out = compile_and_run(
        r#"<?php
$x = 3.7;
settype($x, "integer");
echo $x;
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies `get_debug_type()` returns PHP 8's short scalar type names for statically typed values.
#[test]
fn test_get_debug_type_scalars() {
    let out = compile_and_run(
        r#"<?php
echo get_debug_type(5) . "|" . get_debug_type(1.5) . "|" . get_debug_type("s")
    . "|" . get_debug_type(true) . "|" . get_debug_type(null) . "|" . get_debug_type([1, 2]);
"#,
    );
    assert_eq!(out, "int|float|string|bool|null|array");
}

/// Verifies `get_debug_type()` returns the class name for an object (here a statically typed one).
#[test]
fn test_get_debug_type_object_returns_class_name() {
    let out = compile_and_run(
        r#"<?php
class Mailer {}
echo get_debug_type(new Mailer());
"#,
    );
    assert_eq!(out, "Mailer");
}

/// Verifies `get_debug_type()` dispatches on the runtime tag of a boxed Mixed value, including the
/// class name when the Mixed holds an object.
#[test]
fn test_get_debug_type_mixed_values() {
    let out = compile_and_run(
        r#"<?php
class Widget {}
$bag = ["i" => 5, "s" => "x", "f" => 2.5, "b" => false, "n" => null, "arr" => [1], "o" => new Widget()];
echo get_debug_type($bag["i"]) . "|" . get_debug_type($bag["s"]) . "|" . get_debug_type($bag["f"])
    . "|" . get_debug_type($bag["b"]) . "|" . get_debug_type($bag["n"]) . "|" . get_debug_type($bag["arr"])
    . "|" . get_debug_type($bag["o"]);
"#,
    );
    assert_eq!(out, "int|string|float|bool|null|array|Widget");
}

/// Verifies `get_debug_type()` returns the runtime class name of a dynamically constructed object
/// (whose static type is Mixed), and that the name is case-insensitive like other builtins.
#[test]
fn test_get_debug_type_dynamic_new_and_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
class Service {}
$c = "Service";
$o = new $c();
echo get_debug_type($o) . "|" . GET_DEBUG_TYPE(7);
"#,
    );
    assert_eq!(out, "Service|int");
}

// --- Missing type function tests ---

// --- Non-literal defined()/constant()/enum_exists() registry lookups ---

/// Verifies a non-literal `defined($name)` resolves a builtin constant name
/// through the runtime registry and reports it as present. The name is derived
/// from `$argc` so it stays non-literal and is not folded at compile time.
#[test]
fn test_defined_nonliteral_known_constant_is_true() {
    let out = compile_and_run(
        r#"<?php
$name = $argc > 0 ? "PHP_INT_SIZE" : "NOPE";
echo defined($name) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "yes");
}

/// Verifies a non-literal `defined($name)` reports an undefined constant as
/// absent via the runtime registry lookup.
#[test]
fn test_defined_nonliteral_unknown_constant_is_false() {
    let out = compile_and_run(
        r#"<?php
$name = $argc > 0 ? "ELEPHC_TOTALLY_UNDEFINED_XYZ" : "NOPE";
echo defined($name) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "no");
}

/// Verifies a non-literal `constant($name)` returns the value of a user-declared
/// scalar constant resolved through the runtime registry.
#[test]
fn test_constant_nonliteral_returns_scalar_value() {
    let out = compile_and_run(
        r#"<?php
const ELEPHC_ANSWER = 42;
$name = $argc > 0 ? "ELEPHC_ANSWER" : "NOPE";
echo constant($name);
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies a non-literal `constant($name)` returns the value of a user-declared
/// string constant, exercising the boxed-string registry payload and runtime
/// `__rt_str_persist` ownership path.
#[test]
fn test_constant_nonliteral_returns_string_value() {
    let out = compile_and_run(
        r#"<?php
const ELEPHC_GREETING = "hi there";
$name = $argc > 0 ? "ELEPHC_GREETING" : "NOPE";
echo constant($name);
"#,
    );
    assert_eq!(out, "hi there");
}

/// Verifies a non-literal `constant($name)` for an undefined constant throws a
/// catchable `\Error` carrying the PHP 8 `Undefined constant "X"` message.
#[test]
fn test_constant_nonliteral_miss_throws_error() {
    let out = compile_and_run(
        r#"<?php
$name = $argc > 0 ? "ELEPHC_NO_SUCH_CONST" : "NOPE";
try {
    echo constant($name);
} catch (\Error $e) {
    echo "caught:" . $e->getMessage();
}
"#,
    );
    assert_eq!(out, "caught:Undefined constant \"ELEPHC_NO_SUCH_CONST\"");
}

/// Verifies a non-literal `enum_exists($name)` finds a declared enum through the
/// runtime registry lookup.
#[test]
fn test_enum_exists_nonliteral_known_enum_is_true() {
    let out = compile_and_run(
        r#"<?php
enum Suit { case Hearts; case Spades; }
$name = $argc > 0 ? "Suit" : "NOPE";
echo enum_exists($name) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "yes");
}

/// Verifies a non-literal `enum_exists($name)` reports an undeclared enum as
/// absent.
#[test]
fn test_enum_exists_nonliteral_unknown_enum_is_false() {
    let out = compile_and_run(
        r#"<?php
enum Suit { case Hearts; }
$name = $argc > 0 ? "ElephcNoSuchEnum" : "NOPE";
echo enum_exists($name) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "no");
}

/// Verifies the string-literal `defined('PHP_INT_SIZE')` fast path still folds to
/// a compile-time boolean (true) with no behavior change.
#[test]
fn test_defined_literal_known_constant_still_folds_true() {
    let out = compile_and_run("<?php echo defined('PHP_INT_SIZE') ? 'yes' : 'no';");
    assert_eq!(out, "yes");
}

/// Verifies the string-literal `defined('UNKNOWN')` fast path still folds to a
/// compile-time boolean (false).
#[test]
fn test_defined_literal_unknown_constant_still_folds_false() {
    let out = compile_and_run("<?php echo defined('ELEPHC_NOPE_LITERAL') ? 'yes' : 'no';");
    assert_eq!(out, "no");
}

/// Verifies the string-literal `enum_exists('Suit')` fast path still folds to a
/// compile-time boolean (true).
#[test]
fn test_enum_exists_literal_still_folds_true() {
    let out = compile_and_run(
        r#"<?php
enum Suit { case Hearts; }
echo enum_exists('Suit') ? 'yes' : 'no';
"#,
    );
    assert_eq!(out, "yes");
}

/// Verifies a string-literal `constant('NAME')` for a user-declared scalar folds
/// to its value during lowering without a runtime registry lookup.
#[test]
fn test_constant_literal_folds_scalar_value() {
    let out = compile_and_run(
        r#"<?php
const ELEPHC_SEVEN = 7;
echo constant('ELEPHC_SEVEN');
"#,
    );
    assert_eq!(out, "7");
}
