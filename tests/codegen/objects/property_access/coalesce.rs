//! Purpose:
//! Regression tests for PHP property reads used as the value side of null coalescing.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `??` probes properties with `isset` semantics, so an undeclared property yields its fallback
//!   without a checker diagnostic or an observable property-read warning.

use super::*;

/// Verifies an undeclared property on a known object behaves as absent under `??`.
#[test]
fn test_undefined_property_null_coalesce_uses_fallback() {
    let out = compile_and_run(
        r#"<?php
class OptionalLoggerOwner {
    public function logger(): mixed {
        return $this->logger ?? null;
    }
}
echo (new OptionalLoggerOwner())->logger() === null ? "null" : "value";
"#,
    );
    assert_eq!(out, "null");
}
