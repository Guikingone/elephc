//! Purpose:
//! Verifies scalar indexed-array overwrites across unique, COW, growth, and reference shapes.
//!
//! Called from:
//! - `cargo test` through the codegen integration harness.
//!
//! Key details:
//! - Unique in-bounds writes may use the runtime fast path.
//! - Shared arrays must still split, while reference aliases must observe the same mutation.

use crate::support::*;

/// Scalar writes preserve PHP values while selecting fast and fallback ownership paths.
#[test]
fn test_scalar_array_set_fast_path_preserves_cow_growth_and_references() {
    let out = compile_and_run(
        r#"<?php
$ints = [1, 2, 3];
$ints[1] = 9;
$floats = [1.5, 2.5];
$floats[0] = 3.75;
$bools = [true, false];
$bools[1] = true;
$source = [10.5, 20.5];
$copy = $source;
$copy[0] = 99.5;
$grow = [1.0];
$grow[1] = 2.0;
$refs = [4.0, 5.0];
$alias =& $refs;
$refs[1] = 8.0;
for ($i = 0; $i < 1000; $i++) {
    $floats[$i % 2] += 0.25;
}
echo implode(',', $ints), '|', implode(',', $floats), '|';
echo ($bools[0] ? '1' : '0'), ($bools[1] ? '1' : '0'), '|';
echo implode(',', $source), '|', implode(',', $copy), '|';
echo implode(',', $grow), '|', implode(',', $alias);
"#,
    );
    assert_eq!(
        out,
        "1,9,3|128.75,127.5|11|10.5,20.5|99.5,20.5|1,2|4,8"
    );
}
