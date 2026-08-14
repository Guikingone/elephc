//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of control flow branches and loops, including if true, if false, and if else.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies that if (true) executes the branch.
#[test]
fn test_if_true() {
    let out = compile_and_run("<?php if (1 == 1) { echo \"yes\"; }");
    assert_eq!(out, "yes");
}

/// Verifies that if (false) skips the branch, producing no output.
#[test]
fn test_if_false() {
    let out = compile_and_run("<?php if (1 == 2) { echo \"yes\"; }");
    assert_eq!(out, "");
}

/// Verifies if/else selects the else branch when the condition is false.
#[test]
fn test_if_else() {
    let out = compile_and_run("<?php if (1 == 2) { echo \"a\"; } else { echo \"b\"; }");
    assert_eq!(out, "b");
}

/// Verifies elseif chain takes the second branch when first condition is false.
#[test]
fn test_if_elseif_else() {
    let out = compile_and_run(
        "<?php $x = 2; if ($x == 1) { echo \"one\"; } elseif ($x == 2) { echo \"two\"; } else { echo \"other\"; }",
    );
    assert_eq!(out, "two");
}

/// Verifies elseif/else falls through to the else branch when all conditions are false.
#[test]
fn test_if_else_falls_through() {
    let out = compile_and_run(
        "<?php $x = 99; if ($x == 1) { echo \"a\"; } elseif ($x == 2) { echo \"b\"; } else { echo \"c\"; }",
    );
    assert_eq!(out, "c");
}

// --- while ---

/// Verifies a basic while loop iterates 5 times and prints 01234.
#[test]
fn test_while_loop() {
    let out = compile_and_run("<?php $i = 0; while ($i < 5) { echo $i; $i = $i + 1; }");
    assert_eq!(out, "01234");
}

/// Verifies while (false) body never executes.
#[test]
fn test_while_zero_iterations() {
    let out = compile_and_run("<?php while (0) { echo \"no\"; }");
    assert_eq!(out, "");
}

/// Verifies break exits the while loop when $i reaches 3, producing 012.
#[test]
fn test_while_break() {
    let out = compile_and_run(
        "<?php $i = 0; while ($i < 10) { if ($i == 3) { break; } echo $i; $i = $i + 1; }",
    );
    assert_eq!(out, "012");
}

/// Verifies continue skips to the next iteration, skipping the echo at $i==3.
#[test]
fn test_while_continue() {
    let out = compile_and_run(
        "<?php $i = 0; while ($i < 5) { $i = $i + 1; if ($i == 3) { continue; } echo $i; }",
    );
    assert_eq!(out, "1245");
}

/// Verifies break 2 exits both the inner and outer loop from within the innermost loop.
#[test]
fn test_multilevel_break_exits_nested_loops() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0; $i < 3; $i++) {
    echo "i" . $i . ":";
    for ($j = 0; $j < 3; $j++) {
        if ($i == 1) { break 2; }
        echo $j;
    }
}
echo "end";
"#,
    );
    assert_eq!(out, "i0:012i1:end");
}

/// Verifies continue 2 jumps to the outer loop's update expression, skipping the inner loop body and outer echo.
#[test]
fn test_multilevel_continue_targets_outer_loop_update() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0; $i < 3; $i++) {
    echo "i" . $i . ":";
    for ($j = 0; $j < 3; $j++) {
        if ($j == 1) { continue 2; }
        echo $j;
    }
    echo "x";
}
echo "end";
"#,
    );
    assert_eq!(out, "i0:0i1:0i2:0end");
}

/// Verifies continue 2 from within a switch targets the outer for loop, not the switch itself.
#[test]
fn test_multilevel_continue_from_switch_targets_outer_loop() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0; $i < 3; $i++) {
    echo "a";
    switch ($i) {
        case 1:
            echo "b";
            continue 2;
        default:
            echo "c";
    }
    echo "d";
}
"#,
    );
    assert_eq!(out, "acdabacd");
}

/// Verifies break 2 through a try/finally runs the finally block exactly once before exiting.
#[test]
fn test_multilevel_break_through_finally_runs_finally_once() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0; $i < 2; $i++) {
    for ($j = 0; $j < 2; $j++) {
        try {
            echo "t";
            break 2;
        } finally {
            echo "f";
        }
    }
    echo "x";
}
echo "e";
"#,
    );
    assert_eq!(out, "tfe");
}

// --- for ---

/// Verifies a basic for loop with manual increment in body iterates 5 times and prints 01234.
#[test]
fn test_for_loop() {
    let out = compile_and_run("<?php for ($i = 0; $i < 5; $i = $i + 1) { echo $i; }");
    assert_eq!(out, "01234");
}

/// Verifies comma-separated `for` clauses execute left-to-right, including after `continue`.
#[test]
fn test_for_multiple_init_and_update_expressions() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0, $j = 3; $i < 3; ++$i, --$j) {
    if ($i === 1) { continue; }
    echo $i . ":" . $j . ";";
}
echo $i . ":" . $j;
"#,
    );
    assert_eq!(out, "0:3;2:1;3:0");
}

/// Verifies that call expressions in `for` clauses execute for their side effects.
#[test]
fn test_for_call_expression_init_and_update() {
    let out = compile_and_run(
        r#"<?php
$paths = ['a', 'b', 'c'];
for (next($paths); null !== key($paths); next($paths)) {
    echo current($paths);
}
"#,
    );
    assert_eq!(out, "bc");
}

/// Verifies a literal-led short-circuit assignment statement preserves its RHS side effect.
#[test]
fn test_literal_led_short_circuit_assignment_statement() {
    let out = compile_and_run("<?php $value = -1; 0 > $value && $value += 0x40; echo $value;");
    assert_eq!(out, "63");
}

/// Verifies break exits the for loop when $i reaches 3, producing 012.
#[test]
fn test_for_break() {
    let out = compile_and_run(
        "<?php for ($i = 0; $i < 10; $i = $i + 1) { if ($i == 3) { break; } echo $i; }",
    );
    assert_eq!(out, "012");
}

// --- FizzBuzz ---

/// Verifies nested if/elseif/else chain correctly maps 1–15 to Fizz/Buzz/FizzBuzz/decimal output.
#[test]
fn test_fizzbuzz() {
    let source = r#"<?php
$i = 1;
while ($i <= 15) {
    if ($i % 15 == 0) {
        echo "FizzBuzz\n";
    } elseif ($i % 3 == 0) {
        echo "Fizz\n";
    } elseif ($i % 5 == 0) {
        echo "Buzz\n";
    } else {
        echo $i;
        echo "\n";
    }
    $i = $i + 1;
}
"#;
    let out = compile_and_run(source);
    assert_eq!(
        out,
        "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n"
    );
}

// --- Increment/Decrement ---

/// Verifies for loop with ++$i post-increment in update expression prints 01234.
#[test]
fn test_for_with_increment() {
    let out = compile_and_run("<?php for ($i = 0; $i < 5; $i++) { echo $i; }");
    assert_eq!(out, "01234");
}

/// Verifies pre-increment (++$i) updates before the loop body echo, producing 123.
#[test]
fn test_while_with_pre_increment() {
    let out = compile_and_run("<?php $i = 0; while ($i < 3) { ++$i; echo $i; }");
    assert_eq!(out, "123");
}

// --- Functions ---

/// Verifies null is falsy in an if condition, selecting the else branch.
#[test]
fn test_if_null_is_falsy() {
    let out = compile_and_run(
        r#"<?php
$x = null;
if ($x) {
    echo "true";
} else {
    echo "false";
}
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies null as a while condition prevents loop entry and prints ok.
#[test]
fn test_while_null_no_loop() {
    let out = compile_and_run("<?php $x = null; while ($x) { echo \"bad\"; } echo \"ok\";");
    assert_eq!(out, "ok");
}

/// Makes an assignment on the right of `||` visible when the false outcome selects `else`.
#[test]
fn test_false_or_outcome_exposes_rhs_assignment_in_else() {
    let out = compile_and_run(
        "<?php
        function render(bool $lead): void {
            if ($lead || !($value = 'ready')) {
                echo 'then';
            } else {
                echo $value;
            }
        }
        render(false);",
    );
    assert_eq!(out, "ready");
}

/// Preserves the value assigned as the left operand of an `instanceof` condition.
#[test]
fn test_instanceof_condition_preserves_assignment_effect() {
    let out = compile_and_run(
        "<?php
        class Marker {}
        function render(object $input): void {
            if (($value = $input) instanceof Marker) {
                echo $value::class;
            }
        }
        render(new Marker());",
    );
    assert_eq!(out, "Marker");
}

/// Makes assignments from a successful `&&` chain visible to the true ternary arm.
#[test]
fn test_ternary_true_arm_sees_short_circuit_assignment() {
    let out = compile_and_run(
        "<?php
        $result = true
            && (($value = 'ready') === 'ready')
            && strlen($value) > 0
                ? $value
                : 'missing';
        echo $result;",
    );
    assert_eq!(out, "ready");
}

/// Makes an assignment in a false equality condition visible to the false ternary arm.
#[test]
fn test_ternary_false_arm_sees_condition_assignment() {
    let out = compile_and_run(
        "<?php
        function prefix(string $value): string {
            return false === ($position = strrpos($value, '\\\\'))
                ? ''
                : substr($value, 0, $position);
        }
        echo prefix('Alpha\\\\Beta');",
    );
    assert_eq!(out, "Alpha");
}

/// Preserves a ternary's branch-local assignment inside a property `??=` initializer.
#[test]
fn test_property_null_coalesce_ternary_sees_condition_assignment() {
    let out = compile_and_run(
        "<?php
        final class PrefixCache {
            private ?string $cached = null;
            public function resolve(string $value): string {
                return $this->cached ??= false === ($position = strrpos($value, '\\\\'))
                    ? ''
                    : substr($value, 0, $position);
            }
        }
        echo (new PrefixCache())->resolve('Alpha\\\\Beta');",
    );
    assert_eq!(out, "Alpha");
}

/// Boxes a local whose value changes from null to an indexable object across a loop back-edge.
#[test]
fn test_loop_carried_local_widens_runtime_representation() {
    let out = compile_and_run(
        r#"<?php
class LoopStorageSource {
    public $value;
}
$source = new LoopStorageSource();
$source->value = new ArrayObject(['ready']);
$slot = null;
foreach ([0, 1] as $iteration) {
    if ($slot) {
        echo $slot[0];
    }
    $slot = $source->value;
}
"#,
    );
    assert_eq!(out, "ready");
}

// --- Ternary operator ---
