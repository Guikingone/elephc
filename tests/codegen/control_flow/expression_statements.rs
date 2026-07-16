//! Purpose:
//! Integration tests for end-to-end codegen of bare expression statements whose leading
//! token is a value or unary operator (e.g. `0 > $T && $T += 0x40;`, `new C();`, `-$x;`),
//! or a variable that is an operand of a larger expression (e.g. `$s > 0 ? f() : $x = 1;`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - PHP allows any expression as a statement. These fixtures exercise the statement
//!   dispatcher's bare-expression fallback for non-variable, non-keyword leading tokens, as
//!   well as variable-led statements whose next token is not an assignment operator, and
//!   assert PHP-equivalent stdout. One fixture uses `$argc` so the construct survives
//!   AST-level constant folding and actually reaches codegen.

use super::*;

/// Verifies the short-circuit `cond && action;` idiom runs the action when the literal-led
/// condition is true: `-5 < 0` holds, so `$T += 0x40` (64) makes `$T == 59`. This is the
/// Symfony intl-normalizer pattern (`0 > $T && $T += 0x40;`).
#[test]
fn test_value_led_short_circuit_runs_action() {
    let out = compile_and_run("<?php $T = -5; 0 > $T && $T += 0x40; echo $T;");
    assert_eq!(out, "59");
}

/// Verifies the same idiom skips the action when the literal-led condition is false:
/// `0 > 5` is false, so `$T += 0x40` never runs and `$T` stays `5`.
#[test]
fn test_value_led_short_circuit_skips_action_when_false() {
    let out = compile_and_run("<?php $T = 5; 0 > $T && $T += 0x40; echo $T;");
    assert_eq!(out, "5");
}

/// Verifies a literal-led short-circuit statement survives constant folding by using a
/// runtime-unknown operand (`$argc`, which is 1 when run with no args): `0 < 1` holds, so
/// `$n += 10` yields `11`.
#[test]
fn test_value_led_short_circuit_with_runtime_unknown() {
    let out = compile_and_run("<?php $n = $argc; 0 < $n && $n += 10; echo $n;");
    assert_eq!(out, "11");
}

/// Verifies the literal-led `cond || action;` form: `0 < $x` is false (with `$x == 0`), so the
/// right side `$x = 42` executes and `$x` becomes `42`.
#[test]
fn test_value_led_or_runs_action() {
    let out = compile_and_run("<?php $x = 0; 0 < $x || $x = 42; echo $x;");
    assert_eq!(out, "42");
}

/// Verifies a bare `new C();` statement (no assignment) executes the constructor for its
/// side effect, like PHP. Previously this errored at statement position.
#[test]
fn test_bare_new_object_statement_runs_constructor() {
    let out = compile_and_run(
        "<?php class C { public function __construct() { echo \"ctor\"; } } new C(); echo \"-done\";",
    );
    assert_eq!(out, "ctor-done");
}

/// Verifies a unary-operator-led statement (`-$x;`) parses and runs as a discarded
/// expression statement, leaving `$x` unchanged.
#[test]
fn test_unary_led_statement_is_discarded() {
    let out = compile_and_run("<?php $x = 7; -$x; echo $x;");
    assert_eq!(out, "7");
}

/// Verifies a call-result negation drives a short-circuit action: `f()` prints `f` and
/// returns false, so `!false` is true and `print("g")` runs, giving `fg`.
#[test]
fn test_negated_call_short_circuit_statement() {
    let out = compile_and_run(
        "<?php function f() { echo \"f\"; return false; } !f() && print(\"g\");",
    );
    assert_eq!(out, "fg");
}

/// Verifies a bare ternary-expression statement whose leading variable is a comparison
/// operand (mirroring Symfony's `ProgressBar::setProgress`, e.g.
/// `$startAt > 0 ? f($startAt) : $this->percent = 0.0;`). The condition is false, so the
/// false branch's assignment `$x = 5.0` runs; cross-checked with `php -r`.
#[test]
fn test_variable_led_ternary_statement_false_branch_assigns() {
    let out = compile_and_run(
        "<?php $s = 0; $x = 1.0; $s > 0 ? printf(\"t\") : $x = 5.0; echo $x;",
    );
    assert_eq!(out, "5");
}

/// Verifies the true-branch variant of the same bare ternary-expression statement: the
/// condition is true, so `printf(\"t\")` runs and the false branch's assignment never
/// executes, leaving `$x` at its original `1`. Cross-checked with `php -r`.
#[test]
fn test_variable_led_ternary_statement_true_branch_skips_assign() {
    let out = compile_and_run(
        "<?php $s = 3; $x = 1.0; $s > 0 ? printf(\"t\") : $x = 5.0; echo $x;",
    );
    assert_eq!(out, "t1");
}

/// Verifies the general case: a bare comparison statement whose leading token is a
/// variable (`$s > 0;`) compiles and runs as a discarded expression statement, producing
/// no output. Previously this errored with "Expected '=' after variable name".
#[test]
fn test_variable_led_comparison_statement_compiles_and_runs() {
    let out = compile_and_run("<?php $s = 5; $s > 0; echo \"ok\";");
    assert_eq!(out, "ok");
}

// --- Expression-position array-element / append assignment in ternary branches ---
//
// PHP assignment is an expression: `$a[] = v` and `$a[k] = v` are legal inside a ternary
// branch or any other expression position, not just as a bare statement. Previously
// `$a[] = v` in expression position failed to parse at all ("Unexpected token: RBracket")
// and `$c ? $a[k]=v1 : $a[k]=v2;` failed with "Expected ':' in ternary operator" because the
// statement-level postfix-assignment scanner (`find_top_level_assignment` in
// `src/parser/stmt/assign/postfix.rs`) did not track ternary `?`/`:` nesting and tried to
// parse the dangling prefix before the first `=` as a standalone expression. Every fixture
// here is cross-checked against `php -r` (see the padawan task's spec probes g1/g4/g5).

/// g1: append-assign as a ternary branch, used as the RHS of a plain assignment. `$a` starts
/// with one element; the true branch pushes a second, so `count($a)` is `2` — matching PHP.
#[test]
fn test_ternary_branch_append_assignment_expr_grows_array() {
    let out = compile_and_run("<?php $a = [0]; $x = true ? $a[] = 5 : 0; echo count($a);");
    assert_eq!(out, "2");
}

/// g4 (PhpDumper:1937 gate shape, true branch): `($c = f($v)) ? $ops[] = ... : ++$ops[0];`
/// used as a bare STATEMENT. The condition is true, so the append branch runs and `$a`
/// grows from one element to two.
#[test]
fn test_ternary_statement_append_vs_incdec_true_branch_appends() {
    let out = compile_and_run(
        "<?php $a = [0]; $c = true; $c ? $a[] = \"x\" : ++$a[0]; echo count($a);",
    );
    assert_eq!(out, "2");
}

/// Same gate shape, false branch: the append is skipped and `++$a[0]` runs instead,
/// incrementing the existing element in place (`$a` stays length 1, `$a[0]` becomes `1`).
#[test]
fn test_ternary_statement_append_vs_incdec_false_branch_increments() {
    let out = compile_and_run(
        "<?php $a = [0]; $c = false; $c ? $a[] = \"x\" : ++$a[0]; echo $a[0];",
    );
    assert_eq!(out, "1");
}

/// g5: indexed-element assignment (not append) in BOTH ternary branches. Previously this
/// was a hard parse error ("Expected ':' in ternary operator") because the statement-level
/// scanner mis-detected the first `=` inside the ternary's own branch as the whole
/// statement's assignment operator. True branch stores `7`.
#[test]
fn test_ternary_branch_indexed_element_assignment_true_branch() {
    let out = compile_and_run("<?php $a = [0]; $c = true; $c ? $a[0] = 7 : $a[0] = 8; echo $a[0];");
    assert_eq!(out, "7");
}

/// Same shape, false branch: stores `8` instead, proving the true branch's store does not
/// also leak into (or get read from) the untaken branch.
#[test]
fn test_ternary_branch_indexed_element_assignment_false_branch() {
    let out = compile_and_run("<?php $a = [0]; $c = false; $c ? $a[0] = 7 : $a[0] = 8; echo $a[0];");
    assert_eq!(out, "8");
}

/// Value-yield: PHP's assignment expression evaluates to the assigned value, so
/// `$a[] = 5` used as a parenthesized expression yields `5`, matching a plain `$x = 5`.
#[test]
fn test_append_assignment_expression_yields_assigned_value() {
    let out = compile_and_run("<?php $a = [0]; $x = ($a[] = 5); echo $x;");
    assert_eq!(out, "5");
}

/// Heterogeneous append: pushing a `Str` into an `array<int>` from the append branch must
/// take the same Mixed-promotion path the bare statement form (`$a[] = v;`) already uses,
/// so the push succeeds (rather than the checker rejecting a type mismatch) and `count($a)`
/// grows to `2`.
#[test]
fn test_ternary_branch_append_heterogeneous_element_type() {
    let out = compile_and_run(
        "<?php $a = [0]; $c = true; $c ? $a[] = \"x\" : ++$a[0]; echo count($a);",
    );
    assert_eq!(out, "2");
}

/// Heap-string value-yield through parens: `$x = ($a[] = $s . "z")` must hand `$x` the
/// live concatenated string, byte-for-byte. Regression guard for the correction-round-1
/// UAF: the first desugar yielded the hidden RHS temp via a `$t = $t` SELF-assignment,
/// and `store_local` releases a slot's current heap payload before re-acquiring the same
/// pointer, so `$x` received freed memory and printed blanks. The yield is now a copy into
/// a distinct hidden local. Cross-checked with `php -r` → `abababz abababz`.
#[test]
fn test_append_assignment_expression_yields_heap_string_via_parens() {
    let out = compile_and_run(
        "<?php $a = []; $s = str_repeat(\"ab\", 3); $x = ($a[] = $s . \"z\"); echo $x, \" \", $a[0];",
    );
    assert_eq!(out, "abababz abababz");
}

/// Heap-string value-yield through a TAKEN ternary branch: the same UAF shape as the parens
/// variant but reached via `lower_ternary`'s per-branch block placement, proving the yielded
/// string survives the branch merge. Cross-checked with `php -r` → `abababz abababz`.
#[test]
fn test_ternary_branch_append_yields_heap_string() {
    let out = compile_and_run(
        "<?php $a = []; $s = str_repeat(\"ab\", 3); $c = true; \
         $x = $c ? $a[] = $s . \"z\" : \"n\"; echo $x, \" \", $a[0];",
    );
    assert_eq!(out, "abababz abababz");
}

/// Heap-string value-yield consumed by ANOTHER container push: `$b[] = ($a[] = $s . \"!\")`
/// must store the same live bytes in both arrays (the outer statement-level push consumes
/// the inner expression-position append's yielded value). Cross-checked with `php -r`
/// → `cdcd! cdcd!`.
#[test]
fn test_append_yield_pushed_into_another_container() {
    let out = compile_and_run(
        "<?php $a = []; $b = []; $s = str_repeat(\"cd\", 2); \
         $b[] = ($a[] = $s . \"!\"); echo $b[0], \" \", $a[0];",
    );
    assert_eq!(out, "cdcd! cdcd!");
}

/// PHP evaluates an assignment's lvalue chain BEFORE its RHS: in
/// `getBox($box)->items[] = rhs()` used in expression position, `getBox()` must print "L"
/// before `rhs()` prints "R". Regression guard for the correction-round-1 eval-order bug:
/// the property-container prelude bound the RHS temp before the object expression, printing
/// "RL". The receiver is now stabilized into its own hidden temp first (same treatment as
/// indexed non-local targets). Cross-checked with `php -r` → `LR51` (yield `5`, one item).
#[test]
fn test_property_append_expression_evaluates_object_before_rhs() {
    let out = compile_and_run(
        "<?php
        class Box { public $items = []; }
        function getBox($b) { echo \"L\"; return $b; }
        function rhs() { echo \"R\"; return 5; }
        $box = new Box();
        $x = (getBox($box)->items[] = rhs());
        echo $x, count($box->items);",
    );
    assert_eq!(out, "LR51");
}

/// A namespaced function called in the append RHS inside a ternary branch must resolve to
/// the CURRENT namespace's function, not the global one. Regression guard for the
/// correction-round-1 name-resolver hole: the append desugar stores the RHS inside
/// `Assignment.prelude` at parse time, and the name resolver cloned preludes verbatim, so
/// `pick()` inside `namespace App` silently called the global `pick()` (printed 222).
/// Cross-checked with the php CLI → `111`.
#[test]
fn test_ternary_append_rhs_resolves_namespaced_function() {
    let out = compile_and_run(
        "<?php
        namespace App {
            function pick() { return 111; }
        }
        namespace {
            function pick() { return 222; }
        }
        namespace App {
            $a = [0];
            $c = true;
            $c ? $a[] = pick() : 0;
            echo $a[1];
        }",
    );
    assert_eq!(out, "111");
}

/// A namespaced class constructed in the append RHS (`$a[] = new Box(7)` inside
/// `namespace App`) must namespace-resolve to `App\Box`. Regression guard for the same
/// name-resolver prelude hole as the function variant: the unresolved name previously
/// surfaced as an "unknown class Box" EIR error. The constructor echoes to prove the
/// resolved class actually ran with its argument. Cross-checked with the php CLI → `B7 1`.
#[test]
fn test_ternary_append_rhs_namespaced_class_new_compiles_and_runs() {
    let out = compile_and_run(
        "<?php
        namespace App {
            class Box {
                public $v;
                public function __construct($v) { $this->v = $v; echo \"B\", $v; }
            }
        }
        namespace App {
            $a = [];
            $c = true;
            $c ? $a[] = new Box(7) : 0;
            echo \" \", count($a);
        }",
    );
    assert_eq!(out, "B7 1");
}

/// Foreach-loop variant mirroring the exact `--web` gate shape at PhpDumper.php:1937:
/// `($c = f($v)) ? $ops[] = "s$c" : ++$ops[0];` inside a loop body, where the ternary
/// condition itself is an assignment expression. `$ops` starts as `[0]`; each iteration
/// either appends a new "s{$c}" element (when `f($v)` is truthy) or increments `$ops[0]`
/// (when it is falsy/zero). Cross-checked with `php -r`.
#[test]
fn test_ternary_append_in_foreach_mirrors_phpdumper_gate() {
    let out = compile_and_run(
        "<?php
        function f($v) {
            if ($v > 3) return $v;
            return 0;
        }
        $ops = [0];
        foreach ([1, 5, 2, 8] as $v) {
            ($c = f($v)) ? $ops[] = \"s$c\" : ++$ops[0];
        }
        echo implode(\",\", $ops), \"\\n\";
        echo count($ops);",
    );
    assert_eq!(out, "2,s5,s8\n3");
}
