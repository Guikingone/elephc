//! Purpose:
//! End-to-end codegen tests for PHP 8.2 DNF (disjunctive normal form) types: a parenthesized
//! intersection group used as a union member, e.g. `(A&B)|null`, in property, parameter, and
//! return type positions.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `(A&B)|null` canonicalizes to a nullable intersection; the value is typed as the intersection's
//!   first member, so `null` narrowing and member dispatch behave like the plain `?A` form.
//! - Every expected output is cross-checked against the PHP interpreter.

use super::*;

/// Verifies a DNF property type (`(A&B)|null`) accepts a null default, then an assigned implementor,
/// and that `!== null` narrowing observes the transition (PHP prints "ny").
#[test]
fn test_dnf_property_assign_and_narrow() {
    let out = compile_and_run(
        "<?php
        interface A {} interface B {}
        class C implements A, B {}
        class Box {
            protected (A&B)|null $parent = null;
            public function set(C $c): void { $this->parent = $c; }
            public function has(): bool { return $this->parent !== null; }
        }
        $b = new Box(); echo $b->has() ? \"y\" : \"n\"; $b->set(new C()); echo $b->has() ? \"y\" : \"n\";
        ",
    );
    assert_eq!(out, "ny");
}

/// Verifies a DNF parameter type (`(A&B)|null`) accepts both `null` and an implementor of the
/// intersection, discriminated by `=== null` (PHP prints "nullobj").
#[test]
fn test_dnf_param_null_vs_object() {
    let out = compile_and_run(
        "<?php
        interface A {} interface B {}
        class C implements A, B {}
        function g((A&B)|null $x): string { return $x === null ? \"null\" : \"obj\"; }
        echo g(null), g(new C());
        ",
    );
    assert_eq!(out, "nullobj");
}

/// Verifies a DNF return type (`(A&B)|null`) may return either `null` or an implementor of the
/// intersection (PHP prints "nullobj").
#[test]
fn test_dnf_return_type() {
    let out = compile_and_run(
        "<?php
        interface A {} interface B {}
        class C implements A, B {}
        function h(bool $n): (A&B)|null { return $n ? null : new C(); }
        $r = h(true); echo $r === null ? \"null\" : \"obj\";
        $r = h(false); echo $r === null ? \"null\" : \"obj\";
        ",
    );
    assert_eq!(out, "nullobj");
}

/// Verifies a multi-group DNF union (`(A&B)|(P&Q)`) accepts an implementor of either intersection
/// arm (PHP prints "okok").
#[test]
fn test_dnf_multi_group_union_accepts_both_arms() {
    let out = compile_and_run(
        "<?php
        interface A {} interface B {} interface P {} interface Q {}
        class AB implements A, B {}
        class PQ implements P, Q {}
        function m((A&B)|(P&Q) $x): string { return \"ok\"; }
        echo m(new AB()), m(new PQ());
        ",
    );
    assert_eq!(out, "okok");
}

/// Verifies a DNF group mixing a base class and an interface (`(Base&I)|null`) accepts null and an
/// instance that both extends the class and implements the interface (PHP prints "ny").
#[test]
fn test_dnf_class_and_interface_intersection() {
    let out = compile_and_run(
        "<?php
        interface I {}
        class Base {}
        class Impl extends Base implements I {}
        function ci((Base&I)|null $x): string { return $x === null ? \"n\" : \"y\"; }
        echo ci(null), ci(new Impl());
        ",
    );
    assert_eq!(out, "ny");
}

/// Compiles and runs the checked-in `examples/dnf-types/main.php` fixture.
#[test]
fn test_example_dnf_types_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/dnf-types/main.php"));
    assert_eq!(out, "ny\n");
}

/// Verifies a flat, non-nested DNF group with three members `(A&B&C)` (no inner parentheses)
/// parses to a single `TypeExpr::Intersection` with three members and runs end-to-end: a value
/// implementing all three interfaces satisfies the parameter (PHP prints "ok"). Regression guard
/// for the nested-paren rejection in `parse_dnf_group`/`parse_dnf_member`: a flat multi-member
/// group must keep working while `((A&B)&C)`/`(A&(B&C))` are rejected.
#[test]
fn test_dnf_flat_three_member_group_runs() {
    let out = compile_and_run(
        "<?php
        interface A {} interface B {} interface C {}
        class ABC implements A, B, C {}
        function h((A&B&C)|null $x): string { return $x === null ? \"null\" : \"ok\"; }
        echo h(new ABC());
        ",
    );
    assert_eq!(out, "ok");
}
