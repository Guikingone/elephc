//! Purpose:
//! End-to-end codegen tests for `types::constructor_owner` — the rule that decides which class's
//! `__construct` PHP runs when a descendant of a class with a NON-PUBLIC constructor is
//! instantiated, and which members that descendant still reports.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A PRIVATE method is not inherited, so the descendant's own method map has no `__construct`
//!   entry: `method_exists()` and `get_class_methods()` report it gone while the constructor
//!   still runs and `ReflectionClass::getConstructor()` still finds it on the ancestor. A
//!   PROTECTED one IS inherited, so the descendant carries the entry and every member query
//!   answers from it — the two halves are pinned side by side here because they diverge.
//! - Expected output is verbatim reference PHP 8.5 for the same program.

use super::*;

/// A PROTECTED ancestor constructor is INHERITED, so the descendant's own method map carries
/// the entry: the constructor runs, `method_exists()` is `true` on both classes, and the
/// reflected declaring class is still the ancestor that wrote it.
#[test]
fn test_protected_ancestor_constructor_is_inherited_and_runs() {
    let out = compile_and_run(
        r#"<?php
class ProtOwner {
    protected function __construct(public int $n = 3) {}
    public static function makeChild(): ProtOwner { return new ProtChild(); }
}
class ProtChild extends ProtOwner {}
echo ProtOwner::makeChild()->n, "|";
var_dump(
    method_exists('ProtOwner', '__construct'),
    method_exists('ProtChild', '__construct')
);
"#,
    );
    assert_eq!(out, "3|bool(true)\nbool(true)\n");
}

/// A protected constructor is not PUBLIC, so the descendant is not instantiable from outside
/// the hierarchy even though it inherits the entry.
#[test]
fn test_protected_ancestor_constructor_leaves_the_descendant_uninstantiable() {
    let out = compile_and_run(
        r#"<?php
class ProtInstOwner { protected function __construct() {} }
class ProtInstChild extends ProtInstOwner {}
var_dump(
    (new ReflectionClass('ProtInstChild'))->isInstantiable(),
    (new ReflectionClass('ProtInstChild'))->getConstructor()?->getDeclaringClass()->getName()
);
"#,
    );
    assert_eq!(out, "bool(false)\nstring(13) \"ProtInstOwner\"\n");
}

/// A PRIVATE ancestor constructor is the opposite: the descendant does not inherit the entry,
/// so `method_exists()` is `false` on it while staying `true` on the class that declares it.
///
/// This is the half that makes `constructor_owner` necessary at all — the descendant looks like
/// it has no constructor to every member query, and only the owner walk finds the one PHP runs.
#[test]
fn test_private_ancestor_constructor_is_not_a_member_of_the_descendant() {
    let out = compile_and_run(
        r#"<?php
class PrivOwner { private function __construct() {} }
class PrivChild extends PrivOwner {}
var_dump(
    method_exists('PrivOwner', '__construct'),
    method_exists('PrivChild', '__construct')
);
"#,
    );
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// `get_class_methods()` agrees with `method_exists()`: the inherited private constructor is
/// absent from the descendant's list, while an inherited PUBLIC method is still listed.
///
/// The contrast is what makes this a pin rather than a tautology — `get_class_methods()` is
/// SCOPE-AWARE, so from global scope a private constructor is omitted from its own declaring
/// class too (php-src returns `0` for both classes with only that constructor), and only the
/// public method proves the list is being populated at all. The function is reachable through
/// the eval bridge rather than AOT, which is why it is spelled through `eval()`.
#[test]
fn test_get_class_methods_omits_an_inherited_private_constructor() {
    let out = compile_and_run(
        r#"<?php
class GcmOwner {
    private function __construct() {}
    public function visible(): int { return 1; }
}
class GcmChild extends GcmOwner {}
$child = eval('return get_class_methods("GcmChild");');
$owner = eval('return get_class_methods("GcmOwner");');
echo count($child), "|", count($owner), "|", $child[0], "|", $owner[0];
"#,
    );
    assert_eq!(out, "1|1|visible|visible");
}

/// A descendant's own PUBLIC constructor replaces a private ancestor's: the owner walk stops at
/// the descendant, so instantiating it from global scope is allowed and runs the descendant's.
///
/// The rejecting counterpart — a descendant whose own constructor is PRIVATE, which HIDES an
/// inherited public one — is a compile error and lives in
/// `error_tests::classes_traits::test_error_a_descendants_own_private_constructor_hides_the_ancestors`.
#[test]
fn test_a_descendants_own_public_constructor_replaces_a_private_ancestors() {
    let out = compile_and_run(
        r#"<?php
class ReplOwner {
    private function __construct() { echo "owner "; }
}
class ReplChild extends ReplOwner {
    public function __construct() { echo "child "; }
}
$object = new ReplChild();
echo $object instanceof ReplOwner ? "yes" : "no";
"#,
    );
    assert_eq!(out, "child yes");
}
