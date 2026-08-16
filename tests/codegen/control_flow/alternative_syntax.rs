//! Purpose:
//! Integration tests for PHP's alternative control-structure syntax
//! (`if:`/`endif;`, `while:`/`endwhile;`, `for:`/`endfor;`, `foreach:`/`endforeach;`,
//! `switch:`/`endswitch;`) compiled end to end and compared against PHP's output.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected string was produced by running the same fixture under `LC_ALL=C php`
//!   (PHP 8.4.20); the alternative form must be observationally identical to the brace form.
//! - Fixtures cover empty bodies, nesting in both directions, and `break`/`continue`, which are
//!   the edge cases where a desugaring mistake would show up.

use super::*;

/// Verifies the alternative `if (…): … endif;` form executes its body.
#[test]
fn test_alternative_if() {
    let out = compile_and_run("<?php if (true): echo \"y\"; endif;");
    assert_eq!(out, "y");
}

/// Verifies an alternative `if` whose condition is false produces no output.
#[test]
fn test_alternative_if_false() {
    let out = compile_and_run("<?php if (false): echo \"y\"; endif;");
    assert_eq!(out, "");
}

/// Verifies `elseif:` selects the matching branch in an alternative `if` chain.
#[test]
fn test_alternative_if_elseif_else() {
    let out = compile_and_run(
        "<?php $v = 2; if ($v === 1): echo \"one\"; elseif ($v === 2): echo \"two\"; else: echo \"other\"; endif;",
    );
    assert_eq!(out, "two");
}

/// Verifies `else:` runs when every preceding alternative branch condition is false.
#[test]
fn test_alternative_if_else_branch() {
    let out = compile_and_run(
        "<?php $v = 9; if ($v === 1): echo \"one\"; elseif ($v === 2): echo \"two\"; else: echo \"other\"; endif;",
    );
    assert_eq!(out, "other");
}

/// Verifies the alternative `foreach` form iterates a list.
#[test]
fn test_alternative_foreach() {
    let out = compile_and_run("<?php foreach ([1,2,3] as $x): echo $x; endforeach;");
    assert_eq!(out, "123");
}

/// Verifies the alternative `foreach` form supports the `$key => $value` binding.
#[test]
fn test_alternative_foreach_key_value() {
    let out = compile_and_run("<?php foreach ([\"a\"=>1,\"b\"=>2] as $k => $v): echo \"$k$v\"; endforeach;");
    assert_eq!(out, "a1b2");
}

/// Verifies the alternative `while` form loops until its condition is false.
#[test]
fn test_alternative_while() {
    let out = compile_and_run("<?php $i = 0; while ($i < 3): $i++; echo $i; endwhile;");
    assert_eq!(out, "123");
}

/// Verifies the alternative `for` form runs init, condition, and update as usual.
#[test]
fn test_alternative_for() {
    let out = compile_and_run("<?php for ($i = 0; $i < 3; $i++): echo $i; endfor;");
    assert_eq!(out, "012");
}

/// Verifies the alternative `switch` form dispatches to the matching case.
#[test]
fn test_alternative_switch() {
    let out = compile_and_run(
        "<?php $v = 1; switch ($v): case 1: echo \"one\"; break; case 2: echo \"two\"; break; default: echo \"d\"; endswitch;",
    );
    assert_eq!(out, "one");
}

/// Verifies the alternative `switch` form falls back to `default` when no case matches.
#[test]
fn test_alternative_switch_default() {
    let out = compile_and_run(
        "<?php $v = 9; switch ($v): case 1: echo \"one\"; break; default: echo \"d\"; endswitch;",
    );
    assert_eq!(out, "d");
}

/// Verifies grouped case labels still fall through in the alternative `switch` form.
#[test]
fn test_alternative_switch_case_fallthrough() {
    let out =
        compile_and_run("<?php $v = 1; switch ($v): case 1: case 2: echo \"low\"; break; endswitch;");
    assert_eq!(out, "low");
}

/// Verifies every alternative form accepts an empty body, the main edge case for the
/// terminator-driven statement loop.
#[test]
fn test_alternative_empty_bodies() {
    let out = compile_and_run(
        "<?php if (false): endif; while (false): endwhile; for(;false;): endfor; foreach([] as $z): endforeach; switch(1): endswitch; echo \"empty\";",
    );
    assert_eq!(out, "empty");
}

/// Verifies `break` and `continue` behave normally inside an alternative loop body.
#[test]
fn test_alternative_loop_break_and_continue() {
    let out = compile_and_run(
        "<?php for ($i=0;$i<4;$i++): if ($i===1): continue; endif; if ($i===3): break; endif; echo $i; endfor;",
    );
    assert_eq!(out, "02");
}

/// Verifies alternative loops nest inside one another.
#[test]
fn test_alternative_forms_nest() {
    let out = compile_and_run(
        "<?php $n=0; foreach([[1,2],[3,4]] as $row): foreach($row as $c): $n += $c; endforeach; endforeach; echo $n;",
    );
    assert_eq!(out, "10");
}

/// Verifies a brace-form `if` nests inside an alternative `foreach` body.
#[test]
fn test_brace_form_nested_in_alternative_form() {
    let out = compile_and_run(
        "<?php foreach ([1,2] as $a): if ($a === 1) { echo \"b$a\"; } else { echo \"a$a\"; } endforeach;",
    );
    assert_eq!(out, "b1a2");
}

/// Verifies an alternative `if` nests inside a brace-form `foreach` body.
#[test]
fn test_alternative_form_nested_in_brace_form() {
    let out = compile_and_run(
        "<?php foreach ([1,2] as $a) { if ($a === 1): echo \"b$a\"; else: echo \"a$a\"; endif; }",
    );
    assert_eq!(out, "b1a2");
}

/// Verifies alternative syntax works inside a function body, including `return` from a branch.
#[test]
fn test_alternative_if_inside_function() {
    let out = compile_and_run(
        "<?php function f(int $n): string { if ($n > 0): return \"pos\"; else: return \"nonpos\"; endif; } echo f(1), f(-1);",
    );
    assert_eq!(out, "posnonpos");
}
