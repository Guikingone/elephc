//! Purpose:
//! Checks the checker's binding decisions for `for` initializers.
//!
//! Called from:
//! - The diagnostic integration test root.
//!
//! Key details:
//! - A `for` initializer runs unconditionally, so it binds at the statement's depth.

use super::*;

/// A `for` counter retyped after its loop warns like the same assignment written before it.
///
/// The initializer used to be checked at the loop body's conditional depth, so this valid PHP
/// failed with `cannot reassign $i from int|float to string` while the hoisted form compiled.
#[test]
fn test_for_initializer_counter_retypes_after_the_loop() {
    expect_warning(
        "<?php for ($i = 0; $i < $argc; $i++) {} $i = \"done\"; echo $i;",
        "changes type from int|float to string",
    );
}

/// A straight-line `unset` after the loop now ends the counter binding the initializer created.
#[test]
fn test_for_initializer_counter_is_killed_by_a_later_unset() {
    let result = check_source_full(
        "<?php for ($i = 0; $i < $argc; $i++) {} unset($i); $i = \"fresh\"; echo $i;",
    )
    .expect("a killed counter re-binds at any type");
    assert_eq!(result.local_bind_kill_sites.len(), 1, "{:?}", result.local_bind_kill_sites);
    assert!(
        !result.warnings.iter().any(|warning| warning.message.contains("changes type")),
        "the binding after a kill is fresh, not a retype: {:?}",
        result.warnings
    );
}
