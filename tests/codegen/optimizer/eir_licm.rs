//! Purpose:
//! Integration tests for the EIR loop-invariant code motion pass (`licm`) driven
//! by the fixed-point pass driver. These exercise real loop CFGs (for, nested,
//! while, and a loop with an invariant-looking subexpression) and assert correct
//! output, guarding that LICM — together with the dominance and loop analyses it
//! builds on — never changes loop behavior.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Mutable PHP loop variables still use effectful `load_local` instructions.
//!   Read-only scalar inputs and single-assignment entry locals can now be proven
//!   immutable and hoisted with dependent pure computations; issue-623 structural
//!   coverage lives in the adjacent `checked_int_sink` module. `$argc` is 1 with
//!   no CLI arguments.

use super::*;

/// A `for` loop accumulating into a slot keeps its result: `$argc + (0+1+2+3+4)`.
#[test]
fn test_licm_for_loop_sum_preserved() {
    let out = compile_and_run("<?php $sum=$argc; for($i=0;$i<5;$i++){$sum=$sum+$i;} echo $sum;");
    assert_eq!(out, "11");
}

/// A nested loop runs the inner body the full product of iterations.
#[test]
fn test_licm_nested_loop_preserved() {
    let out = compile_and_run(
        "<?php $t=0; for($i=0;$i<3;$i++){ for($j=0;$j<3;$j++){ $t=$t+1; } } echo $t;",
    );
    assert_eq!(out, "9");
}

/// A `while` loop doubling a value runs to completion.
#[test]
fn test_licm_while_loop_preserved() {
    let out = compile_and_run("<?php $i=$argc; $p=1; while($i<=4){ $p=$p*2; $i++; } echo $p;");
    assert_eq!(out, "16");
}

/// A loop whose body recomputes an invariant-looking subexpression each iteration
/// still produces the correct total.
#[test]
fn test_licm_invariant_subexpression_preserved() {
    let out = compile_and_run(
        "<?php $n=$argc; $k=$argc; $s=0; for($i=0;$i<3;$i++){ $s=$s+($n+$k); } echo $s;",
    );
    assert_eq!(out, "6");
}
