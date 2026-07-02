//! Purpose:
//! End-to-end tests for the AST optimizer's closed-world `class_exists`/`interface_exists`/
//! `trait_exists`/`enum_exists` fold and the checker's absent-class tolerance, verifying that a
//! `class_exists`-guarded block referencing an absent optional-dependency class is dead-code
//! eliminated so codegen never sees `new AbsentClass`, while an existing class folds true.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - These fixtures only compile+run because the guarded `new AbsentClass` is pruned before EIR
//!   lowering. If the fold or DCE regressed, ir_lower would fail on the undefined class and the
//!   test would panic at compile time. Outputs are cross-checked against `php`.

use super::*;

/// Verifies a `class_exists(Absent::class)`-guarded block is folded and DCE'd: the `false` fold
/// makes `!class_exists(...)` constant-true, so the `else` branch's `new NoSuchClass()` is pruned
/// and never reaches codegen. PHP prints "absent" (the class is undefined, so `class_exists`
/// returns false).
#[test]
fn test_class_exists_guarded_absent_class_is_dce_d() {
    let out = compile_and_run(
        r#"<?php
if (!class_exists(NoSuchClass::class)) {
    echo "absent";
} else {
    $x = new NoSuchClass();
    echo $x->foo();
}
"#,
    );
    assert_eq!(out, "absent");
}

/// Verifies a method whose signature type-hints an absent class compiles when its body's use of
/// that class is `class_exists`-guarded (so the `new`/return tail is DCE'd) and the method is never
/// called. The absent type hints degrade to `Mixed`; the program runs its main line.
#[test]
fn test_absent_class_typehint_guarded_method_never_called_compiles() {
    let out = compile_and_run(
        r#"<?php
class Runner {
    public function run(\NoSuch\Thing $x): \NoSuch\Thing {
        if (!class_exists(\NoSuch\Thing::class)) {
            throw new \Exception('no');
        }
        return $x;
    }
}
echo "main-ok";
"#,
    );
    assert_eq!(out, "main-ok");
}

/// Regression guard: `class_exists(\stdClass::class)` on a class that exists in the closed world
/// folds to `true`, so the guarded `echo` survives. PHP prints "yes".
#[test]
fn test_class_exists_existing_class_folds_true() {
    let out = compile_and_run(
        r#"<?php
if (class_exists(\stdClass::class)) {
    echo "yes";
}
"#,
    );
    assert_eq!(out, "yes");
}

/// Verifies a genuine ProcessHelper-shaped guarded block inside an uncalled method does not break
/// linking: the `new \Acme\Process(...)` in the guarded tail is pruned, so codegen never emits a
/// constructor for the absent class and the binary links and runs. PHP prints "linked".
#[test]
fn test_guarded_new_absent_class_in_uncalled_method_links() {
    let out = compile_and_run(
        r#"<?php
class ProcessRunner {
    public function run($cmd) {
        if (!class_exists(\Acme\Process::class)) {
            throw new \LogicException('The Process component is not installed');
        }
        $process = new \Acme\Process($cmd);
        return $process->start();
    }
}
echo "linked";
"#,
    );
    assert_eq!(out, "linked");
}

/// Verifies the fold composes with a plain string-literal argument (not just `::class`): an absent
/// class name string folds `class_exists("Absent")` to `false`, pruning the guarded body. PHP
/// prints "gone".
#[test]
fn test_class_exists_string_literal_absent_folds_false() {
    let out = compile_and_run(
        r#"<?php
if (class_exists("Totally\\Absent\\Klass")) {
    $x = new \Totally\Absent\Klass();
    echo $x->run();
} else {
    echo "gone";
}
"#,
    );
    assert_eq!(out, "gone");
}
