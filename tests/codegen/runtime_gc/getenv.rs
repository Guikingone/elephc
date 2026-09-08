//! Purpose:
//! Heap-debug regression coverage for `getenv()` whole-environment results.
//!
//! Called from:
//! - `cargo test --test codegen_tests runtime_gc::getenv` through the runtime-GC suite.
//!
//! Key details:
//! - `__rt_getenv_all` allocates a hash plus persisted keys and values, then boxes
//!   the hash as Mixed. Releasing only the Mixed cell leaked the hash and every
//!   entry; each iteration must leave a clean heap.

use crate::support::*;

/// Verifies repeated no-argument `getenv()` results release the environment hash.
#[test]
fn test_getenv_all_owned_results_are_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$n = 0;
for ($i = 0; $i < 3; $i++) {
    $all = getenv();
    $n = count($all);
    unset($all);
}
echo $n > 0 ? "clean" : "empty";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "clean");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected getenv() results to leave a clean heap, got: {}",
        out.stderr
    );
}
