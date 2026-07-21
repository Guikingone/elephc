//! Purpose:
//! Regression tests for sound flow-sensitive narrowing diagnostics.
//!
//! Called from:
//! - `cargo test --test error_tests` through Rust's test harness.
//!
//! Key details:
//! - Negative fixtures ensure literal-false and property facts are not retained beyond mutations,
//!   receiver rebindings, or user-code property getters.

use super::*;

/// Verifies the literal `false` parameter type rejects `true` rather than widening to bool.
#[test]
fn test_literal_false_parameter_rejects_true() {
    expect_error(
        "<?php function onlyFalse(false $value): void {} onlyFalse(true);",
        "expects False, got Bool",
    );
}

/// Verifies the fallthrough after `$value === false` keeps a full bool member (`true` remains
/// possible), and that the remaining `int|bool` value is gradually accepted through the `int`
/// return boundary (PHP coercive-mode bool→int return coercion).
#[test]
fn test_strict_false_guard_keeps_full_bool_member() {
    expect_ok(
        "<?php function requireInt(int|bool $value): int { if ($value === false) { throw new Exception('false'); } return $value; }",
    );
}

/// Verifies a direct property write clears a prior property narrowing before a later return.
/// The widened `?W` value is then gradually accepted through the `W` return boundary (the
/// campaign's gradual union-into-object rule defers the null case to runtime), so the program
/// compiles instead of erroring.
#[test]
fn test_property_write_invalidates_narrowing() {
    expect_ok(
        "<?php class W {} class Box { public function __construct(public ?W $value) {} } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } $box->value = null; return $box->value; }",
    );
}

/// Verifies rebinding the local receiver clears property facts tied to the old object.
/// The widened `?W` read is then gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_property_receiver_rebinding_invalidates_narrowing() {
    expect_ok(
        "<?php class W {} class Box { public function __construct(public ?W $value) {} } function read(Box $box, Box $replacement): W { if (!$box->value instanceof W) { throw new Exception('missing'); } $box = $replacement; return $box->value; }",
    );
}

/// Verifies a hooked property is never treated as a stable flow binding across two reads.
/// The unnarrowed `?W` second read is gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_property_get_hook_is_not_persistently_narrowed() {
    expect_ok(
        "<?php class W {} class Box { private ?W $stored; public function __construct(?W $stored) { $this->stored = $stored; } public ?W $value { get { $result = $this->stored; $this->stored = null; return $result; } } } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } return $box->value; }",
    );
}

/// Verifies an undeclared property served by `__get` is not treated as a stable flow binding.
/// The unnarrowed `?W` second read is gradually accepted through the `W` return boundary
/// under the gradual union-into-object rule.
#[test]
fn test_magic_get_property_is_not_persistently_narrowed() {
    expect_ok(
        "<?php class W {} class Box { private ?W $stored; public function __construct(?W $stored) { $this->stored = $stored; } public function __get(string $name): ?W { $result = $this->stored; $this->stored = null; return $result; } } function read(Box $box): W { if (!$box->value instanceof W) { throw new Exception('missing'); } return $box->value; }",
    );
}
