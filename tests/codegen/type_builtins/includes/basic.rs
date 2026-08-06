//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of type-related builtins, includes basic, including include basic, require basic, and include with parens.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Multi-file fixtures exercise include/require resolution, temporary project layout, and native binary output.

use super::*;

/// Compiles main.php that includes helper.php and calls the exported function.
#[test]
fn test_include_basic() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php include 'helper.php'; echo greet();"),
            ("helper.php", "<?php function greet() { return \"hello\"; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "hello");
}

/// Compiles main.php that requires math.php and calls the exported function.
#[test]
fn test_require_basic() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php require 'math.php'; echo add(3, 4);"),
            ("math.php", "<?php function add($a, $b) { return $a + $b; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "7");
}

/// Verifies `include` with parentheses (functional syntax) works correctly.
#[test]
fn test_include_with_parens() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php include('helper.php'); echo greet();"),
            ("helper.php", "<?php function greet() { return \"hi\"; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "hi");
}

/// Verifies top-level code in an included file executes at the include point, interleaving with main file output.
#[test]
fn test_include_top_level_code() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php echo \"before\"; include 'mid.php'; echo \"after\";",
            ),
            ("mid.php", "<?php echo \"middle\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "beforemiddleafter");
}

/// Verifies `include_once` only executes the file the first time; subsequent calls in the same runtime are no-ops.
#[test]
fn test_include_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
include_once 'counter.php';
include_once 'counter.php';
echo $x;
"#,
            ),
            ("counter.php", "<?php $x = 42;"),
        ],
        "main.php",
    );
    assert_eq!(out, "42");
}

/// Verifies `require_once` only executes the file once; function is callable after first load.
#[test]
fn test_require_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
require_once 'lib.php';
require_once 'lib.php';
echo double(5);
"#,
            ),
            ("lib.php", "<?php function double($n) { return $n * 2; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "10");
}

/// Verifies constants and functions declared in a `require_once` file are accessible after loading.
#[test]
fn test_require_once_const_visible_inside_included_function() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
require_once 'lib.php';
echo LIB_CONST;
echo from_func();
"#,
            ),
            (
                "lib.php",
                r#"<?php
const LIB_CONST = 42;
function from_func() { return LIB_CONST; }
"#,
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "4242");
}

/// Verifies `include_once` in a constant-false branch does not claim the file; later `include_once` still executes it.
#[test]
fn test_include_once_skipped_branch_does_not_claim_file() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
if (false) {
    include_once 'piece.php';
}
include_once 'piece.php';
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies `include_once` in a loop only executes the file once across all iterations.
#[test]
fn test_include_once_in_loop_executes_file_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
$i = 0;
while ($i < 3) {
    include_once 'tick.php';
    $i = $i + 1;
}
"#,
            ),
            ("tick.php", "<?php echo \"tick\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "tick");
}

/// Verifies `require_once` inside a function has globalOnce semantics; subsequent calls do not re-execute.
#[test]
fn test_require_once_in_function_is_global_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
function load_piece() {
    require_once 'piece.php';
}
load_piece();
load_piece();
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies `require_once` inside a class method has globalOnce semantics across calls on the same instance.
#[test]
fn test_require_once_in_method_is_global_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
class Loader {
    public function load() {
        require_once 'piece.php';
    }
}
$loader = new Loader();
$loader->load();
$loader->load();
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies `require_once` inside a closure has globalOnce semantics across closure invocations.
#[test]
fn test_require_once_in_closure_is_global_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
$load = function() {
    require_once 'piece.php';
};
$load();
$load();
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies a regular `include` inside a closure marks the file as loaded, causing a later `include_once` to skip it.
#[test]
fn test_regular_include_in_closure_marks_later_include_once() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
$load = function() {
    include 'piece.php';
};
$load();
include_once 'piece.php';
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies declarations from a regular `include` are visible to a subsequent `include_once` (no duplicate error).
#[test]
fn test_regular_include_marks_later_include_once_declarations() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
include 'lib.php';
include_once 'lib.php';
echo seven();
"#,
            ),
            ("lib.php", "<?php function seven() { return 7; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "7");
}

/// Verifies `include_once` in a constant-false branch does not claim the file; later `include_once` still executes and finds the declaration.
#[test]
fn test_skipped_regular_include_does_not_make_include_once_skip() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
if (false) {
    include 'piece.php';
}
include_once 'piece.php';
"#,
            ),
            ("piece.php", "<?php echo \"piece\";"),
        ],
        "main.php",
    );
    assert_eq!(out, "piece");
}

/// Verifies `return require X;` includes the file (its declarations become available) and the
/// expression yields `1`, the value PHP returns for an include with no explicit `return`.
#[test]
fn test_require_as_return_value() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function boot(): int { return require 'helper.php'; } echo boot(); echo ':'; echo greet();",
            ),
            ("helper.php", "<?php function greet() { return \"hi\"; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "1:hi");
}

/// Verifies `$x = require X;` includes the file and assigns the include's value `1`.
#[test]
fn test_require_as_assignment_value() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $loaded = require 'math.php'; echo $loaded; echo ':'; echo add(2, 5);",
            ),
            ("math.php", "<?php function add($a, $b) { return $a + $b; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "1:7");
}

/// Verifies `$x = require_once X;` works as a value-position include with the once semantics.
#[test]
fn test_require_once_as_assignment_value() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $a = require_once 'lib.php'; echo $a; echo ':'; echo val();",
            ),
            ("lib.php", "<?php function val() { return 9; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "1:9");
}

/// Verifies that `$x = require X;` captures the included file's top-level `return` value (an
/// integer here), matching PHP's "include returns a value" semantics.
#[test]
fn test_require_value_captures_returned_int() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php $n = require 'num.php'; echo $n + 1;"),
            ("num.php", "<?php return 41;"),
        ],
        "main.php",
    );
    assert_eq!(out, "42");
}

/// Verifies that `return require X;` returns the included file's returned array, readable by key.
#[test]
fn test_require_value_captures_returned_array() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php function cfg(): array { return require 'config.php'; } $c = cfg(); echo $c['port'];",
            ),
            ("config.php", "<?php return ['host' => 'localhost', 'port' => 5432];"),
        ],
        "main.php",
    );
    assert_eq!(out, "5432");
}

/// Verifies that an expression-position `require` shares the caller's scope: the included file
/// can READ a variable defined in the caller (PHP runs includes in the calling scope).
#[test]
fn test_require_value_reads_caller_scope() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $base = 10; $v = require 'inc.php'; echo $v;",
            ),
            ("inc.php", "<?php return $base * 2;"),
        ],
        "main.php",
    );
    assert_eq!(out, "20");
}

/// Verifies that an expression-position `require` shares the caller's scope for WRITES: a value
/// assigned to an existing caller variable inside the included file is visible after the include.
#[test]
fn test_require_value_writes_caller_scope() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $acc = 1; $r = require 'inc.php'; echo $acc; echo ':'; echo $r;",
            ),
            ("inc.php", "<?php $acc = $acc + 41; return 7;"),
        ],
        "main.php",
    );
    assert_eq!(out, "42:7");
}

/// Verifies that a variable first assigned inside an expression-position `require` leaks into the
/// caller's scope afterward, matching PHP's shared-scope include semantics.
#[test]
fn test_require_value_new_var_leaks_to_caller() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $r = require 'inc.php'; echo $created; echo ':'; echo $r;",
            ),
            ("inc.php", "<?php $created = 99; return 1;"),
        ],
        "main.php",
    );
    assert_eq!(out, "99:1");
}

/// Verifies that an included file with no top-level `return` yields `1` while still hoisting its
/// declarations globally.
#[test]
fn test_require_value_without_return_yields_one() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php $r = require 'lib.php'; echo $r; echo ':'; echo helper();"),
            ("lib.php", "<?php function helper() { return 'H'; }"),
        ],
        "main.php",
    );
    assert_eq!(out, "1:H");
}

/// Milestone for the value-include hoister: a `??=` lazy-init whose default operand is a `require`
/// is a NESTED expression-position include (`$t = $t ?? require F`), so the resolver's direct
/// fast-path does not apply — the hoister must lift the include into a temp evaluated before the
/// statement. The included file is a pure `return [ ...array... ];` data table. Calling the
/// initializer TWICE proves the eager re-evaluation of the include is harmless: the value read
/// back (`$t[1][0]` == 3) is stable across calls, identical to PHP. (Cross-checked with
/// `php`: prints `3:3`.) The equivalent static/instance-property forms lift the include correctly
/// too but are currently blocked further down the pipeline — see the resolver report — so the
/// milestone is covered via a local `??=`, the spec's sanctioned fallback.
#[test]
fn test_value_include_null_coalesce_assign_stable_across_calls() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                r#"<?php
function cell(): int {
    $t = null;
    $t ??= require 'data.php';
    return $t[1][0];
}
echo cell();
echo ':';
echo cell();
"#,
            ),
            ("data.php", "<?php return [[1, 2], [3, 4]];"),
        ],
        "main.php",
    );
    assert_eq!(out, "3:3");
}

/// Verifies a `require` used as a NESTED operand of a binary expression (`$x = 10 + (require F)`),
/// which the direct fast-path cannot capture, is hoisted into a temp and folded into the sum.
/// The included file returns the integer `5`, so the result is `15` (cross-checked with `php`).
#[test]
fn test_value_include_nested_in_binary_op() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php $x = 10 + (require 'five.php'); echo $x;"),
            ("five.php", "<?php return 5;"),
        ],
        "main.php",
    );
    assert_eq!(out, "15");
}

/// Verifies a top-level `$t = $t ?? require F` (nested include in a `NullCoalesce` default)
/// is hoisted and yields the included array, readable by nested index (`$t[1][0]` == 3).
#[test]
fn test_value_include_top_level_null_coalesce() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $t = null; $t = $t ?? require 'data.php'; echo $t[1][0];",
            ),
            ("data.php", "<?php return [[1, 2], [3, 4]];"),
        ],
        "main.php",
    );
    assert_eq!(out, "3");
}

/// Regression guard for the direct value-include fast-path (`$x = require F;`), which the hoister
/// change must leave untouched: the include's top-level `return 41;` is captured directly into
/// `$n` with no extra hoist temp, and echoes `41`.
#[test]
fn test_value_include_direct_assignment_fast_path_regression() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php $n = require 'num.php'; echo $n;"),
            ("num.php", "<?php return 41;"),
        ],
        "main.php",
    );
    assert_eq!(out, "41");
}

/// A `return` nested inside an included file's top-level control flow returns from the INCLUDE,
/// not from the includer: PHP yields that value as the `require` expression's result and keeps
/// executing the caller afterwards. Guard for the shape where the taken branch returns.
#[test]
fn test_include_return_inside_if_yields_include_value_and_caller_continues() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $v = require 'boot.php'; echo $v; echo ':'; echo 'after';",
            ),
            ("boot.php", "<?php if (true) { return 7; } return 9;"),
        ],
        "main.php",
    );
    assert_eq!(out, "7:after");
}

/// The mirror case: the guarded `return` is skipped, so the included file falls through to its
/// trailing top-level `return`, and the caller still runs to completion.
#[test]
fn test_include_return_inside_untaken_if_falls_through_to_trailing_return() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $v = require 'boot.php'; echo $v; echo ':'; echo 'after';",
            ),
            ("boot.php", "<?php if (false) { return 7; } return 9;"),
        ],
        "main.php",
    );
    assert_eq!(out, "9:after");
}

/// A nested `return` must also STOP the included file: statements following the guarded return
/// in the include's top-level scope are dead once it fires, exactly as in PHP.
#[test]
fn test_include_return_inside_if_stops_the_included_file() {
    let out = compile_and_run_files(
        &[
            ("main.php", "<?php $v = require 'boot.php'; echo $v;"),
            (
                "boot.php",
                "<?php if (true) { echo 'in'; return 7; } echo 'tail'; return 9;",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "in7");
}

/// The Symfony polyfill bootstrap idiom: a dispatch file whose two mutually exclusive branches
/// each `return require` a DIFFERENT target declaring the SAME function. Only the live branch may
/// be flattened, so the declaration must not collide, and the caller must still run.
#[test]
fn test_include_conditional_return_require_dispatch_picks_one_target() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $v = require 'boot.php'; echo $v; echo ':'; echo poly(); echo ':done';",
            ),
            (
                "boot.php",
                "<?php if (\\PHP_VERSION_ID >= 80000) { return require __DIR__ . '/b80.php'; }\nreturn require __DIR__ . '/b72.php';",
            ),
            (
                "b80.php",
                "<?php function poly(): string { return 'new'; } return 80;",
            ),
            (
                "b72.php",
                "<?php function poly(): string { return 'old'; } return 72;",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "80:new:done");
}

/// A bare `return;` nested in the included file's control flow ends the include with PHP's
/// no-value default of `1`, and must not return from the caller.
#[test]
fn test_include_bare_return_inside_if_yields_one_and_caller_continues() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php $v = require 'boot.php'; echo $v; echo ':'; echo 'after';",
            ),
            ("boot.php", "<?php if (true) { return; } echo 'unreached';"),
        ],
        "main.php",
    );
    assert_eq!(out, "1:after");
}
