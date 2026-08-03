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

/// Tests that `declare()` with no directive is rejected with a clear diagnostic.
#[test]
fn test_error_declare_requires_directive_name() {
    expect_error(
        "<?php declare();",
        "Expected a directive name in 'declare(...)'",
    );
}

/// Tests that declare values cannot be variables because PHP requires literals.
#[test]
fn test_error_declare_value_must_be_literal() {
    expect_error(
        "<?php declare(ticks=$ticks);",
        "declare(ticks) value must be a literal",
    );
}

/// Tests that a compound expression is not accepted as a declare literal.
#[test]
fn test_error_declare_rejects_literal_expression() {
    expect_error(
        "<?php declare(ticks=1 + 0);",
        "declare(ticks) value must be a literal",
    );
}

/// Tests that callable expressions are rejected instead of being parsed and silently discarded.
#[test]
fn test_error_declare_rejects_call_value() {
    expect_error(
        "<?php declare(ticks=side_effect());",
        "declare(ticks) value must be a literal",
    );
}

/// Tests that strict_types accepts only the integer literals zero and one.
#[test]
fn test_error_declare_strict_types_requires_zero_or_one() {
    expect_error(
        "<?php declare(strict_types=2);",
        "strict_types declaration must have 0 or 1 as its value",
    );
}

/// Tests that strict_types must precede every executable or declaration statement.
#[test]
fn test_error_declare_strict_types_must_be_first() {
    expect_error(
        "<?php echo 1; declare(strict_types=1);",
        "strict_types declaration must be the very first statement in the script",
    );
}

/// Tests that strict_types cannot control a braced body.
#[test]
fn test_error_declare_strict_types_rejects_block_mode() {
    expect_error(
        "<?php declare(strict_types=1) { echo 1; }",
        "strict_types declaration must not use block mode",
    );
}

/// Tests that alternative declare syntax requires its `enddeclare` terminator.
#[test]
fn test_error_declare_alternative_syntax_requires_enddeclare() {
    expect_error(
        "<?php declare(ticks=1): echo 1;",
        "Expected 'enddeclare' after declare block",
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

/// A static *array-element* reference source (`&self::$a[$k]`, formerly a deferred slice)
/// is now supported: the aliased element reads/writes through the shared cell (verified
/// behaviorally by the references codegen tests); here the shape must simply type-check.
#[test]
fn test_reference_to_static_array_element_source_supported() {
    expect_ok(
        "<?php class C { public static array $a = [1, 2]; static function t() { $k = 0; $e = &self::$a[$k]; return $e; } } echo C::t();",
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

/// Tests that unary `~` on a concrete string rejects it with the
/// "Bitwise NOT requires integer operand" error. Concrete-string unary NOT is a pre-existing
/// gap (mirrors the binary bitwise family, whose runtime-polymorphic path only covers dynamic
/// `Mixed`/union operands, not concrete strings); only a dynamic operand routes to the runtime
/// `__rt_mixed_bitwise_not` helper (see `test_bitwise_not_mixed_operand_ok`).
#[test]
fn test_error_bitwise_not_string() {
    expect_error(
        r#"<?php echo ~"hello";"#,
        "Bitwise NOT requires integer operand",
    );
}

/// Tests that unary `~` on a concrete array operand stays a loud compile error — an array can
/// never be a bitwise operand.
#[test]
fn test_error_bitwise_not_array() {
    expect_error(
        r#"<?php $a = [1, 2]; echo ~$a;"#,
        "Bitwise NOT requires integer operand",
    );
}

/// Tests that unary `~` on a dynamic `Mixed` operand type-checks cleanly: the string-vs-integer
/// choice is deferred to the runtime `__rt_mixed_bitwise_not` helper, mirroring the binary
/// runtime-polymorphic `&`/`|`/`^` dispatch.
#[test]
fn test_bitwise_not_mixed_operand_ok() {
    expect_ok(r#"<?php function bnot(mixed $x): mixed { return ~$x; } echo bnot(5);"#);
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

/// Tests that an empty array index (`[]`) in READ position is a loud parse error rather
/// than silently reading a nonexistent element. `[]` is exclusively PHP's array-append
/// assignment TARGET marker; PHP itself rejects `echo $a[];` with "Cannot use [] for
/// reading" (verified with `php -r`), and elephc's parser matches that message exactly.
#[test]
fn test_error_empty_array_index_in_read_position() {
    expect_error("<?php $a = [1]; echo $a[];", "Cannot use [] for reading");
}

// --- Static closures ---

// --- By-reference array-literal entries ---

/// Tests that a call result cannot be a by-reference array-literal entry source
/// (`[&f()]`). PHP rejects this at the grammar level regardless of the callee's
/// signature with "Can't use function return value in write context" (verified with
/// `php -r`), and elephc's parser matches that message.
#[test]
fn test_error_ref_entry_call_source_rejected() {
    expect_error(
        "<?php function f(): int { return 1; } $a = [&f()];",
        "Can't use function return value in write context",
    );
}

/// Tests that a non-lvalue by-reference array-literal entry source (`[&1]`) is a loud
/// parse error naming the accepted shapes rather than a silent value copy.
#[test]
fn test_error_ref_entry_non_lvalue_source_rejected() {
    expect_error(
        "<?php $a = [&1];",
        "By-reference array entry source must be a variable, array element, or property",
    );
}

/// Tests that a spread inside an array literal that also has a by-reference entry is a
/// loud error: the ref-bearing literal desugars to per-entry statements, and no statement
/// form preserves a spread's string keys, so silence would mis-lower.
#[test]
fn test_error_ref_entry_literal_with_spread_rejected() {
    expect_error(
        "<?php $xs = [1, 2]; $v = 3; $a = [...$xs, &$v];",
        "Spread (...) inside an array literal with a by-reference entry is not supported",
    );
}

/// Tests that a string-valued by-reference entry source keeps the committed SLICE-2 loud
/// error (a kind-6 cell holds one value word, so a `{ptr,len}` string source would drop
/// its length): the literal desugar must not turn it into a silent value copy.
#[test]
fn test_error_ref_entry_string_valued_source_stays_loud() {
    expect_error(
        "<?php $s = \"hi\"; $a = [\"k\" => &$s];",
        "Reference to a string-valued source in a local array element is not yet supported",
    );
}

/// Tests that a `global`-imported by-reference entry source is a loud error: a global
/// lives in program-global storage, not a frame slot, and the kind-6 cell machinery only
/// adopts frame locals today, so binding one would silently read stale data.
#[test]
fn test_error_ref_entry_global_imported_source_rejected() {
    expect_error(
        "<?php function go(): void { global $cfg; $t = [\"c\" => &$cfg]; }\n$cfg = [1];\ngo();",
        "Reference to a superglobal or global-imported source in a local array element is not yet supported",
    );
}

/// Tests that a parenthesized by-reference array-literal entry source (`[&($v)]`) is a
/// loud parse error: PHP's grammar only accepts a variable-rooted token immediately after
/// `&` in this position and parse-rejects the parenthesized form (verified with `php -l`).
#[test]
fn test_error_ref_entry_parenthesized_source_rejected() {
    expect_error(
        "<?php $v = 1; $a = [&($v)];",
        "By-reference array entry source must be a variable, array element, or property",
    );
}

/// Tests that a by-reference PARAMETER as a ref-entry source is a loud error: its slot holds
/// the caller-provided raw reference address, not a kind-6 cell, so adopting it would alias
/// pointer garbage through the entry instead of the caller's value.
#[test]
fn test_error_ref_entry_byref_param_source_rejected() {
    expect_error(
        "<?php function mk(int &$x): array { return ['s' => &$x]; }\n$v = 13;\n$q = mk($v);",
        "Reference to a by-reference parameter or by-ref capture in a local array element is not yet supported",
    );
}

/// Tests that a by-reference `use` CAPTURE as a ref-entry source is a loud error: like a
/// by-ref parameter, the capture slot carries a raw reference address that the kind-6 cell
/// machinery cannot adopt.
#[test]
fn test_error_ref_entry_byref_capture_source_rejected() {
    expect_error(
        "<?php $b = 5;\n$f = function () use (&$b): array { return ['s' => &$b]; };\n$q = $f();",
        "Reference to a by-reference parameter or by-ref capture in a local array element is not yet supported",
    );
}

/// Tests that a by-reference element in an expression-position destructuring pattern
/// (`if ([&$x] = $arr)`) stays a loud parse error — by-ref destructuring is unsupported
/// (statement form included), and the expression desugar must inherit that loudly.
#[test]
fn test_error_expr_destructure_by_ref_element_rejected() {
    expect_error(
        "<?php $arr = [1, 2]; if ([&$x] = $arr) { echo $x; }",
        "Unexpected token: Ampersand",
    );
}

/// Tests that mixing keyed and unkeyed entries in an expression-position destructuring
/// pattern is rejected with the same diagnostic as the statement form (PHP fatals on the
/// mix as well).
#[test]
fn test_error_expr_destructure_mixed_keyed_unkeyed_rejected() {
    expect_error(
        "<?php $arr = [1, 2]; if ([$a, \"k\" => $b] = $arr) { echo 1; }",
        "Cannot mix keyed and unkeyed list entries",
    );
}

/// Tests that an empty destructuring pattern in expression position (`[] = $x`) is
/// rejected like PHP's "Cannot use empty list" fatal.
#[test]
fn test_error_expr_destructure_empty_pattern_rejected() {
    expect_error(
        "<?php $x = [1, 2]; if ([] = $x) { echo 1; }",
        "Cannot use empty list",
    );
}

/// Tests that the SLICE-2 one-word-cell rule is enforced by WORD COUNT, not by naming `Str`.
///
/// A kind-6 reference cell holds a single inner value word. `Str` (pointer + length) was the only
/// multi-word source the guard named, but `TaggedScalar` — the representation of `?int`, an inline
/// payload + tag pair — is equally multi-word, and it slipped through to codegen where
/// `runtime_value_tag` has no static tag for a runtime-tagged type and hit its `unreachable!`,
/// crashing the compiler on valid PHP. The refusal must name the type so the diagnostic is
/// actionable.
#[test]
fn test_error_ref_entry_nullable_int_source_is_refused_by_word_count() {
    expect_error(
        "<?php function f(?int $p): void { $a = []; $a['x'] = &$p; }",
        "Reference to a multi-word source (int|null) in a local array element is not yet supported",
    );
}
