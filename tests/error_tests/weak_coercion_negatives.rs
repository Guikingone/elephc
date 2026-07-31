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

/// SUPERSEDED by the checked downcast for UNION SOURCES: a union of unrelated object types into a
/// concrete object parameter is accepted, because the boundary now emits a runtime guard. Property
/// access is still by static offset — which is exactly why the guard's ok-edge re-materializes a
/// raw object pointer with `Op::ObjectCast` — but a sibling class no longer reaches that offset:
/// `instanceof RouteCollection` rules it out and throws PHP's own catchable `TypeError`, which is
/// what PHP does with this program too. Runtime coverage of both arms lives in
/// `tests/codegen/oop/checked_downcast_argument.rs`.
#[test]
fn test_object_union_into_sibling_object_parameter_accepted_with_runtime_guard() {
    expect_ok(
        r#"<?php
class RouteCollection {}
class Route {}
function add(RouteCollection $c): void {}
function getIt(bool $b): RouteCollection|Route { return $b ? new RouteCollection() : new Route(); }
add(getIt(true));
"#,
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

/// F6a: an `Obj|false` return value flows into a `?Obj`-shaped (`Obj|null`) declared return. Both
/// are boxed-`Mixed` unions and the target carries an object member, so `boxed_union_source_member_flows`
/// accepts the `false` sentinel as a memory-safe box-pointer copy (a stray `false` reaching an object
/// use fatals loudly as a PHP `TypeError`). Mirrors HeaderBag::getDate / Response::getExpires.
#[test]
fn test_object_false_union_into_nullable_object_return_accepted() {
    expect_ok(
        r#"<?php
class D {}
function make(bool $ok): D|false { return $ok ? new D() : false; }
function getIt(bool $ok): ?D { return make($ok); }
"#,
    );
}

/// F4: a `string|false` scalar union returned from a `?string`-shaped declared return is accepted —
/// the return-boundary codegen weak-coerces `false`→"" (`__rt_mixed_cast_string`). Every source
/// member is string-coercible (no null/array/object) and every target member is string/void, so the
/// coercion is sound. Mirrors AbstractDumper::dump / Filesystem::readlink.
#[test]
fn test_string_false_union_into_nullable_string_return_accepted() {
    expect_ok(
        r#"<?php
function rl(bool $ok): string|false { return $ok ? "t" : false; }
function readlink2(bool $ok): ?string { return rl($ok); }
"#,
    );
}

/// F6a/F4 sentinel: an `int|false` scalar union into an `:int` return stays loud. The target is a
/// bare scalar with no object member (so the F6a object arm does not fire) and no string member (so
/// the F4 `?string` coercion does not apply); PHP would coerce the int but `TypeError` on `false`,
/// so a compile-time rejection is the sound choice. Guards F6a/F4 against over-broadening.
#[test]
fn test_int_false_union_into_int_return_stays_loud() {
    expect_error(
        r#"<?php
function make(bool $ok): int|false { return $ok ? 5 : false; }
function getInt(bool $ok): int { return make($ok); }
echo getInt(true);
"#,
        "expects Int, got Union([Int, False])",
    );
}

/// F4 sentinel: a `string|array` union into a `?string` return stays loud. The `array` member is not
/// string-coercible, so the F4 scalar-union → `?string` coercion deliberately excludes it (a
/// boxed-array payload has no sound `string` cast); PHP raises a `TypeError`. Guards F4's
/// "every source member string-coercible" precondition.
#[test]
fn test_string_array_union_into_nullable_string_return_stays_loud() {
    expect_error(
        r#"<?php
function mk(bool $ok): string|array { return $ok ? "s" : [1, 2]; }
function g(bool $ok): ?string { return mk($ok); }
echo g(true);
"#,
        "expects Union([Str, Void]), got Union([Str, Array(Mixed)])",
    );
}
