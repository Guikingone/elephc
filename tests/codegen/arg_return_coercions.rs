//! Purpose:
//! Integration regression tests for PHP weak-mode value-boundary coercions at call arguments and
//! returns: a concrete scalar or Stringable object flowing into a `string` parameter/return, and a
//! scalar/Stringable/union flowing into a boxed-`Mixed` union parameter/return.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and stdout is asserted against PHP's value.
//!   These lock the coercions added to `weak_boundary_coercion_accepts` (checker) plus the
//!   `string`-cast argument coercion in `coerce_scalar_arg_to_param_storage`/`coerce_operands_to_params`.

use crate::support::*;

/// A concrete `int` argument coerces to a `string` parameter (`(string)$int` == `IToStr`).
#[test]
fn test_int_argument_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
function f(string $s): string { return "[$s]"; }
echo f(42), f(7);
"#,
    );
    assert_eq!(out, "[42][7]");
}

/// A concrete `float` argument coerces to a `string` parameter.
#[test]
fn test_float_argument_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
function f(string $s): string { return $s; }
echo f(1.5), ":", f(2.0);
"#,
    );
    assert_eq!(out, "1.5:2");
}

/// A Stringable object argument coerces to a `string` parameter through `__toString()`.
#[test]
fn test_stringable_object_argument_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
class Name { public function __construct(private string $n) {} public function __toString(): string { return $this->n; } }
function f(string $s): string { return "<$s>"; }
$v = new Name("hi");
echo f($v), f(new Name("fresh"));
"#,
    );
    assert_eq!(out, "<hi><fresh>");
}

/// A Stringable object that inherits `__toString()` from a concrete parent still coerces.
#[test]
fn test_inherited_stringable_object_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
class C { public function __toString(): string { return "C"; } }
class D extends C {}
function f(string $s): string { return "[$s]"; }
echo f(new D());
"#,
    );
    assert_eq!(out, "[C]");
}

/// A Stringable object return coerces to a declared `string` return type through `__toString()`.
#[test]
fn test_stringable_object_coerces_to_string_return() {
    let out = compile_and_run(
        r#"<?php
class Name { public function __construct(private string $n) {} public function __toString(): string { return $this->n; } }
function g(): string { return new Name("ret"); }
echo g();
"#,
    );
    assert_eq!(out, "ret");
}

/// A concrete `int` coerces into a `string|null` union parameter (boxed and coerced at use).
#[test]
fn test_int_argument_coerces_to_string_union_parameter() {
    let out = compile_and_run(
        r#"<?php
function g(string|null $x): string { return $x ?? "null"; }
echo g(42), ":", g(0);
"#,
    );
    assert_eq!(out, "42:0");
}

/// A Stringable object coerces into a union parameter that offers `string` (boxed, coerced at use).
#[test]
fn test_stringable_object_coerces_to_string_union_parameter() {
    let out = compile_and_run(
        r#"<?php
class Name { public function __construct(public string $n) {} public function __toString(): string { return $this->n; } }
function f(string|array $x): string { if (is_array($x)) { return "arr"; } return "str:$x"; }
echo f(new Name("hi"));
"#,
    );
    assert_eq!(out, "str:hi");
}

/// Concrete `float`/`bool`/`false` scalars box into a union with a `string` member and coerce at use
/// (notably `false` stringifies to `""`, matching PHP, unlike an eager int-like `(string)false`).
#[test]
fn test_scalar_arguments_box_into_string_union_parameter() {
    let out = compile_and_run(
        r#"<?php
function h(string|array|null $v): string {
    if (is_array($v)) { return "a"; }
    if ($v === null) { return "N"; }
    return "v:$v";
}
echo h(3.5), "|", h(true), "|", h(false), "|", h("s");
"#,
    );
    assert_eq!(out, "v:3.5|v:1|v:|v:s");
}

/// The real Symfony `AbstractString::startsWith(string|iterable $prefix)` shape: an
/// abstract-base-typed Stringable object (`AbstractString`, concrete `ByteString` at runtime) boxes
/// into the union slot and dispatches `__toString()` virtually at use, matching PHP.
#[test]
fn test_abstract_typed_stringable_object_into_string_iterable_union() {
    let out = compile_and_run(
        r#"<?php
abstract class AbstractString { abstract public function __toString(): string; }
class ByteString extends AbstractString {
    public function __construct(private string $s) {}
    public function __toString(): string { return $this->s; }
}
function startsWith(string|iterable $prefix): string {
    if (is_iterable($prefix)) { return "iter"; }
    return "prefix=$prefix";
}
function make(): AbstractString { return new ByteString("hello"); }
echo startsWith(make()), "|", startsWith(new ByteString("world")), "|", startsWith(["a", "b"]);
"#,
    );
    assert_eq!(out, "prefix=hello|prefix=world|iter");
}

/// The coercions are refcount-clean across a loop: the object/int `string` casts release their
/// owned temporaries so no descriptor leaks. (Compiles and runs; a leak would trip heap-debug.)
#[test]
fn test_string_coercions_are_refcount_clean_in_loop() {
    let out = compile_and_run(
        r#"<?php
class Ref { public function __construct(private string $id) {} public function __toString(): string { return $this->id; } }
function need(string $s): string { return $s; }
$acc = "";
for ($i = 0; $i < 3; $i++) { $acc .= need(new Ref("r$i")) . need($i); }
echo $acc;
"#,
    );
    assert_eq!(out, "r00r11r22");
}

/// A `string|false` value coerces into a `string` RETURN: PHP weak-mode maps `false` to `""` and
/// passes a string through. This is Symfony's non-`strict_types` Dotenv/Path/Yaml boundary shape.
/// The union is a boxed `Mixed`, so the return boundary emits `__rt_mixed_cast_string`, whose bool
/// arm yields `""` for `false` — matching `php -n` exactly.
#[test]
fn test_string_false_union_coerces_to_string_return() {
    let out = compile_and_run(
        r#"<?php
function raw(bool $b): string|false { return $b ? "hi" : false; }
function norm(bool $b): string { return raw($b); }
$s = norm(false);
echo norm(true), "|", $s, "|", strlen($s), "\n";
"#,
    );
    assert_eq!(out, "hi||0\n");
}

/// A `string|false` value coerces into a `string` PARAMETER (the call-argument boundary emits the
/// same `__rt_mixed_cast_string`), `false` becoming `""`.
#[test]
fn test_string_false_union_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
function src(bool $b): string|false { return $b ? "hi" : false; }
function need(string $s): string { return "[$s]"; }
echo need(src(true)), need(src(false)), "end";
"#,
    );
    assert_eq!(out, "[hi][]end");
}

/// An `int|false` scalar union coerces into a `string` parameter: the `int` stringifies to its
/// decimal and `false` to `""`, both via the boxed-`Mixed` cast dispatch.
#[test]
fn test_int_false_union_coerces_to_string_parameter() {
    let out = compile_and_run(
        r#"<?php
function src(bool $b): int|false { return $b ? 42 : false; }
function need(string $s): string { return "<$s>"; }
echo need(src(true)), need(src(false));
"#,
    );
    assert_eq!(out, "<42><>");
}

/// A scalar union with NO `string` member (`int|float`) still coerces into a `string` parameter:
/// each scalar stringifies through the boxed-`Mixed` cast (`int`→decimal, `float`→`ftoa`).
/// (Loud-rejection guards for array/object members and the TaggedScalar `int|null` exclusion live
/// in `error_tests::weak_coercion_negatives`.)
#[test]
fn test_scalar_union_without_string_member_coerces_to_string() {
    let out = compile_and_run(
        r#"<?php
function src(bool $b): int|float { return $b ? 3 : 2.5; }
function need(string $s): string { return "<$s>"; }
echo need(src(true)), need(src(false));
"#,
    );
    assert_eq!(out, "<3><2.5>");
}

/// A value-returning body that runs off its end raises PHP's catchable `TypeError`, with PHP's
/// exact message — fully-qualified function name, fully-qualified declared type — rather than
/// being refused at compile time or returning a fabricated value.
///
/// All three expectations are `php -n` 8.5 output. `crate::return_type_guard` synthesizes the
/// throw; without it, `ir_lower::function::terminate_open_block` would hand the caller a
/// placeholder value for the fall-through path.
#[test]
fn test_falling_off_a_value_returning_function_raises_phps_type_error() {
    let out = compile_and_run(
        r#"<?php
namespace App\Sub;

class Item { public int $n = 7; }

function firstOf(array $items): Item {
    foreach ($items as $item) {
        return $item;
    }
}

class Holder {
    public function pick(bool $ok): int { if ($ok) { return 5; } }
}

echo firstOf([new Item()])->n, "|";
try { firstOf([]); } catch (\TypeError $e) { echo $e->getMessage(), "|"; }
echo (new Holder())->pick(true), "|";
try { (new Holder())->pick(false); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "7|App\\Sub\\firstOf(): Return value must be of type App\\Sub\\Item, none returned|\
         5|App\\Sub\\Holder::pick(): Return value must be of type int, none returned"
    );
}

/// A GENERATOR legitimately runs off its end and PHP raises nothing, so no guard is synthesized.
/// Without the `body_contains_yield` exclusion this would throw where PHP returns cleanly.
#[test]
fn test_generator_running_off_its_end_raises_nothing() {
    let out = compile_and_run(
        r#"<?php
function gen(): \Generator { yield 1; yield 2; }
$total = 0;
foreach (gen() as $v) { $total += $v; }
echo $total;
"#,
    );
    assert_eq!(out, "3");
}
