//! Purpose:
//! End-to-end regressions for issue #1113: `is_subclass_of()` and `is_a()` take a class NAME as
//! readily as an object, and answering `false` for the name form is a silent wrong answer.
//!
//! Called from:
//! - `cargo test --test codegen_tests class_relation_names` through Rust's test harness.
//!
//! Key details:
//! - `static_relation_holds` opened by requiring `PhpType::Object`, so a string first operand
//!   returned `false` before the parent and interface walks below it were ever reached. Those
//!   walks were already correct — they just need a class name, and do not care which operand
//!   shape it arrived in.
//! - PHP's `$allow_string` defaults differ between the two builtins: `is_subclass_of` takes
//!   names unless told otherwise, `is_a` only when told to. Measured, `is_a("D", "B")` is false
//!   and `is_a("D", "B", true)` is true.
//! - An INTERFACE name is reachable only through the string form — there is no instance of an
//!   interface to pass — and its parents live in `interface_infos`, not `class_infos`, so it
//!   needs its own walk.
//! - Still `false`, and deliberately: a NON-LITERAL name. That needs a name-keyed table the
//!   emitted program can consult at runtime, which is the second half of #1113 and what
//!   `ReflectionAttribute::IS_INSTANCEOF` actually needs.

use crate::support::compile_and_run;

/// Verifies the six rows the issue was filed with, plus the two the object form already got
/// right, so a regression on either side is visible.
#[test]
fn test_class_name_subject_answers_like_php() {
    let out = compile_and_run(
        r#"<?php
class Base {}
class Derived extends Base {}
class Deeper extends Derived {}
interface I {}
class Impl implements I {}

$o = new Derived();
echo is_subclass_of($o, "Base") ? "y" : "n";
echo is_subclass_of("Derived", "Base") ? "y" : "n";
echo is_subclass_of("Deeper", "Base") ? "y" : "n";
echo is_subclass_of("Impl", "I") ? "y" : "n";
echo is_subclass_of("Base", "Base") ? "y" : "n";
echo is_subclass_of("Impl", "Base") ? "y" : "n";
echo is_a($o, "Base") ? "y" : "n";
echo is_a("Derived", "Base", true) ? "y" : "n";
"#,
    );

    assert_eq!(out, "yyyynnyy");
}

/// Verifies PHP's two different `$allow_string` defaults, and that an explicit flag overrides
/// each of them.
#[test]
fn test_allow_string_defaults_differ_between_the_two_builtins() {
    let out = compile_and_run(
        r#"<?php
class Base {}
class Derived extends Base {}

echo is_subclass_of("Derived", "Base") ? "y" : "n";
echo is_subclass_of("Derived", "Base", true) ? "y" : "n";
echo is_subclass_of("Derived", "Base", false) ? "y" : "n";
echo is_a("Derived", "Base") ? "y" : "n";
echo is_a("Derived", "Base", true) ? "y" : "n";
echo is_a("Derived", "Base", false) ? "y" : "n";
echo is_a("Base", "Base", true) ? "y" : "n";
"#,
    );

    assert_eq!(out, "yynnyny");
}

/// Verifies an interface name as the subject, which the object form can never ask about, and
/// which walks a different table.
#[test]
fn test_an_interface_name_is_a_valid_subject() {
    let out = compile_and_run(
        r#"<?php
interface I {}
interface J extends I {}
interface K extends J {}
interface Unrelated {}
class ImplJ implements J {}

echo is_subclass_of("J", "I") ? "y" : "n";
echo is_subclass_of("K", "I") ? "y" : "n";
echo is_subclass_of("I", "J") ? "y" : "n";
echo is_subclass_of("J", "Unrelated") ? "y" : "n";
echo is_subclass_of("ImplJ", "I") ? "y" : "n";
echo is_a("J", "I", true) ? "y" : "n";
echo is_a("I", "I", true) ? "y" : "n";
"#,
    );

    assert_eq!(out, "yynnyyy");
}

/// Verifies names are matched the way PHP matches them — case-insensitively, with a leading
/// separator ignored — and that an unknown name answers `false` rather than claiming a relation.
#[test]
fn test_names_fold_like_php_and_unknown_names_answer_false() {
    let out = compile_and_run(
        r#"<?php
namespace App;

class Base {}
class Derived extends Base {}

echo \is_subclass_of("app\\dErIvEd", "App\\Base") ? "y" : "n";
echo \is_subclass_of("App\\Derived", "app\\bAsE") ? "y" : "n";
echo \is_subclass_of("\\App\\Derived", "\\App\\Base") ? "y" : "n";
echo \is_subclass_of("App\\NoSuchClass", "App\\Base") ? "y" : "n";
echo \is_subclass_of("App\\Derived", "App\\NoSuchTarget") ? "y" : "n";
echo \is_subclass_of("", "App\\Base") ? "y" : "n";
"#,
    );

    assert_eq!(out, "yyynnn");
}
