//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of magic constants, including file is absolute path ending in test php, dir is absolute path with no trailing slash, and dir concat produces single folded string.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Multi-file fixtures exercise include/require resolution, temporary project layout, and native binary output.

use crate::support::*;

// `__FILE__` and `__DIR__` are substituted to the canonical path of the
// containing source file. The test harness uses `<temp>/test.php` as the
// synthetic main path, so __FILE__ ends with "test.php" and __DIR__ is the
// canonical temp directory.

/// Verifies `__FILE__` is an absolute path ending in `test.php`. The temp
/// directory provides an absolute path; __FILE__ is substituted at lowering.
#[test]
fn test_dunder_file_is_absolute_path_ending_in_test_php() {
    let out = compile_and_run("<?php echo __FILE__;");
    assert!(
        out.starts_with('/'),
        "__FILE__ should be absolute, got {:?}",
        out
    );
    assert!(
        out.ends_with("test.php"),
        "__FILE__ should end with test.php, got {:?}",
        out
    );
}

/// Verifies `__DIR__` is an absolute path with no trailing slash. The temp
/// directory provides an absolute path; __DIR__ is substituted at lowering.
#[test]
fn test_dunder_dir_is_absolute_path_with_no_trailing_slash() {
    let out = compile_and_run("<?php echo __DIR__;");
    assert!(out.starts_with('/'), "__DIR__ should be absolute, got {:?}", out);
    assert!(
        !out.ends_with('/'),
        "__DIR__ should not end with a separator, got {:?}",
        out
    );
}

/// Verifies `__DIR__ . '/sub/file.php'` is folded into a single literal at
/// compile time. The optimizer concatenates the constant dir with the relative
/// path and emits one string constant.
#[test]
fn test_dunder_dir_concat_produces_single_folded_string() {
    // The optimizer should fold `__DIR__ . '/sub'` into a single literal.
    let out = compile_and_run("<?php echo __DIR__ . '/sub/file.php';");
    assert!(out.starts_with('/'));
    assert!(out.ends_with("/sub/file.php"));
}

/// Verifies `__dir__` and `__LiNe__` (alternative casing) resolve correctly.
/// Magic constants are case-insensitive per PHP semantics.
#[test]
fn test_magic_constants_are_case_insensitive() {
    let out = compile_and_run("<?php echo __dir__ . '|'; echo __LiNe__;");
    assert!(out.starts_with('/'), "__dir__ should resolve to a path, got {:?}", out);
    assert!(out.ends_with("|1"), "__LiNe__ should resolve to line 1, got {:?}", out);
}

// `__LINE__` is substituted at parse time using the span line.

/// Verifies `__LINE__` equals 1 at the first line of a PHP script.
#[test]
fn test_dunder_line_at_first_line() {
    let out = compile_and_run("<?php echo __LINE__;");
    assert_eq!(out, "1");
}

/// Verifies `__LINE__` accounts for blank lines before the reference. With
/// three blank lines before `echo`, the span line is 4.
#[test]
fn test_dunder_line_after_blank_lines() {
    let out = compile_and_run("<?php\n\n\necho __LINE__;\n");
    assert_eq!(out, "4");
}

/// Verifies `__LINE__` inside a function body reports the line within the
/// function, not the top-level script. The echo is on line 3 of the script.
#[test]
fn test_dunder_line_inside_function_body() {
    let out = compile_and_run(
        "<?php\nfunction f() {\n    echo __LINE__;\n}\nf();\n",
    );
    assert_eq!(out, "3");
}

// `__FUNCTION__` returns the (FQN) function name inside a function, empty
// outside.

/// Verifies `__FUNCTION__` is empty when accessed outside any function body.
#[test]
fn test_dunder_function_outside_any_function_is_empty() {
    let out = compile_and_run("<?php echo '[' . __FUNCTION__ . ']';");
    assert_eq!(out, "[]");
}

/// Verifies `__FUNCTION__` inside a plain function returns the function name.
#[test]
fn test_dunder_function_inside_plain_function() {
    let out = compile_and_run(
        "<?php\nfunction greet() {\n    echo __FUNCTION__;\n}\ngreet();\n",
    );
    assert_eq!(out, "greet");
}

/// Verifies `__FUNCTION__` inside a namespaced function returns the fully-
/// qualified name (e.g. `App\Util\greet`).
#[test]
fn test_dunder_function_inside_namespaced_function_uses_fqn() {
    let out = compile_and_run(
        "<?php\nnamespace App\\Util;\nfunction greet() {\n    echo __FUNCTION__;\n}\ngreet();\n",
    );
    assert_eq!(out, "App\\Util\\greet");
}

/// Verifies `__FUNCTION__` inside a closure returns a PHP-style closure
/// marker string (e.g. `{closure:test.php:2}`) rather than empty or a path.
#[test]
fn test_dunder_function_inside_closure_returns_closure_marker() {
    let out = compile_and_run(
        "<?php\n$f = function() {\n    echo __FUNCTION__;\n};\n$f();\n",
    );
    assert!(
        out.contains("{closure:") && out.ends_with("test.php:2}"),
        "expected PHP-style closure marker, got {:?}",
        out
    );
}

// `__CLASS__` returns the (FQN) class name, empty outside.

/// Verifies `__CLASS__` is empty when accessed outside any class body.
#[test]
fn test_dunder_class_outside_any_class_is_empty() {
    let out = compile_and_run("<?php echo '[' . __CLASS__ . ']';");
    assert_eq!(out, "[]");
}

/// Verifies `__CLASS__` inside a method returns the class name (no namespace).
#[test]
fn test_dunder_class_inside_method() {
    let out = compile_and_run(
        "<?php\nclass C {\n    public function m() { echo __CLASS__; }\n}\n$o = new C(); $o->m();\n",
    );
    assert_eq!(out, "C");
}

/// Verifies `__CLASS__` inside a namespaced class returns the fully-qualified
/// name (e.g. `App\C`).
#[test]
fn test_dunder_class_inside_namespaced_class_uses_fqn() {
    let out = compile_and_run(
        "<?php\nnamespace App;\nclass C {\n    public function m() { echo __CLASS__; }\n}\n$o = new C(); $o->m();\n",
    );
    assert_eq!(out, "App\\C");
}

// `__METHOD__` returns "Class::method" inside a method.

/// Verifies `__METHOD__` is empty when accessed outside any function or method.
#[test]
fn test_dunder_method_outside_any_function_is_empty() {
    let out = compile_and_run("<?php echo '[' . __METHOD__ . ']';");
    assert_eq!(out, "[]");
}

/// Verifies `__METHOD__` inside a method returns `Class::method`.
#[test]
fn test_dunder_method_inside_method_is_class_qualified() {
    let out = compile_and_run(
        "<?php\nclass C {\n    public function go() { echo __METHOD__; }\n}\n$o = new C(); $o->go();\n",
    );
    assert_eq!(out, "C::go");
}

/// Verifies `__METHOD__` inside a namespaced method returns the FQN form
/// (e.g. `App\C::go`).
#[test]
fn test_dunder_method_inside_namespaced_method_uses_fqn() {
    let out = compile_and_run(
        "<?php\nnamespace App;\nclass C {\n    public function go() { echo __METHOD__; }\n}\n$o = new C(); $o->go();\n",
    );
    assert_eq!(out, "App\\C::go");
}

/// Verifies `__METHOD__` inside a plain function (not a method) returns the
/// function name, not a class-qualified form.
#[test]
fn test_dunder_method_inside_plain_function_is_function_name() {
    let out = compile_and_run(
        "<?php\nfunction f() { echo __METHOD__; }\nf();\n",
    );
    assert_eq!(out, "f");
}

// `__NAMESPACE__` returns the current namespace, empty outside.

/// Verifies `__NAMESPACE__` is empty when accessed outside any namespace
/// declaration.
#[test]
fn test_dunder_namespace_outside_namespace_is_empty() {
    let out = compile_and_run("<?php echo '[' . __NAMESPACE__ . ']';");
    assert_eq!(out, "[]");
}

/// Verifies `__NAMESPACE__` inside a namespace declaration returns that
/// namespace (e.g. `App\Util`).
#[test]
fn test_dunder_namespace_inside_namespace_decl() {
    let out = compile_and_run(
        "<?php\nnamespace App\\Util;\necho __NAMESPACE__;\n",
    );
    assert_eq!(out, "App\\Util");
}

// `__TRAIT__` returns the trait name inside a trait method.

/// Verifies `__TRAIT__` is empty when accessed outside any trait.
#[test]
fn test_dunder_trait_outside_trait_is_empty() {
    let out = compile_and_run("<?php echo '[' . __TRAIT__ . ']';");
    assert_eq!(out, "[]");
}

/// Verifies `__TRAIT__` inside a trait method returns the trait name.
#[test]
fn test_dunder_trait_inside_trait_method() {
    let out = compile_and_run(
        "<?php\ntrait Greetable {\n    public function name() { echo __TRAIT__; }\n}\nclass C { use Greetable; }\n$o = new C(); $o->name();\n",
    );
    assert_eq!(out, "Greetable");
}

/// Verifies `__CLASS__` inside a trait method that is used by a class returns
/// the importing class name, not the trait name. Combines `__CLASS__` and
/// `__METHOD__` in one expression to confirm both resolve to `C`.
#[test]
fn test_dunder_class_inside_trait_method_uses_importing_class() {
    let out = compile_and_run(
        "<?php\ntrait Greetable {\n    public function name() { echo __CLASS__ . '|' . __METHOD__; }\n}\nclass C { use Greetable; }\n$o = new C(); $o->name();\n",
    );
    assert_eq!(out, "C|Greetable::name");
}

/// Verifies `__CLASS__` in a property default inside a trait resolves to the
/// importing class (`C`), not the trait (`Named`). Regression for trait property
/// initializers that reference magic constants.
#[test]
fn test_dunder_class_inside_trait_property_uses_importing_class() {
    let out = compile_and_run(
        "<?php\ntrait Named {\n    public string $name = __CLASS__;\n}\nclass C { use Named; }\n$o = new C(); echo $o->name;\n",
    );
    assert_eq!(out, "C");
}

/// Verifies `__CLASS__`, `__METHOD__`, and `__TRAIT__` inside a namespaced
/// trait method all resolve to their FQN forms using the importing class's
/// namespace (e.g. `App\C`, `App\C::info`, `App\Named`). Confirms namespace
/// context is preserved through the trait use chain.
#[test]
fn test_dunder_class_inside_namespaced_trait_uses_importing_class_fqn() {
    let out = compile_and_run(
        "<?php\nnamespace App;\ntrait Named {\n    public string $name = __CLASS__;\n    public function info() { echo __CLASS__ . '|' . __METHOD__ . '|' . __TRAIT__; }\n}\nclass C { use Named; }\n$o = new C(); echo $o->name . '|'; $o->info();\n",
    );
    assert_eq!(out, "App\\C|App\\C|App\\Named::info|App\\Named");
}

/// Verifies `__FUNCTION__` and `__METHOD__` inside a closure defined within
/// a method use PHP-style closure markers that embed the enclosing method's
/// context (e.g. `{closure:C::m():4}`). Both constants resolve identically
/// inside closures in this configuration.
#[test]
fn test_dunder_function_inside_closure_in_method_uses_php_style_name() {
    let out = compile_and_run(
        "<?php\nclass C {\n    public function m() {\n        $f = function() { echo __FUNCTION__ . '|' . __METHOD__; };\n        $f();\n    }\n}\n$o = new C(); $o->m();\n",
    );
    assert_eq!(out, "{closure:C::m():4}|{closure:C::m():4}");
}

/// Verifies `__FUNCTION__` in a short ternary (`expr ?: fallback`) and in
/// the alternate branch (`'' ?: __FUNCTION__`) is lowered correctly without
/// special-case syntax breaking constant substitution. Regression for
/// short-circuit operators mishandling magic constant operands.
#[test]
fn test_magic_constant_inside_short_ternary_is_lowered() {
    let out = compile_and_run(
        "<?php\nfunction f() {\n    echo __FUNCTION__ ?: 'fallback';\n    echo '|';\n    echo '' ?: __FUNCTION__;\n}\nf();\n",
    );
    assert_eq!(out, "f|f");
}

// Magic constants inside class-constant initializers (`const X = <expr>;`) must
// be lowered with the enclosing class/interface/enum scope, exactly like method
// bodies, so they never reach type inference un-lowered. Before the lowering pass
// descended into constant members these panicked with "MagicConstant must be
// lowered before type inference".

/// Verifies `__CLASS__` inside a class constant resolves to the declaring class
/// name (php-verified: `Baz`). Regression for the ReflectionCaster:27 ICE gate.
#[test]
fn test_dunder_class_in_class_constant() {
    let out = compile_and_run(
        "<?php\nclass Baz {\n    const N = __CLASS__;\n}\necho Baz::N;\n",
    );
    assert_eq!(out, "Baz");
}

/// Verifies `__CLASS__` concatenated inside an array-literal class constant
/// resolves and folds (php-verified: `Foo::method`). Mirrors the exact shape of
/// symfony/var-dumper ReflectionCaster's `UNSET_CLOSURE_FILE_INFO` constant.
#[test]
fn test_dunder_class_in_class_constant_array_literal() {
    let out = compile_and_run(
        "<?php\nclass Foo {\n    const INFO = ['k' => __CLASS__ . '::method'];\n}\necho Foo::INFO['k'];\n",
    );
    assert_eq!(out, "Foo::method");
}

/// Verifies `__CLASS__` and `__NAMESPACE__` inside a namespaced class constant
/// resolve to the fully-qualified class name and the namespace (php-verified:
/// `App\Sub\C` and `App\Sub`).
#[test]
fn test_dunder_class_and_namespace_in_namespaced_class_constant() {
    let out = compile_and_run(
        "<?php\nnamespace App\\Sub;\nclass C {\n    const A = __CLASS__;\n    const B = __NAMESPACE__;\n}\necho C::A . '|' . C::B;\n",
    );
    assert_eq!(out, "App\\Sub\\C|App\\Sub");
}

/// Verifies `__CLASS__` inside an interface constant resolves to the interface's
/// own name (php-verified: `I`), not empty.
#[test]
fn test_dunder_class_in_interface_constant() {
    let out = compile_and_run(
        "<?php\ninterface I {\n    const A = __CLASS__;\n}\necho I::A;\n",
    );
    assert_eq!(out, "I");
}

/// Verifies `__CLASS__` inside an enum constant resolves to the enum's own name
/// (php-verified: `E`).
#[test]
fn test_dunder_class_in_enum_constant() {
    let out = compile_and_run(
        "<?php\nenum E: string {\n    case A = 'a';\n    const K = __CLASS__;\n}\necho E::K;\n",
    );
    assert_eq!(out, "E");
}

/// Verifies `__CLASS__` inside a backed-enum case value resolves to the enum's
/// own name (php-verified: `E`). Case values are class-like initializers too.
#[test]
fn test_dunder_class_in_enum_case_value() {
    let out = compile_and_run(
        "<?php\nenum E: string {\n    case A = __CLASS__;\n}\necho E::A->value;\n",
    );
    assert_eq!(out, "E");
}

/// Verifies `__FUNCTION__` and `__METHOD__` inside a class constant are the empty
/// string (php-verified: no enclosing function/method), while `__CLASS__` still
/// resolves. Confirms class-const initializers reuse the method-body scope where
/// only the class name is set.
#[test]
fn test_dunder_function_and_method_in_class_constant_are_empty() {
    let out = compile_and_run(
        "<?php\nclass C {\n    const F = __FUNCTION__;\n    const M = __METHOD__;\n    const K = __CLASS__;\n}\necho '[' . C::F . '][' . C::M . '][' . C::K . ']';\n",
    );
    assert_eq!(out, "[][][C]");
}

/// Verifies `__LINE__` inside a class constant resolves to the declaration line
/// (php-verified: line 3 here). `__LINE__` is lowered at parse time, so this also
/// confirms the const-member walk does not disturb parser-lowered constants.
#[test]
fn test_dunder_line_in_class_constant() {
    let out = compile_and_run(
        "<?php\nclass L {\n    const N = __LINE__;\n}\necho L::N;\n",
    );
    assert_eq!(out, "3");
}

/// Verifies `__CLASS__` in a plain (non-trait) property default resolves to the
/// declaring class (php-verified: `P`), and `__FUNCTION__` there is empty. This
/// path already worked (property defaults are walked in class scope); the test
/// pins the behavior alongside the new class-constant coverage.
#[test]
fn test_dunder_class_in_plain_property_default() {
    let out = compile_and_run(
        "<?php\nclass P {\n    public string $x = __CLASS__;\n    public string $f = __FUNCTION__;\n}\n$p = new P(); echo '[' . $p->x . '][' . $p->f . ']';\n",
    );
    assert_eq!(out, "[P][]");
}

/// Verifies a class-constant `__CLASS__` (lowered to the declaring class) and a
/// method-body `__CLASS__` (lowered the same way) agree in the same class. Guards
/// against the const-member walk diverging from the method-body walk.
#[test]
fn test_dunder_class_in_const_and_method_agree() {
    let out = compile_and_run(
        "<?php\nclass Bar {\n    const C = __CLASS__;\n    public function who(): string { return __CLASS__; }\n}\n$o = new Bar();\necho Bar::C . '|' . $o->who();\n",
    );
    assert_eq!(out, "Bar|Bar");
}

// `__FILE__` and `__DIR__` from an *included* file should reflect that
// file's path, not the main file's.

/// Verifies `__FILE__` inside an included file reflects the included file's
/// path, not the main script's path. Uses a two-file fixture with `require`.
#[test]
fn test_dunder_file_inside_include_uses_included_files_path() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib/inner.php';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho __FILE__;\n",
            ),
        ],
        "main.php",
    );
    assert!(
        out.ends_with("/lib/inner.php"),
        "expected included file's __FILE__, got {:?}",
        out
    );
}

/// Verifies `__DIR__` inside an included file reflects the included file's
/// directory (`lib/`), not the main script's directory. Uses a two-file
/// fixture with `require`.
#[test]
fn test_dunder_dir_inside_include_uses_included_files_dir() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire 'lib/inner.php';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho __DIR__;\n",
            ),
        ],
        "main.php",
    );
    assert!(
        out.ends_with("/lib"),
        "expected included file's __DIR__ (ending in /lib), got {:?}",
        out
    );
}

/// Verifies `__NAMESPACE__` inside an included file is independent of the
/// caller's namespace. Main file has `namespace App` but included file has
/// no namespace, so the included file's `__NAMESPACE__` is empty.
#[test]
fn test_included_file_magic_namespace_does_not_inherit_caller_namespace() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nnamespace App;\nrequire 'lib/inner.php';\necho '[' . __NAMESPACE__ . ']';\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho '[' . __NAMESPACE__ . ']';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "[][App]");
}

/// Verifies `__NAMESPACE__` inside an included file with its own namespace
/// declaration (`namespace Lib`) does not affect the main file's
/// `__NAMESPACE__`. Each file maintains its own namespace context.
#[test]
fn test_included_file_namespace_does_not_leak_to_caller_magic_constants() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nnamespace App;\nrequire 'lib/inner.php';\necho '[' . __NAMESPACE__ . ']';\n",
            ),
            (
                "lib/inner.php",
                "<?php\nnamespace Lib;\necho '[' . __NAMESPACE__ . ']';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "[Lib][App]");
}

/// Verifies `__FUNCTION__` inside an included file does not inherit the
/// calling function's context. When `inner.php` is required from within
/// function `load()`, the included file's `__FUNCTION__` remains empty because
/// the include happens at runtime and does not place the included code inside
/// the calling function's scope.
#[test]
fn test_included_file_magic_function_does_not_inherit_calling_function() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nfunction load() { require 'lib/inner.php'; }\nload();\n",
            ),
            (
                "lib/inner.php",
                "<?php\necho '[' . __FUNCTION__ . ']';\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "[]");
}
