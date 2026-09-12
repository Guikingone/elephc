//! Purpose:
//! Regression tests for generators reached ONLY through the runtime eval bridge: a class or
//! function containing `yield`, declared in a file whose path is known only at run time (an
//! `include $variable`), so the compiled entry point's own AST never spells `yield` or
//! `Generator` anywhere a static scan can see.
//!
//! Called from:
//!  - `cargo test` via the integration test harness; aggregated under
//!    `tests::codegen::generators` in `tests/codegen/generators/mod.rs`.
//!
//! Key details:
//!  - Before the fix, calling a generator function or method reached ONLY through a dynamic
//!    include died with `Fatal error: eval() runtime failed: unsupported MethodCall expression`
//!    (or `unsupported Call expression` for a plain function). The label was a red herring: the
//!    interpreter's own generator machinery (step-list lowering, resumption, `yield from`, ...)
//!    is complete and already covered by `crates/elephc-magician`'s own fragment tests. The
//!    actual defect was two static "does this program need the builtin `Generator` class"
//!    gates that only checked the compiled entry file's OWN literal AST for a `yield` or the
//!    spelled name `Generator` (`types::checker::builtin_class_gate::program_may_reference_generator`
//!    and `codegen::runtime_metadata::classes::runtime_referenced_class_names`), so neither ever
//!    fired when the only `yield` lived in a dynamically included file. Without a registered
//!    `Generator` class, `new_object("Generator")` returned null, `eval_generator_new` turned
//!    that into a bare `RuntimeFatal` with no message, and the outermost `MethodCall`/`Call`
//!    expression picked up the generic "unsupported X expression" label -- exactly the kind of
//!    mislabeling this campaign has hit before (a double free, a premature destruct, a missing
//!    scope push, an ownership undercount). This is the Symfony `--web` 404 blocker:
//!    `Kernel::doInitializeBundles()` runs `foreach ($this->registerBundles() as $bundle)` where
//!    `registerBundles()` is a generator method reached only through the same dynamic-include
//!    path every autoloaded vendor file uses.
//!  - Expected values are real `LC_ALL=C php -n` 8.5.6 output.

use crate::support::*;

/// Verifies a generator METHOD reached only through a dynamic include iterates correctly.
///
/// `main.php` contains no `yield` and never spells `Generator`, so both static gates that used
/// to decide whether the builtin `Generator` class exists at all saw nothing. `php -n` 8.5.6
/// prints `yield_in_method:3`.
#[test]
fn test_generator_method_reached_only_through_dynamic_include() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include $path; } load('piece.php');",
            ),
            (
                "piece.php",
                r#"<?php
class Y { public function items(): iterable { yield 1; yield 2; } }
$y = new Y();
$s = 0;
foreach ($y->items() as $v) { $s += $v; }
echo "yield_in_method:", $s, "\n";
"#,
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "yield_in_method:3\n");
}

/// Verifies a generator FUNCTION reached only through a dynamic include iterates correctly, and
/// that the returned object satisfies `instanceof Generator`/`Traversable`/`Iterator` even
/// though the compiled entry point never names any of the three.
///
/// `php -n` 8.5.6 prints `yield_in_function:a` then `GTI`.
#[test]
fn test_generator_function_reached_only_through_dynamic_include() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function load($path) { include $path; } load('piece.php');",
            ),
            (
                "piece.php",
                r#"<?php
function gen(): iterable { yield "a"; }
$g = gen();
foreach ($g as $v) { echo "yield_in_function:", $v, "\n"; }
echo $g instanceof Generator ? "G" : "-";
echo $g instanceof Traversable ? "T" : "-";
echo $g instanceof Iterator ? "I" : "-";
"#,
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "yield_in_function:a\nGTI");
}
