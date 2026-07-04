//! Purpose:
//! Integration or regression tests for diagnostic coverage of misc, including bitwise compound assignment requires ints, duplicate use alias is rejected, and has line number.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

/// Tests that `&=` compound assignment rejects a string left-hand operand.
/// The error message is "Bitwise operators require integer operands".
#[test]
fn test_error_bitwise_compound_assignment_requires_ints() {
    expect_error(
        "<?php $x = \"flags\"; $x &= 1;",
        "Bitwise operators require integer operands",
    );
}

/// Tests that direct reference assignment rejects a non-variable source.
#[test]
fn test_error_reference_assignment_requires_variable_source() {
    expect_error(
        "<?php $a = 1; $b =& 1;",
        "Reference assignment source must be a variable",
    );
}

/// Tests that a reference assignment rejects a computed (non-lvalue) source such as
/// `$a + 1`, which is neither a variable, an array/property element, nor a call.
#[test]
fn test_error_reference_assignment_rejects_computed_source() {
    expect_error(
        "<?php $a = 1; $b =& $a + 1;",
        "Reference assignment source must be a variable",
    );
}

/// A plain static property is a valid reference source (`$e = &self::$n`), but a static
/// *array-element* source (`&self::$a[$k]`) is a deferred slice: it is rejected rather than
/// silently miscompiled. Asserts the deferral message stays explicit.
#[test]
fn test_error_reference_to_static_array_element_source_deferred() {
    expect_error(
        "<?php class C { public static array $a = [1, 2]; static function t() { $k = 0; $e = &self::$a[$k]; return $e; } } echo C::t();",
        "Reference assignment source must be a variable",
    );
}

/// A static-property reference source is only supported into a plain-variable target
/// (`$e = &self::$n`); aliasing it into a complex lvalue (`$this->p = &self::$n`) is a
/// deferred slice and must error cleanly instead of becoming a value copy.
#[test]
fn test_error_reference_static_property_into_complex_target_deferred() {
    expect_error(
        "<?php class C { public static int $n = 5; public int $p = 0; function t() { $this->p = &self::$n; return $this->p; } } $c = new C(); echo $c->t();",
        "Reference assignment source must be a variable",
    );
}

/// Tests that two `use` statements with the same alias name produce a
/// "Duplicate import alias" error.
#[test]
fn test_error_duplicate_use_alias_is_rejected() {
    expect_error(
        "<?php namespace App; use Lib\\One as Tool; use Lib\\Two as Tool; echo 1;",
        "Duplicate import alias: Tool",
    );
}

/// Verifies that lexer errors report the correct line number in the span.
/// The input has two newlines before the unterminated string, so the error
/// should be on line 3.
#[test]
fn test_error_has_line_number() {
    let result = tokenize("<?php\n\n\"unterminated");
    let err = result.unwrap_err();
    assert_eq!(err.span.line, 3, "Error should be on line 3");
}

/// Verifies that lexer errors carry a column number greater than zero.
#[test]
fn test_error_has_column() {
    let result = tokenize("<?php `");
    let err = result.unwrap_err();
    assert!(err.span.col > 0, "Error should have a column number");
}

/// Tests that `gettype()` with no arguments produces the expected arity error.
#[test]
fn test_error_gettype_wrong_args() {
    expect_error("<?php gettype();", "gettype() takes exactly 1 argument");
}

/// Tests that `empty()` with no arguments produces the expected arity error.
#[test]
fn test_error_empty_wrong_args() {
    expect_error("<?php empty();", "empty() takes exactly 1 argument");
}

/// Tests that `unset()` with no arguments produces the expected arity error.
#[test]
fn test_error_unset_wrong_args() {
    expect_error("<?php unset();", "unset() takes at least 1 argument");
}

/// Tests that `settype()` with only one argument produces the expected arity error.
#[test]
fn test_error_settype_wrong_args() {
    expect_error("<?php settype(42);", "settype() takes exactly 2 arguments");
}

/// Tests that `&` with a string left-hand operand rejects it with the
/// "Bitwise operators require integer operands" error.
#[test]
fn test_error_bitwise_and_string() {
    expect_error(
        r#"<?php echo "hello" & 1;"#,
        "Bitwise operators require integer operands",
    );
}

/// Tests that a string shift (`<<`/`>>`) stays on the integer path and is still rejected.
/// PHP string bitwise only covers `&`/`|`/`^`; shifts are never string operators, so
/// `"abc" << 1` must still report "Bitwise operators require integer operands" — the
/// both-string string-bitwise rule must not leak into the shift operators.
#[test]
fn test_error_bitwise_shift_string_unaffected() {
    expect_error(
        r#"<?php echo "abc" << 1;"#,
        "Bitwise operators require integer operands",
    );
}

/// Tests that unary `~` on a string rejects it with the
/// "Bitwise NOT requires integer operand" error.
#[test]
fn test_error_bitwise_not_string() {
    expect_error(
        r#"<?php echo ~"hello";"#,
        "Bitwise NOT requires integer operand",
    );
}

/// Tests that the spaceship operator `<=>` with an array operand is rejected with the
/// "Spaceship operator requires numeric or string operands" error. PHP 8 string ordering is
/// now valid (lowered through `__rt_php_compare`); array/object ordering stays rejected.
#[test]
fn test_error_spaceship_array() {
    expect_error(
        r#"<?php $a = [1]; echo $a <=> 1;"#,
        "Spaceship operator requires numeric or string operands",
    );
}

/// Tests that using `$this` inside a `static` method produces the expected
/// "Cannot use $this inside a static method" error.
#[test]
fn test_error_static_this() {
    expect_error(
        "<?php class Demo { public static function bad() { return $this; } } Demo::bad();",
        "Cannot use $this inside a static method",
    );
}

/// Tests that a child class method that changes the parameter count when
/// overriding a parent method produces the expected error. Dropping a parameter
/// (the child accepts fewer calls than the parent) is a genuine LSP violation
/// and stays rejected. Cross-checked with `php` (fatal: must be compatible).
#[test]
fn test_error_override_cannot_change_parameter_count() {
    expect_error(
        "<?php class Base { public function ping($x) { return $x; } } class Child extends Base { public function ping() { return 1; } }",
        "Cannot change parameter count when overriding method: Child::ping",
    );
}

/// Tests that a child override adding a *required* parameter over a
/// zero-parameter parent method is rejected: the parent's callers never supply
/// it, so the child accepts fewer calls. Cross-checked with `php` (fatal: must
/// be compatible).
#[test]
fn test_error_override_cannot_add_required_param() {
    expect_error(
        "<?php class Base { public function ping() { return 1; } } class Child extends Base { public function ping($x) { return $x; } }",
        "Cannot add a required parameter when overriding method: Child::ping",
    );
}

/// Tests that a child override making an *optional* parent parameter *required*
/// is rejected: callers who omit the argument would break. Cross-checked with
/// `php` (fatal: must be compatible).
#[test]
fn test_error_override_cannot_make_optional_param_required() {
    expect_error(
        "<?php class Base { public function ping($x = 1) { return $x; } } class Child extends Base { public function ping($x) { return $x; } }",
        "Cannot make an optional parameter required when overriding method: Child::ping",
    );
}

/// Tests that a child override removing the parent's variadic tail is rejected:
/// it would reject calls that pass extra arguments the parent accepts.
/// Cross-checked with `php` (fatal: must be compatible).
#[test]
fn test_error_override_cannot_remove_variadic() {
    expect_error(
        "<?php class Base { public function ping($a, ...$xs) { return $a; } } class Child extends Base { public function ping($a) { return $a; } }",
        "Cannot change variadic parameter shape when overriding method: Child::ping",
    );
}

/// Tests that a child override changing the by-reference-ness of an overlapping
/// parameter is rejected: PHP requires by-ref parameters to match exactly for
/// overlapping positions. Cross-checked with `php` (fatal: must be compatible).
#[test]
fn test_error_override_cannot_change_by_reference_param() {
    expect_error(
        "<?php class Base { public function ping(&$x) { $x = 1; } } class Child extends Base { public function ping($x) { return $x; } }",
        "Cannot change pass-by-reference parameters when overriding method: Child::ping",
    );
}

/// Tests that a hex literal with no digits after `0x` produces the expected
/// "Expected hex digits after '0x'" error.
#[test]
fn test_error_hex_no_digits() {
    expect_error("<?php echo 0x;", "Expected hex digits after '0x'");
}

// --- Mixed return type errors ---

// Note: mixed return types are now widened (Str > Float > Int) instead of
// producing an error. The test_return_type_mixed_branches codegen test
// covers the widening behavior.

// --- Math trig/log error tests ---

/// Tests that `is_null()` with no arguments produces the expected arity error.
#[test]
fn test_error_is_null_wrong_args() {
    expect_error("<?php is_null();", "is_null() takes exactly 1 argument");
}

/// Tests that reassigning a nullable typed local variable (`?int`) with a
/// string produces a "cannot reassign $value" error.
#[test]
fn test_error_nullable_typed_local_rejects_invalid_reassignment() {
    expect_error(
        "<?php ?int $value = null; $value = \"x\";",
        "cannot reassign $value",
    );
}

/// Tests that `require` with a variable as the path produces a
/// "compile-time-constant string" error.
#[test]
fn test_include_path_with_variable_errors() {
    let err = resolver_error("<?php $path = 'x'; require $path;");
    assert!(
        err.message.contains("compile-time-constant string"),
        "message did not mention compile-time-constant: {}",
        err.message
    );
}

/// Tests that `require` with a function call as the path produces a
/// "compile-time-constant string" error.
#[test]
fn test_include_path_with_function_call_errors() {
    let err = resolver_error("<?php require getenv('PATH');");
    assert!(
        err.message.contains("compile-time-constant string"),
        "message did not mention compile-time-constant: {}",
        err.message
    );
}

// --- Static closures ---
