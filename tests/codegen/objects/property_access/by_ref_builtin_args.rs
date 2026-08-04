//! Purpose:
//! Regression tests for passing an object property (or a container element reached through
//! one) as the by-reference argument of a mutating array builtin. These calls used to be
//! silent no-ops: the property's array was loaded, separated by copy-on-write inside the
//! runtime helper, mutated, and then discarded because nothing stored it back.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Expected values are real `LC_ALL=C php` 8.4 output for the same fixtures.
//! - The receiver shapes here cover the property-resolution paths the lowering has to walk:
//!   a declared property, an inherited one, a constructor-promoted one, `self::$prop` from a
//!   static method, and an element of a property that holds nested arrays.

use super::*;

/// A constructor-promoted, declared-type property is still a writable place for a
/// by-reference builtin argument.
#[test]
fn test_usort_on_promoted_constructor_property() {
    let out = compile_and_run(
        r#"<?php
class B { public function __construct(public array $items) {} }
$b = new B([3,1,2]);
usort($b->items, fn($x, $y) => $x <=> $y);
echo implode(",", $b->items);
"#,
    );
    assert_eq!(out, "1,2,3");
}

/// A property inherited from a parent class resolves through the same visible-property
/// lookup, so `sort()` mutates the subclass instance's storage.
#[test]
fn test_sort_on_inherited_property() {
    let out = compile_and_run(
        r#"<?php
class A { public $items = [3,1,2]; }
class B extends A {}
$b = new B();
sort($b->items);
echo implode(",", $b->items);
"#,
    );
    assert_eq!(out, "1,2,3");
}

/// A `self::$prop` receiver inside a static method: the static receiver resolves to the
/// enclosing class before the write-back targets the same static slot.
#[test]
fn test_sort_on_self_static_property_inside_static_method() {
    let out = compile_and_run(
        r#"<?php
class B {
    public static $items = [3,1,2];
    public static function go(): void { sort(self::$items); }
}
B::go();
echo implode(",", B::$items);
"#,
    );
    assert_eq!(out, "1,2,3");
}

/// An element of a property that holds nested arrays: the read and the write-back both walk
/// property-then-element, and only the addressed row is mutated.
#[test]
fn test_sort_on_element_of_array_property() {
    let out = compile_and_run(
        r#"<?php
class B { public $rows = [[3,1,2],[9,8]]; }
$b = new B();
sort($b->rows[0]);
echo implode(",", $b->rows[0]), "|", implode(",", $b->rows[1]);
"#,
    );
    assert_eq!(out, "1,2,3|9,8");
}

/// Two different array properties of the same object are mutated independently, so the
/// synthetic temporaries do not alias each other.
#[test]
fn test_sorting_two_properties_of_one_object() {
    let out = compile_and_run(
        r#"<?php
class B { public $a = [3,1,2]; public $b = [6,5,4]; }
$o = new B();
sort($o->a);
rsort($o->b);
echo implode(",", $o->a), "|", implode(",", $o->b);
"#,
    );
    assert_eq!(out, "1,2,3|6,5,4");
}
