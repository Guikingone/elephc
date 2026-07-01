//! Purpose:
//! Unit tests for the tree-shaking skeleton harvest and its structural queries.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`cargo test tree_shake`).
//!
//! Key details:
//! - Each test parses a small PHP source string through the real lexer + parser (no
//!   name resolver), then asserts over the harvested `Skeleton`. Sources are un-namespaced
//!   so raw declaration names are already canonical, matching the post-resolver key form.

use super::*;

/// Parses a PHP source string into a program and harvests its skeleton.
fn skel(src: &str) -> Skeleton {
    let tokens = crate::lexer::tokenize(src).expect("tokenize should succeed");
    let program = crate::parser::parse(&tokens).expect("parse should succeed");
    harvest_skeleton(&program)
}

/// Verifies the harvest captures each class-like, its kind, hierarchy links, modifier
/// flags, and its method table with per-method visibility and the static flag.
#[test]
fn captures_classes_hierarchy_and_method_flags() {
    use crate::parser::ast::Visibility;
    let s = skel(
        "<?php
        abstract class Animal {
            public function speak() {}
            protected static function helper() {}
        }
        final class Dog extends Animal implements Pettable {}
        interface Pettable {
            public function pet();
        }
        ",
    );

    let animal = s.classes.get("Animal").expect("Animal harvested");
    assert_eq!(animal.kind, ClassKind::Class);
    assert!(animal.is_abstract);
    assert!(!animal.is_final);
    let speak = animal.methods.get("speak").expect("speak method");
    assert_eq!(speak.name, "speak");
    assert_eq!(speak.visibility, Visibility::Public);
    assert!(!speak.is_static);
    let helper = animal.methods.get("helper").expect("helper method");
    assert_eq!(helper.visibility, Visibility::Protected);
    assert!(helper.is_static);

    let dog = s.classes.get("Dog").expect("Dog harvested");
    assert_eq!(dog.parent.as_deref(), Some("Animal"));
    assert!(dog.interfaces.iter().any(|i| i == "Pettable"));
    assert!(dog.is_final);

    let pettable = s.classes.get("Pettable").expect("Pettable harvested");
    assert_eq!(pettable.kind, ClassKind::Interface);
}

/// Verifies a free function is harvested into the function table by canonical name.
#[test]
fn captures_free_functions() {
    let s = skel("<?php function do_thing() {}");
    assert!(s.functions.contains_key("do_thing"));
}

/// Verifies `subtypes` walks a transitive `extends` chain (A <- B <- C) and that
/// `implementers` reports a class implementing an interface.
#[test]
fn subtypes_and_implementers_walk_closed_hierarchy() {
    let s = skel(
        "<?php
        class A {}
        class B extends A {}
        class C extends B {}
        interface I {}
        class X implements I {}
        ",
    );

    let subs = s.subtypes("A");
    for expected in ["A", "B", "C"] {
        assert!(subs.contains(&expected.to_string()), "subtypes(A) missing {expected}");
    }

    let impls = s.implementers("I");
    assert!(impls.contains(&"X".to_string()), "implementers(I) missing X");
    // The interface itself is a subtype of I, and X reaches I through `implements`.
    assert!(s.subtypes("I").contains(&"X".to_string()));
}

/// Verifies `is_instantiable` is true only for a concrete class, and false for abstract
/// classes, interfaces, traits, and absent types.
#[test]
fn is_instantiable_only_for_concrete_classes() {
    let s = skel(
        "<?php
        abstract class Abs {}
        interface Iface {}
        trait Tr {}
        class Conc {}
        ",
    );
    assert!(s.is_instantiable("Conc"));
    assert!(!s.is_instantiable("Abs"));
    assert!(!s.is_instantiable("Iface"));
    assert!(!s.is_instantiable("Tr"));
    assert!(!s.is_instantiable("Missing"));
}

/// Verifies `resolve_method` finds an inherited method on its declaring ancestor, an
/// overridden method on the nearest class, is case-insensitive, and yields `None` for a
/// missing method.
#[test]
fn resolve_method_walks_parents_case_insensitively() {
    let s = skel(
        "<?php
        class Base {
            public function foo() {}
            public function bar() {}
            public function Greet() {}
        }
        class Sub extends Base {
            public function bar() {}
        }
        ",
    );

    let (foo_owner, _) = s.resolve_method("Sub", "foo").expect("inherited foo resolves");
    assert_eq!(foo_owner, "Base");

    let (bar_owner, _) = s.resolve_method("Sub", "bar").expect("overridden bar resolves");
    assert_eq!(bar_owner, "Sub");

    // PHP method lookup is case-insensitive; the key is lowercase.
    let (greet_owner, _) = s.resolve_method("Sub", "greet").expect("Greet resolves via lowercase");
    assert_eq!(greet_owner, "Base");

    assert!(s.resolve_method("Sub", "missing").is_none());
}

/// Verifies trait methods are folded onto a using class (and transitively through a trait
/// that itself uses another trait), so the class reports them as its own.
#[test]
fn trait_methods_fold_onto_using_class() {
    let s = skel(
        "<?php
        trait Greetable {
            public function greet() {}
        }
        class Person {
            use Greetable;
            public function name() {}
        }
        trait Inner {
            public function inner() {}
        }
        trait Outer {
            use Inner;
            public function outer() {}
        }
        class Host {
            use Outer;
        }
        ",
    );

    let person = s.classes.get("Person").expect("Person harvested");
    assert!(person.methods.contains_key("greet"), "direct trait method folded");
    assert!(person.methods.contains_key("name"), "own method present");

    let host = s.classes.get("Host").expect("Host harvested");
    assert!(host.methods.contains_key("outer"), "used trait method folded");
    assert!(host.methods.contains_key("inner"), "transitive trait method folded");
}

/// Verifies an external (undeclared) parent/interface does not panic and marks a
/// hierarchy boundary: subtype and method walks stop at it, and querying the absent type
/// returns just itself.
#[test]
fn external_super_type_is_a_boundary_not_a_panic() {
    let s = skel("<?php class Orphan extends \\DoesNotExist implements \\AlsoMissing {}");

    let orphan = s.classes.get("Orphan").expect("Orphan harvested");
    assert_eq!(orphan.parent.as_deref(), Some("DoesNotExist"));
    assert!(orphan.interfaces.iter().any(|i| i == "AlsoMissing"));

    // Orphan has no declared subtypes; only itself is returned.
    assert_eq!(s.subtypes("Orphan"), vec!["Orphan".to_string()]);
    // The external parent is not in the map, so resolution stops with no match.
    assert!(s.resolve_method("Orphan", "whatever").is_none());
    // Querying the absent type itself does not panic: it returns the type plus the
    // declared classes that reach it (Orphan `extends \DoesNotExist`).
    let external_subs = s.subtypes("DoesNotExist");
    assert!(external_subs.contains(&"DoesNotExist".to_string()));
    assert!(external_subs.contains(&"Orphan".to_string()));
}

/// Verifies the harvest descends into a namespace block and a conditional (`if`) body,
/// so declarations nested in those statement bodies are still recorded.
#[test]
fn harvest_descends_into_nested_statement_bodies() {
    let s = skel(
        "<?php
        namespace App {
            class Inside {}
            if (true) {
                class Conditional {}
            }
        }
        ",
    );
    assert!(s.classes.contains_key("Inside"), "class in namespace block harvested");
    assert!(s.classes.contains_key("Conditional"), "class in if-body harvested");
}
