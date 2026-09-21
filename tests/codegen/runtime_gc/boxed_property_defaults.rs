//! Purpose:
//! Regression tests for issue #688: an array literal defaulting a `mixed` or union-typed
//! property allocates its container and then boxes it into a Mixed cell, and the box takes
//! its OWN reference — so the object-construction path must use the owned boxer.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each fixture runs under `--heap-debug` and asserts `leak summary: clean`.
//! - Objects are constructed in a LOOP on purpose: the plain boxer retains without
//!   releasing and leaks exactly one block per object, which a single construction hides.
//! - Both literal spellings are covered together — keyed (`["k" => 1]`) and positional
//!   (`[1, 2]`) reach different emitters — so a future change cannot fix one and lose the
//!   other's ownership.

use crate::support::compile_and_run_with_heap_debug;

/// Verifies the boxed array-literal defaults release the container they allocate.
///
/// Each object allocates the literal and boxes it into a Mixed cell, and the box takes its own
/// reference — so the OWNED boxer is required. The plain one retains without releasing and leaks
/// one block per object, which a single-iteration total hides.
#[test]
fn test_array_literal_defaults_on_union_properties_are_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public ?array $x = ["k" => 1, "j" => "s"]; }
class B { public mixed $y = ["a" => 1]; }
class C { public ?array $z = [1, 2]; }
for ($i = 0; $i < 32; $i++) {
    $a = new A();
    $b = new B();
    $c = new C();
}
echo count($a->x), count($b->y), count($c->z), "\n";
"#,
    );
    assert_eq!(out.stdout, "212\n", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "boxed array-literal property default leaked: {}",
        out.stderr
    );
}

/// Follow-up for #1152: a keyed default on an INSTANCE `array|string` union slot.
///
/// `LiteralDefaultValue::BoxedAssocArray` reaches the union emitter too, but only the
/// `?array` / `mixed` spellings had keyed fixtures — the union slot was covered by its
/// POSITIONAL sibling alone. The container is allocated per object and boxed into the slot,
/// so a plain boxer here leaks one block per construction.
#[test]
fn test_keyed_array_default_on_a_union_property_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class T { public array|string $u = ["k" => 2, "j" => "s"]; }
for ($i = 0; $i < 32; $i++) {
    $t = new T();
}
echo count($t->u), ";";
var_dump($t->u["k"], $t->u["j"]);
"#,
    );
    assert_eq!(
        out.stdout,
        "2;int(2)\nstring(1) \"s\"\n",
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "a keyed default on a union slot leaked: {}",
        out.stderr
    );
}

/// Follow-up for #1152: keyed defaults on STATIC `?array` and `array|string` slots.
///
/// The static emitter is a separate path, and it too had only positional keyed coverage. A
/// static's storage is initialized once and never released — that is true of a positional
/// default as well, so `leak summary: clean` is the wrong question here. The right one is
/// whether the cost is FIXED: the same program is run with the read loop at 32 and at 64
/// iterations and the live totals have to be identical, which a per-read allocation or a
/// re-initialized default would break.
///
/// Every value expectation is the host PHP 8.5.10 output for the same fixture.
#[test]
fn test_keyed_array_defaults_on_static_properties_cost_one_fixed_allocation() {
    let program = |iterations: usize| {
        format!(
            r#"<?php
class S {{ public static ?array $x = ["k" => 1, "j" => "s"]; }}
class U {{ public static array|string $v = ["m" => 3]; }}
$n = 0;
for ($i = 0; $i < {iterations}; $i++) {{
    $n += count(S::$x) + count(U::$v);
    foreach (S::$x as $k => $v) {{ $n++; }}
}}
echo $n, ";";
var_dump(S::$x["k"], S::$x["j"], U::$v["m"]);
"#
        )
    };

    let short = compile_and_run_with_heap_debug(&program(32));
    assert_eq!(
        short.stdout,
        "160;int(1)\nstring(1) \"s\"\nint(3)\n",
        "stderr: {}",
        short.stderr
    );

    let long = compile_and_run_with_heap_debug(&program(64));
    assert_eq!(
        long.stdout,
        "320;int(1)\nstring(1) \"s\"\nint(3)\n",
        "stderr: {}",
        long.stderr
    );

    let live = |stderr: &str| {
        stderr
            .lines()
            .find(|line| line.starts_with("HEAP DEBUG: leak summary:"))
            .unwrap_or_else(|| panic!("no heap summary in: {stderr}"))
            .to_string()
    };
    assert_eq!(
        live(&short.stderr),
        live(&long.stderr),
        "a static keyed default must cost the same however often it is read"
    );
}
