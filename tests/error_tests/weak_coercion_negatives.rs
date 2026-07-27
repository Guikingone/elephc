//! Purpose:
//! Diagnostic regression tests locking the value-boundary coercions that MUST stay loud: the weak
//! argument/return coercions added in `weak_boundary_coercion_accepts` deliberately exclude these
//! because the call/return codegen boundary cannot realize them with defined, memory-safe semantics
//! (or PHP itself raises a `TypeError`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each snippet must still be rejected by the type checker; a silent acceptance would risk a
//!   SIGSEGV, a link failure, or a value diverging from PHP.

use super::*;

/// A non-Stringable object into a `string` parameter stays loud (PHP raises a `TypeError`; there is
/// no `__toString()` to dispatch).
#[test]
fn test_non_stringable_object_into_string_parameter_stays_loud() {
    expect_error(
        r#"<?php
class Bare {}
function f(string $s): string { return $s; }
echo f(new Bare());
"#,
        "expects Str, got Object",
    );
}

/// An `array` into a `string` parameter stays loud (PHP raises a `TypeError`; `array`→`string` is
/// not a value coercion, only a warned `"Array"` in string CONTEXTS, never at a typed boundary).
#[test]
fn test_array_into_string_parameter_stays_loud() {
    expect_error(
        r#"<?php
function f(string $s): string { return $s; }
echo f([1, 2, 3]);
"#,
        "expects Str, got Array",
    );
}

/// An abstract-base Stringable static type into a plain `string` parameter stays loud: the eager
/// `(string)$obj` cast would emit a DIRECT call to the abstract `__toString` symbol, which has no
/// body (a link failure). Concrete Stringable types are accepted; abstract ones are not.
#[test]
fn test_abstract_stringable_into_string_parameter_stays_loud() {
    expect_error(
        r#"<?php
abstract class Base { abstract public function __toString(): string; }
class Impl extends Base { public function __toString(): string { return "impl"; } }
function f(string $s): string { return $s; }
function make(): Base { return new Impl(); }
echo f(make());
"#,
        "expects Str, got Object(\"Base\")",
    );
}

/// A nullable-int union SOURCE (`int|null`, a TAGGED-SCALAR representation) into a boxed-`Mixed`
/// union parameter stays loud: the two representations differ, and the boundary emits no re-boxing,
/// so copying the tagged scalar into a `Mixed` slot would be read as a pointer (a crash).
#[test]
fn test_nullable_int_union_source_into_boxed_union_stays_loud() {
    expect_error(
        r#"<?php
function h(string|array|null $v): string { return is_array($v) ? "a" : ($v ?? "N"); }
function src(bool $b): int|null { return $b ? 5 : null; }
echo h(src(true));
"#,
        "got Union([Int, Void])",
    );
}

/// A union of unrelated object types into a concrete object parameter stays loud: object property
/// access is by static offset, so a sibling class would read the wrong offset (a SIGSEGV risk).
/// PHP raises a `TypeError` here too.
#[test]
fn test_object_union_into_sibling_object_parameter_stays_loud() {
    expect_error(
        r#"<?php
class RouteCollection {}
class Route {}
function add(RouteCollection $c): void {}
function getIt(bool $b): RouteCollection|Route { return $b ? new RouteCollection() : new Route(); }
add(getIt(true));
"#,
        "expects Object(\"RouteCollection\"), got Union([Object(\"RouteCollection\"), Object(\"Route\")])",
    );
}

/// A scalar into a scalar-free union (`array|null`) stays loud: there is no scalar member for the
/// value to weak-coerce to, so PHP raises a `TypeError`.
#[test]
fn test_scalar_into_scalar_free_union_stays_loud() {
    expect_error(
        r#"<?php
function k(array|null $x): int { return 1; }
k(5);
"#,
        "expects Union([Array(Mixed), Void]), got Int",
    );
}

/// A nullable-int union (`int|null`) into a `string` parameter stays loud. Unlike `string|false`
/// and other scalar-only unions — which use a boxed-`Mixed` representation and ARE weak-coerced to
/// `string` via `__rt_mixed_cast_string` (see `codegen::arg_return_coercions`) — `int|null` uses
/// the TAGGED-SCALAR codegen representation, so the scalar-union → `string` weak coercion
/// deliberately excludes it (the `IToStr` path would mis-stringify its null case). PHP would coerce
/// the int but `TypeError` on null, so a compile-time rejection is the sound, conservative choice.
#[test]
fn test_tagged_scalar_int_null_union_into_string_stays_loud() {
    expect_error(
        r#"<?php
function src(bool $b): int|null { return $b ? 5 : null; }
function need(string $s): string { return $s; }
echo need(src(true));
"#,
        "expects Str, got Union([Int, Void])",
    );
}
