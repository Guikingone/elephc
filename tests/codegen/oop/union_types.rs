//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP union types, including union typed local gettype and reassignment, nullable typed local null coalesce, and union typed local truthiness dispatch.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies that gettype() reflects the narrowest runtime type of a union-typed local,
/// and that reassignment to an alternate union member updates the reported type.
/// Fixture: `int|string $value = 1` → gettype returns "integer", then `$value = "two"` → gettype returns "string".
#[test]
fn test_union_typed_local_gettype_and_reassignment() {
    let out = compile_and_run(
        r#"<?php
function demo() {
    int|string $value = 1;
    echo gettype($value);
    echo ":";
    $value = "two";
    echo gettype($value);
    echo ":";
    echo $value;
}

demo();
"#,
    );
    assert_eq!(out, "integer:string:two");
}

/// Verifies null coalesce on a nullable-typed local: fallback is used when the value is null,
/// and the actual value is returned after assignment to a non-null int.
/// Fixture: `?int $value = null` → `?? 41` yields 41; then `$value = 1` → `?? 41` yields 1.
#[test]
fn test_nullable_typed_local_null_coalesce() {
    let out = compile_and_run(
        r#"<?php
function demo() {
    ?int $value = null;
    echo $value ?? 41;
    $value = 1;
    echo $value ?? 41;
}

demo();
"#,
    );
    assert_eq!(out, "411");
}

/// Verifies truthiness dispatch for a union-typed local: string "0" is falsy, int 7 is truthy.
/// Regression: ensures the codegen emits correct branch logic for both string and int payloads.
#[test]
fn test_union_typed_local_truthiness_dispatch() {
    let out = compile_and_run(
        r#"<?php
function demo() {
    int|string $value = "0";
    if ($value) {
        echo 1;
    } else {
        echo 0;
    }
    $value = 7;
    if ($value) {
        echo 1;
    } else {
        echo 0;
    }
}

demo();
"#,
    );
    assert_eq!(out, "01");
}

/// Verifies empty() dispatch for a union-typed local: string "0" is empty, string "7" is not.
/// Regression: ensures empty() correctly distinguishes falsy-but-non-empty strings from empty strings.
#[test]
fn test_union_typed_local_empty_dispatch() {
    let out = compile_and_run(
        r#"<?php
function demo() {
    int|string $value = "0";
    echo empty($value) ? 1 : 0;
    $value = "7";
    echo empty($value) ? 1 : 0;
}

demo();
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies that a union-typed property accepts an integer literal default, boxing it into the
/// property's Mixed storage. Regression: this previously failed codegen with an "object_new for
/// default value ... Union" error.
#[test]
fn test_union_property_int_literal_default() {
    let out = compile_and_run(
        "<?php class C { public int|string $v = 1; } $c = new C(); var_dump($c->v);",
    );
    assert_eq!(out, "int(1)\n");
}

/// Verifies that a union-typed property accepts a negative integer literal default.
#[test]
fn test_union_property_negative_int_default() {
    let out = compile_and_run(
        "<?php class C { public int|string $v = -7; } $c = new C(); var_dump($c->v);",
    );
    assert_eq!(out, "int(-7)\n");
}

/// Verifies that a union-typed property accepts a float literal default.
#[test]
fn test_union_property_float_literal_default() {
    let out = compile_and_run(
        "<?php class C { public float|int $v = 1.5; } $c = new C(); var_dump($c->v);",
    );
    assert_eq!(out, "float(1.5)\n");
}

/// Verifies that a union-typed property accepts a boolean literal default.
#[test]
fn test_union_property_bool_literal_default() {
    let out = compile_and_run(
        "<?php class C { public bool|int $v = true; } $c = new C(); var_dump($c->v);",
    );
    assert_eq!(out, "bool(true)\n");
}

/// Verifies that a string literal default for a union-typed property still works (it did before
/// this fix), exercising the sibling boxed-string path.
#[test]
fn test_union_property_string_literal_default() {
    let out = compile_and_run(
        "<?php class C { public string|int $v = \"hi\"; } $c = new C(); var_dump($c->v);",
    );
    assert_eq!(out, "string(2) \"hi\"\n");
}

/// Verifies method dispatch on an object-or-false union return (`A|false`): the
/// object branch dispatches the method on the single object class. Mirrors the
/// `DateTime::createFromFormat()` pattern (returns `DateTime|false`).
#[test]
fn test_object_false_union_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
class A { public int $v = 5; function who(): string { return "A"; } }
function make(bool $b): A|false { return $b ? new A() : false; }
echo make(true)->who(), make(true)->v;
"#,
    );
    assert_eq!(out, "A5");
}

/// Verifies property and method access on a nullable object union (`A|null`)
/// still works after the union-receiver relaxation (regression guard).
#[test]
fn test_object_null_union_access() {
    let out = compile_and_run(
        r#"<?php
class A { public int $v = 7; function who(): string { return "A"; } }
function make(bool $b): A|null { return $b ? new A() : null; }
echo make(true)->who(), make(true)->v;
"#,
    );
    assert_eq!(out, "A7");
}

/// Verifies runtime class-id dispatch on a union of two distinct object classes
/// (`A|B`): the method and property resolve to whichever class the value holds.
#[test]
fn test_two_class_union_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
class A { public int $v = 1; function who(): string { return "A"; } }
class B { public int $v = 2; function who(): string { return "B"; } }
function make(bool $b): A|B { return $b ? new A() : new B(); }
echo make(true)->who(), make(false)->who(), make(true)->v, make(false)->v;
"#,
    );
    assert_eq!(out, "AB12");
}

/// Verifies a three-member object union with a scalar sentinel (`A|B|false`)
/// dispatches the object branches by runtime class id.
#[test]
fn test_two_class_false_union_dispatch() {
    let out = compile_and_run(
        r#"<?php
class A { public int $v = 1; function who(): string { return "A"; } }
class B { public int $v = 2; function who(): string { return "B"; } }
function make(int $w): A|B|false { if ($w == 0) return new A(); if ($w == 1) return new B(); return false; }
echo make(0)->who(), make(1)->who(), make(0)->v, make(1)->v;
"#,
    );
    assert_eq!(out, "AB12");
}

/// Verifies PHP-faithful lenient method dispatch on a multi-class object union where only ONE
/// member declares the called method (`A|B`, only `A::who()` exists). PHP dispatches on the
/// runtime class, so this type-checks and, when the runtime value actually is an `A`, runs and
/// returns php-identically. Regression guard for the checker relaxation (SPEC G1): previously the
/// checker required the method on EVERY union member and rejected this call outright.
#[test]
fn test_union_method_dispatch_single_resolving_member_runs() {
    let out = compile_and_run(
        r#"<?php
class A { function who(): string { return "A"; } }
class B { }
function make(bool $b): A|B { return $b ? new A() : new B(); }
echo make(true)->who();
"#,
    );
    assert_eq!(out, "A");
}

/// Verifies the runtime no-match fatal for lenient union method dispatch: `A|B` where only `A`
/// declares `who()`, called on a runtime value that is actually `B`. Codegen's Mixed-receiver
/// class-id dispatch (`lower_mixed_method_call` in `codegen_ir/lower_inst.rs`) falls through to
/// the existing `emit_method_call_on_null_fatal` clean fatal — this is the soundness-gate
/// requirement from SPEC G1: never garbage/SIGSEGV, always a loud, clean, nonzero-exit failure.
#[test]
fn test_union_method_dispatch_no_match_member_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
class A { function who(): string { return "A"; } }
class B { }
function make(bool $b): A|B { return $b ? new A() : new B(); }
echo make(false)->who();
"#,
    );
    assert!(
        err.contains("Call to a member function who() on null"),
        "{err}"
    );
}

/// Verifies lenient union method dispatch when TWO members both resolve the method with
/// DIFFERENT return types (`A::m(): int`, `B::m(): string`): the call type-checks against both,
/// the static result type is the union of their return types, and each runtime branch returns
/// its own concrete value php-identically.
#[test]
fn test_union_method_dispatch_multi_resolving_different_return_types() {
    let out = compile_and_run(
        r#"<?php
class A { function m(): int { return 1; } }
class B { function m(): string { return "b"; } }
function make(bool $b): A|B { return $b ? new A() : new B(); }
var_dump(make(true)->m());
var_dump(make(false)->m());
"#,
    );
    assert_eq!(out, "int(1)\nstring(1) \"b\"\n");
}

/// Verifies the exact SPEC G1 root-cause shape: a 3-member union (`Definition|Alias|null`)
/// where only `Definition` declares the called method, accessed through the plain `->` object
/// path via a property. Mirrors Symfony's `AbstractConfigurator::$definition`.
#[test]
fn test_union_method_dispatch_three_member_union_with_null() {
    let out = compile_and_run(
        r#"<?php
class Definition { function addTag(string $t): string { return "tag:$t"; } }
class Alias { function setPublic(bool $p): string { return $p ? "public" : "private"; } }
class Holder {
    protected Definition|Alias|null $definition = null;
    function set(): void { $this->definition = new Definition(); }
    function tag(): string { return $this->definition->addTag("x"); }
}
$h = new Holder();
$h->set();
echo $h->tag();
"#,
    );
    assert_eq!(out, "tag:x");
}

/// Regression: a `mixed`/union-typed property accepts a scalar literal *default* (int, bool, float)
/// at object construction, boxing it into a `Mixed` cell. Previously `object_new` raised
/// "unsupported EIR backend feature: object_new for default value of property ... with PHP type
/// Mixed/Union". Each property must round-trip its default and a later reassignment.
#[test]
fn test_union_property_scalar_literal_defaults() {
    let out = compile_and_run(
        r#"<?php
class Bag {
    public int|false $count = false;
    public mixed $tag = 7;
    public int|float $ratio = 1.5;
}
$b = new Bag();
echo ($b->count === false) ? "F" : "T";
echo "|", $b->tag, "|", $b->ratio;
$b->count = 42;
echo "|", $b->count;
"#,
    );
    assert_eq!(out, "F|7|1.5|42");
}
